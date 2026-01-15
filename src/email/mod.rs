use crate::config::types::EmailConfig;
use anyhow::anyhow;
use async_imap::Session;
use async_imap::types::Fetch;
use async_native_tls::TlsStream;
use chrono::{FixedOffset, NaiveDate, NaiveTime};
use futures::{StreamExt, TryStreamExt};
use mail_parser::{Header, HeaderName, HeaderValue, MessageParser};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{error, info};

#[derive(Debug)]
pub struct EmailService {
	email: String,
	password: String,
	server: String,
	port: u16,

	tx: Sender<IncomingEmail>,
}

fn dt_to_chrono(dt: &mail_parser::DateTime) -> Option<chrono::DateTime<FixedOffset>> {
	let offset = FixedOffset::east_opt(
		(if dt.tz_before_gmt { -1 } else { 1 })
			* (dt.tz_hour as i32 * 3600 + dt.tz_minute as i32 * 60),
	)?;

	let date = NaiveDate::from_ymd_opt(dt.year.into(), dt.month.into(), dt.day.into())?;
	let time = NaiveTime::from_hms_opt(dt.hour.into(), dt.minute.into(), dt.second.into())?;

	Some(chrono::DateTime::from_naive_utc_and_offset(
		date.and_time(time),
		offset,
	))
}

#[derive(Debug, Clone)]
pub struct Address {
	pub name: Option<String>,
	pub address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IncomingEmail {
	pub subject: Option<String>,
	pub from: Vec<Address>,
	pub to: Vec<Address>,
	pub cc: Vec<Address>,
	pub reply_to: Vec<Address>,
	pub date: Option<chrono::DateTime<FixedOffset>>,
	pub message_id: Option<String>,
	pub in_reply_to: Option<String>,
	pub references: Option<String>,
	pub html: Option<String>,
	pub text: Option<String>,
}

impl EmailService {
	pub fn new(config: &EmailConfig) -> Self {
		let (tx, _) = broadcast::channel(64);

		Self {
			email: config.email.clone(),
			password: config.password.clone(),
			server: config.server.clone(),
			port: config.port,
			tx,
		}
	}

	pub fn subscribe(&self) -> Receiver<IncomingEmail> {
		self.tx.subscribe()
	}

	fn get_header_str<'x>(headers: &'x [Header<'x>], name: &HeaderName) -> Option<String> {
		headers
			.iter()
			.find(|h| h.name == *name)
			.and_then(|h| match &h.value {
				HeaderValue::Text(t) => Some(t.as_ref().to_string()),
				_ => None,
			})
	}

	fn get_header_address(headers: &[Header], name: &HeaderName) -> Vec<Address> {
		headers
			.iter()
			.filter(|h| h.name == *name)
			.flat_map(|h| match &h.value {
				HeaderValue::Address(addr_list) => addr_list
					.iter()
					.map(|a| Address {
						name: a.name.as_deref().map(ToString::to_string),
						address: a.address.as_deref().map(ToString::to_string),
					})
					.collect::<Vec<_>>(),
				_ => Vec::new(),
			})
			.collect()
	}

	fn get_header_date<'x>(
		headers: &'x [Header<'x>],
		name: &HeaderName,
	) -> Option<chrono::DateTime<FixedOffset>> {
		headers
			.iter()
			.find(|h| h.name == *name)
			.and_then(|h| match &h.value {
				HeaderValue::DateTime(addr) => Some(addr),
				_ => None,
			})
			.and_then(dt_to_chrono)
	}

	async fn process_messages(
		imap_session: &mut Session<TlsStream<TcpStream>>,
		tx: &Sender<IncomingEmail>,
	) -> anyhow::Result<()> {
		let _mailbox = imap_session.select("INBOX").await?;

		let uids = imap_session.uid_search("ALL").await?;
		let uid_set = uids
			.iter()
			.map(|u| u.to_string())
			.collect::<Vec<_>>()
			.join(",");

		if uid_set.is_empty() {
			return Ok(());
		}

		let message_stream = imap_session
			.uid_fetch(&uid_set, "(FLAGS ENVELOPE BODY.PEEK[])")
			.await?;
		let messages: Vec<Fetch> = message_stream.try_collect().await?;

		for msg in messages.iter() {
			if let Some(uid) = msg.uid {
				let parsed = MessageParser::default()
					.parse(msg.body().ok_or(anyhow!("msg body is none"))?)
					.ok_or(anyhow!("failed to parse body"))?;
				let headers = parsed.headers();

				let from = Self::get_header_address(headers, &HeaderName::From);
				let to = Self::get_header_address(headers, &HeaderName::To);
				let subject = Self::get_header_str(headers, &HeaderName::Subject);
				let date = Self::get_header_date(headers, &HeaderName::Date);

				let cc = Self::get_header_address(headers, &HeaderName::Cc);
				let reply_to = Self::get_header_address(headers, &HeaderName::ReplyTo);
				let message_id = Self::get_header_str(headers, &HeaderName::MessageId);
				let in_reply_to = Self::get_header_str(headers, &HeaderName::InReplyTo);
				let references = Self::get_header_str(headers, &HeaderName::References);

				let html: Option<String> = parsed.body_html(0).map(|h| h.to_string());
				let text: Option<String> = parsed.body_text(0).map(|t| t.to_string());

				let email = IncomingEmail {
					subject,
					from,
					to,
					cc,
					reply_to,
					date,
					message_id,
					in_reply_to,
					references,
					html,
					text,
				};

				info!("New mail: {:?}", email);

				if let Err(e) = tx.send(email) {
					error!("Failed to broadcast email (no receivers?): {e}");
				}

				imap_session.uid_mv(uid.to_string(), "Processed").await?;
			}
		}
		Ok(())
	}

	pub async fn connect_and_login(&self) -> anyhow::Result<Session<TlsStream<TcpStream>>> {
		let imap_addr = (self.server.clone(), self.port);
		let tcp_stream = TcpStream::connect(imap_addr.clone()).await?;
		let tls = async_native_tls::TlsConnector::new();
		let tls_stream = tls.connect(self.server.clone(), tcp_stream).await?;

		let client = async_imap::Client::new(tls_stream);
		Ok(client
			.login(self.email.clone(), self.password.clone())
			.await
			.map_err(|e| e.0)?)
	}

	/// Ensures the "Processed" Mailbox exists where processed mails get moved too
	pub async fn ensure_processed_mailbox(
		imap_session: &mut Session<TlsStream<TcpStream>>,
	) -> Result<(), async_imap::error::Error> {
		let mut has_processed = false;
		{
			let mut mailboxes_stream = imap_session.list(None, Some("*")).await?;

			while let Some(result) = mailboxes_stream.next().await {
				match result {
					Ok(name) => {
						if name.name() == "Processed" {
							has_processed = true;
							break;
						}
					}
					Err(e) => {
						error!("Error listing mailboxes: {e}");
					}
				}
			}
		}

		if !has_processed {
			let _ = imap_session.create("Processed").await;
		}

		Ok(())
	}

	pub async fn run(&self) -> anyhow::Result<()> {
		let mut imap_session = self.connect_and_login().await?;

		Self::ensure_processed_mailbox(&mut imap_session).await?;

		Self::process_messages(&mut imap_session, &self.tx).await?;

		// Main Message Loop for new messages
		loop {
			// Wait for IDLE to wake client up (new msg rarely smth else)
			let mut idle = imap_session.idle();
			idle.init().await?;

			{
				let wait_future = idle.wait();
				wait_future.0.await?;
			}

			imap_session = idle.done().await?;

			Self::process_messages(&mut imap_session, &self.tx).await?;
		}
	}
}
