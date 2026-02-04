use uuid::Uuid;

use crate::webserver::render::{Color, Objects, Style, builders::TextBlobBuilder};

pub fn footer(trace_id: Uuid) -> Vec<Objects> {
	vec![
		TextBlobBuilder::new("\n").into(),
		TextBlobBuilder::new("           ")
			.style(Style::new().fg(Color::Black).bg(Color::Black))
			.into(),
		TextBlobBuilder::new(" Trace ID: ")
			.style(Style::new().fg(Color::BrightBlack))
			.into(),
		TextBlobBuilder::new(format!("{}\n", trace_id)).into(),
		TextBlobBuilder::new("           ")
			.style(Style::new().fg(Color::BrightRed).bg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new(" Version: ")
			.style(Style::new().fg(Color::BrightRed))
			.into(),
		TextBlobBuilder::new(format!("{}\n", crate::GIT_SHA.to_owned())).into(),
		TextBlobBuilder::new("           ")
			.style(Style::new().fg(Color::Yellow).bg(Color::Yellow))
			.into(),
		TextBlobBuilder::new(" Made By: ")
			.style(Style::new().fg(Color::Yellow))
			.into(),
		TextBlobBuilder::new("2kybe3\n").into(),
	]
}
