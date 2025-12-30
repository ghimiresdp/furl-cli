use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Default)]
pub struct FurlConfig {
    config_dir: PathBuf,
}

pub struct DownloadMetadata {}

impl FurlConfig {
    pub fn init() -> Result<Self, std::io::Error> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find config directory.",
                )
            })?
            .join(format!(
                "furl-cli/{}",
                if cfg!(debug_assertions) { "dev" } else { "" }
            ));
        fs::create_dir_all(&config_dir)?;

        Ok(Self { config_dir })
    }
    pub fn get_db_path(&self) -> PathBuf {
        self.config_dir.join("db.json")
    }
    // TODO: ADD CUSTOM CONFIG IN THE FUTURE
    // pub fn get_config_file(&self) -> PathBuf {
    //     self.config_dir.join("config.toml")
    // }
}
