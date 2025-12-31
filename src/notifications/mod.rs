use crate::config::types::NotificationConfig;
use crate::notifications::error::NotificationError;
use crate::notifications::log::LogNotifier;
use crate::notifications::providers::discord::DiscordNotifier;
use futures::future::join_all;
use providers::gotify::GotifyNotifier;
use std::fmt::Debug;
use tracing::error;

pub mod error;
mod log;
mod providers;

#[async_trait::async_trait]
pub trait Notifier: Send + Sync + Debug {
	async fn send(&self, notification: &Notification) -> Result<(), NotificationError>;

	fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Notification {
	pub title: String,
	pub message: String,
}

impl Notification {
	pub fn new<S: Into<String>>(title: S, message: S) -> Self {
		Notification {
			title: title.into(),
			message: message.into(),
		}
	}
}

#[derive(Debug)]
pub struct Notifications {
	notifiers: Vec<Box<dyn Notifier + Send + Sync>>,
}

impl Notifications {
	pub fn new(config: &NotificationConfig) -> Self {
		let mut notifiers: Vec<Box<dyn Notifier + Send + Sync>> = Vec::new();

		if config.log.enabled {
			notifiers.push(Box::new(LogNotifier::new()));
		}

		if config.gotify.enabled {
			if let Some(url) = config.gotify.url.clone() {
				notifiers.push(Box::new(GotifyNotifier::new(url)));
			} else {
				error!("Gotify enabled but missing URL, skipping GotifyNotifier");
			}
		}

		if config.discord.enabled {
			if let Some(url) = config.discord.url.clone() {
				notifiers.push(Box::new(DiscordNotifier::new(url)));
			} else {
				error!("Discord enabled but missing URL, skipping DiscordNotifier");
			}
		}

		Self { notifiers }
	}

	pub async fn notify<S: Into<Notification>>(&self, notification: S) {
		let notification = notification.into();
		let futures = self.notifiers.iter().map(|notifier| {
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

		join_all(futures).await;
	}
}
