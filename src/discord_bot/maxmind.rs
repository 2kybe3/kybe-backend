use std::net::IpAddr;

use crate::discord_bot::{Context, Error, reply_or_attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn maxmind(
    ctx: Context<'_>,
    #[description = "The Ip To get Info for"] ip: String,
) -> Result<(), Error> {
    let ip = match ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            reply_or_attach(&ctx, format!("Invalid IP format: {:?}", e), "error", "txt").await;
            return Ok(());
        }
    };

    let result = ctx
        .data()
        .mm
        .lookup(ip)
        .and_then(|res| serde_json::to_string_pretty(&res).map_err(|e| anyhow::anyhow!(e)));

    match result {
        Ok(res) => {
            reply_or_attach(&ctx, res, "res", "json").await;
        }
        Err(e) => {
            reply_or_attach(&ctx, e.to_string(), "error", "txt").await;
        }
    }

    Ok(())
}
