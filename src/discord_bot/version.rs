use crate::db::command_traces::CommandTrace;
use crate::discord_bot::{Context, Error, finalize_command_trace};

#[poise::command(
	slash_command,
	install_context = "Guild|User",
	interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn version(ctx: Context<'_>) -> Result<(), Error> {
	let mut trace = CommandTrace::start(&ctx, "version");

	let response = format!("version: `{}`", crate::GIT_SHA.to_owned());
	ctx.reply(response.clone()).await?;
	trace.output = Some(response);

	finalize_command_trace(&ctx, &mut trace).await?;

	Ok(())
}
