use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
	#[error("failed to get current directory")]
	CurrentDir(#[source] std::io::Error),

	#[error("failed to read config.toml")]
	ReadFile(#[source] std::io::Error),

	#[error("failed to write default config.toml")]
	WriteFile(#[source] std::io::Error),

	#[error("invalid config.toml")]
	Parse(#[from] toml::de::Error),

	#[error("failed to download default config")]
	Download(#[from] reqwest::Error),

	#[error("failed to serialize config")]
	Serialize(#[from] toml::ser::Error),
}
