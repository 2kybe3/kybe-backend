use std::marker::PhantomData;

use crate::webserver::render::{LinkTo, Object, Style};

pub struct NoStyle;
pub struct HasStyle;

pub struct NoLink;
pub struct HasLink;

pub struct TextBlobBuilder<'a, S, L> {
	text: &'a str,
	style: Option<Style>,
	link_to: Option<LinkTo<'a>>,
	_state: PhantomData<(S, L)>,
}

impl<'a> TextBlobBuilder<'a, NoStyle, NoLink> {
	pub fn new(text: &'a str) -> Self {
		Self {
			text,
			style: None,
			link_to: None,
			_state: PhantomData::<(NoStyle, NoLink)>,
		}
	}
}

impl<'a, L> TextBlobBuilder<'a, NoStyle, L> {
	pub fn style(self, style: Style) -> TextBlobBuilder<'a, HasStyle, L> {
		TextBlobBuilder {
			text: self.text,
			style: Some(style),
			link_to: self.link_to,
			_state: PhantomData::<(HasStyle, L)>,
		}
	}
}

impl<'a, S> TextBlobBuilder<'a, S, NoLink> {
	pub fn link_to(self, link_to: LinkTo<'a>) -> TextBlobBuilder<'a, S, HasLink> {
		TextBlobBuilder {
			text: self.text,
			style: self.style,
			link_to: Some(link_to),
			_state: PhantomData::<(S, HasLink)>,
		}
	}
}

impl<'a, S, L> From<TextBlobBuilder<'a, S, L>> for Object<'a> {
	fn from(value: TextBlobBuilder<'a, S, L>) -> Object<'a> {
		Object::TextBlob {
			text: value.text,
			style: value.style.unwrap_or_default(),
			link_to: value.link_to,
		}
	}
}
