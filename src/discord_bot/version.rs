use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::finalize_command_trace;

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "version");

	if let Err(e) = ctx.defer().await {
		trace.status = CommandStatus::Error;
		trace.error = Some(format!("Defer failed: {:?}", e));

		finalize_command_trace!(ctx, trace);
		return Err(e.into());
	}

	ctx.reply(crate::GIT_SHA).await?;
	trace.output = Some(crate::GIT_SHA.to_owned());

	finalize_command_trace!(ctx, trace);

	Ok(())
}
