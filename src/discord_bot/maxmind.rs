use std::net::IpAddr;

use crate::discord_bot::{Context, Error, attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn maxmind(
    ctx: Context<'_>,
    #[description = "The Ip To get Info for"] ip: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let ip = match ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            ctx.reply(format!("Invalid IP format: {:?}", e)).await?;
            return Ok(());
        }
    };

    let result = ctx.data().mm.lookup(ip);
    match result {
        Ok(res) => {
            attach(&ctx, serde_json::to_string_pretty(&res)?, "res.json").await;
        }
        Err(e) => {
            ctx.reply(format!("MaxMind Error: {:?}", e)).await?;
        }
    }

    Ok(())
}
