use std::path::PathBuf;

use furl_core::DownloadConfig;

/// Reads a single configuration value by its TOML key name (`download_dir`,
/// `max_chunk_size`, or `threads`). Returns `None` if `key` isn't recognized.
pub fn get(config: &DownloadConfig, key: &str) -> Option<String> {
    match key {
        "download_dir" => Some(config.download_dir.display().to_string()),
        "max_chunk_size" => Some(config.max_chunk_size.to_string()),
        "threads" => Some(config.threads.to_string()),
        _ => None,
    }
}

/// Returns `config` with `key` set to `value`. Errors if `key` isn't
/// recognized or `value` doesn't parse into that field's type.
pub fn set(config: DownloadConfig, key: &str, value: &str) -> Result<DownloadConfig, String> {
    match key {
        "download_dir" => Ok(config.set_download_dir(PathBuf::from(value))),
        "max_chunk_size" => value
            .parse()
            .map(|v| config.set_max_chunk_size(v))
            .map_err(|err| format!("invalid value for max_chunk_size: {err}")),
        "threads" => value
            .parse()
            .map(|v| config.set_threads(v))
            .map_err(|err| format!("invalid value for threads: {err}")),
        _ => Err(format!("unknown configuration key '{key}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_reads_known_keys() {
        let config = DownloadConfig::new().set_threads(16);

        assert_eq!(get(&config, "threads"), Some("16".to_string()));
        assert_eq!(get(&config, "max_chunk_size"), Some("10485760".to_string()));
        assert_eq!(get(&config, "download_dir"), Some(".".to_string()));
    }

    #[test]
    fn get_returns_none_for_unknown_key() {
        let config = DownloadConfig::default();

        assert_eq!(get(&config, "not_a_real_key"), None);
    }

    #[test]
    fn set_updates_known_keys() {
        let config = DownloadConfig::default();

        let config = set(config, "threads", "32").unwrap();
        assert_eq!(config.threads, 32);

        let config = set(config, "download_dir", "/tmp/downloads").unwrap();
        assert_eq!(config.download_dir, PathBuf::from("/tmp/downloads"));

        let config = set(config, "max_chunk_size", "2048").unwrap();
        assert_eq!(config.max_chunk_size, 2048);
    }

    #[test]
    fn set_rejects_unknown_key_and_bad_values() {
        assert!(set(DownloadConfig::default(), "not_a_real_key", "1").is_err());
        assert!(set(DownloadConfig::default(), "threads", "not-a-number").is_err());
        assert!(set(DownloadConfig::default(), "threads", "9999").is_err());
    }
}
