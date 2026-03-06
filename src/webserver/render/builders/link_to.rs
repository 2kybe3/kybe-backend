use crate::webserver::render::{LinkTo, Style};

pub struct LinkToBuilder {
	link: String,
	seperator_style: Option<Style>,
	link_style: Option<Style>,
}

impl LinkToBuilder {
	pub fn new(link: impl Into<String>) -> LinkToBuilder {
		LinkToBuilder {
			link: link.into(),
			seperator_style: None,
			link_style: None,
		}
	}

	pub fn seperator_style(self, seperator_style: Style) -> LinkToBuilder {
		LinkToBuilder {
			link: self.link,
			seperator_style: Some(seperator_style),
			link_style: self.link_style,
		}
	}

	pub fn link_style(self, link_style: Style) -> LinkToBuilder {
		LinkToBuilder {
			link: self.link,
			seperator_style: self.seperator_style,
			link_style: Some(link_style),
		}
	}
}

impl From<LinkToBuilder> for LinkTo {
	fn from(l: LinkToBuilder) -> LinkTo {
		LinkTo {
			link: l.link,
			seperator_style: l.seperator_style,
			link_style: l.link_style,
		}
	}
}
