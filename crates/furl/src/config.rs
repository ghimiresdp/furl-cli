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
/// needed. Failures are non-fatal; a warning is printed and the caller
/// proceeds with the in-memory config regardless.
fn save_config(path: &PathBuf, config: &DownloadConfig) {
    if let Some(parent) = path.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        eprintln!(
            "Warning: could not create config directory {}: {err}",
            parent.display()
        );
        return;
    }

    if let Err(err) = fs::write(path, format_config(config)) {
        eprintln!(
            "Warning: could not write config file at {}: {err}",
            path.display()
        );
    }
}

/// Renders a `DownloadConfig` as TOML, in the same shape a user could paste
/// into their config file.
pub fn format_config(config: &DownloadConfig) -> String {
    toml_edit::ser::to_string_pretty(config)
        .unwrap_or_else(|err| format!("# error serializing config: {err}\n"))
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
}
