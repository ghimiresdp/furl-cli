use std::{
    cmp::min,
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use reqwest::{
    self, Url,
    header::{CONTENT_DISPOSITION, CONTENT_RANGE, HeaderMap, RANGE},
};

// use tokio::stream;
use futures_util::{StreamExt, future};
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
    sync::Mutex,
};

const _1MB: u64 = 1024 * 1024;
const _10MB: u64 = 10 * 1024 * 1024;

pub trait ProgressReporter: Send + Sync {
    fn on_start(&self, chunk_index: usize, total_bytes: u64);
    fn on_progress(&self, chunk_index: usize, bytes_downloaded: u64);
    fn on_finish(&self, chunk_index: usize);
}

// No-op for consumers who don't care about progress
pub struct NoopReporter;

impl ProgressReporter for NoopReporter {
    fn on_start(&self, _chunk_index: usize, _total_bytes: u64) {}
    fn on_progress(&self, _chunk_index: usize, _bytes_downloaded: u64) {}
    fn on_finish(&self, _chunk_index: usize) {}
}

#[derive(Debug)]
struct Chunk {
    start_byte: u64,
    end_byte: u64,
    downloaded: u64,
}

impl Chunk {
    fn new(start_byte: u64, end_byte: u64) -> Self {
        Self {
            start_byte,
            end_byte,
            downloaded: 0,
        }
    }
}

#[derive(Clone)]
pub struct DownloadConfig {
    pub max_chunk_size: u64,
}

impl DownloadConfig {
    pub fn new() -> Self {
        Self {
            max_chunk_size: _10MB,
        }
    }
    pub fn set_max_chunk_size(mut self, size: u64) -> Self {
        self.max_chunk_size = size;
        self
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Downloader {
    url: String,
    headers: HeaderMap,
    file_size: Option<u64>,
    filename: Option<String>,
    chunks: Arc<Mutex<Vec<Chunk>>>, // this stores downloaded chunk size
    reporter: Arc<dyn ProgressReporter + Send + Sync>,
    config: Arc<DownloadConfig>,
}

pub trait HeaderUtils {
    /// # `extract_filename`
    ///
    /// When response header provides content disposition or any other keys to
    /// provide file name or file type, we can extract it from here.
    ///
    /// We can also guess file name from the url and content-type too.
    fn extract_filename(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// # `extract_file_size`
    ///
    /// When response header provides content-range, it is easy to extract the
    /// actual file size in bytes.
    ///
    /// example response: `Content-Range` `bytes 0-0/360996864`
    ///
    /// From the above response header, we can extract value in bytes
    ///
    /// It will help us downloading partial data with multiple threads.
    ///
    fn extract_file_size(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>>;
}

/// Since HeaderMap is imported from Reqwest, we need to define trait and then
/// implement it to the HeaderMap struct.
impl HeaderUtils for HeaderMap {
    fn extract_filename(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(disposition) = &self.get(CONTENT_DISPOSITION) {
            let value = disposition.to_str()?;
            if let Some(filename) = value.split("filename=").nth(1) {
                return Ok(filename.trim_matches('"').to_string());
            }
        }
        Err(Box::from("Unable to extract filename".to_owned()))
        // TODO: guess filename from content type
    }

    /// Returns the file size in bytes.
    ///
    /// If the content-length is is not found or not extracted, it just returns
    /// Error
    fn extract_file_size(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let &cr = &self
            .get(CONTENT_RANGE)
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Content_range not found"))?;
        let content_range =
            cr.to_str()?.split("/").last().ok_or_else(|| {
                Box::<dyn Error + Send + Sync>::from("Invalid Content_range_format")
            })?;
        Ok(content_range.parse()?)
    }
}

/// # Extract file name from Urls
/// This method is used when we do not have any headers passed for file name
/// For example: if content disposition is not provided, but there is a valid
/// filename in the request url
pub fn extract_filename_from_url(url: &str) -> Option<String> {
    if let Ok(parsed_url) = Url::parse(url)
        && let Some(segment) = parsed_url.path_segments().and_then(|mut s| s.next_back())
        && !segment.is_empty()
    {
        return Some(segment.to_string());
    }

    None
}

impl Downloader {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            headers: HeaderMap::new(),
            file_size: None,
            filename: None,
            chunks: Arc::new(Mutex::new(Vec::new())),
            reporter: Arc::new(NoopReporter),
            config: Arc::new(DownloadConfig::default()),
        }
    }

    pub fn with_config(mut self, config: DownloadConfig) -> Self {
        self.config = Arc::new(config);
        self
    }

    pub fn with_reporter<R: ProgressReporter + Send + Sync + 'static>(
        mut self,
        reporter: R,
    ) -> Self {
        self.reporter = Arc::new(reporter);
        self
    }

    async fn get_chunk(
        &self,
        range: Option<(u64, u64)>,
        reporter: Arc<dyn ProgressReporter + Send + Sync>,
        file: Option<Arc<Mutex<File>>>,
        chunk_index: Option<usize>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut builder = client.get(&self.url);
        if let Some((start, end)) = range {
            builder = builder.header(RANGE, &format!("bytes={start}-{end}"));
            reporter.on_start(chunk_index.unwrap_or(0), end - start + 1);
        }
        let response = builder.send().await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut chunk_data = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            chunk_data.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            reporter.on_progress(chunk_index.unwrap_or(0), downloaded);

            // Update chunk progress in shared state
            if let Some(idx) = chunk_index {
                let mut chunks = self.chunks.lock().await;
                if idx < chunks.len() {
                    chunks[idx].downloaded = downloaded;
                }
            }
        }

        // Write to file at correct position
        if let (Some(file), Some((start, _))) = (file, range) {
            let mut f = file.lock().await;
            f.seek(SeekFrom::Start(start)).await?;
            f.write_all(&chunk_data).await?;
        }

        reporter.on_finish(chunk_index.unwrap_or(0));

        Ok(downloaded)
    }

    /// downloads the file into the provided path
    /// # Arguments
    ///
    /// * `path`: download path
    /// * `threads`: number of threads to use for downloading.
    ///   if you pass None, or Some(0), it will defaults to 8
    ///
    /// Note: If the download size is less than 1 MB, then it will completely
    /// ignore threads, and download it as a single thread.
    ///
    /// If the file size is unknown at the moment it gets the header, it will
    /// also ignore threads and skips the progress bar and just shows a simple
    /// ticker as a feedback to let user know that the process is not is in a
    /// deadlock state.
    ///
    pub async fn download(
        &mut self,
        path: &str,
        filename: Option<String>,
        threads: Option<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let threads: u64 = match threads {
            Some(0) => 8,
            Some(count) => count as u64,
            None => 8,
        };
        // get response headers to get file name, length, etc.
        let response = client
            .get(&self.url)
            .header(RANGE, "bytes=0-0")
            .send()
            .await?;
        self.headers = response.headers().clone().to_owned();

        let filename = if let Some(filename) = filename {
            filename
        } else {
            match &self.headers.extract_filename() {
                Ok(filename) => filename.to_owned(),
                Err(_) => match extract_filename_from_url(&self.url) {
                    Some(filename) => filename,
                    None => "download.dat".to_owned(),
                },
            }
        };
        println!("Downloading \"{filename}\"");
        self.filename = Some(format!("{path}/{filename}").replace("//", "/"));

        if let Ok(file_size) = self.headers.extract_file_size() {
            self.file_size = Some(file_size);
            println!("file size: {}", file_size);
        } else {
            println!("Unable to determine the file size. skipping threads");
        }

        let file = Arc::new(Mutex::new(
            File::create(self.filename.as_ref().unwrap()).await?,
        ));

        // handle chunks with threads
        if let Some(file_size) = self.file_size {
            // allocate file's size if size is known
            // this helps seeking to the position and writing the chunk at that position
            file.lock().await.set_len(file_size).await?;

            let mut start = 0;
            // Determine chunk size: default to per-thread slice, cap at 10 MB for memory,
            // or use full file if smaller than 1 MB (no threading benefit).
            let chunk_size = if file_size < _1MB {
                println!("ℹ️ The file is smaller than 1 MB, so skipping threads.");
                file_size
            } else {
                (file_size / threads).min(self.config.max_chunk_size)
            };

            // split chunks to download
            while start < file_size {
                let end = min(start + chunk_size, file_size);
                self.chunks.lock().await.push(Chunk::new(start, end));
                start = end + 1;
            }

            let num_chunks = self.chunks.lock().await.len();
            println!("Created {} chunks for download", num_chunks);

            // Create tasks for concurrent downloading
            let mut tasks = Vec::new();
            let chunks_clone = Arc::clone(&self.chunks);

            // Use a fixed-size worker pool and an atomic index so we don't spawn one task per chunk.
            // Each worker atomically pulls the next chunk index and processes it, which creates
            // queue-like behavior while keeping the number of concurrent tasks limited to `threads`.
            let index = Arc::new(AtomicUsize::new(0));

            // limit number of workers to at most num_chunks
            let threads_to_spawn = std::cmp::min(threads as usize, num_chunks);

            for _ in 0..threads_to_spawn {
                let chunks = Arc::clone(&chunks_clone);
                let file_clone = Arc::clone(&file);
                let url = self.url.clone();
                let index_clone = Arc::clone(&index);
                let reporter_clone = Arc::clone(&self.reporter);
                let config = Arc::clone(&self.config);

                let task = tokio::spawn(async move {
                    let mut worker_total: u64 = 0;
                    loop {
                        // fetch next chunk index
                        let i = index_clone.fetch_add(1, Ordering::SeqCst);
                        if i >= num_chunks {
                            break;
                        }

                        // Get chunk info
                        let (start, end) = {
                            let chunks_guard = chunks.lock().await;
                            (chunks_guard[i].start_byte, chunks_guard[i].end_byte)
                        };

                        // Create a downloader instance for this chunk
                        let downloader = Downloader {
                            url: url.clone(),
                            headers: HeaderMap::new(),
                            file_size: None,
                            filename: None,
                            chunks: Arc::clone(&chunks),
                            reporter: Arc::clone(&reporter_clone),
                            config: Arc::clone(&config),
                        };

                        // Download the chunk and accumulate the bytes downloaded by this worker
                        let downloaded = downloader
                            .get_chunk(
                                Some((start, end)),
                                Arc::clone(&reporter_clone),
                                Some(Arc::clone(&file_clone)),
                                Some(i),
                            )
                            .await?;
                        worker_total += downloaded;
                    }

                    Ok::<u64, Box<dyn std::error::Error + Send + Sync>>(worker_total)
                });

                tasks.push(task);
            }

            // Wait for all downloads to complete
            println!("Starting concurrent downloads...");
            let results = future::try_join_all(tasks)
                .await
                .map_err(|e| format!("Task join error: {}", e))?;

            let total_downloaded: u64 = results
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum();

            println!("Download completed! Total bytes: {}", total_downloaded);
        } else {
            // continue without threads when the file size is unknown
            // and just display ticks instead of progressbar since the size is unknown.
            let file_clone = Arc::clone(&file);
            println!("\n");
            let _ = self
                .get_chunk(None, Arc::clone(&self.reporter), Some(file_clone), None)
                .await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use tokio::runtime::Runtime;

    #[test]
    fn test_extract_filename_from_url() {
        let url = "https://example.com/path/to/file.txt";
        assert_eq!(extract_filename_from_url(url), Some("file.txt".to_string()));
        let url2 = "https://example.com/path/to/";
        assert_eq!(extract_filename_from_url(url2), None);
    }

    #[test]
    fn test_header_extract_filename() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            "attachment; filename=\"myfile.bin\"".parse().unwrap(),
        );
        let name = headers.extract_filename().unwrap();
        assert_eq!(name, "myfile.bin");
    }

    #[test]
    fn test_header_extract_file_size() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_RANGE,
            "bytes 0-0/12345".parse().unwrap(),
        );
        let size = headers.extract_file_size().unwrap();
        assert_eq!(size, 12345u64);
    }

    #[test]
    fn test_downloader_new_and_defaults() {
        let d = Downloader::new("https://example.com/file");
        assert_eq!(d.url, "https://example.com/file");
        assert!(d.filename.is_none());
        assert!(d.file_size.is_none());
    }

    // Placeholder async test for download-related behavior; does not perform network IO.
    #[test]
    fn test_download_placeholder() {
        // Create a runtime to run async parts if needed.
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut downloader = Downloader::new("https://example.com/file");
            // Set internal fields to avoid real network operations in this placeholder.
            downloader.headers = HeaderMap::new();
            downloader.filename = Some("tmp_download.bin".to_string());
            downloader.file_size = Some(0);

            // Ensure setters/readers behave as expected in a minimal scenario.
            assert_eq!(downloader.filename.as_deref(), Some("tmp_download.bin"));
            assert_eq!(downloader.file_size, Some(0));
        });
    }
    #[test]
    fn test_custom_download_config() {
        let config = DownloadConfig::new().set_max_chunk_size(5 * 1024 * 1024);
        assert_eq!(config.max_chunk_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_default_download_config() {
        let config = DownloadConfig::new();
        assert_eq!(config.max_chunk_size, 10 * 1024 * 1024);
    }
}
