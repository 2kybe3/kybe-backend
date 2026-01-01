use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error, reply_or_attach};
use crate::finalize_command_trace;
use futures::{Stream, StreamExt};
use log::info;
use poise::CreateReply;
use poise::serenity_prelude::CreateAttachment;
use serde_json::Value;
use tokio::time::Instant;
use tracing::error;

async fn autocomplete_tag<'a>(
	ctx: Context<'_>,
	partial: &'a str,
) -> impl Stream<Item = String> + 'a {
	let now = Instant::now();
	let tags = {
		let mut catass = ctx.data().catass.write().await;
		catass.tags().await
	};

	futures::stream::iter(tags)
		.filter(move |tag| futures::future::ready(tag.starts_with(partial)))
		.map(|tag| tag.to_string())
}

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn cat(
	ctx: Context<'_>,
	#[description = "Get cat with a specific tag"]
	#[autocomplete = autocomplete_tag]
	tag: Option<String>,
	#[description = "Adds a text to the image"] says: Option<String>,
	verbose: Option<bool>,
) -> anyhow::Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "cat");

	trace.input = serde_json::json!({
		"tag": tag,
		"says": says,
		"verbose": verbose,
	});

	let verbose = verbose.unwrap_or(false);

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	let res = {
		let catass = ctx.data().catass.read().await;
		catass.get_cat_url(tag.as_deref(), says.as_deref()).await
	};

	match res {
		Ok(res) => {
			if let Value::Object(map) = &mut trace.data {
				map.insert("cataas".to_string(), serde_json::to_value(res.clone())?);
			}

			let bytes_result = {
				let catass = ctx.data().catass.read().await;
				catass.get_image(&res.url).await
			};
			match bytes_result {
				Ok(bytes) => {
					let ext = match res.mimetype.as_str() {
						"image/png" => "png",
						"image/jpeg" => "jpg",
						"image/gif" => "gif",
						_ => "img",
					};

					let attachment = CreateAttachment::bytes(bytes, format!("cat.{ext}"));
					let reply = if verbose {
						CreateReply::default()
							.attachment(attachment)
							.content(format!(
								"```json\n{}\n```",
								serde_json::to_string_pretty(&res)?
							))
					} else {
						CreateReply::default().attachment(attachment)
					};

					if let Err(e) = ctx.send(reply).await {
						error!("Failed to send response: {:?}", e);
						let _ = ctx
							.say("Failed to send the full response due to an error.")
							.await;
					}
					trace.output = Some("Image".into());
				}
				Err(e) => {
					trace.error = Some(format!("{:?}", e));
					trace.status = CommandStatus::Error;

					if verbose {
						let res = format!("{:#?}", res);
						trace.output = Some(res.clone());
						reply_or_attach(&ctx, res, "verbose.txt").await;
					} else {
						trace.output = Some(res.url.clone());
						ctx.reply(res.url).await?;
					}
				}
			}
		}
		Err(e) => {
			trace.status = CommandStatus::Error;
			trace.error = Some(format!("{:?}", e));
			trace.output = Some("Error evaluating expression".into());
			ctx.reply("Error evaluating expression").await?;
		}
	}

	finalize_command_trace!(ctx, trace);

	Ok(())
}
