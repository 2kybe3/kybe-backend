use crate::config::error::ConfigError;
use crate::config::types::{
	Config, DatabaseConfig, DiscordBotConfig, DiscordConfig, EmailConfig, GotifyConfig,
	LastFMConfig, LoggerConfig, MaxMindConfig, NotificationConfig, TranslatorConfig, UmamiConfig,
	WebserverConfig,
};
use std::env;
use std::time::Instant;
use tokio::fs;
use tracing::{error, info, warn};

pub mod error;
pub mod types;

const DEFAULT_CONFIG_URL: &str =
	"https://raw.githubusercontent.com/2kybe3/kybe-backend/refs/heads/main/config.toml.example";

impl Config {
	pub async fn init() -> Result<Self, ConfigError> {
		let args: Vec<String> = env::args().collect();
		if args.iter().any(|arg| arg == "--generate-example") {
			let time = Instant::now();
			info!("Generating config.toml.example");
			Self::create_local_default().await?;
			info!(
				"Generated config.toml.example in {} MS",
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
				info!("creating default config.toml");
				Self::create_default().await?;
				info!("config created in {} ms", start.elapsed().as_millis());
				if let Err(e) = Self::load().await {
					warn!(
						"default config failed to load! Please open a issue https://github.com/2kybe3/kybe-backend/issues\n Error: {:?}",
						e
					);
					std::process::exit(1);
				} else {
					info!("Default config.toml created! Please edit");
					std::process::exit(0);
				}
			}

			Err(e) => Err(e),
		}
	}

	pub async fn load_env_overrides(mut cfg: Config) -> Config {
		macro_rules! env_override_str {
			($field:expr, $env_name:literal) => {
				if let Ok(val) = env::var($env_name) {
					tracing::info!("Overriding {} from environment", $env_name);
					$field = val;
				}
			};
		}

		env_override_str!(cfg.database.postgres_url, "KYBE_DATABASE_POSTGRES_URL");

		cfg
	}

	pub async fn load() -> Result<Self, ConfigError> {
		let path = env::current_dir()
			.map_err(ConfigError::CurrentDir)?
			.join("config.toml");
		let contents = fs::read_to_string(&path)
			.await
			.map_err(ConfigError::ReadFile)?;
		let res = toml::from_str(&contents)?;
		Ok(Self::load_env_overrides(res).await)
	}
	pub async fn create_default() -> Result<(), ConfigError> {
		let path = env::current_dir()
			.map_err(ConfigError::CurrentDir)?
			.join("config.toml");

		let resp = reqwest::get(DEFAULT_CONFIG_URL).await?;
		let content = resp.text().await?;

		fs::write(path, content)
			.await
			.map_err(ConfigError::WriteFile)?;
		Ok(())
	}

	pub async fn create_local_default() -> Result<(), ConfigError> {
		let path = env::current_dir()
			.map_err(ConfigError::CurrentDir)?
			.join("config.toml.example");

		let default_config = Config::default();
		let content = toml::to_string_pretty(&default_config).map_err(ConfigError::Serialize)?;

		toml::from_str::<Config>(&content)?;

		fs::write(path, content)
			.await
			.map_err(ConfigError::WriteFile)?;
		Ok(())
	}
}

impl Default for Config {
	fn default() -> Self {
		Config {
			notification: NotificationConfig {
				discord: DiscordConfig {
					enabled: false,
					url: Some("https://discord.com/api/webhooks/.../...".into()),
				},
				gotify: GotifyConfig {
					enabled: false,
					url: Some("https://gotify.kybe.xyz/message?token=<token>".into()),
				},
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
			database: DatabaseConfig {
				postgres_url: "postgres://postgres:password@localhost/test".into(),
				max_connections: 5,
			},
			maxmind: MaxMindConfig {
				city_enable: false,
				city: "".into(),
				asn_enable: false,
				asn: "".into(),
			},
			lastfm: LastFMConfig {
				enable: false,
				token: Some("".into()),
				username: Some("".into()),
				interval_secs: Some(10),
			},
			logger: LoggerConfig {
				file_logger_enabled: true,
				file_logger_path: Some("./log".into()),
			},
			webserver: WebserverConfig {
				behind_proxy: false,
				trust_proxy_header: Some("X-Forwarded-For".into()),
				umami: UmamiConfig {
					script_path: Some("https://kybe.xyz/script.js".into()),
					id: Some("umami-id".into()),
				},
			},
			email: EmailConfig {
				enable: false,
				server: "mail.kybe.xyz".into(),
				port: 587,
				email: "system@kybe.xyz".into(),
				password: "".into(),
			},
		}
	}
}
