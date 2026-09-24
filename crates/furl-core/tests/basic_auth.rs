//! End-to-end tests for HTTP Basic Authentication against a minimal
//! in-process HTTP server, so they run offline and in CI without Docker.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use furl_core::{DownloadConfig, Downloader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const USERNAME: &str = "alice";
const PASSWORD: &str = "s3cret";
/// base64 of `alice:s3cret`
const EXPECTED_AUTH: &str = "Basic YWxpY2U6czNjcmV0";

/// Counters the tests assert on, shared with the server task.
#[derive(Default)]
struct Stats {
    authorized: AtomicUsize,
    rejected: AtomicUsize,
}

/// Serves `body` behind Basic Auth. With `ranges`, it answers `Range`
/// requests with `206` and a `Content-Range` header (what enables furl's
/// multi-chunk mode); without, it always sends the whole body with `200`,
/// like servers that don't support ranges.
async fn spawn_server(body: Vec<u8>, ranges: bool) -> (String, Arc<Stats>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let body = Arc::new(body);
    let stats = Arc::new(Stats::default());

    let server_stats = Arc::clone(&stats);
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let body = Arc::clone(&body);
            let stats = Arc::clone(&server_stats);
            tokio::spawn(async move {
                let request = read_request_head(&mut socket).await;
                let header = |name: &str| {
                    request.lines().find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case(name)
                            .then(|| value.trim().to_string())
                    })
                };

                if header("authorization").as_deref() != Some(EXPECTED_AUTH) {
                    stats.rejected.fetch_add(1, Ordering::SeqCst);
                    let response = "HTTP/1.1 401 Unauthorized\r\n\
                        WWW-Authenticate: Basic realm=\"test\"\r\n\
                        Content-Length: 12\r\nConnection: close\r\n\r\nUnauthorized";
                    socket.write_all(response.as_bytes()).await.unwrap();
                    return;
                }
                stats.authorized.fetch_add(1, Ordering::SeqCst);

                let range = header("range").filter(|_| ranges).and_then(|range| {
                    let (start, end) = range.strip_prefix("bytes=")?.split_once('-')?;
                    let start: usize = start.parse().ok()?;
                    let end: usize = end.parse().ok()?;
                    Some((start, end.min(body.len() - 1)))
                });

                let (status, extra, payload) = match range {
                    Some((start, end)) => (
                        "206 Partial Content",
                        format!("Content-Range: bytes {start}-{end}/{}\r\n", body.len()),
                        &body[start..=end],
                    ),
                    None => ("200 OK", String::new(), &body[..]),
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    payload.len()
                );
                socket.write_all(head.as_bytes()).await.unwrap();
                socket.write_all(payload).await.unwrap();
            });
        }
    });

    (format!("http://{addr}"), stats)
}

async fn read_request_head(socket: &mut tokio::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while !buf.ends_with(b"\r\n\r\n") {
        if socket.read(&mut byte).await.unwrap() == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    String::from_utf8_lossy(&buf).into_owned()
}

/// Deterministic, non-repeating-ish content so misplaced chunks are caught.
fn test_body(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 31 % 251) as u8).collect()
}

/// A fresh temp directory, unique per call so parallel tests never collide.
fn temp_dir() -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("furl-auth-test-{}-{nanos}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

async fn download(
    downloader: Downloader,
) -> (
    Result<(), Box<dyn std::error::Error + Send + Sync>>,
    PathBuf,
) {
    let dir = temp_dir();
    let mut downloader =
        downloader.with_config(DownloadConfig::default().set_max_chunk_size(256 * 1024));
    let result = downloader
        .download(dir.to_str().unwrap(), Some("out.bin".to_string()), Some(4))
        .await;
    (result, dir.join("out.bin"))
}

#[tokio::test]
async fn multi_chunk_download_authenticates_every_request() {
    // 3 MB with a 256 KB chunk cap -> 12 chunks across 4 workers
    let body = test_body(3 * 1024 * 1024);
    let (base, stats) = spawn_server(body.clone(), true).await;

    let downloader =
        Downloader::new(format!("{base}/file.bin")).with_basic_auth(USERNAME, Some(PASSWORD));
    let (result, path) = download(downloader).await;

    result.unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), body);
    assert_eq!(stats.rejected.load(Ordering::SeqCst), 0);
    // the header probe plus one request per chunk
    assert!(stats.authorized.load(Ordering::SeqCst) > 10);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn small_file_download_with_auth() {
    let body = test_body(10 * 1024);
    let (base, stats) = spawn_server(body.clone(), true).await;

    let downloader =
        Downloader::new(format!("{base}/small.bin")).with_basic_auth(USERNAME, Some(PASSWORD));
    let (result, path) = download(downloader).await;

    result.unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), body);
    assert_eq!(stats.rejected.load(Ordering::SeqCst), 0);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn download_without_range_support_writes_whole_body() {
    let body = test_body(1536 * 1024);
    let (base, stats) = spawn_server(body.clone(), false).await;

    let downloader =
        Downloader::new(format!("{base}/stream.bin")).with_basic_auth(USERNAME, Some(PASSWORD));
    let (result, path) = download(downloader).await;

    result.unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), body);
    assert_eq!(stats.rejected.load(Ordering::SeqCst), 0);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn credentials_in_url_are_used() {
    let body = test_body(2 * 1024 * 1024);
    let (base, stats) = spawn_server(body.clone(), true).await;
    let url = base.replace("http://", &format!("http://{USERNAME}:{PASSWORD}@"));

    let (result, path) = download(Downloader::new(format!("{url}/file.bin"))).await;

    result.unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), body);
    assert_eq!(stats.rejected.load(Ordering::SeqCst), 0);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn wrong_password_fails_with_401() {
    let (base, stats) = spawn_server(test_body(2 * 1024 * 1024), true).await;

    let downloader =
        Downloader::new(format!("{base}/file.bin")).with_basic_auth(USERNAME, Some("wrong"));
    let (result, path) = download(downloader).await;

    let err = result.unwrap_err().to_string();
    assert!(err.contains("401"), "unexpected error: {err}");
    assert_eq!(stats.authorized.load(Ordering::SeqCst), 0);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn missing_credentials_fail_with_401() {
    let (base, _) = spawn_server(test_body(1024), true).await;

    let (result, path) = download(Downloader::new(format!("{base}/file.bin"))).await;

    let err = result.unwrap_err().to_string();
    assert!(err.contains("401"), "unexpected error: {err}");
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
