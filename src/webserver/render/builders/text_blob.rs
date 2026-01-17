use std::marker::PhantomData;

use crate::webserver::render::{LinkTo, Object, Style};

pub struct NoStyle;
pub struct HasStyle;

pub struct NoLink;
pub struct HasLink;

pub struct TextBlobBuilder<S, L> {
	text: String,
	style: Option<Style>,
	link_to: Option<LinkTo>,
	_state: PhantomData<(S, L)>,
}

impl TextBlobBuilder<NoStyle, NoLink> {
	pub fn new(text: impl Into<String>) -> Self {
		Self {
			text: text.into(),
			style: None,
			link_to: None,
			_state: PhantomData::<(NoStyle, NoLink)>,
		}
	}
}

impl<L> TextBlobBuilder<NoStyle, L> {
	pub fn style(self, style: Style) -> TextBlobBuilder<HasStyle, L> {
		TextBlobBuilder {
			text: self.text,
			style: Some(style),
			link_to: self.link_to,
			_state: PhantomData::<(HasStyle, L)>,
		}
	}
}

impl<S> TextBlobBuilder<S, NoLink> {
	pub fn link_to(self, link_to: LinkTo) -> TextBlobBuilder<S, HasLink> {
		TextBlobBuilder {
			text: self.text,
			style: self.style,
			link_to: Some(link_to),
			_state: PhantomData::<(S, HasLink)>,
		}
	}
}

impl<S, L> From<TextBlobBuilder<S, L>> for Object {
	fn from(t: TextBlobBuilder<S, L>) -> Self {
		Object::TextBlob {
			text: t.text,
			style: t.style.unwrap_or_default(),
			link_to: t.link_to,
		}
	}
}

impl From<&str> for TextBlobBuilder<NoStyle, NoLink> {
	fn from(text: &str) -> Self {
		Self::new(text)
	}
}
