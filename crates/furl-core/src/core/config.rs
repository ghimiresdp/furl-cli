use std::path::PathBuf;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct DownloadConfig {
    pub download_dir: PathBuf,
    pub max_chunk_size: u64,
    pub threads: u8,
}

impl DownloadConfig {
    pub fn new() -> Self {
        Self {
            download_dir: PathBuf::from("."),
            max_chunk_size: 10 * 1024 * 1024,
            threads: 4,
        }
    }
    pub fn set_max_chunk_size(mut self, size: u64) -> Self {
        self.max_chunk_size = size;
        self
    }
    pub fn set_threads(mut self, threads: u8) -> Self {
        self.threads = threads;
        self
    }
    pub fn set_download_dir(mut self, dir: PathBuf) -> Self {
        self.download_dir = dir;
        self
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self::new()
    }
}
