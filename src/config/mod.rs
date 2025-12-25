use std::env;
use tokio::fs;
use crate::config::error::ConfigError;
use crate::config::types::Config;

pub mod types;
pub mod error;

const DEFAULT_CONFIG_URL: &str =
    "https://raw.githubusercontent.com/2kybe3/kybe-backend/refs/heads/main/config.toml.example";

impl Config {
    pub async fn init() -> Result<Self, ConfigError> {
        match Self::load().await {
            Ok(cfg) => Ok(cfg),

            Err(ConfigError::ReadFile(e))
                if e.kind() == std::io::ErrorKind::NotFound => {
                Self::create_default().await?;
                Self::load().await
            }

            Err(e) => Err(e)
        }
    }

    pub async fn load() -> Result<Self, ConfigError> {
        let path = env::current_dir().map_err(ConfigError::CurrentDir)?.join("config.toml");
        let contents = fs::read_to_string(&path).await.map_err(ConfigError::ReadFile)?;
        Ok(toml::from_str(&contents)?)
    }

    pub async fn create_default() -> Result<(), ConfigError> {
        let path = env::current_dir().map_err(ConfigError::CurrentDir)?.join("config.toml");

        let resp = reqwest::get(DEFAULT_CONFIG_URL).await?;
        let content = resp.text().await?;

        toml::from_str::<Config>(&content)?;

        fs::write(path, content).await.map_err(ConfigError::WriteFile)?;
        Ok(())
    }
}