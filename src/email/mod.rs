use lettre::{
	AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
	transport::smtp::authentication::Credentials,
};

use crate::config::types::EmailConfig;

#[derive(Debug)]
pub struct EmailService {
	email: String,
	password: String,
	server: String,
	port: u16,
}

impl EmailService {
	pub fn new(config: &EmailConfig) -> Self {
		Self {
			email: config.email.clone(),
			password: config.password.clone(),
			server: config.server.clone(),
			port: config.port,
		}
	}

	pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
		let mail = Message::builder()
			.from(self.email.parse()?)
			.to(to.parse()?)
			.subject(subject)
			.body(body.to_string())?;

		let creds = Credentials::new(self.email.clone(), self.password.clone());

		let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.server)?
			.port(self.port)
			.credentials(creds)
			.build();

		mailer.send(mail).await?;
		Ok(())
	}
}
