use reqwest::{Client, StatusCode};
use serde::Serialize;
use crate::notifications::{Notification, Notifier};
use crate::notifications::error::NotificationError;

pub struct GotifyNotifier {
    url: String,
    token: String,
}

#[derive(Serialize)]
struct GotifyPayload<'a> {
    title: &'a str,
    message: &'a str,
    priority: u8,
}

impl GotifyNotifier {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }
}

#[async_trait::async_trait]
impl Notifier for GotifyNotifier {
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        let payload = GotifyPayload {
            title: &notification.title,
            message: &notification.message,
            priority: notification.priority,
        };

        let url = format!("{}/message?token={}", self.url, self.token);

        let client = Client::new();
        let res = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|_| NotificationError::Transport)?;

        match res.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(NotificationError::Auth),
            StatusCode::BAD_REQUEST => Err(NotificationError::InvalidConfig),
            _ => Err(NotificationError::Transport),
        }
    }

    fn name(&self) -> &'static str {
        "Gotify"
    }
}