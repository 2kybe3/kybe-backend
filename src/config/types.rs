use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub notification: NotificationConfig,
}

#[derive(Deserialize)]
pub struct NotificationConfig {
    pub log: LogConfig,
    pub gotify: GotifyConfig,
    pub discord: DiscordConfig,
}

#[derive(Deserialize)]
pub struct LogConfig {
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct GotifyConfig {
    pub enabled: bool,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct DiscordConfig {
    pub enabled: bool,
    pub url: Option<String>,
}