use tracing::error;
use crate::notifications::error::NotificationError;
use crate::notifications::gotify::GotifyNotifier;
use crate::notifications::log::LogNotifier;

pub mod gotify;
pub mod error;
mod log;

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
    pub gotify_enabled: bool,
    pub gotify_url: String,
    pub gotify_token: String,
    pub log_enabled: bool,
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

        Self { notifiers }
    }

    pub async fn notify(&self, notification: Notification) {
        for notifier in &self.notifiers {
            if let Err(err) = notifier.send(&notification).await {
                error!("Failed to send notification via {}: {:?}", notifier.name(), err);
            }
        }
    }
}
