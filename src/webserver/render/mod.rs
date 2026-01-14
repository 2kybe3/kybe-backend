mod ansi;
mod builders;
mod color;
mod html;

pub use builders::{CodeBlockBuilder, LinkToBuilder, TextBlobBuilder};
pub use color::{Color, Style};

use crate::webserver::render::builders::{NoLanguage, NoLink, NoStyle, NoTitle};

pub struct LinkTo<'a> {
	link: &'a str,
	seperator_style: Option<Style>,
	link_style: Option<Style>,
}

pub enum Object<'a> {
	TextBlob {
		text: &'a str,
		style: Style,
		link_to: Option<LinkTo<'a>>,
	},
	CodeBlock {
		title: Option<&'a str>,
		language: Option<&'a str>,
		code: &'a str,
	},
}

impl<'a> Object<'a> {
	pub fn text(text: &'a str) -> TextBlobBuilder<'a, NoStyle, NoLink> {
		TextBlobBuilder::new(text)
	}

	pub fn code(code: &'a str) -> CodeBlockBuilder<'a, NoTitle, NoLanguage> {
		CodeBlockBuilder::new(code)
	}
}

pub struct Page<'a> {
	objects: Vec<Object<'a>>,
}

impl<'a> Page<'a> {
	pub fn new(objects: Vec<Object<'a>>) -> Page<'a> {
		Page { objects }
	}
}

impl<'a> FromIterator<Object<'a>> for Page<'a> {
	fn from_iter<T: IntoIterator<Item = Object<'a>>>(iter: T) -> Self {
		Self::new(iter.into_iter().collect())
	}
}
