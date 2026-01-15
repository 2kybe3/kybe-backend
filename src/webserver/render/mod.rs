mod ansi;
mod builders;
mod color;
mod html;

use std::collections::HashMap;

pub use builders::{
	COLOR_MAPPING, CanvasBuilder, CodeBlockBuilder, LinkToBuilder, TextBlobBuilder,
};
pub use color::{Color, Style};

use crate::webserver::render::builders::{NoLanguage, NoLink, NoStyle, NoTitle};

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
	pub fn text(text: impl Into<String>) -> TextBlobBuilder<NoStyle, NoLink> {
		TextBlobBuilder::new(text)
	}

	pub fn code(code: impl Into<String>) -> CodeBlockBuilder<NoTitle, NoLanguage> {
		CodeBlockBuilder::new(code)
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

impl FromIterator<Object> for Page {
	fn from_iter<T: IntoIterator<Item = Object>>(iter: T) -> Self {
		Self::new(iter.into_iter().collect())
	}
}
