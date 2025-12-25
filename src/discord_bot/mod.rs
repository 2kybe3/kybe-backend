use std::sync::Arc;
use poise::FrameworkError;
use tracing::error;
use crate::notifications::{Notification, Notifications};

#[derive(Clone)]
#[derive(Debug)]
pub struct Data {
    pub notifications: Arc<Notifications>,
}

pub async fn init_bot(notifications: Arc<Notifications>) {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: None,
                ..Default::default()
            },
            on_error: |error: FrameworkError<'_, Data, anyhow::Error>| Box::pin(async move {
                if let Some(ctx) = error.ctx() {
                    let notifications = &ctx.data().notifications;

                    notifications
                        .notify(Notification::new("Serenity", &format!("Error: {}", error.to_string())))
                        .await
                } else {
                    error!("Error without context: {:?}", error);
                }
            }),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { notifications: Arc::clone(&notifications) })
            })
        })
        .build();
}