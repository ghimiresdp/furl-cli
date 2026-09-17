use std::fs;
use std::path::PathBuf;

use furl_core::DownloadConfig;

/// Path to the user's furl config file, e.g. `~/.config/.furl/config.toml` on
/// Linux. Returns `None` if the platform exposes no config directory.
fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(".furl").join("config.toml"))
}

/// Loads `DownloadConfig` from the user's config file. Any value missing
/// from the file, an unreadable/missing file, or a platform with no config
/// directory all fall back to `DownloadConfig::default()`. On first run
/// (no config file yet), the defaults are also written to disk so the file
/// exists for the user to edit afterwards.
pub fn load_config() -> DownloadConfig {
    let Some(path) = config_file_path() else {
        return DownloadConfig::default();
    };

    load_config_from(&path)
}

fn load_config_from(path: &PathBuf) -> DownloadConfig {
    if !path.exists() {
        let config = DownloadConfig::default();
        save_config(path, &config);
        return config;
    }

    let Ok(contents) = fs::read_to_string(path) else {
        return DownloadConfig::default();
    };

    toml_edit::de::from_str(&contents).unwrap_or_else(|err| {
        eprintln!(
            "Warning: could not parse config file at {}: {err}. Using defaults.",
            path.display()
        );
        DownloadConfig::default()
    })
}

/// Writes `config` to `path` as TOML, creating the parent directory if
/// needed.
fn write_config(path: &PathBuf, config: &DownloadConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format_config(config))
}

/// Writes `config` to `path`. Failures are non-fatal here: a warning is
/// printed and the caller proceeds with the in-memory config regardless.
/// Used for the first-run auto-save in `load_config_from`.
fn save_config(path: &PathBuf, config: &DownloadConfig) {
    if let Err(err) = write_config(path, config) {
        eprintln!(
            "Warning: could not write config file at {}: {err}",
            path.display()
        );
    }
}

/// Writes `config` to the user's config file, for use by `furl config <key>
/// <value>` where a failure to save should be reported to the user.
pub fn persist(config: &DownloadConfig) -> Result<(), String> {
    let path =
        config_file_path().ok_or_else(|| "no config directory on this platform".to_string())?;

    write_config(&path, config)
        .map_err(|err| format!("could not write config file at {}: {err}", path.display()))
}

/// Renders a `DownloadConfig` as TOML, in the same shape a user could paste
/// into their config file.
pub fn format_config(config: &DownloadConfig) -> String {
    toml_edit::ser::to_string_pretty(config)
        .unwrap_or_else(|err| format!("# error serializing config: {err}\n"))
}

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
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A config.toml path under a fresh temp directory, unique per call so
    /// tests running in parallel never collide.
    fn temp_config_path() -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "furl-config-test-{}-{nanos}-{n}",
            std::process::id()
        ));
        dir.join("config.toml")
    }

    #[test]
    fn load_config_from_creates_file_with_defaults_on_first_run() {
        let path = temp_config_path();
        assert!(!path.exists());

        let config = load_config_from(&path);

        assert_eq!(config, DownloadConfig::default());
        assert!(path.exists());
        assert_eq!(
            toml_edit::de::from_str::<DownloadConfig>(&fs::read_to_string(&path).unwrap())
                .unwrap(),
            DownloadConfig::default()
        );

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn load_config_from_reads_existing_file() {
        let path = temp_config_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "threads = 16\n").unwrap();

        let config = load_config_from(&path);

        assert_eq!(config.threads, 16);
        assert_eq!(
            config.max_chunk_size,
            DownloadConfig::default().max_chunk_size
        );

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn load_config_from_falls_back_to_defaults_on_invalid_toml() {
        let path = temp_config_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "this is not valid toml ===").unwrap();

        let config = load_config_from(&path);

        assert_eq!(config, DownloadConfig::default());
        // the invalid file is left untouched, not overwritten with defaults
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "this is not valid toml ==="
        );

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn format_config_round_trips_through_parsing() {
        let config = DownloadConfig::new().set_threads(16).set_max_chunk_size(1);

        let formatted = format_config(&config);
        let parsed: DownloadConfig = toml_edit::de::from_str(&formatted).unwrap();

        assert_eq!(parsed, config);
    }

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
    fn persist_writes_to_the_resolved_config_path() {
        let path = temp_config_path();
        // exercise the real write path directly, since `persist` always
        // resolves the path via `dirs::config_dir()`
        let config = DownloadConfig::default().set_threads(42);

        write_config(&path, &config).unwrap();

        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("threads = 42"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
