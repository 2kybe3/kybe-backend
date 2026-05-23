use std::collections::HashMap;

use crate::webserver::render::{Color, Style};

pub type ColorMapping = HashMap<String, Color>;

pub struct LinkTo {
	link: String,
	separator_style: Option<Style>,
	link_style: Option<Style>,
}

impl LinkTo {
	pub fn new(link: String, separator_style: Option<Style>, link_style: Option<Style>) -> Self {
		Self {
			link,
			separator_style,
			link_style,
		}
	}
	pub fn link(&self) -> &str {
		&self.link
	}

	pub fn separator_style(&self) -> Option<&Style> {
		self.separator_style.as_ref()
	}

	pub fn link_style(&self) -> Option<&Style> {
		self.link_style.as_ref()
	}
}

pub enum Object {
	TextBlob {
		text: String,
		style: Style,
		link_to: Option<LinkTo>,
	},
	CodeBlock {
		title: Option<String>,
		language: Option<String>,
		code: String,
	},
	Image {
		url: String,
		alt: String,
		width: i64,
		height: i64,
	},
	Canvas {
		data: String,
		color_mapping: ColorMapping,
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
