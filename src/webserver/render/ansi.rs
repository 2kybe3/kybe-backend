use crate::webserver::render::{
	ColorMapping, LinkTo, Object, Page,
	color::{Color, Style},
};

impl Style {
	pub fn ansi_code(self) -> String {
		let mut codes = vec![];
		if self.bold {
			codes.push("1")
		}
		if self.dim {
			codes.push("2")
		}

		codes.push(match self.fg {
			Color::Default => "39",

			Color::Black => "30",
			Color::Red => "31",
			Color::Green => "32",
			Color::Yellow => "33",
			Color::Blue => "34",
			Color::Magenta => "35",
			Color::Cyan => "36",
			Color::White => "37",

			Color::BrightBlack => "90",
			Color::BrightRed => "91",
			Color::BrightGreen => "92",
			Color::BrightYellow => "93",
			Color::BrightBlue => "94",
			Color::BrightMagenta => "95",
			Color::BrightCyan => "96",
			Color::BrightWhite => "97",
		});

		codes.push(match self.bg {
			Color::Default => "49",

			Color::Black => "40",
			Color::Red => "41",
			Color::Green => "42",
			Color::Yellow => "43",
			Color::Blue => "44",
			Color::Magenta => "45",
			Color::Cyan => "46",
			Color::White => "47",

			Color::BrightBlack => "100",
			Color::BrightRed => "101",
			Color::BrightGreen => "102",
			Color::BrightYellow => "103",
			Color::BrightBlue => "104",
			Color::BrightMagenta => "105",
			Color::BrightCyan => "106",
			Color::BrightWhite => "107",
		});

		format!("\x1b[0m\x1b[{}m", codes.join(";"))
	}
}

impl Page {
	fn render_ansi_text_blob(text: &str, style: &Style, link_to: &Option<LinkTo>) -> String {
		let mut output = String::new();

		output.push_str(&style.ansi_code());

		let mut text = text.to_string();

		if let Some(link_to) = link_to
			&& link_to.link.starts_with("http")
			&& !text.contains(&link_to.link)
			&& let Some(http_index) = text.find("http")
		{
			let mut colored = String::new();
			if let Some(style) = link_to.seperator_style {
				colored.push_str(&style.ansi_code());
			}
			colored.push_str(" => ");
			if let Some(style) = link_to.link_style {
				colored.push_str(&style.ansi_code());
			} else {
				// If no link style is set use the previous style (the text)
				colored.push_str(&style.ansi_code());
			}

			let trimmed = text.trim_start();
			let newline_index = trimmed[http_index..].find("\n").map(|i| i + http_index);

			if let Some(index) = newline_index {
				let rest = text.split_off(index);
				text.push_str(&colored);
				text.push_str(&link_to.link);
				text.push_str(&rest);
			} else {
				text.push_str(&colored);
				text.push_str(&link_to.link);
			}
		}

		output.push_str(&text);

		output.push_str(&Style::default().ansi_code());

		output
	}

	fn render_ansi_code_block(
		title: &Option<String>,
		language: &Option<String>,
		code: &[Object],
	) -> String {
		let mut output = String::new();

		if title.is_some() || language.is_some() {
			output.push_str("--title--\n");
			output.push_str(&Style::new().fg(Color::Cyan).ansi_code());
			if let Some(title) = title {
				output.push_str(&format!(
					"title: {}{}",
					title,
					if language.is_some() { "," } else { "" }
				));
			}
			if let Some(language) = language {
				if title.is_some() {
					output.push(' ');
				}
				output.push_str(&format!("lang: {}", language));
			};

			output.push_str(&Style::default().ansi_code());
			output.push('\n');
			output.push_str("--code---\n");
		} else {
			output.push_str("\n--code---\n");
		}

		output.push_str(
			&code
				.iter()
				.map(Self::render_ansi_object)
				.collect::<Vec<_>>()
				.join(""),
		);
		output.push_str("\n---------\n\n");

		output
	}

	fn render_ansi_canvas(data: &str, color_mapping: &ColorMapping) -> Option<String> {
		let mut output = String::new();
		let mut buffer = String::new();
		let mut last_color = Color::Default;

		for ch in data.chars() {
			buffer.push(ch);

			if buffer == "NL" {
				output.push('\n');
				buffer.clear();
				continue;
			}

			if let Some(&color) = color_mapping.get(&buffer) {
				if color != last_color {
					output.push_str(&Style::new().fg(color).bg(color).ansi_code());
				}

				output.push(' ');
				last_color = color;
				buffer.clear();
				continue;
			}

			let max_key_len = color_mapping.keys().map(|k| k.len()).max().unwrap_or(0);
			if buffer.len() > max_key_len {
				return None;
			}
		}

		if !buffer.is_empty() {
			return None;
		}

		Some(output)
	}

	pub fn render_ansi_object(obj: &Object) -> String {
		match obj {
			super::Object::TextBlob {
				text,
				style,
				link_to,
				..
			} => Self::render_ansi_text_blob(text, style, link_to),
			super::Object::CodeBlock {
				title,
				language,
				code,
			} => Self::render_ansi_code_block(title, language, code),
			super::Object::Canvas {
				data,
				color_mapping,
			} => Self::render_ansi_canvas(data, color_mapping).unwrap_or(
				Self::render_ansi_text_blob(
					"Error rendering Canvas",
					&Style::new().fg(Color::Red),
					&None,
				),
			),
		}
	}

	pub fn render_ansi(&self) -> String {
		self.objects
			.iter()
			.map(Self::render_ansi_object)
			.collect::<Vec<_>>()
			.join("")
	}
}
