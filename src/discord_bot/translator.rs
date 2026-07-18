use crate::discord_bot::{Context, Error, reply_or_attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn detect(
    ctx: Context<'_>,
    #[description = "The text"] text: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let Some(translator) = ctx.data().translator.as_ref() else {
        ctx.reply("Translation is not enabled!").await?;
        return Ok(());
    };

    match translator.detect(text).await {
        Ok(res) => {
            let summary = res
                .iter()
                .map(|d| format!("{} ({:.0}%)", d.language, d.confidence))
                .collect::<Vec<_>>()
                .join(" -> ");

            reply_or_attach(&ctx, summary, "detected_languages.txt").await;
        }
        Err(e) => {
            ctx.reply(format!("Error detecting language: {:?}", e))
                .await?;
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
    ctx.defer().await?;

    let Some(translator) = ctx.data().translator.as_ref() else {
        ctx.reply("Translation is not enabled!").await?;
        return Ok(());
    };

    match translator.languages().await {
        Ok(res) => {
            reply_or_attach(
                &ctx,
                format!("```\n{}```", serde_json::to_string_pretty(&res)?),
                "languages_supported.json",
            )
            .await;
        }
        Err(e) => {
            ctx.reply(format!("Error getting languages {:?}", e))
                .await?;
        }
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
) -> Result<(), Error> {
    ctx.defer().await?;

    let Some(translator) = ctx.data().translator.as_ref() else {
        ctx.reply("Translation is not enabled!").await?;
        return Ok(());
    };

    let mut source = source.unwrap_or("auto".to_string());
    let target = target.unwrap_or("de".to_string());

    match translator.translate(&source, &target, &text).await {
        Ok(res) => {
            if let Some(det) = &res.detected_language {
                source = det.language.clone();
            }

            reply_or_attach(
                &ctx,
                format!("{} → {} \"{}\"", source, target, res.translated_text),
                "translation.txt",
            )
            .await;
        }
        Err(e) => {
            ctx.reply(format!("Error translating: {:?}", e)).await?;
        }
    }

    Ok(())
}
