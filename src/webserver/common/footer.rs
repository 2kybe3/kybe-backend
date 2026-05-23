use uuid::Uuid;

use crate::webserver::render::{
	Style,
	builders::{LinkToBuilder, TextBlobBuilder},
	color::{bit4::Bit4Color, bit24::Bit24Color},
	object::Objects,
};

pub const GERMAN_FLAG_BLACK: Bit24Color = Bit24Color::new(0, 0, 0);
pub const GERMAN_FLAG_RED: Bit24Color = Bit24Color::new(221, 0, 0);
pub const GERMAN_FLAG_GOLD: Bit24Color = Bit24Color::new(255, 204, 0);

pub fn footer(trace_id: Uuid) -> Vec<Objects> {
	let spacer = " ".repeat(12);
	vec![
		TextBlobBuilder::new("\n").into(),
		TextBlobBuilder::new(&spacer)
			.style(Style::new().fg(GERMAN_FLAG_BLACK).bg(GERMAN_FLAG_BLACK))
			.into(),
		TextBlobBuilder::new(" Version: ")
			.style(Style::new().fg(Bit4Color::BRIGHT_BLACK))
			.into(),
		TextBlobBuilder::new(format!("{}\n", crate::GIT_SHA.to_owned()))
			.link_to(
				LinkToBuilder::new(format!(
					"https://git.kybe.xyz/2kybe3/kybe-backend/src/commit/{}",
					crate::GIT_SHA.to_owned()
				))
				.into(),
			)
			.into(),
		TextBlobBuilder::new(&spacer)
			.style(Style::new().fg(GERMAN_FLAG_RED).bg(GERMAN_FLAG_RED))
			.into(),
		TextBlobBuilder::new(" Trace ID: ")
			.style(Style::new().fg(GERMAN_FLAG_RED))
			.into(),
		TextBlobBuilder::new(format!("{}\n", trace_id)).into(),
		TextBlobBuilder::new(&spacer)
			.style(Style::new().fg(GERMAN_FLAG_GOLD).bg(GERMAN_FLAG_GOLD))
			.into(),
		TextBlobBuilder::new(" Made By: ")
			.style(Style::new().fg(GERMAN_FLAG_GOLD))
			.into(),
		TextBlobBuilder::new("2kybe3\n").into(),
	]
}
