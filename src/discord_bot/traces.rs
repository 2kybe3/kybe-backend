use crate::db::command_traces::{CommandStatus, CommandTrace};
use crate::discord_bot::{Context, Error};
use crate::{finalize_command_trace, reply_or_attach};
use std::time::Instant;
use uuid::Uuid;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn get_trace(ctx: Context<'_>, trace_id: String) -> Result<(), Error> {
    let start = Instant::now();
    let mut trace = CommandTrace::start(&ctx, "get_trace");

    trace.input = serde_json::json!({
        "trace_id": trace_id,
    });

    if let Err(e) = ctx.defer().await {
        trace.status = CommandStatus::Error;
        trace.error = Some(format!("Defer failed: {:?}", e));

        finalize_command_trace!(ctx, trace);
        return Err(e.into());
    }

    if ctx.author().id.to_string() != ctx.data().config.discord_bot.admin_id {
        trace.status = CommandStatus::Disabled;
        trace.output = Some("You are not allowed to use this command!".into());

        ctx.reply("You are not allowed to use this command!").await?;
        finalize_command_trace!(ctx, trace);
        return Ok(());
    }

    let uuid = match Uuid::parse_str(&trace_id) {
        Ok(u) => u,
        Err(e) => {
            trace.status = CommandStatus::Error;
            trace.error = Some(format!("Invalid UUID: {:?}", e));
            trace.output = Some("Invalid trace ID format".into());
            ctx.reply("Invalid trace ID format (must be a valid UUID)").await?;

            finalize_command_trace!(ctx, trace);
            return Ok(());
        }
    };

    match ctx.data().database.get_command_trace(uuid).await {
        Ok(Some(db_trace)) => {
            let res = format!("{:#?}", db_trace);
            let duration = start.elapsed();
            trace.output = Some(format!("Trace fetched in {:?}", duration));
            reply_or_attach!(ctx, res, "trace.txt");
        }
        Ok(None) => {
            trace.output = Some("No trace found".into());
            ctx.reply(format!("No trace found for ID: `{}`", uuid)).await?;
        }
        Err(e) => {
            trace.status = CommandStatus::Error;
            trace.error = Some(format!("DB error: {:?}", e));
            trace.output = Some(format!("Failed to fetch trace `{}`", uuid));
            ctx.reply(format!("Failed to fetch trace `{}`: database error", uuid)).await?;
        }
    }

    finalize_command_trace!(ctx, trace);

    Ok(())
}

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn get_latest_trace(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    let mut trace = CommandTrace::start(&ctx, "get_latest_trace");

    if let Err(e) = ctx.defer().await {
        trace.status = CommandStatus::Error;
        trace.error = Some(format!("Defer failed: {:?}", e));
        finalize_command_trace!(ctx, trace);
        return Err(e.into());
    }

    if ctx.author().id.to_string() != ctx.data().config.discord_bot.admin_id {
        trace.status = CommandStatus::Disabled;
        trace.output = Some("Not allowed".into());
        ctx.reply("You are not allowed to use this command").await?;
        finalize_command_trace!(ctx, trace);
        return Ok(());
    }

    match ctx.data().database.get_latest_command_trace().await {
        Ok(Some(db_trace)) => {
            let output = format!("{:#?}", db_trace);
            let duration = start.elapsed();
            trace.output = Some(format!("Latest Trace fetched in {:?}", duration));
            reply_or_attach!(ctx, output, "latest_trace.txt");
        }
        Ok(None) => {
            trace.output = Some("No traces found".into());
            ctx.reply("No traces found").await?;
        }
        Err(e) => {
            trace.status = CommandStatus::Error;
            trace.error = Some(format!("{:?}", e));
            trace.output = Some("DB error".into());
            ctx.reply("Failed to fetch latest trace").await?;
        }
    }

    finalize_command_trace!(ctx, trace);
    Ok(())
}
