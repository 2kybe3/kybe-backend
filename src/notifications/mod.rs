use futures::future::join_all;
use tracing::error;
use crate::notifications::error::NotificationError;
use providers::gotify::GotifyNotifier;
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

#[derive(Debug)]
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

pub struct NotificationsConfig {
    pub log_enabled: bool,
    pub gotify_enabled: bool,
    pub gotify_url: String,
    pub gotify_token: String,
    pub discord_enabled: bool,
    pub discord_url: String,
}

impl Notifications {
    pub fn new(config: NotificationsConfig) -> Self {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

        if config.log_enabled {
            notifiers.push(Box::new(LogNotifier::new()));
        }

        if config.gotify_enabled {
            let gotify = GotifyNotifier::new(config.gotify_url, config.gotify_token);
            notifiers.push(Box::new(gotify));
        }

        if config.discord_enabled {
            let discord = DiscordNotifier::new(config.discord_url);
            notifiers.push(Box::new(discord))
        }

        Self { notifiers }
    }

    pub async fn notify(&self, notification: Notification) {
        let futures = self.notifiers.iter().map(|notifier| {
            let notification = &notification;
            async move {
                if let Err(err) = notifier.send(notification).await {
                    error!("Failed to send notification via {}: {:?}", notifier.name(), err);
                }
            }
        });

        join_all(futures).await;
    }
}
