mod calculator;
mod cataas;
mod traces;
mod translator;
mod version;

use crate::config::types::Config;
use crate::notifications::{Notification, Notifications};
use poise::{CreateReply, FrameworkError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;

use crate::cataas::CatAss;
use crate::db::Database;
use crate::translator::Translator;
use poise::serenity_prelude as serenity;
use reqwest::Client;

type Error = anyhow::Error;
pub(crate) type Context<'a> = poise::Context<'a, Data, Error>;

pub const MAX_MSG_LENGTH: usize = 2000;

#[derive(Debug)]
pub struct Data {
	pub notifications: Arc<Notifications>,
	#[allow(dead_code)]
	pub config: Arc<Config>,
	pub translator: Option<Arc<Translator>>,
	#[allow(dead_code)]
	pub database: Database,

	pub client: Arc<Client>,
	pub catass: CatAss,
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
				version::version(),
				cataas::cat(),
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

				let client = Arc::new(Client::builder()
					.user_agent("kybe_backend / ".to_string() + crate::GIT_SHA)
					.timeout(Duration::from_secs(5))
					.read_timeout(Duration::from_secs(5))
					.connect_timeout(Duration::from_secs(5))
					.build()?);

				let catass = CatAss::new(client.clone());

                Ok(Data {
                    notifications,
                    config,
                    translator,
                    database,
					client,
					catass,
                })
            })
        })
        .build();

	let intents = serenity::GatewayIntents::non_privileged();

	let client = serenity::ClientBuilder::new(&token, intents)
		.framework(framework)
		.await;

	let mut client = client?;
	client.start().await?;
	Ok(())
}

pub async fn reply_or_attach(ctx: &Context<'_>, text: String, filename: impl Into<String>) {
	let result = if text.chars().count() <= MAX_MSG_LENGTH {
		ctx.reply(&text).await
	} else {
		let attachment = poise::serenity_prelude::CreateAttachment::bytes(text, filename);
		let reply = CreateReply::default().attachment(attachment);
		ctx.send(reply).await
	};

	if let Err(e) = result {
		error!("Failed to send response: {:?}", e);
		let _ = ctx
			.say("Failed to send the full response due to an error.")
			.await;
	}
}

#[macro_export]
macro_rules! finalize_command_trace {
    ($ctx:expr, $trace:expr) => {
        $trace.duration_ms = chrono::Utc::now().signed_duration_since($trace.started_at).num_milliseconds();
        $ctx.data().database.save_command_trace(&$trace).await?;

        if $trace.status == CommandStatus::Error {
            tracing::error!(trace = ?$trace, "command finished with error");
        } else {
            tracing::debug!(trace = ?$trace, "command finished");
        }
    };
}
