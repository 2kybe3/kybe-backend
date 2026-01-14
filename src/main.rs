pub mod auth;
mod config;
mod db;
mod discord_bot;
pub mod email;
pub mod external;
mod notifications;
pub mod translator;
mod webserver;

use crate::auth::AuthService;
use crate::config::types::{Config, LoggerConfig};
use crate::db::Database;
use crate::email::EmailService;
use crate::notifications::Notifications;
use std::backtrace::Backtrace;
use std::io::stdout;
use std::sync::Arc;
use tracing::dispatcher::DefaultGuard;
use tracing::warn;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};

const GIT_SHA: &str = include_str!("../assets/git.sha");

macro_rules! exit_error {
	($notifications:expr, $title:expr, $msg:expr) => {{
		tracing::error!("{}: {}", $title, $msg);

		$notifications
			.notify(crate::notifications::Notification::new($title, $msg))
			.await;

		std::process::exit(1);
	}};
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
	// Init logger in two phases:
	// 1. without file logging to print info about loading the config
	// 2. with file logging derived from the config persistent for the rest of the application
	let bootstrap_guard = init_logger_bootstrap()?;
	let config = Config::init().await?;
	init_logger(&config.logger, bootstrap_guard)?;

	let notifications = Arc::new(Notifications::new(&config.notification));
	let database = match Database::init(Arc::clone(&config)).await {
		Ok(db) => db,
		Err(e) => exit_error!(
			notifications,
			"Database",
			format!("init failed: {e}\nBacktrace: {}", Backtrace::capture())
		),
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

/// Creates a default tracing logger that is used as long as DefaultGuard is not dropped
fn init_logger_bootstrap() -> anyhow::Result<DefaultGuard> {
	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
		EnvFilter::new("info").add_directive("kybe_backend=debug".parse().unwrap())
	});

	let subscriber = tracing_subscriber::fmt()
		.with_max_level(tracing::Level::TRACE)
		.with_thread_ids(true)
		.with_file(true)
		.with_line_number(true)
		.with_target(true)
		.with_env_filter(filter)
		.finish();

	Ok(tracing::subscriber::set_default(subscriber))
}

/// Takes a old_logger (from init_logger_bootstrap) and creates a new logger which includes file
/// logging which location we should have from the config
///
/// Drops the old_logger before setting the new logger
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
		.with_file(true)
		.with_line_number(true)
		.with_target(true)
		.with_env_filter(filter)
		.with_writer(writer)
		.finish();

	drop(old_logger);
	tracing::subscriber::set_global_default(subscriber)?;
	Ok(())
}
