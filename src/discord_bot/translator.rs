use crate::discord_bot::{Context, Error};
use crate::reply_or_attach;

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
                reply_or_attach!(
                    ctx,
                    res.iter()
                        .map(|d| format!("{} ({:.0}%)", d.language, d.confidence))
                        .collect::<Vec<_>>()
                        .join(" -> "),
                    "detected_languages.txt"
                );
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
        reply_or_attach!(
            ctx,
            serde_json::to_string_pretty(&res).unwrap(),
            "languages_supported.json"
        );
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
                reply_or_attach!(
                    ctx,
                    serde_json::to_string_pretty(&res).unwrap(),
                    "languages_supported.json"
                )
            } else {
                if res.detected_language.is_some() {
                    source = res.detected_language.unwrap().language.clone();
                }
                reply_or_attach!(
                    ctx,
                    format!("{} -> {} \"{}\"", source, target, res.translated_text),
                    "detected_languages.txt"
                );
            }
        }
        Err(e) => {
            reply_or_attach!(ctx, format!("Error translating: {:?}", e), "error.txt");
        }
    }

    Ok(())
}
