#![warn(clippy::unwrap_used)]

pub mod external;
pub mod maxmind;
pub mod prometheus;
pub mod translator;

mod config;
mod discord_bot;
mod logger;
mod ssh;
mod webserver;

use crate::config::types::Config;
use crate::external::lastfm::LastFM;
use crate::maxmind::MaxMind;
use futures::future::try_join_all;
use once_cell::sync::Lazy;
use std::env;
use std::sync::Arc;
use tracing::warn;

pub static GIT_SHA: Lazy<String> =
    Lazy::new(|| env::var("KYBE_BACKEND_GIT_SHA").unwrap_or("dev".to_string()));

pub async fn notify_error(title: impl AsRef<str>, msg: impl Into<String>, exit: bool) {
    tracing::error!("{}: {}", title.as_ref(), msg.into());

    if exit {
        std::process::exit(1)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logger in two phases:
    // 1. without file logging to print info about loading the config
    // 2. with file logging derived from the config persistent for the rest of the application
    let bootstrap_guard = logger::init_logger_bootstrap()?;
    let config = Arc::new(Config::init().await?);
    logger::init_logger(&config.logger, bootstrap_guard)?;

    prometheus::register_custom_metrics();

    let mut handles = Vec::new();

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

    if config.discord_bot.enable {
        let config = Arc::clone(&config);
        let mm = Arc::clone(&mm);

        handles.push(tokio::spawn(async move {
            if let Err(e) = discord_bot::init_bot(config, mm).await {
                notify_error("Discord Bot", format!("init failed: {e}",), true).await;
            }
        }));
    }

    {
        let config = Arc::clone(&config);
        handles.push(tokio::spawn(async move {
            if let Err(e) = ssh::init(Arc::clone(&config)).await {
                notify_error("SSH", format!("init failed: {e}"), true).await;
            };
        }));
    }

    handles.push(tokio::spawn(async move {
        if let Err(e) = webserver::init_webserver(config, mm, lastfm).await {
            notify_error("Discord Bot", format!("init failed: {e}"), true).await;
        }
    }));

    try_join_all(handles).await?;
    Ok(())
}
