use crate::db::traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::{finalize_command_trace, reply_or_attach};
use std::time::Instant;

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
    let start = Instant::now();
    let mut log = CommandTrace::start(&ctx, "detect");

    log.input = serde_json::json!({
        "text": text,
        "verbose": verbose.unwrap_or(false),
    });

    if let Err(e) = ctx.defer().await {
        log.status = CommandStatus::Error;
        log.error = Some(format!("Defer failed: {:?}", e));

        finalize_command_trace!(ctx, log, start);
        return Err(e.into());
    }

    if let Some(translator) = ctx.data().translator.as_ref() {
        match translator.detect(text).await {
            Ok(res) => {
                let verbose = verbose.unwrap_or(false);
                if verbose {
                    let output = format!("{:?}", res);
                    log.output = Some(output.clone());
                    ctx.reply(output).await?;
                } else {
                    let summary = res
                        .iter()
                        .map(|d| format!("{} ({:.0}%)", d.language, d.confidence))
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    log.output = Some(summary.clone());

                    reply_or_attach!(ctx, summary, "detected_languages.txt");
                }
            }
            Err(e) => {
                log.status = CommandStatus::Error;
                log.error = Some(format!("{:?}", e));
                log.output = Some(format!("Error detecting language: (trace id {})", log.trace_id));
                ctx.reply(format!("Error detecting language: (trace id {})", log.trace_id)).await?;
            }
        }
    } else {
        log.status = CommandStatus::Disabled;
        log.output = Some("Translation is not enabled!".to_string());
        ctx.reply("Translation is not enabled!").await?;
    }

    finalize_command_trace!(ctx, log, start);

    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn languages(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    let mut log = CommandTrace::start(&ctx, "languages");

    if let Err(e) = ctx.defer().await {
        log.status = CommandStatus::Error;
        log.error = Some(format!("Defer failed: {:?}", e));

        finalize_command_trace!(ctx, log, start);
        return Err(e.into());
    }

    if let Some(translator) = ctx.data().translator.as_ref() {
        match translator.languages().await {
            Ok(res) => {
                let output = serde_json::to_string_pretty(&res)?;
                log.output = Some(format!("{} languages supported", res.len()));
                reply_or_attach!(ctx, output, "languages_supported.json");
            }
            Err(e) => {
                log.status = CommandStatus::Error;
                log.error = Some(format!("{:?}", e));
                log.output = Some(format!("Error getting languages (trace id: {})", log.trace_id));
                ctx.reply(format!("Error getting languages (trace id: {})", log.trace_id)).await?;
            }
        }
    } else {
        log.status = CommandStatus::Disabled;
        log.output = Some("Translation is not enabled!".into());
        ctx.reply("Translation is not enabled!").await?;
    }

    finalize_command_trace!(ctx, log, start);

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
    let start = Instant::now();
    let mut log = CommandTrace::start(&ctx, "translate");

    let source_str = source.as_deref().unwrap_or("auto");
    let target_str = target.as_deref().unwrap_or("de");

    log.input = serde_json::json!({
        "source": source_str,
        "target": target_str,
        "text": text,
        "verbose": verbose.unwrap_or(false),
    });

    if let Err(e) = ctx.defer().await {
        log.status = CommandStatus::Error;
        log.error = Some(format!("Defer failed: {:?}", e));

        finalize_command_trace!(ctx, log, start);
        return Err(e.into());
    }

    if let Some(translator) = ctx.data().translator.as_ref() {
        let mut source = source.unwrap_or("auto".to_string());
        let target = target.unwrap_or("de".to_string());

        match translator.translate(&source, &target, &text).await {
            Ok(res) => {
                let verbose = verbose.unwrap_or(false);
                if verbose {
                    let output = serde_json::to_string_pretty(&res)?;
                    log.output = Some(output.clone());
                    reply_or_attach!(ctx, output, "translation.json");
                } else {
                    if let Some(det) = &res.detected_language {
                        source = det.language.clone();
                    }
                    let output = format!("{} → {} \"{}\"", source, target, res.translated_text);
                    log.output = Some(output.clone());
                    reply_or_attach!(ctx, output, "translation.txt");
                }
            }
            Err(e) => {
                log.status = CommandStatus::Error;
                log.error = Some(format!("{:?}", e));
                log.output = Some(format!("Error translating (trace id: {})", log.trace_id));
                ctx.reply(format!("Error translating (trace id: {})", log.trace_id)).await?;
            }
        }
    } else {
        log.status = CommandStatus::Disabled;
        log.output = Some("Translation is not enabled!".to_string());
        ctx.reply("Translation is not enabled!").await?;
    }

    finalize_command_trace!(ctx, log, start);

    Ok(())
}
