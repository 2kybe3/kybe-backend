pub mod auth;
mod config;
mod db;
mod discord_bot;
pub mod email;
mod notifications;
pub mod translator;
mod webserver;

use crate::auth::AuthService;
use crate::config::types::{Config, LoggerConfig};
use crate::db::Database;
use crate::email::EmailService;
use crate::notifications::{Notification, Notifications};
use std::io::stdout;
use std::sync::Arc;
use tracing::dispatcher::DefaultGuard;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};

const GIT_SHA: &str = env!("KYBE_GIT_SHA");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
	let bootstrap_guard = init_logger_bootstrap()?;
	let config = Config::init().await?;
	init_logger(&config.logger, bootstrap_guard)?;

	let notifications = Arc::new(Notifications::new(&config.notification));
	let database = match Database::init(Arc::clone(&config), Arc::clone(&notifications)).await {
		Ok(db) => {
			db.delete_old_unverified_users_loop().await;
			db
		}
		Err(e) => {
			error!("Database init failed: {e}");
			notifications
				.notify(Notification::new(
					"Database",
					&format!("Database init failed: {e}"),
				))
				.await;
			std::process::exit(1);
		}
	};

	let email_service = Arc::new(EmailService::new(&config.email));
	let email_service_loop_handle = email_service.run_loop();

	let bot_handle = tokio::spawn(discord_bot::init_bot(
		Arc::clone(&notifications),
		Arc::clone(&config),
		database.clone(),
	));

	let webserver_handle = tokio::spawn(webserver::init_webserver(
		notifications,
		config,
		Arc::new(AuthService::new(database.clone())),
		database,
	));

	tokio::try_join!(webserver_handle, bot_handle, email_service_loop_handle)?;
	Ok(())
}

fn init_logger_bootstrap() -> anyhow::Result<DefaultGuard> {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
		EnvFilter::new("info").add_directive("kybe_backend=debug".parse().unwrap())
	});

	let subscriber = tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_thread_ids(true)
		.with_env_filter(filter)
		.finish();

	Ok(tracing::subscriber::set_default(subscriber))
}

fn init_logger(config: &LoggerConfig, old_logger: DefaultGuard) -> anyhow::Result<()> {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
		EnvFilter::new("info").add_directive("kybe_backend=debug".parse().unwrap())
	});

	let writer: BoxMakeWriter = if config.file_logger_enabled {
		match &config.file_logger_path {
			Some(path) if !path.trim().is_empty() => BoxMakeWriter::new(
				stdout.and(tracing_appender::rolling::daily(path, "kybe_backend.log")),
			),
			_ => {
				warn!("file_logger_enabled but file_logger_path is empty or not set, disabling!");
				BoxMakeWriter::new(stdout)
			}
		}
	} else {
		BoxMakeWriter::new(stdout)
	};

	let subscriber = tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_thread_ids(true)
		.with_env_filter(filter)
		.with_writer(writer)
		.finish();

	drop(old_logger);
	tracing::subscriber::set_global_default(subscriber)?;
	Ok(())
}
