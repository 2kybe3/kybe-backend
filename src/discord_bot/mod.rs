mod calculator;
mod translator;

use crate::config::types::Config;
use crate::notifications::{Notification, Notifications};
use poise::FrameworkError;
use std::sync::Arc;
use tracing::error;

use crate::translator::Translator;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub const MAX_MSG_LENGTH: usize = 2000;

#[derive(Clone, Debug)]
pub struct Data {
    pub notifications: Arc<Notifications>,
    pub config: Arc<Config>,
    pub translator: Option<Arc<Translator>>,
}

pub async fn init_bot(notifications: Arc<Notifications>, config: Arc<Config>) -> Result<(), Error> {
    let token = config.discord_bot.token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                calculator::calculate(),
                translator::detect(),
                translator::languages(),
                translator::translate(),
            ],
            on_error: |error: FrameworkError<'_, Data, Error>| {
                Box::pin(async move {
                    if let Some(ctx) = error.ctx() {
                        let notifications = &ctx.data().notifications;

                        notifications
                            .notify(Notification::new(
                                "Discord Bot Error",
                                &format!("Error: {}", error),
                            ))
                            .await
                    } else {
                        error!("Error without context: {:?}", error);
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let translator = config
                    .discord_bot
                    .translator
                    .clone()
                    .try_into()
                    .ok()
                    .map(Arc::new);
                if translator.is_none() && config.discord_bot.translator.enabled {
                    tracing::warn!(
                        "Failed to initialize translator – translation commands will be unavailable"
                    );
                }
                Ok(Data {
                    notifications: Arc::clone(&notifications),
                    config: Arc::clone(&config),
                    translator,
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

#[macro_export]
macro_rules! roa {
    ($ctx:expr, $s:expr, $filename:expr) => {{
        let text = $s.to_string();

        if $s.chars().count() <= $crate::discord_bot::MAX_MSG_LENGTH {
            $ctx.reply(&text).await.unwrap();
        } else {
            let attachment = CreateAttachment::bytes(text, $filename);
            let reply = CreateReply::default().attachment(attachment);
            $ctx.send(reply).await.unwrap();
        }
    }};
}
