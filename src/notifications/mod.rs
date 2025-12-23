use tracing::error;
use crate::notifications::error::NotificationError;
use crate::notifications::gotify::GotifyNotifier;

pub mod gotify;
pub mod error;

pub trait Notifier {
    fn send(&self, notification: &Notification) -> Result<(), NotificationError>;
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

impl Notifications {
    pub fn new() -> Self {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

        let gotify_enabled = true;
        let gotify_url = "https://example.com".to_string();
        let gotify_token = "test".to_string();

        if gotify_enabled {
            let gotify = GotifyNotifier::new(gotify_url, gotify_token);
            notifiers.push(Box::new(gotify));
        }

        Self { notifiers }
    }

    pub fn notify(&self, notification: Notification) {
        for notifier in &self.notifiers {
            if let Err(err) = notifier.send(&notification) {
                error!("Failed to send notification: {:?}", err);
            }
        }
    }
}
