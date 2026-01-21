use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
	#[error("failed to get current directory: {0}")]
	CurrentDir(#[source] std::io::Error),

	#[error("failed to read config.toml: {0}")]
	ReadFile(#[source] std::io::Error),

	#[error("failed to write default config.toml: {0}")]
	WriteFile(#[source] std::io::Error),

	#[error("invalid config.toml: {0}")]
	Parse(#[from] toml::de::Error),

	#[error("failed to download default config: {0}")]
	Download(#[from] reqwest::Error),

	#[error("failed to serialize config: {0}")]
	Serialize(#[from] toml::ser::Error),
}
