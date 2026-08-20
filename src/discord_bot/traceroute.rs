use poise::{CreateReply, serenity_prelude::CreateAttachment};
use staticmap::{StaticMapBuilder, tools::LineBuilder};

use crate::discord_bot::{Context, Error, reply_or_attach};

use std::net::IpAddr;

#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn traceroute(
    ctx: Context<'_>,
    #[description = "The Target IP"] target_ip: String,
) -> Result<(), Error> {
    let target_ip = match target_ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(e) => {
            reply_or_attach(&ctx, format!("Invalid IP format: {:?}", e), "error", "txt").await;
            return Ok(());
        }
    };

    ctx.defer().await?;

    let tracer = trippy_core::Builder::new(target_ip)
        .protocol(trippy_core::Protocol::Icmp)
        .max_rounds(Some(1))
        .first_ttl(1)
        .max_ttl(30)
        .build()?;

    tracer.run()?;

    let snapshot = tracer.snapshot();
    let hops = snapshot.hops();

    if hops.len() < 2 {
        ctx.reply("Less then 2 hops").await?;
        return Ok(());
    }

    let geo_hops: Vec<(f64, f64)> = hops
        .iter()
        .filter_map(|hop| {
            let ip = hop.addrs().next()?;

            let lookup: crate::maxmind::LookupResponse =
                ctx.data().mm.lookup(ip.to_owned()).ok()?;
            let location = lookup.city.map(|city| city.location)?;

            let lat = location.latitude?;
            let lon = location.longitude?;

            Some((lat, lon))
        })
        .collect();

    if geo_hops.len() < 2 {
        ctx.reply("No 2 hops with geo information").await?;
        return Ok(());
    }

    let image_bytes = generate_map(&geo_hops)?;
    let attachment = CreateAttachment::bytes(image_bytes, "map.png");

    ctx.send(CreateReply::new().attachment(attachment)).await?;

    Ok(())
}

fn generate_map(geo_hops: &[(f64, f64)]) -> Result<Vec<u8>, Error> {
    let (clamped_zoom, center_lat, center_lon) =
        calculate_zoom_and_center(geo_hops, 1000, 1000 / 3);
    let mut map = StaticMapBuilder::default()
        .width(1000)
        .height(1000 / 3)
        .zoom(clamped_zoom)
        .lat_center(center_lat)
        .lon_center(center_lon)
        .build()?;

    let mut last_lat = geo_hops[0].0;
    let mut last_lon = geo_hops[0].1;

    for (lat, lon) in geo_hops.iter().skip(1) {
        let line = LineBuilder::new()
            .lat_coordinates([last_lat, *lat])
            .lon_coordinates([last_lon, *lon])
            .build()?;

        map.add_tool(line);

        last_lat = *lat;
        last_lon = *lon;
    }

    Ok(map.encode_png()?)
}

fn calculate_zoom_and_center(
    geo_hops: &[(f64, f64)],
    map_width_px: u32,
    map_height_px: u32,
) -> (u8, f64, f64) {
    if geo_hops.is_empty() {
        return (1, 0.0, 0.0);
    }

    let mut min_lat = f64::MAX;
    let mut max_lat = f64::MIN;
    let mut min_lon = f64::MAX;
    let mut max_lon = f64::MIN;

    for &(lat, lon) in geo_hops {
        min_lat = min_lat.min(lat);
        max_lat = max_lat.max(lat);
        min_lon = min_lon.min(lon);
        max_lon = max_lon.max(lon);
    }

    let center_lat = (min_lat + max_lat) / 2.0;
    let center_lon = (min_lon + max_lon) / 2.0;

    let lat_span = (max_lat - min_lat).max(0.01) * 1.2;
    let lon_span = (max_lon - min_lon).max(0.01) * 1.2;

    let zoom_lon = ((map_width_px as f64 * 360.0) / (lon_span * 256.0)).log2();
    let zoom_lat = ((map_height_px as f64 * 180.0) / (lat_span * 256.0)).log2();

    let zoom = zoom_lon.min(zoom_lat).floor() as i32;
    let clamped_zoom = zoom.clamp(0, 18) as u8;

    (clamped_zoom, center_lat, center_lon)
}
