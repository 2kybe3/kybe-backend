use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub notification: NotificationConfig,
}

#[derive(Deserialize, Serialize)]
pub struct NotificationConfig {
    pub log: LogConfig,
    pub gotify: GotifyConfig,
    pub discord: DiscordConfig,
}

#[derive(Deserialize, Serialize)]
pub struct LogConfig {
    pub enabled: bool,
}

#[derive(Deserialize, Serialize)]
pub struct GotifyConfig {
    pub enabled: bool,
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub url: Option<String>,
}
