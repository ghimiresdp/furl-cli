use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufWriter, Error, ErrorKind},
    path::PathBuf,
};
// use toml::Table;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn get_config_dir() -> Option<PathBuf> {
        Some(dirs::config_dir()?.join(format!(
            "furl-cli/{}",
            if cfg!(debug_assertions) { "dev" } else { "" }
        )))
    }

    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Self {
            ..FurlConfig::default()
        };
        if let Some(config_dir) = Self::get_config_dir() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(config)
    }
    pub fn get_db_path(&self) -> Option<PathBuf> {
        let path = Self::get_config_dir()?.join("db.json");
        Some(path)
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(path) = Self::get_config_dir() {
            let config_path = path.join("config.json");
            if let Ok(config) = std::fs::read_to_string(config_path.clone()) {
                let config = serde_json5::from_str::<FurlConfig>(&config)?;
                return Ok(config);
            } else {
                let config = FurlConfig::default();
                config.save()?;
                return Ok(config);
            }
        }
        return Err(Error::new(ErrorKind::NotFound, "Config directory not found").into());
    }
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::get_config_dir() {
            let file = File::create(path.join("config.json"))?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, self)?;
            return Ok(());
        }
        return Err(Error::new(ErrorKind::NotFound, "Config directory not found").into());
    }
}
