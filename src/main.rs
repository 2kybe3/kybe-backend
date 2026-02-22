#![warn(clippy::unwrap_used)]

pub mod auth;
mod config;
mod db;
mod discord_bot;
pub mod email;
pub mod external;
mod logger;
pub mod maxmind;
mod notifications;
pub mod translator;
mod webserver;

use crate::auth::AuthService;
use crate::config::types::Config;
use crate::db::Database;
use crate::email::EmailService;
use crate::external::lastfm::LastFM;
use crate::maxmind::MaxMind;
use crate::notifications::{Notification, Notifications};
use futures::future::try_join_all;
use once_cell::sync::Lazy;
use std::backtrace::Backtrace;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing::warn;

pub static GIT_SHA: Lazy<&str> = Lazy::new(|| include_str!("../assets/git.sha").trim());
static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static NOTIFICATIONS_INSTANCE: OnceLock<Arc<Notifications>> = OnceLock::new();

pub async fn notify_error(
	notifications: &Notifications,
	title: impl AsRef<str>,
	msg: impl Into<String>,
	exit: bool,
) {
	let mut msg = msg.into();

	let bt = Backtrace::force_capture();
	let bt_str = format!("{bt:#?}").trim().to_owned();

	let url = external::null_pointer::upload_to_0x0(&bt_str, Some(Duration::from_hours(1))).await;

	if exit {
		match url {
			Some(u) => msg.push_str(&format!("\nBacktrace: {}", u)),
			None => msg.push_str("\nBacktrace (upload failed)"),
		}
	}

	msg.push_str(&format!("\nVersion: {}", *GIT_SHA));

	tracing::error!("{}: {}", title.as_ref(), &msg);
	notifications
		.notify(Notification::new(title.as_ref(), &msg), exit)
		.await;

	if exit {
		std::process::exit(1)
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	std::panic::set_hook(Box::new(|info| {
		if PANIC_IN_PROGRESS.swap(true, Ordering::Relaxed) {
			return;
		}

		let bt = Backtrace::force_capture();
		let bt_str = format!("{bt:#?}").trim().to_owned();

		eprintln!("panic occurred: {info}\nBacktrace: {bt_str}");

		futures::executor::block_on(async move {
			let url =
				external::null_pointer::upload_to_0x0(&bt_str, Some(Duration::from_hours(1))).await;

			let notification = match url {
				Some(u) => Notification::new(
					"Fatal panic",
					format!("{}\nBacktrace: {}\nVersion: {}", info, u, *GIT_SHA),
				),
				None => Notification::new(
					"Fatal panic",
					format!("{}\nBacktrace (upload failed)\nVersion: {}", info, *GIT_SHA),
				),
			};

			if let Some(notifications) = NOTIFICATIONS_INSTANCE.get() {
				notifications.notify(notification, true).await;
			} else {
				eprintln!("panic before notifications ready: {notification:#?}");
			}
		});
	}));

	// Init logger in two phases:
	// 1. without file logging to print info about loading the config
	// 2. with file logging derived from the config persistent for the rest of the application
	let bootstrap_guard = logger::init_logger_bootstrap()?;
	let config = Arc::new(Config::init().await?);
	logger::init_logger(&config.logger, bootstrap_guard)?;

	let mut handles = Vec::new();

	let notifications = Arc::new(Notifications::new(&config.notification));
	NOTIFICATIONS_INSTANCE
		.set(Arc::clone(&notifications))
		.expect("Notifications already set");

	let mm = Arc::new(MaxMind::new(config.maxmind.clone())?);
	let lastfm = if config.lastfm.enable {
		let lastfm = LastFM::new(&config.lastfm).map(Arc::new);
		if let Some(ref lastfm) = lastfm {
			Arc::clone(lastfm).run_cacher().await;
		}
		lastfm
	} else {
		None
	};

	let database = match Database::init(Arc::clone(&config)).await {
		Ok(db) => db,
		Err(e) => {
			notify_error(
				&notifications,
				"Database",
				format!("init failed: {e}"),
				true,
			)
			.await;
			unreachable!()
		}
	};

	let args: Vec<String> = env::args().collect();
	if args.iter().any(|arg| arg == "--sync-maxmind") {
		database.sync_maxmind(Arc::clone(&mm)).await?;
	}

	#[allow(unused)]
	let mut email_service = None;
	#[allow(unused)]
	if config.email.enable {
		email_service = Some(Arc::new(EmailService::new(&config.email)));
	}

	let auth = Arc::new(AuthService::new(database.clone()));

	if config.discord_bot.enable {
		let notifications = Arc::clone(&notifications);
		let config = Arc::clone(&config);
		let mm = Arc::clone(&mm);
		let database = database.clone();

		handles.push(tokio::spawn(async move {
			if let Err(e) = discord_bot::init_bot(notifications.clone(), config, database, mm).await
			{
				notify_error(
					&notifications,
					"Discord Bot",
					format!("init failed: {e}",),
					true,
				)
				.await;
			}
		}));
	}

	let auth_clone = Arc::clone(&auth);
	handles.push(tokio::spawn(async move {
		if let Err(e) = webserver::init_webserver(config, auth_clone, database, mm, lastfm).await {
			notify_error(
				&notifications,
				"Discord Bot",
				format!("init failed: {e}"),
				true,
			)
			.await;
		}
	}));

	try_join_all(handles).await?;
	Ok(())
}
