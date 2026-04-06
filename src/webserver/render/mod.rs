mod ansi;
pub mod builders;
mod color;
mod html;
mod theme;

use std::collections::HashMap;

pub use color::{Color, Style};
pub use theme::Theme;

use crate::config::types::UmamiConfig;

pub struct LinkTo {
	link: String,
	separator_style: Option<Style>,
	link_style: Option<Style>,
}

type ColorMapping = HashMap<String, Color>;

pub enum Object {
	TextBlob {
		text: String,
		style: Style,
		link_to: Option<LinkTo>,
		copyable: bool,
	},
	CodeBlock {
		title: Option<String>,
		language: Option<String>,
		code: Vec<Object>,
	},
	Canvas {
		data: String,
		color_mapping: ColorMapping,
	},
	Image {
		url: String,
		alt: String,
		width: i64,
		height: i64,
	},
}

pub enum Objects {
	One(Object),
	Many(Vec<Object>),
}

impl IntoIterator for Objects {
	type Item = Object;
	type IntoIter = std::vec::IntoIter<Object>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Objects::One(obj) => vec![obj].into_iter(),
			Objects::Many(vec) => vec.into_iter(),
		}
	}
}

impl From<Vec<Object>> for Objects {
	fn from(v: Vec<Object>) -> Self {
		Objects::Many(v)
	}
}

impl<T: Into<Object>> From<T> for Objects {
	fn from(o: T) -> Self {
		Objects::One(o.into())
	}
}

pub struct Page {
	objects: Vec<Object>,
}

impl Page {
	pub fn new(objects: Vec<Object>) -> Page {
		Page { objects }
	}

	// (is_html, data)
	pub fn render(self, user_agent: &str, title: &str, umami: &UmamiConfig) -> (bool, String) {
		if user_agent.contains("curl") {
			(false, self.render_ansi())
		} else {
			(true, self.render_html_page(title, umami))
		}
	}
}

impl Page {
	pub fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = Objects>,
	{
		let objects = iter.into_iter().flatten().collect();
		Self::new(objects)
	}
}
