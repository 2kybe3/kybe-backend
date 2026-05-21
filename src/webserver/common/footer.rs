use uuid::Uuid;

use crate::webserver::render::{
	Color, Objects, Style,
	builders::{LinkToBuilder, TextBlobBuilder},
};

pub fn footer(trace_id: Uuid) -> Vec<Objects> {
	let spacer = " ".repeat(12);
	vec![
		TextBlobBuilder::new("\n").into(),
		TextBlobBuilder::new(&spacer)
			.style(
				Style::new()
					.fg(Color::GERMAN_FLAG_BLACK)
					.bg(Color::GERMAN_FLAG_BLACK),
			)
			.into(),
		TextBlobBuilder::new(" Version: ")
			.style(Style::new().fg(Color::BRIGHT_BLACK))
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
			.style(
				Style::new()
					.fg(Color::GERMAN_FLAG_RED)
					.bg(Color::GERMAN_FLAG_RED),
			)
			.into(),
		TextBlobBuilder::new(" Trace ID: ")
			.style(Style::new().fg(Color::GERMAN_FLAG_RED))
			.into(),
		TextBlobBuilder::new(format!("{}\n", trace_id)).into(),
		TextBlobBuilder::new(&spacer)
			.style(
				Style::new()
					.fg(Color::GERMAN_FLAG_GOLD)
					.bg(Color::GERMAN_FLAG_GOLD),
			)
			.into(),
		TextBlobBuilder::new(" Made By: ")
			.style(Style::new().fg(Color::GERMAN_FLAG_GOLD))
			.into(),
		TextBlobBuilder::new("2kybe3\n").into(),
	]
}
