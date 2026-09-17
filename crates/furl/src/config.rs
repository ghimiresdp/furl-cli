use std::fs;
use std::path::PathBuf;

use furl_core::DownloadConfig;

/// Path to the user's furl config file, e.g. `~/.config/furl/config.toml` on
/// Linux. Returns `None` if the platform exposes no config directory.
fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("furl").join("config.toml"))
}

/// Loads `DownloadConfig` from the user's config file. Any value missing
/// from the file, an unreadable/missing file, or a platform with no config
/// directory all fall back to `DownloadConfig::default()`.
pub fn load_config() -> DownloadConfig {
    let Some(path) = config_file_path() else {
        return DownloadConfig::default();
    };

    let Ok(contents) = fs::read_to_string(&path) else {
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

/// Renders a `DownloadConfig` as TOML, in the same shape a user could paste
/// into their config file.
pub fn format_config(config: &DownloadConfig) -> String {
    toml_edit::ser::to_string_pretty(config)
        .unwrap_or_else(|err| format!("# error serializing config: {err}\n"))
}
