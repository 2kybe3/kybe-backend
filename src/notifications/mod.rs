use futures::future::join_all;
use tracing::error;
use crate::notifications::error::NotificationError;
use providers::gotify::GotifyNotifier;
use crate::config::types::NotificationConfig;
use crate::notifications::log::LogNotifier;
use crate::notifications::providers::discord::DiscordNotifier;

pub mod error;
mod log;
mod providers;

#[async_trait::async_trait]
pub trait Notifier {
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError>;

    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: u8,
}

impl Notification {
    pub fn new(title: String, message: String, priority: u8) -> Self {
        Notification {
            title,
            message,
            priority
        }
    }
}

pub struct Notifications {
    notifiers: Vec<Box<dyn Notifier>>
}

impl Notifications {
    pub fn new(config: &NotificationConfig) -> Self {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

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

    pub async fn notify(&self, notification: Notification) {
        let futures = self.notifiers.iter().map(|notifier| {
            let notification = notification.clone();
            async move {
                if let Err(err) = notifier.send(&notification).await {
                    error!("Failed to send notification via {}: {:?}", notifier.name(), err);
                }
            }
        });

        join_all(futures).await;
    }
}
