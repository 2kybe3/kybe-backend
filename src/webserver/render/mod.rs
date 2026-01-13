mod ansi;
mod builders;
mod color;
mod html;

pub use builders::{CodeBlockBuilder, TextBlobBuilder};
pub use color::{Color, Style};

pub enum Object<'a> {
	TextBlob {
		text: &'a str,
		style: Style,
		// TODO: add more options like (link color seperator color (ascii render))
		link_to: Option<&'a str>,
	},
	CodeBlock {
		title: Option<&'a str>,
		language: Option<&'a str>,
		code: &'a str,
	},
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
