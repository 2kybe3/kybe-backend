use std::net::IpAddr;

use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error, attach, defer, finalize_command_trace};

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn maxmind(
	ctx: Context<'_>,
	#[description = "The Ip To get Info for"] ip: String,
) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "maxmind");
	defer(&ctx, &mut trace).await?;

	trace.input = serde_json::json!({
		"ip": ip,
	});

	let ip = match ip.parse::<IpAddr>() {
		Ok(ip) => ip,
		Err(e) => {
			trace.status = CommandStatus::Error;
			trace.error = Some(format!("Invalid IP: {:?}", e));
			trace.output = Some("Invalid IP format".into());
			ctx.reply("Invalid IP format").await?;

			finalize_command_trace(&ctx, &mut trace).await?;
			return Ok(());
		}
	};

	let result = ctx.data().mm.lookup(ip);
	match result {
		Ok(res) => {
			let res = serde_json::to_string_pretty(&res)?;
			trace.output = Some(res.clone());
			attach(&ctx, res, "res.json").await;

			finalize_command_trace(&ctx, &mut trace).await?;
		}
		Err(e) => {
			trace.status = CommandStatus::Error;
			trace.error = Some(format!("MaxMind error: {:?}", e));
			trace.output = Some("Maxmind Error".into());
			ctx.reply("MaxMind Error").await?;

			finalize_command_trace(&ctx, &mut trace).await?;
		}
	}

	Ok(())
}
