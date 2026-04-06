use crate::webserver::render::{LinkTo, Style};

pub struct LinkToBuilder {
	link: String,
	separator_style: Option<Style>,
	link_style: Option<Style>,
}

impl LinkToBuilder {
	pub fn new(link: impl Into<String>) -> LinkToBuilder {
		LinkToBuilder {
			link: link.into(),
			separator_style: None,
			link_style: None,
		}
	}

	pub fn separator_style(self, separator_style: Style) -> LinkToBuilder {
		LinkToBuilder {
			link: self.link,
			separator_style: Some(separator_style),
			link_style: self.link_style,
		}
	}

	pub fn link_style(self, link_style: Style) -> LinkToBuilder {
		LinkToBuilder {
			link: self.link,
			separator_style: self.separator_style,
			link_style: Some(link_style),
		}
	}
}

impl From<LinkToBuilder> for LinkTo {
	fn from(l: LinkToBuilder) -> LinkTo {
		LinkTo {
			link: l.link,
			separator_style: l.separator_style,
			link_style: l.link_style,
		}
	}
}
