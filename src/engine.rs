use std::{cmp::min, error::Error, sync::Arc, time::Duration};

use reqwest::{
    self, Url,
    header::{CONTENT_DISPOSITION, CONTENT_RANGE, HeaderMap, RANGE},
};
// use tokio::stream;
use futures_util::{StreamExt, future};
use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::Mutex;

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

#[derive(Debug)]
pub struct Downloader {
    url: String,
    headers: HeaderMap,
    file_size: Option<u64>,
    filename: Option<String>,
    chunks: Arc<Mutex<Vec<Chunk>>>, // this stores downloaded chunk size
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
        return Err(Box::from("Unable to extract filename".to_owned()));
        // TODO: guess filename from content type
    }

    fn extract_file_size(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let &cr = &self
            .get(CONTENT_RANGE)
            .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Content_range not found"))?;
        let content_range =
            cr.to_str()?.split("/").into_iter().last().ok_or_else(|| {
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
    if let Ok(parsed_url) = Url::parse(&url) {
        if let Some(segment) = parsed_url.path_segments().and_then(|s| s.last()) {
            if !segment.is_empty() {
                return Some(segment.to_string());
            }
        }
    }
    return None;
}

impl Downloader {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            headers: HeaderMap::new(),
            file_size: None,
            filename: None,
            chunks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_chunk(
        &self,
        range: Option<(u64, u64)>,
        progress_bar: Option<ProgressBar>,
        file: Option<Arc<Mutex<File>>>,
        chunk_index: Option<usize>,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut builder = client.get(&self.url);
        if let Some((start, end)) = range {
            builder = builder.header(RANGE, &format!("bytes={start}-{end}"));
        }
        let response = builder.send().await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut chunk_data = Vec::new();

        // progress_bar
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            chunk_data.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            if let Some(bar) = &progress_bar {
                bar.inc(chunk.len() as u64);
            }

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

        if let Some(bar) = progress_bar {
            bar.finish();
        }

        Ok(downloaded)
    }

    pub async fn download(
        &mut self,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        // get response headers to get file name, length, etc.
        let response = client
            .get(&self.url)
            .header(RANGE, "bytes=0-0")
            .send()
            .await?;
        self.headers = response.headers().clone().to_owned();

        let filename = match &self.headers.extract_filename() {
            Ok(filename) => filename.to_owned(),
            Err(_) => match extract_filename_from_url(&self.url) {
                Some(filename) => filename,
                None => "download.bin".to_owned(),
            },
        };
        println!("⛔filename: {filename}");
        // trim trailing / from original path
        self.filename = Some(format!("{path}/{filename}").replace("//", "/"));

        if let Ok(file_size) = self.headers.extract_file_size() {
            self.file_size = Some(file_size);
            println!("⛔file size: {}", HumanBytes(file_size));
        } else {
            println!("⛔ Unable to determine the file size. skipping threads")
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
            let byte_size = file_size / 8;

            // split chunks to download
            while start < file_size {
                let end = min(start + byte_size, file_size);
                self.chunks.lock().await.push(Chunk::new(start, end));
                start = end + 1;
            }

            let num_chunks = self.chunks.lock().await.len();
            println!("Created {} chunks for download", num_chunks);

            let multi_progress = Arc::new(MultiProgress::new());

            // Create tasks for concurrent downloading
            let mut tasks = Vec::new();
            let chunks_clone = Arc::clone(&self.chunks);

            for i in 0..num_chunks {
                let chunks = Arc::clone(&chunks_clone);
                let file_clone = Arc::clone(&file);
                let url = self.url.clone();
                let multi_progress_clone = Arc::clone(&multi_progress);

                let task = tokio::spawn(async move {
                    // Get chunk info
                    let (start, end) = {
                        let chunks_guard = chunks.lock().await;
                        if i >= chunks_guard.len() {
                            return Err("Chunk index out of bounds".into());
                        }
                        (chunks_guard[i].start_byte, chunks_guard[i].end_byte)
                    };

                    // Create progress bar for this chunk
                    let chunk_size = end - start + 1;
                    let progress_bar = multi_progress_clone.add(ProgressBar::new(chunk_size));
                    progress_bar.set_style(ProgressStyle::with_template(
                        &format!("[Chunk {}] {{wide_bar:40.cyan/blue}} {{binary_bytes}}/{{binary_total_bytes}} ({{percent}}%)", i)
                    ).unwrap());

                    // Create a downloader instance for this chunk
                    let downloader = Downloader {
                        url,
                        headers: HeaderMap::new(),
                        file_size: None,
                        filename: None,
                        chunks: chunks,
                    };

                    // Download the chunk
                    downloader
                        .get_chunk(
                            Some((start, end)),
                            Some(progress_bar),
                            Some(file_clone),
                            Some(i),
                        )
                        .await
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

            println!(
                "Download completed! Total bytes: {}",
                HumanBytes(total_downloaded)
            );
        } else {
            let file_clone = Arc::clone(&file);
            let bar = ProgressBar::new_spinner();
            bar.enable_steady_tick(Duration::from_millis(100));
            println!("");
            bar.set_style(
                ProgressStyle::with_template(&format!(
                    "{{spinner:.cyan}} {:?} ({{binary_bytes}} downloaded)",
                    self.filename.as_ref().unwrap()
                ))
                .unwrap()
                // .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
                // set tick character as a moon's phase as progress indicator
                .tick_chars("🌑🌒🌓🌔🌕🌖🌗🌘"),
            );
            let _ = self
                .get_chunk(None, Some(bar), Some(file_clone), None)
                .await;
        }

        Ok(())
    }
}
