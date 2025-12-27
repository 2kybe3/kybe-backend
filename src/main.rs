mod config;
mod db;
mod discord_bot;
mod notifications;
pub mod translator;

use crate::config::types::{Config, LoggerConfig};
use crate::db::Database;
use crate::discord_bot::init_bot;
use crate::notifications::{Notification, Notifications};
use std::io::stdout;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::dispatcher::DefaultGuard;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};

fn init_logger_bootstrap() -> Result<DefaultGuard, Box<dyn std::error::Error>> {
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

fn init_logger(
    config: &LoggerConfig,
    old_logger: DefaultGuard,
) -> Result<(), Box<dyn std::error::Error>> {
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

async fn init_config() -> Result<Arc<Config>, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--generate-example") {
        let time = Instant::now();
        info!("Generating config.toml.example");
        Config::create_local_default().await?;
        info!("Generated config.toml.example in {} NS", time.elapsed().as_nanos());
        std::process::exit(0)
    }

    match Config::init().await {
        Ok(cfg) => Ok(Arc::new(cfg)),
        Err(e) => {
            error!("Failed to load config: {e}");
            std::process::exit(1);
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bootstrap_guard = init_logger_bootstrap()?;
    info!("initializing...");

    let config = init_config().await?;
    init_logger(&config.logger, bootstrap_guard)?;

    let notifications = Arc::new(Notifications::new(&config.notification));
    let database = match Database::init(Arc::clone(&config), Arc::clone(&notifications)).await {
        Ok(db) => db,
        Err(e) => {
            error!("Database init failed: {e}");
            notifications
                .notify(Notification::new("Database", &format!("Database init failed: {e}")))
                .await;
            std::process::exit(1);
        }
    };

    let notifications_clone = Arc::clone(&notifications);
    let bot_handle = tokio::spawn(async move {
        let mut retries = 0;
        let mut last_failure: Option<Instant> = None;

        loop {
            match init_bot(notifications_clone.clone(), config.clone(), database.clone()).await {
                Ok(_) => break,
                Err(e) => {
                    let now = Instant::now();

                    if let Some(last) = last_failure
                        && now.duration_since(last) > Duration::from_mins(5)
                    {
                        retries = 0;
                    }

                    last_failure = Some(now);
                    retries += 1;

                    notifications_clone
                        .notify(Notification::new(
                            "Discord Bot Critical Failure",
                            &format!("attempt {retries}: {e}"),
                        ))
                        .await;

                    if retries >= 5 {
                        break;
                    }

                    tokio::time::sleep(Duration::from_secs(5 * retries)).await;
                }
            }
        }
    });

    bot_handle.await?;
    Ok(())
}
