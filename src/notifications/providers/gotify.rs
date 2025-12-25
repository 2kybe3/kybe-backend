use reqwest::{Client, StatusCode};
use serde::Serialize;
use crate::notifications::{Notification, Notifier};
use crate::notifications::error::NotificationError;

pub struct GotifyNotifier {
    url: String,
    client: Client,
}

impl GotifyNotifier {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GotifyPayload<'a> {
    title: &'a str,
    message: &'a str,
    priority: u8,
}

#[async_trait::async_trait]
impl Notifier for GotifyNotifier {
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        let payload = GotifyPayload {
            title: &notification.title,
            message: &notification.message,
            priority: notification.priority,
        };

        let res = self.client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::Transport(e.to_string()))?;

        let status = res.status();

        if status.is_success() {
            Ok(())
        } else {
            let msg = format!("({}): {}", status, res.text().await.unwrap_or_default());

            let err = match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => NotificationError::Auth(msg),
                StatusCode::BAD_REQUEST => NotificationError::InvalidConfig(msg),
                _ => NotificationError::Transport(msg),
            };
            Err(err)
        }
    }

    fn name(&self) -> &'static str {
        "Gotify"
    }
}