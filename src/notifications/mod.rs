use crate::config::types::NotificationConfig;
use crate::notifications::providers::discord::DiscordNotifier;
use futures::future::join_all;
use providers::gotify::GotifyNotifier;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;

mod providers;

#[derive(Debug, Error)]
pub enum NotificationError {
	#[error("Invalid config: {0}")]
	InvalidConfig(String),

	#[error("Transport error: {0}")]
	Transport(String),

	#[error("Auth error: {0}")]
	Auth(String),
}

#[async_trait::async_trait]
pub trait Notifier: Send + Sync + Debug {
	/// Sends a notification to the provider
	async fn send(&self, notification: &Notification) -> Result<(), NotificationError>;

	/// Name to identify the Notifier
	fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Notification {
	pub title: String,
	pub message: String,
}

impl Notification {
	pub fn new<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
		Notification {
			title: title.into(),
			message: message.into(),
		}
	}
}

#[derive(Debug)]
pub struct Notifications {
	notifiers: Vec<Arc<dyn Notifier + Send + Sync>>,
}

impl Notifications {
	pub fn new(config: &NotificationConfig) -> Self {
		let mut notifiers: Vec<Arc<dyn Notifier + Send + Sync>> = Vec::new();

		if config.gotify.enabled {
			if let Some(url) = config.gotify.url.clone() {
				notifiers.push(Arc::new(GotifyNotifier::new(url)));
			} else {
				error!("Gotify enabled but missing URL, skipping GotifyNotifier");
			}
		}

		if config.discord.enabled {
			if let Some(url) = config.discord.url.clone() {
				notifiers.push(Arc::new(DiscordNotifier::new(url)));
			} else {
				error!("Discord enabled but missing URL, skipping DiscordNotifier");
			}
		}

		Self { notifiers }
	}

	pub async fn notify<S: Into<Notification>>(&self, notification: S, wait: bool) {
		let notification = notification.into();
		let notifiers = self.notifiers.clone();

		let futures = notifiers.into_iter().map(move |notifier| {
			let notification = notification.clone();
			async move {
				if let Err(err) = notifier.send(&notification).await {
					error!(
						"Failed to send notification via {}: {:?}",
						notifier.name(),
						err
					);
				}
			}
		});

		if wait {
			join_all(futures).await;
		} else {
			tokio::spawn(async move {
				join_all(futures).await;
			});
		}
	}
}
