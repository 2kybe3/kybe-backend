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
) -> Result<(), Error> {
    let translator = ctx.data().translator.as_ref();
    if translator.is_none() {
        ctx.reply("Translation is not enabled!").await.unwrap();
        return Ok(());
    }

    let response = translator.unwrap().detect(text).await;

    ctx.say(format!("{:?}", response)).await.unwrap();
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
    #[description = "Source languages (can be auto)"] source: String,
    #[description = "Target language"] target: String,
    #[description = "The text"] text: String,
) -> Result<(), Error> {
    let translator = ctx.data().translator.as_ref();
    if translator.is_none() {
        ctx.reply("Translation is not enabled!").await.unwrap();
        return Ok(());
    }

    match translator.unwrap().translate(source, target, text).await {
        Ok(res) => {
            let attachment = CreateAttachment::bytes(
                serde_json::to_string_pretty(&res).unwrap(),
                "languages_supported.json",
            );
            let reply = CreateReply::default().attachment(attachment);
            ctx.send(reply).await.unwrap();
        }
        Err(e) => {
            ctx.reply(format!("Error translating: {:?}", e)).await.unwrap();
        }
    }

    Ok(())
}

