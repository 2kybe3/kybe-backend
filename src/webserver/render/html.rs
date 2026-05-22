use crate::{
	config::types::UmamiConfig,
	webserver::render::{ColorMapping, LinkTo, Object, Page, Style, color::bit4::Bit4Color},
};

const HTML_TEMPLATE: &str = include_str!("../../../assets/template.html");

impl Page {
	pub fn render_html_page(&self, title: &str, umami: &UmamiConfig) -> String {
		let inner_html = self.render_html();

		let umami = match (umami.id.clone(), umami.script_path.clone()) {
			(Some(id), Some(path)) => format!(
				"<script defer src=\"{}\" data-website-id=\"{}\"></script>",
				path, id
			),
			(Some(id), None) => format!(
				"<script defer src=\"/script.js\" data-website-id=\"{}\"></script>",
				id
			),
			_ => "".into(),
		};

		HTML_TEMPLATE
			.replace("{{title}}", &html_escape::encode_text(title))
			.replace("{{content}}", &inner_html)
			.replace("{{umami}}", &umami)
	}

	pub fn render_html_text_blob(
		text: &str,
		style: &Style,
		link_to: &Option<LinkTo>,
		copyable: bool,
	) -> String {
		let (start, end) = match link_to {
			Some(link_to) => (
				&*format!(
					"<a class=\"[class]\" style=\"[style]\" href={}>",
					link_to.link
				),
				"</a>",
			),
			None => ("<span class=\"[class]\" style=\"[style]\">", "</span>"),
		};
		format!(
			"{}{}{}",
			&start
				.replace("[style]", &style.html_style())
				.replace("[class]", if copyable { "copyable" } else { "" }),
			&html_escape::encode_text(text).replace("\n", "<br>"),
			end
		)
	}

	pub fn render_html_code_block(
		title: &Option<String>,
		language: &Option<String>,
		code: &[Object],
	) -> String {
		let mut output = String::new();
		if title.is_some() || language.is_some() {
			let mut parts = vec![];
			let header_style = Style::new().fg(Bit4Color::CYAN);
			if let Some(t) = title {
				parts.push(format!(
					"Title: {}{}",
					html_escape::encode_text(&t),
					if language.is_some() { "," } else { "" }
				));
			}
			if let Some(lang) = language {
				parts.push(format!("Lang: {}", html_escape::encode_text(&lang)));
			}

			output.push_str(&format!(
				"<div style=\"{}\">{}</div>",
				header_style.html_style(),
				parts.join(" ")
			));
		}

		output.push_str("<pre><code>");
		output.push_str(
			&code
				.iter()
				.map(Self::render_html_object)
				.collect::<Vec<_>>()
				.join(""),
		);
		output.push_str("</code></pre>");
		output
	}

	fn render_html_image(url: &str, alt: &str, width: i64, height: i64) -> String {
		format!(
			"<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
			url,
			html_escape::encode_text(&alt),
			width,
			height,
		)
	}

	fn render_html_canvas(data: &str, color_mapping: &ColorMapping) -> Option<String> {
		let mut output = String::new();
		let mut buffer = String::new();

		for ch in data.chars() {
			buffer.push(ch);

			if buffer == "NL" {
				output.push('\n');
				buffer.clear();
				continue;
			}

			if let Some(&color) = color_mapping.get(&buffer) {
				output.push_str(&format!(
					"<span style=\"{}\">{}</span>",
					Style::new().fg(color).bg(color).html_style(),
					&html_escape::encode_text(" ")
				));
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

	pub fn render_html_object(obj: &Object) -> String {
		match obj {
			super::Object::TextBlob {
				text,
				style,
				link_to,
				copyable,
			} => Self::render_html_text_blob(text, style, link_to, *copyable),
			super::Object::CodeBlock {
				title,
				language,
				code,
			} => Self::render_html_code_block(title, language, code),
			super::Object::Canvas {
				data,
				color_mapping,
			} => Self::render_html_canvas(data, color_mapping).unwrap_or(
				Self::render_html_text_blob(
					"Error rendering Canvas",
					&Style::new().fg(Bit4Color::RED),
					&None,
					true,
				),
			),
			super::Object::Image {
				url,
				alt,
				width,
				height,
			} => Self::render_html_image(url, alt, *width, *height),
		}
	}

	pub fn render_html(&self) -> String {
		self.objects
			.iter()
			.map(Self::render_html_object)
			.collect::<Vec<_>>()
			.join("")
	}
}
