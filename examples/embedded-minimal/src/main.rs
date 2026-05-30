use furl_core::{DownloadConfig, Downloader, GraphicalProgressReporter};

#[tokio::main]
async fn main() {
    let url = "https://raw.githubusercontent.com/ghimiresdp/furl-cli/refs/heads/main/res/images/example.png";

    let config = DownloadConfig::default().set_max_chunk_size(5 * 1024 * 1024); // 5 MB

    let mut downloader = Downloader::new(url)
        .with_reporter(GraphicalProgressReporter::new())
        .with_config(config);

    if downloader.download(".", None, Some(4)).await.is_ok() {
        println!("Download completed successfully!");
    } else {
        println!("Download failed.");
    }
}
