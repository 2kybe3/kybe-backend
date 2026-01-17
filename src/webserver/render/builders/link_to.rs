use std::marker::PhantomData;

use crate::webserver::render::{LinkTo, Style};

pub struct HasSeperatorStyle;
pub struct NoSeperatorStyle;

#[allow(unused)]
pub struct HasLinkStyle;
pub struct NoLinkStyle;

pub struct LinkToBuilder<S, L> {
	link: String,
	seperator_style: Option<Style>,
	link_style: Option<Style>,
	_state: PhantomData<(S, L)>,
}

impl LinkToBuilder<NoSeperatorStyle, NoLinkStyle> {
	pub fn new(link: impl Into<String>) -> LinkToBuilder<NoSeperatorStyle, NoLinkStyle> {
		LinkToBuilder {
			link: link.into(),
			seperator_style: None,
			link_style: None,
			_state: PhantomData::<(NoSeperatorStyle, NoLinkStyle)>,
		}
	}
}

impl<L> LinkToBuilder<NoSeperatorStyle, L> {
	pub fn seperator_style(self, seperator_style: Style) -> LinkToBuilder<HasSeperatorStyle, L> {
		LinkToBuilder {
			link: self.link,
			seperator_style: Some(seperator_style),
			link_style: self.link_style,
			_state: PhantomData::<(HasSeperatorStyle, L)>,
		}
	}
}

impl<S> LinkToBuilder<S, NoLinkStyle> {
	pub fn link_style(self, link_style: Style) -> LinkToBuilder<S, HasLinkStyle> {
		LinkToBuilder {
			link: self.link,
			seperator_style: self.seperator_style,
			link_style: Some(link_style),
			_state: PhantomData::<(S, HasLinkStyle)>,
		}
	}
}

impl<S, L> From<LinkToBuilder<S, L>> for LinkTo {
	fn from(l: LinkToBuilder<S, L>) -> LinkTo {
		LinkTo {
			link: l.link,
			seperator_style: l.seperator_style,
			link_style: l.link_style,
		}
	}
}
