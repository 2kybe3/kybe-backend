use crate::notifications::error::NotificationError;
use crate::notifications::{Notification, Notifier};
use reqwest::{Client, StatusCode};
use webhook::models::Message;

pub struct DiscordNotifier {
    url: String,
    client: Client,
}

impl DiscordNotifier {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Notifier for DiscordNotifier {
    async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
        let mut payload = Message::new();
        payload.embed(|embed| {
            embed
                .title(&notification.title)
                .description(&notification.message)
        });

        let res = self
            .client
            .post(&self.url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::Transport(e.to_string()))?;

        let status = res.status();

        if status.is_success() {
            return Ok(());
        } else {
            let msg = format!("({}) {}", status, res.text().await.unwrap());

            let err = match status {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => NotificationError::Auth(msg),
                StatusCode::BAD_REQUEST => NotificationError::InvalidConfig(msg),
                _ => NotificationError::Transport(msg),
            };
            Err(err)
        }
    }

    fn name(&self) -> &'static str {
        "Discord"
    }
}
