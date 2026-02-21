use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error, reply_or_attach};
use crate::finalize_command_trace;

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
	let mut trace = CommandTrace::start(&ctx, "detect");

	trace.input = serde_json::json!({
		"text": text,
		"verbose": verbose.unwrap_or(false),
	});

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	if let Some(translator) = ctx.data().translator.as_ref() {
		match translator.detect(text).await {
			Ok(res) => {
				let verbose = verbose.unwrap_or(false);
				if verbose {
					let output = format!("{:?}", res);
					trace.output = Some(output.clone());
					ctx.reply(output).await?;
				} else {
					let summary = res
						.iter()
						.map(|d| format!("{} ({:.0}%)", d.language, d.confidence))
						.collect::<Vec<_>>()
						.join(" -> ");
					trace.output = Some(summary.clone());

					reply_or_attach(&ctx, summary, "detected_languages.txt").await;
				}
			}
			Err(e) => {
				trace.status = CommandStatus::Error;
				trace.error = Some(format!("{:?}", e));
				trace.output = Some(format!(
					"Error detecting language: (trace id {})",
					trace.trace_id
				));
				ctx.reply(format!(
					"Error detecting language: (trace id {})",
					trace.trace_id
				))
				.await?;
			}
		}
	} else {
		trace.status = CommandStatus::Disabled;
		trace.output = Some("Translation is not enabled!".to_string());
		ctx.reply("Translation is not enabled!").await?;
	}

	finalize_command_trace!(ctx, trace);

	Ok(())
}

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn languages(ctx: Context<'_>) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "languages");

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	if let Some(translator) = ctx.data().translator.as_ref() {
		match translator.languages().await {
			Ok(res) => {
				let output = serde_json::to_string_pretty(&res)?;
				trace.output = Some(format!("{} languages supported", res.len()));
				reply_or_attach(
					&ctx,
					format!("```\n{}```", output),
					"languages_supported.json",
				)
				.await;
			}
			Err(e) => {
				trace.status = CommandStatus::Error;
				trace.error = Some(format!("{:?}", e));
				trace.output = Some(format!(
					"Error getting languages (trace id: {})",
					trace.trace_id
				));
				ctx.reply(format!(
					"Error getting languages (trace id: {})",
					trace.trace_id
				))
				.await?;
			}
		}
	} else {
		trace.status = CommandStatus::Disabled;
		trace.output = Some("Translation is not enabled!".into());
		ctx.reply("Translation is not enabled!").await?;
	}

	finalize_command_trace!(ctx, trace);

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
	let mut trace = CommandTrace::start(&ctx, "translate");

	let source_str = source.as_deref().unwrap_or("auto");
	let target_str = target.as_deref().unwrap_or("de");

	trace.input = serde_json::json!({
		"source": source_str,
		"target": target_str,
		"text": text,
		"verbose": verbose.unwrap_or(false),
	});

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
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
					trace.output = Some(output.clone());
					reply_or_attach(
						&ctx,
						format!("```json\n{}\n```", output),
						"translation.json",
					)
					.await;
				} else {
					if let Some(det) = &res.detected_language {
						source = det.language.clone();
					}
					let output = format!("{} → {} \"{}\"", source, target, res.translated_text);
					trace.output = Some(output.clone());
					reply_or_attach(&ctx, output, "translation.txt").await;
				}
			}
			Err(e) => {
				trace.status = CommandStatus::Error;
				trace.error = Some(format!("{:?}", e));
				trace.output = Some(format!("Error translating (trace id: {})", trace.trace_id));
				ctx.reply(format!("Error translating (trace id: {})", trace.trace_id))
					.await?;
			}
		}
	} else {
		trace.status = CommandStatus::Disabled;
		trace.output = Some("Translation is not enabled!".to_string());
		ctx.reply("Translation is not enabled!").await?;
	}

	finalize_command_trace!(ctx, trace);

	Ok(())
}
