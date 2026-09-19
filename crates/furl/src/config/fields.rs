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
        "max_chunk_size" => parse_size(value)
            .map(|v| config.set_max_chunk_size(v))
            .map_err(|err| format!("invalid value for max_chunk_size: {err}")),
        "threads" => value
            .parse()
            .map(|v| config.set_threads(v))
            .map_err(|err| format!("invalid value for threads: {err}")),
        _ => Err(format!("unknown configuration key '{key}'")),
    }
}

/// Parses a human-readable byte size such as `10485760`, `10KB`, `5.5MB`, or
/// `1 GB` into a raw byte count. A bare number with no unit is treated as an
/// exact byte count. Units are case-insensitive and use binary multiples (1
/// KB = 1024 bytes). Shared by `furl config max_chunk_size` and the
/// `--chunksize` CLI flag, so both interpret sizes the same way.
pub(crate) fn parse_size(input: &str) -> Result<u64, String> {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let input = input.trim();
    let split_at = input
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(input.len());
    let (number, unit) = input.split_at(split_at);

    let number: f64 = number
        .parse()
        .map_err(|_| format!("'{input}' is not a valid size"))?;

    let multiplier = match unit.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1.0,
        "K" | "KB" => KB,
        "M" | "MB" => MB,
        "G" | "GB" => GB,
        "T" | "TB" => TB,
        other => {
            return Err(format!(
                "unknown size unit '{other}', expected one of B, KB, MB, GB, TB"
            ));
        }
    };

    let bytes = number * multiplier;
    if bytes < 1.0 {
        return Err("size must be at least 1 byte".to_string());
    }

    Ok(bytes.round() as u64)
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

    #[test]
    fn set_max_chunk_size_accepts_human_readable_units() {
        let config = DownloadConfig::default();

        let config = set(config, "max_chunk_size", "512B").unwrap();
        assert_eq!(config.max_chunk_size, 512);

        let config = set(config, "max_chunk_size", "10KB").unwrap();
        assert_eq!(config.max_chunk_size, 10 * 1024);

        let config = set(config, "max_chunk_size", "5mb").unwrap();
        assert_eq!(config.max_chunk_size, 5 * 1024 * 1024);

        let config = set(config, "max_chunk_size", "1.5 MB").unwrap();
        assert_eq!(config.max_chunk_size, (1.5 * 1024.0 * 1024.0) as u64);

        let config = set(config, "max_chunk_size", "1GB").unwrap();
        assert_eq!(config.max_chunk_size, 1024 * 1024 * 1024);

        let config = set(config, "max_chunk_size", "1TB").unwrap();
        assert_eq!(config.max_chunk_size, 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn set_max_chunk_size_rejects_invalid_sizes() {
        assert!(set(DownloadConfig::default(), "max_chunk_size", "0").is_err());
        assert!(set(DownloadConfig::default(), "max_chunk_size", "0MB").is_err());
        assert!(set(DownloadConfig::default(), "max_chunk_size", "-5MB").is_err());
        assert!(set(DownloadConfig::default(), "max_chunk_size", "abc").is_err());
        assert!(set(DownloadConfig::default(), "max_chunk_size", "10XB").is_err());
    }
}
