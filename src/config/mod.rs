use crate::config::error::ConfigError;
use crate::config::types::{
    Config, DiscordBotConfig, LastFMConfig, LoggerConfig, MaxMindConfig, TranslatorConfig,
    UmamiConfig, WebserverConfig, WolframAlphaConfig,
};
use std::env;
use std::time::Instant;
use tokio::fs;
use tracing::{error, info, warn};

pub mod error;
pub mod types;

const DEFAULT_CONFIG_URL: &str =
    "https://git.kybe.xyz/2kybe3/kybe-backend/src/branch/main/config/config.toml.example";

impl Config {
    pub async fn init() -> Result<Self, ConfigError> {
        let args: Vec<String> = env::args().collect();
        if args.iter().any(|arg| arg == "--generate-example") {
            let time = Instant::now();
            info!("Generating config/config.toml.example");
            Self::create_local_default().await?;
            info!(
                "Generated config/config.toml.example in {} MS",
                time.elapsed().as_millis()
            );
            std::process::exit(0)
        }

        match Self::load_or_create().await {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                error!("Failed to load config: {e}");
                std::process::exit(1);
            }
        }
    }

    pub async fn load_or_create() -> Result<Self, ConfigError> {
        let start = Instant::now();
        match Self::load().await {
            Ok(cfg) => {
                info!("config loaded in {} ms", start.elapsed().as_millis());
                Ok(cfg)
            }

            Err(ConfigError::ReadFile(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("creating default config/config.toml");
                Self::create_default().await?;
                info!("config created in {} ms", start.elapsed().as_millis());
                if let Err(e) = Self::load().await {
                    warn!(
                        "default config failed to load! Please open a issue https://github.com/2kybe3/kybe-backend/issues\n Error: {:?}",
                        e
                    );
                    std::process::exit(1);
                } else {
                    info!("Default config/config.toml created! Please edit");
                    std::process::exit(0);
                }
            }

            Err(e) => Err(e),
        }
    }

    pub async fn load() -> Result<Self, ConfigError> {
        let path = env::current_dir()
            .map_err(ConfigError::CurrentDir)?
            .join("config/config.toml");
        let contents = fs::read_to_string(&path)
            .await
            .map_err(ConfigError::ReadFile)?;
        Ok(toml::from_str(&contents)?)
    }
    pub async fn create_default() -> Result<(), ConfigError> {
        let path = env::current_dir()
            .map_err(ConfigError::CurrentDir)?
            .join("config/config.toml");

        let resp = reqwest::get(DEFAULT_CONFIG_URL).await?;
        let content = resp.text().await?;

        fs::create_dir_all("config")
            .await
            .map_err(ConfigError::CreateDir)?;
        fs::write(path, content)
            .await
            .map_err(ConfigError::WriteFile)?;
        Ok(())
    }

    pub async fn create_local_default() -> Result<(), ConfigError> {
        let path = env::current_dir()
            .map_err(ConfigError::CurrentDir)?
            .join("config/config.toml.example");

        let default_config = Config::default();
        let content = toml::to_string_pretty(&default_config).map_err(ConfigError::Serialize)?;

        toml::from_str::<Config>(&content)?;

        fs::create_dir_all("config")
            .await
            .map_err(ConfigError::CreateDir)?;
        fs::write(path, content)
            .await
            .map_err(ConfigError::WriteFile)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            wolfram_alpha: WolframAlphaConfig {
                enabled: false,
                token: Some("APP_ID".into()),
            },
            discord_bot: DiscordBotConfig {
                enable: false,
                token: "DISCORD_TOKEN".into(),
                admin_id: "921066050009833572".into(),
                translator: TranslatorConfig {
                    enabled: false,
                    url: Some("https://translate.kybe.xyz".into()),
                    token: Some("".into()),
                },
            },
            maxmind: MaxMindConfig {
                city_enable: false,
                city_db_check: false,
                city: "./config/GeoLite2-City.mmdb".into(),
                asn_enable: false,
                asn_db_check: false,
                asn: "./config/GeoLite2-ASN.mmdb".into(),
            },
            lastfm: LastFMConfig {
                enable: false,
                token: Some("".into()),
                username: Some("".into()),
                interval_secs: Some(10),
            },
            logger: LoggerConfig {
                file_logger_enabled: true,
            },
            webserver: WebserverConfig {
                behind_proxy: false,
                proxy_ip: Some("10.0.4.2".into()),
                api_token: "CHANGE_ME".into(),
                proxy_header: Some("X-Forwarded-For".into()),
                behind_i2p: false,
                i2p_ip: Some("".into()),
                i2p_header: Some("X-I2P-DestHash".into()),
                umami: UmamiConfig {
                    script_path: Some("https://kybe.xyz/script.js".into()),
                    id: Some("umami-id".into()),
                },
            },
        }
    }
}
