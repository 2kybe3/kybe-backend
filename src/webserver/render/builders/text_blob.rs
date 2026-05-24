use crate::webserver::render::{Object, Style, object::LinkTo};

pub struct TextBlobBuilder {
    text: String,
    style: Option<Style>,
    link_to: Option<LinkTo>,
}

impl TextBlobBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: None,
            link_to: None,
        }
    }

    pub fn style(self, style: Style) -> TextBlobBuilder {
        TextBlobBuilder {
            text: self.text,
            style: Some(style),
            link_to: self.link_to,
        }
    }

    pub fn link_to(self, link_to: LinkTo) -> TextBlobBuilder {
        TextBlobBuilder {
            text: self.text,
            style: self.style,
            link_to: Some(link_to),
        }
    }
}

impl From<TextBlobBuilder> for Object {
    fn from(t: TextBlobBuilder) -> Self {
        Object::TextBlob {
            text: t.text,
            style: t.style.unwrap_or_default(),
            link_to: t.link_to,
        }
    }
}

impl From<String> for TextBlobBuilder {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for TextBlobBuilder {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
