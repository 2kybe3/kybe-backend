use poise::{CreateReply, serenity_prelude::CreateAttachment};
use staticmap::{StaticMapBuilder, tools::CircleBuilder};

use crate::discord_bot::{Context, Error, reply_or_attach};

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn coords(ctx: Context<'_>, lat: String, lon: String) -> Result<(), Error> {
    let (Ok(lat), Ok(lon)) = (lat.parse(), lon.parse()) else {
        reply_or_attach(&ctx, "invalid lat, long".into(), "error", "txt").await;
        return Ok(());
    };

    ctx.defer().await?;

    let img = {
        let mut map = StaticMapBuilder::default()
            .width(1000)
            .height(1000)
            .zoom(6)
            .lat_center(lat)
            .lon_center(lon)
            .build()?;

        let circle = CircleBuilder::new()
            .lat_coordinate(lat)
            .lon_coordinate(lon)
            .radius(4f32)
            .build()?;

        map.add_tool(circle);

        CreateAttachment::bytes(map.encode_png()?, "map.png")
    };

    ctx.send(CreateReply::new().attachment(img)).await?;

    Ok(())
}
