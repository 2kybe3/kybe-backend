mod calculator;
mod traces;
mod translator;

use crate::config::types::Config;
use crate::notifications::{Notification, Notifications};
use poise::FrameworkError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;

use crate::db::Database;
use crate::translator::Translator;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

pub const MAX_MSG_LENGTH: usize = 2000;

#[derive(Clone, Debug)]
pub struct Data {
    pub notifications: Arc<Notifications>,
    #[allow(dead_code)]
    pub config: Arc<Config>,
    pub translator: Option<Arc<Translator>>,
    #[allow(dead_code)]
    pub database: Database,
}

pub async fn init_bot(notifications: Arc<Notifications>, config: Arc<Config>, database: Database) {
    let mut retries = 0;
    let mut last_failure: Option<Instant> = None;

    loop {
        match init_bot_inner(notifications.clone(), config.clone(), database.clone()).await {
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

                notifications
                    .notify(Notification::new(
                        "Discord Bot Critical Failure",
                        &format!("attempt {retries}: {e}"),
                    ))
                    .await;

                if retries >= 5 {
                    notifications
                        .notify(Notification::new(
                            "Disabling discord bot",
                            &format!("due to error: {e} on retry {retries}"),
                        ))
                        .await;
                    break;
                }

                tokio::time::sleep(Duration::from_secs(5 * retries)).await;
            }
        }
    }
}

async fn init_bot_inner(
    notifications: Arc<Notifications>,
    config: Arc<Config>,
    database: Database,
) -> Result<(), Error> {
    let token = config.discord_bot.token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                calculator::calculate(),
                translator::detect(),
                translator::languages(),
                translator::translate(),
                traces::get_trace(),
                traces::get_latest_trace(),
            ],
            on_error: |error: FrameworkError<'_, Data, Error>| Box::pin(async move {
                if let Some(ctx) = error.ctx() {
                    let notifications = &ctx.data().notifications;

                    notifications.notify(Notification::new(
                        "Discord Bot Error",
                             &format!("Error: {}", error),
                    )).await
                } else {
                    error!("Error without context: {:?}", error);
                }
            }),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let translator_result = config.discord_bot.translator.clone().try_into();

                let translator = match translator_result {
                    Ok(trans) => {
                        Some(Arc::new(trans))
                    }
                    Err(e) => {
                        if config.discord_bot.translator.enabled {
                            tracing::warn!(
                                "Failed to initialize translator: {:#?}\nTranslation commands will be unavailable.",
                                e
                            );

                            notifications
                                .notify(Notification::new(
                                    "Discord Bot - Translator Initialization Failed",
                                    &format!(
                                        "Failed to initialize translator.\n\nError details:\n{:#?}\n\nTranslation commands will be unavailable.",
                                        e
                                    ),
                                ))
                                .await;
                        }
                        None
                    }
                };

                Ok(Data {
                    notifications,
                    config,
                    translator,
                    database,
                })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(&token, intents).framework(framework).await;

    let mut client = client?;
    client.start().await?;
    Ok(())
}

#[macro_export]
macro_rules! reply_or_attach {
    ($ctx:expr, $s:expr, $filename:expr) => {{
        let text = $s.to_string();
        let result = if text.chars().count() <= $crate::discord_bot::MAX_MSG_LENGTH {
            $ctx.reply(&text).await
        } else {
            let attachment = poise::serenity_prelude::CreateAttachment::bytes(text, $filename);
            let reply = poise::CreateReply::default().attachment(attachment);
            $ctx.send(reply).await
        };

        if let Err(e) = result {
            tracing::error!("Failed to send response: {:?}", e);
            let _ = $ctx.say("Failed to send the full response due to an error.").await;
        }
    }};
}

#[macro_export]
macro_rules! finalize_command_trace {
    ($ctx:expr, $log:expr, $start:expr) => {
        $log.duration_ms = $start.elapsed().as_millis().try_into().unwrap_or(0);
        $ctx.data().database.save_command_trace(&$log).await?;

        if $log.status == CommandStatus::Error {
            tracing::error!(log = ?$log, "command finished with error");
        } else {
            tracing::debug!(log = ?$log, "command finished");
        }
    };
}
