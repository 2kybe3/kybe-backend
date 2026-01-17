mod ansi;
pub mod builders;
mod color;
mod html;
mod theme;

use std::collections::HashMap;

use crate::webserver::render::builders::{CodeBlockBuilder, NoLanguage, NoTitle};
pub use color::{Color, Style};
pub use theme::Theme;

pub struct LinkTo {
	link: String,
	seperator_style: Option<Style>,
	link_style: Option<Style>,
}

type ColorMapping = HashMap<String, Color>;

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
	Canvas {
		data: String,
		color_mapping: ColorMapping,
	},
}

impl Object {
	pub fn code(code: impl Into<String>) -> CodeBlockBuilder<NoTitle, NoLanguage> {
		CodeBlockBuilder::new(code)
	}
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
