use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct FurlConfig {
    pub download_dir: PathBuf,
    pub threads: u8,
}

pub struct DownloadMetadata {}

impl Default for FurlConfig {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::from("."),
            threads: 8,
        }
    }
}

impl FurlConfig {
    /// get config directory. if not found, return None.
    ///
    /// Generally, it should return config directory for the respective OS,
    /// however, in case it is not found, or unsupported OS, it will simply
    /// return None.
    ///
    pub fn get_config_dir(&self) -> Option<PathBuf> {
        Some(dirs::config_dir()?.join(format!(
            "furl-cli/{}",
            if cfg!(debug_assertions) { "dev" } else { "" }
        )))
    }

    pub fn init() -> Result<Self, std::io::Error> {
        let config = Self {
            ..FurlConfig::default()
        };
        if let Some(config_dir) = config.get_config_dir() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(config)
    }
    pub fn get_db_path(&self) -> Option<PathBuf> {
        let path = self.get_config_dir()?.join("db.json");
        Some(path)
    }

    pub fn get_config_file(&self) -> Option<PathBuf> {
        let path = self.get_config_dir()?.join("config.toml");
        Some(path)
    }
}
