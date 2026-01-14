use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
	pub notification: NotificationConfig,
	pub discord_bot: DiscordBotConfig,
	pub webserver: WebserverConfig,
	pub database: DatabaseConfig,
	pub logger: LoggerConfig,
	pub email: EmailConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotificationConfig {
	pub gotify: GotifyConfig,
	pub discord: DiscordConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GotifyConfig {
	pub enabled: bool,
	pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DiscordConfig {
	pub enabled: bool,
	pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DiscordBotConfig {
	pub token: String,
	pub translator: TranslatorConfig,
	pub admin_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TranslatorConfig {
	pub enabled: bool,
	pub url: Option<String>,
	pub token: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatabaseConfig {
	pub postgres_url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LoggerConfig {
	pub file_logger_enabled: bool,
	pub file_logger_path: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebserverConfig {
	pub behind_proxy: bool,
	pub trust_proxy_header: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EmailConfig {
	pub email: String,
	pub password: String,
	pub server: String,
	pub port: u16,
}
