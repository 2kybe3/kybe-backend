use crate::webserver::render::{
	Object, Page, PageRenderer, Style, Theme,
	color::bit4::Bit4Color,
	object::{ColorMapping, LinkTo},
};

pub struct AnsiRenderer();

impl<'a> PageRenderer<'a> for AnsiRenderer {
	fn render(page: &Page<'a>) -> String {
		(*page.objects)
			.iter()
			.map(Self::render_object)
			.collect::<Vec<_>>()
			.join("")
	}

	fn render_object(obj: &Object) -> String {
		match obj {
			super::Object::TextBlob {
				text,
				style,
				link_to,
			} => Self::render_text_blob(text, style, link_to),
			super::Object::CodeBlock {
				title,
				language,
				code,
			} => Self::render_code_block(title, language, code),
			super::Object::Canvas {
				data,
				color_mapping,
			} => Self::render_canvas(data, color_mapping),
			super::Object::Image {
				url,
				alt,
				width,
				height,
			} => Self::render_image(url, alt, width, height),
		}
	}

	fn render_text_blob(text: &str, style: &Style, link_to: &Option<LinkTo>) -> String {
		let mut output = style.ansi_code();
		let mut text = text.to_string();

		if let Some(link_to) = link_to
			&& link_to.link().starts_with("http")
			&& !text.contains(link_to.link())
		{
			let mut colored = String::new();
			if let Some(style) = &link_to.separator_style() {
				colored.push_str(&style.ansi_code());
			}
			colored.push_str(" => ");
			if let Some(style) = &link_to.link_style() {
				colored.push_str(&style.ansi_code());
			} else {
				// If no link style is set use the previous style (the text)
				colored.push_str(&style.ansi_code());
			}

			let index = text.find("\n");

			match index {
				Some(index) => {
					text.insert_str(index, &format!("{}{}", &colored, &link_to.link()));
				}
				None => {
					text.push_str(&colored);
					text.push_str(link_to.link());
				}
			};
		}

		output.push_str(&text);

		output.push_str(&Style::default().ansi_code());

		output
	}

	fn render_code_block(title: &Option<String>, language: &Option<String>, code: &str) -> String {
		let mut output = String::new();

		if title.is_some() || language.is_some() {
			output.push_str("---title---\n");
			output.push_str(&Style::new().fg(Bit4Color::CYAN).ansi_code());
			if let Some(title) = &title {
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
			output.push_str("---code----\n");
		} else {
			output.push_str("\n---code----\n");
		}

		// TODO: highlighting
		output.push_str(code);

		output.push_str("\n-----------\n\n");

		output
	}

	fn render_image(url: &str, alt: &str, _width: &i64, _height: &i64) -> String {
		AnsiRenderer::render_object(&(Theme::default().link_colored(alt, url).into()))
	}

	fn render_canvas(data: &str, color_mapping: &ColorMapping) -> String {
		let mut output = String::new();
		let mut buffer = String::new();
		let mut last_color = Bit4Color::DEFAULT.into();

		let mut failed = false;

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
				failed = true;
				break;
			}
		}

		if !buffer.is_empty() && failed {
			Self::render_text_blob(
				"Error rendering Canvas",
				&Style::new().fg(Bit4Color::RED),
				&None,
			)
		} else {
			output
		}
	}
}
