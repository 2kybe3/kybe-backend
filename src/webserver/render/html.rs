use crate::webserver::render::{
	Object, Page, PageRenderer, Style,
	color::bit4::Bit4Color,
	object::{ColorMapping, LinkTo},
};

const HTML_TEMPLATE: &str = include_str!("../../../assets/template.html");

pub struct HtmlRenderer();

impl<'a> PageRenderer<'a> for HtmlRenderer {
	fn render(page: &Page<'a>) -> String {
		let inner_html = page
			.objects
			.iter()
			.map(Self::render_object)
			.collect::<Vec<_>>()
			.join("");

		let umami = match (
			page.config.webserver.umami.id.clone(),
			page.config.webserver.umami.script_path.clone(),
		) {
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
			.replace("{{title}}", &html_escape::encode_text(page.title))
			.replace("{{content}}", &inner_html)
			.replace("{{umami}}", &umami)
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
		let (start, end) = match link_to {
			Some(link_to) => (
				&*format!("<a style=\"[style]\" href={}>", link_to.link()),
				"</a>",
			),
			None => ("<span style=\"[style]\">", "</span>"),
		};
		format!(
			"{}{}{}",
			&start.replace("[style]", &style.html_style()),
			&html_escape::encode_text(text).replace("\n", "<br>"),
			end
		)
	}

	fn render_code_block(title: &Option<String>, language: &Option<String>, code: &str) -> String {
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

		// TODO: highlighting
		output.push_str(code);

		output.push_str("</code></pre>");
		output
	}

	fn render_image(url: &str, alt: &str, width: &i64, height: &i64) -> String {
		format!(
			"<img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
			url,
			html_escape::encode_text(&alt),
			width,
			height,
		)
	}

	fn render_canvas(data: &str, color_mapping: &ColorMapping) -> String {
		let mut output = String::new();
		let mut buffer = String::new();

		let mut failed = false;

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
				failed = true;
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
