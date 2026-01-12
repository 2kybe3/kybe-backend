mod ansi;
mod color;
mod html;

pub use color::{Color, Style};

#[allow(unused)]
pub enum Object<'a> {
	TextBlob {
		text: &'a str,
		style: Style,
		link_to: Option<&'a str>,
	},
	CodeBlock {
		title: Option<&'a str>,
		language: Option<&'a str>,
		code: &'a str,
	},
}

pub struct Page<'a> {
	pub objects: Vec<Object<'a>>,
}

impl<'a> Page<'a> {
	pub fn new(objects: Vec<Object<'a>>) -> Page<'a> {
		Page { objects }
	}
}
