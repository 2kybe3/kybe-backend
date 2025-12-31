use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::{finalize_command_trace, reply_or_attach};

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn calculate(
	ctx: Context<'_>,
	#[description = "Mathematical expression (e.g. 1+2*(3^2) or sqrt(16) or sin(pi/2)"]
    expression: String,
) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "calculate");

	trace.input = serde_json::json!({
		"expression": expression,
	});

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	match meval::eval_str(&expression) {
		Ok(result) => {
			let response = format!("**{}** = **{}**", expression, result);
			trace.output = Some(response.clone());
			reply_or_attach!(ctx, response, "result.txt");
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
