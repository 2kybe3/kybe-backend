use crate::webserver::render::{
	Page,
	color::{Color, Style},
};

// TODO: maybe improve this mapping a bit
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

		format!("\x1b[{}m", codes.join(";"))
	}
}

impl<'a> Page<'a> {
	pub fn render_ansi(&self) -> String {
		let mut output = String::new();

		for obj in &self.objects {
			match obj {
				super::Object::TextBlob { text, style } => {
					if let Some(s) = style {
						output.push_str(&s.ansi_code());
						output.push_str(text);
						output.push_str(&Style::default().ansi_code());
					} else {
						output.push_str(text);
					}
				}
				super::Object::CodeBlock {
					title,
					language,
					code,
				} => {
					if title.is_some() || language.is_some() {
						output.push_str("\n\n--title--\n");
						output.push_str(&Style::new_fg(Color::Cyan).ansi_code());
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
						output.push_str("\n\n--code---\n");
					}

					output.push_str(code);
					output.push_str("\n---------\n\n");
				}
			}
		}

		output
	}
}
