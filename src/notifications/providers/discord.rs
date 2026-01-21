use crate::notifications::{Notification, NotificationError, Notifier};

#[derive(Debug)]
pub struct DiscordNotifier {
	url: String,
}

impl DiscordNotifier {
	pub fn new<S: Into<String>>(url: S) -> Self {
		Self { url: url.into() }
	}
}

#[async_trait::async_trait]
impl Notifier for DiscordNotifier {
	async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
		let client = reqwest::Client::new();
		let payload = serde_json::json!({
			"embeds": [
				{
					"title": notification.title,
					"description": notification.message,
				}
			]
		});

		client
			.post(&self.url)
			.json(&payload)
			.send()
			.await
			.map_err(|e| NotificationError::Transport(format!("request failed: {e:?}")))?
			.error_for_status()
			.map_err(|e| NotificationError::Transport(format!("bad status: {e:?}")))?;

		Ok(())
	}

	fn name(&self) -> &'static str {
		"Discord"
	}
}
