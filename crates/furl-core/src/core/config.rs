#[derive(Clone)]
pub struct DownloadConfig {
    pub max_chunk_size: u64,
}

impl DownloadConfig {
    pub fn new() -> Self {
        Self {
            max_chunk_size: 10 * 1024 * 1024, // 10MB
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
