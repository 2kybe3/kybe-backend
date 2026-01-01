use anyhow::bail;
use toml::Value;

use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::{finalize_command_trace, reply_or_attach};

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn cat(ctx: Context<'_>) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "cat");

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	match get_cat_url(&ctx.data().client).await {
		Ok(url) => {
			trace.output = Some(url.clone());
			reply_or_attach!(ctx, url, "result.txt");
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

pub async fn get_cat_url(client: &reqwest::Client) -> anyhow::Result<String> {
	let resp = client
		.get("https://cataas.com/cat?json=true")
		.send()
		.await?;
	let json: Value = resp.json().await?;

	let url = json
		.get("url")
		.and_then(|v| v.as_str())
		.map(|s| s.to_string());
	match url {
		Some(url) => Ok(url),
		None => bail!("no url field"),
	}
}
