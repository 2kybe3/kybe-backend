use crate::discord_bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude::CreateAttachment;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn detect(
    ctx: Context<'_>,
    #[description = "The text"] text: String,
    #[description = "verbose"] verbose: Option<bool>,
) -> Result<(), Error> {
    let translator = ctx.data().translator.as_ref();
    if translator.is_none() {
        ctx.reply("Translation is not enabled!").await.unwrap();
        return Ok(());
    }

    let response = translator.unwrap().detect(text).await;

    match response {
        Ok(res) => {
            let verbose = verbose.unwrap_or(false);
            if verbose {
                ctx.reply(format!("{:?}", res)).await.unwrap();
            } else {
                ctx.reply(
                    res.iter()
                        .map(|d| format!("{} ({:.0}%)", d.language, d.confidence))
                        .collect::<Vec<_>>()
                        .join(" -> "),
                )
                .await
                .unwrap();
            }
        }
        Err(_) => {
            ctx.reply("Error detecting language").await.unwrap();
        }
    }
    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn languages(ctx: Context<'_>) -> Result<(), Error> {
    let translator = ctx.data().translator.as_ref();
    if translator.is_none() {
        ctx.reply("Translation is not enabled!").await.unwrap();
        return Ok(());
    }

    if let Ok(res) = translator.unwrap().languages().await {
        let attachment = CreateAttachment::bytes(
            serde_json::to_string_pretty(&res).unwrap(),
            "languages_supported.json",
        );
        let reply = CreateReply::default().attachment(attachment);
        ctx.send(reply).await.unwrap();
    } else {
        ctx.reply("Error getting languages").await.unwrap();
    }

    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn translate(
    ctx: Context<'_>,
    #[description = "Source languages (can be auto)"] source: Option<String>,
    #[description = "Target language"] target: Option<String>,
    #[description = "The text"] text: String,
    #[description = "verbose"] verbose: Option<bool>,
) -> Result<(), Error> {
    let translator = ctx.data().translator.as_ref();
    if translator.is_none() {
        ctx.reply("Translation is not enabled!").await.unwrap();
        return Ok(());
    }

    let mut source = source.unwrap_or("auto".to_string());
    let target = target.unwrap_or("de".to_string());

    match translator.unwrap().translate(&source, &target, &text).await {
        Ok(res) => {
            let verbose = verbose.unwrap_or(false);
            if verbose {
                let attachment = CreateAttachment::bytes(
                    serde_json::to_string_pretty(&res).unwrap(),
                    "languages_supported.json",
                );
                let reply = CreateReply::default().attachment(attachment);
                ctx.send(reply).await.unwrap();
            } else {
                if res.detected_language.is_some() {
                    source = res.detected_language.unwrap().language.clone();
                }
                ctx.reply(format!("{} -> {}: {}", source, target, res.translated_text))
                    .await
                    .unwrap();
            }
        }
        Err(e) => {
            ctx.reply(format!("Error translating: {:?}", e))
                .await
                .unwrap();
        }
    }

    Ok(())
}
