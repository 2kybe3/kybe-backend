use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error, reply_or_attach};
use crate::external::cataas::{CATAASCatRequest, Filter, Fit, Position, Type};
use crate::finalize_command_trace;
use futures::{Stream, StreamExt};
use poise::CreateReply;
use poise::serenity_prelude::CreateAttachment;
use serde_json::Value;
use tracing::error;

async fn autocomplete_tag<'a>(
	ctx: Context<'_>,
	partial: &'a str,
) -> impl Stream<Item = String> + 'a {
	let tags = ctx.data().cataas.tags().await.to_owned();

	futures::stream::iter(tags)
		.filter(move |tag| {
			futures::future::ready(tag.starts_with(partial.split(",").last().unwrap_or("")))
		})
		.map(|tag| {
			let partial = match partial.rfind(',') {
				Some(pos) => &partial[..=pos],
				None => "",
			};
			format!("{}{}", partial, tag)
		})
}

#[allow(clippy::too_many_arguments)]
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
	#[description = "Cat Type"] cat_type: Option<Type>,
	#[description = "Filter"] filter: Option<Filter>,
	#[description = "Fit"] fit: Option<Fit>,
	#[description = "Position"] position: Option<Position>,
	#[description = "Width of result image"] width: Option<i32>,
	#[description = "Height of result image"] height: Option<i32>,
	#[description = "Blur the image"] blur: Option<i32>,
	#[description = "With custom filter"] r: Option<i32>,
	#[description = "With custom filter"] g: Option<i32>,
	#[description = "With custom filter"] b: Option<i32>,
	#[description = "With custom filter"] brightness: Option<i32>,
	#[description = "With custom filter"] saturation: Option<i32>,
	#[description = "With custom filter"] hue: Option<i32>,
	#[description = "With custom filter"] lightness: Option<i32>,
	#[description = "Vebose result"] verbose: Option<bool>,
) -> anyhow::Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "cat");

	trace.input = serde_json::json!({
		"tag": tag,
		"says": says,
		"cat_type": cat_type,
		"filter": filter,
		"fit": fit,
		"position": position,
		"width": width,
		"height": height,
		"blur": blur,
		"r": r,
		"g": g,
		"b": b,
		"brightness": brightness,
		"saturation": saturation,
		"hue": hue,
		"lightness": lightness,
		"verbose": verbose,
	});

	let verbose = verbose.unwrap_or(false);

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	let request = CATAASCatRequest {
		cat_type,
		filter,
		fit,
		position,
		width,
		height,
		blur,
		r,
		g,
		b,
		brightness,
		saturation,
		hue,
		lightness,
		json: Some(true),
	};

	let res = ctx
		.data()
		.cataas
		.get_cat_url(&request, tag.as_deref(), says.as_deref())
		.await;

	match res {
		Ok(Some(res)) => {
			if let Value::Object(map) = &mut trace.data {
				map.insert("cataas".to_string(), serde_json::to_value(res.clone())?);
			}

			match ctx.data().cataas.get_image(&res.url).await {
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
		Ok(None) => {
			trace.output = Some("No cat found".into());
			ctx.reply("No cat found").await?;
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
