use poise::{CreateReply, serenity_prelude::CreateEmbed};

use crate::discord_bot::{Context, Error};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn wolframalpha(ctx: Context<'_>, expression: String) -> Result<(), Error> {
    ctx.defer().await?;

    let res = ctx.data().wolframalpha.query(expression).await?;

    let mut reply = CreateReply::default();

    for pod in res {
        let mut description = String::new();
        for subpod in pod.subpods {
            description.push_str(&subpod.plaintext);
        }

        if description.is_empty() {
            continue;
        }

        let embed = CreateEmbed::new().title(pod.title).description(description);
        reply = reply.embed(embed);
    }

    ctx.send(reply).await?;

    Ok(())
}
