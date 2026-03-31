use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
	pub notification: NotificationConfig,
	pub discord_bot: DiscordBotConfig,
	pub webserver: WebserverConfig,
	pub database: DatabaseConfig,
	pub maxmind: MaxMindConfig,
	pub lastfm: LastFMConfig,
	pub logger: LoggerConfig,
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
	pub enable: bool,
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
	pub max_connections: u32,
}

// Max Mind (https://www.maxmind.com/en/geoip-databases)
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MaxMindConfig {
	// Use City DB and Path to it
	pub city_enable: bool,
	pub city_db_check: bool,
	pub city: String,
	// Use ASN DB and Path to it
	pub asn_enable: bool,
	pub asn_db_check: bool,
	pub asn: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LastFMConfig {
	pub enable: bool,
	pub token: Option<String>,
	pub username: Option<String>,
	pub interval_secs: Option<u8>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LoggerConfig {
	pub file_logger_enabled: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebserverConfig {
	pub behind_proxy: bool,
	pub proxy_ip: Option<String>,
	pub api_token: String,
	pub proxy_header: Option<String>,
	pub behind_i2p: bool,
	pub i2p_ip: Option<String>,
	pub i2p_header: Option<String>,
	pub umami: UmamiConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UmamiConfig {
	pub script_path: Option<String>,
	pub id: Option<String>,
}
