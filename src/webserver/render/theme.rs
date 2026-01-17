use crate::webserver::render::{
	Color, Object, Style,
	builders::{HasLink, HasStyle, LinkToBuilder, NoLink, NoStyle, TextBlobBuilder},
};

pub struct Theme {
	pub title: Style,
	pub subtitle: Style,
	pub label: Style,
	#[allow(unused)]
	pub text: Style,
	pub link: Style,
	pub link_separator: Style,
	pub comment: Style,
}

impl Default for Theme {
	fn default() -> Self {
		Self {
			title: Style::new().fg(Color::BrightRed).bold(true),
			subtitle: Style::new().fg(Color::BrightRed),
			label: Style::new().fg(Color::Yellow),
			text: Style::new().fg(Color::White),
			link: Style::new().fg(Color::Green),
			link_separator: Style::new().fg(Color::White),
			comment: Style::new().fg(Color::BrightBlack).dim(true),
		}
	}
}

impl Theme {
	#[allow(unused)]
	pub fn raw<S, L>(&self, text: impl Into<TextBlobBuilder<S, L>>) -> TextBlobBuilder<S, L> {
		text.into()
	}

	pub fn title<L>(
		&self,
		text: impl Into<TextBlobBuilder<NoStyle, L>>,
	) -> TextBlobBuilder<HasStyle, L> {
		text.into().style(self.title)
	}

	pub fn subtitle<L>(
		&self,
		text: impl Into<TextBlobBuilder<NoStyle, L>>,
	) -> TextBlobBuilder<HasStyle, L> {
		text.into().style(self.subtitle)
	}

	#[allow(unused)]
	pub fn text<L>(
		&self,
		text: impl Into<TextBlobBuilder<NoStyle, L>>,
	) -> TextBlobBuilder<HasStyle, L> {
		text.into().style(self.text)
	}

	pub fn comment<L>(
		&self,
		text: impl Into<TextBlobBuilder<NoStyle, L>>,
	) -> TextBlobBuilder<HasStyle, L> {
		text.into().style(self.comment)
	}

	pub fn link_colored(
		&self,
		text: impl Into<TextBlobBuilder<NoStyle, NoLink>>,
		link: &str,
	) -> TextBlobBuilder<HasStyle, HasLink> {
		text.into().style(self.link).link_to(
			LinkToBuilder::new(link)
				.link_style(self.link)
				.seperator_style(self.link_separator)
				.into(),
		)
	}

	#[allow(unused)]
	pub fn link<S>(
		&self,
		text: impl Into<TextBlobBuilder<S, NoLink>>,
		link: &str,
	) -> TextBlobBuilder<S, HasLink> {
		text.into().link_to(
			LinkToBuilder::new(link)
				.link_style(self.link)
				.seperator_style(self.link_separator)
				.into(),
		)
	}

	fn label_text(&self, title: &str) -> Object {
		TextBlobBuilder::new(format!("{}: ", title))
			.style(self.label)
			.into()
	}

	pub fn label(&self, title: &str, data: Vec<Object>) -> Vec<Object> {
		let mut output: Vec<Object> = vec![self.label_text(title)];

		output.extend(data);

		output
	}
}
