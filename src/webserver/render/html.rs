use std::vec;

use crate::webserver::render::{
	Page,
	color::{Color, Style},
};

impl Style {
	pub fn html_hex(&self) -> (&'static str, &'static str) {
		// ( fg, bg )
		(self.fg.html_code(), self.bg.html_code())
	}

	pub fn html_style(&self) -> String {
		let (fg, bg) = self.html_hex();
		let mut styles = vec![format!("color:{}", fg), format!("background-color:{}", bg)];
		if self.bold {
			styles.push("font-weight:bold".into());
		}
		if self.dim {
			styles.push("opacity:0.6".into());
		}
		styles.join("; ")
	}
}

impl Color {
	pub fn html_code(&self) -> &'static str {
		match self {
			Color::Default => "inherit",

			Color::Black => "#000000",
			Color::Red => "#cd3131",
			Color::Green => "#0dbc79",
			Color::Yellow => "#e5e510",
			Color::Blue => "#2472c8",
			Color::Magenta => "#bc3fbc",
			Color::Cyan => "#11a8cd",
			Color::White => "#e5e5e5",

			Color::BrightBlack => "#666666",
			Color::BrightRed => "#f14c4c",
			Color::BrightGreen => "#23d18b",
			Color::BrightYellow => "#f5f543",
			Color::BrightBlue => "#3b8eea",
			Color::BrightMagenta => "#d670d6",
			Color::BrightCyan => "#29b8db",
			Color::BrightWhite => "#ffffff",
		}
	}
}

const HTML_TEMPLATE: &str = include_str!("../../../assets/template.html");

impl<'a> Page<'a> {
	pub fn render_html_page(&self, title: &str) -> String {
		let inner_html = self.render_html();

		HTML_TEMPLATE
			.replace("{{title}}", &html_escape::encode_text(title))
			.replace("{{content}}", &inner_html)
	}

	pub fn render_html(&self) -> String {
		let mut output = String::new();

		for obj in &self.objects {
			match obj {
				super::Object::TextBlob {
					text,
					style,
					link_to,
				} => {
					let (start, end) = match link_to {
						Some(link_to) => (
							&*format!("<a style=\"[style]\" href={}>", link_to.link),
							"</a>",
						),
						None => ("<span style=\"[style]\">", "</span>"),
					};
					output.push_str(&format!(
						"{}{}{}",
						&start.replace("[style]", &style.html_style()),
						&html_escape::encode_text(text),
						end
					));
				}
				super::Object::CodeBlock {
					title,
					language,
					code,
				} => {
					if title.is_some() || language.is_some() {
						output.push_str("<pre><code>");
						let mut parts = vec![];
						if let Some(t) = title {
							parts.push(format!(
								"Title: {}{}",
								html_escape::encode_text(t),
								if language.is_some() { "," } else { "" }
							));
						}
						if let Some(lang) = language {
							parts.push(format!("Lang: {}", html_escape::encode_text(lang)));
						}

						let header_style = Style::new_fg(Color::Cyan);
						output.push_str(&format!(
							"<div style=\"{}\">{}</div>",
							header_style.html_style(),
							parts.join(" ")
						));
					}
					output.push_str("<pre><code>");

					output.push_str(&html_escape::encode_text(code));
					output.push_str("</code></pre>");
					if title.is_some() {
						output.push_str("</code></pre>");
					}
				}
			}
		}

		output
	}
}
