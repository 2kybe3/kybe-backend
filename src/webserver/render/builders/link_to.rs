use std::marker::PhantomData;

use crate::webserver::render::{LinkTo, Style};

pub struct HasSeperatorStyle;
pub struct NoSeperatorStyle;

#[allow(unused)]
pub struct HasLinkStyle;
pub struct NoLinkStyle;

pub struct LinkToBuilder<'a, S, L> {
	link: &'a str,
	seperator_style: Option<Style>,
	link_style: Option<Style>,
	_state: PhantomData<(S, L)>,
}

impl<'a> LinkToBuilder<'a, NoSeperatorStyle, NoLinkStyle> {
	pub fn new(link: &'a str) -> LinkToBuilder<'a, NoSeperatorStyle, NoLinkStyle> {
		LinkToBuilder {
			link,
			seperator_style: None,
			link_style: None,
			_state: PhantomData::<(NoSeperatorStyle, NoLinkStyle)>,
		}
	}
}

impl<'a, L> LinkToBuilder<'a, NoSeperatorStyle, L> {
	pub fn seperator_style(
		self,
		seperator_style: Style,
	) -> LinkToBuilder<'a, HasSeperatorStyle, L> {
		LinkToBuilder {
			link: self.link,
			seperator_style: Some(seperator_style),
			link_style: self.link_style,
			_state: PhantomData::<(HasSeperatorStyle, L)>,
		}
	}
}

#[allow(unused)]
impl<'a, S> LinkToBuilder<'a, S, NoLinkStyle> {
	pub fn link_style(self, link_style: Style) -> LinkToBuilder<'a, S, HasLinkStyle> {
		LinkToBuilder {
			link: self.link,
			seperator_style: self.seperator_style,
			link_style: Some(link_style),
			_state: PhantomData::<(S, HasLinkStyle)>,
		}
	}
}

impl<'a, S, L> From<LinkToBuilder<'a, S, L>> for LinkTo<'a> {
	fn from(l: LinkToBuilder<'a, S, L>) -> LinkTo<'a> {
		LinkTo {
			link: l.link,
			seperator_style: l.seperator_style,
			link_style: l.link_style,
		}
	}
}
