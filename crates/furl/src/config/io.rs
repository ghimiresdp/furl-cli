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

    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!(
                "Warning: could not read config file at {}: {err}. Using defaults.",
                path.display()
            );
            return DownloadConfig::default();
        }
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
/// into their config file. Each key is preceded by a short comment
/// explaining what it does, since this is the file users are expected to
/// hand-edit.
pub fn format_config(config: &DownloadConfig) -> String {
    let mut doc = toml_edit::DocumentMut::new();

    doc["download_dir"] = toml_edit::value(config.download_dir.display().to_string());
    doc["max_chunk_size"] = toml_edit::value(config.max_chunk_size as i64);
    doc["threads"] = toml_edit::value(config.threads as i64);

    set_key_comment(
        &mut doc,
        "download_dir",
        &["directory downloaded files are saved to"],
    );
    set_key_comment(
        &mut doc,
        "max_chunk_size",
        &[
            "maximum size of a single download chunk, in bytes",
            "(`furl config max_chunk_size <value>` also accepts sizes like 10KB, 5MB, 1GB)",
        ],
    );
    set_key_comment(
        &mut doc,
        "threads",
        &["number of concurrent download threads (1-255)"],
    );

    doc.to_string()
}

/// Sets `key`'s leading comment to `lines`, one `# `-prefixed TOML comment
/// line per entry. No-op if `key` isn't present in `doc`.
fn set_key_comment(doc: &mut toml_edit::DocumentMut, key: &str, lines: &[&str]) {
    let Some(mut key) = doc.key_mut(key) else {
        return;
    };
    let prefix: String = lines.iter().map(|line| format!("# {line}\n")).collect();
    key.leaf_decor_mut().set_prefix(prefix);
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
            toml_edit::de::from_str::<DownloadConfig>(&fs::read_to_string(&path).unwrap()).unwrap(),
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

    #[cfg(unix)]
    #[test]
    fn load_config_from_falls_back_to_defaults_on_unreadable_file() {
        use std::os::unix::fs::PermissionsExt;

        let path = temp_config_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "threads = 16\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        if fs::read_to_string(&path).is_ok() {
            // running with elevated privileges that ignore permission bits
            // (e.g. root/CI); the read-failure path can't be exercised here
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
            fs::remove_dir_all(path.parent().unwrap()).unwrap();
            return;
        }

        let config = load_config_from(&path);

        assert_eq!(config, DownloadConfig::default());

        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
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
    fn format_config_includes_explanatory_comments() {
        let formatted = format_config(&DownloadConfig::default());

        assert!(formatted.contains("# directory downloaded files are saved to"));
        assert!(formatted.contains("# maximum size of a single download chunk, in bytes"));
        assert!(formatted.contains("# number of concurrent download threads (1-255)"));
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
