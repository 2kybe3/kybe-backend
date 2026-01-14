use crate::notifications::{Notification, NotificationError, Notifier};
use poise::serenity_prelude::{CreateEmbed, ExecuteWebhook, Http, Webhook};

#[derive(Debug)]
pub struct DiscordNotifier {
	url: String,
	http: Http,
}

impl DiscordNotifier {
	pub fn new<S: Into<String>>(url: S) -> Self {
		let http = Http::new("");
		Self {
			url: url.into(),
			http,
		}
	}
}

#[async_trait::async_trait]
impl Notifier for DiscordNotifier {
	async fn send(&self, notification: &Notification) -> Result<(), NotificationError> {
		let embed = CreateEmbed::new()
			.title(&notification.title)
			.description(&notification.message);

		let execute = ExecuteWebhook::new().embed(embed);

		let webhook = Webhook::from_url(&self.http, &self.url)
			.await
			.map_err(|e| NotificationError::InvalidConfig(e.to_string()))?;

		webhook
			.execute(&self.http, true, execute)
			.await
			.map_err(|e| NotificationError::Transport(e.to_string()))?;

		Ok(())
	}

	fn name(&self) -> &'static str {
		"Discord"
	}
}
