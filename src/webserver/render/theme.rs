use crate::webserver::render::{
    Object, Style,
    builders::{LinkToBuilder, TextBlobBuilder},
    color::bit4::Bit4Color,
};

pub struct Theme {
    pub title: Style,
    pub subtitle: Style,
    pub label: Style,
    pub text: Style,
    pub link: Style,
    pub link_separator: Style,
    #[allow(unused)]
    pub comment: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            title: Style::new().fg(Bit4Color::RED),
            subtitle: Style::new().fg(Bit4Color::BRIGHT_RED),
            label: Style::new().fg(Bit4Color::YELLOW),
            text: Style::new().fg(Bit4Color::WHITE),
            link: Style::new().fg(Bit4Color::GREEN),
            link_separator: Style::new().fg(Bit4Color::WHITE),
            comment: Style::new().fg(Bit4Color::BLACK).dim(true),
        }
    }
}

impl Theme {
    #[allow(unused)]
    pub fn raw(&self, text: impl Into<TextBlobBuilder>) -> TextBlobBuilder {
        text.into()
    }

    pub fn title(&self, text: impl Into<TextBlobBuilder>) -> TextBlobBuilder {
        text.into().style(self.title.clone())
    }

    pub fn subtitle(&self, text: impl Into<TextBlobBuilder>) -> TextBlobBuilder {
        text.into().style(self.subtitle.clone())
    }

    pub fn text(&self, text: impl Into<TextBlobBuilder>) -> TextBlobBuilder {
        text.into().style(self.text.clone())
    }

    #[allow(unused)]
    pub fn comment(&self, text: impl Into<TextBlobBuilder>) -> TextBlobBuilder {
        text.into().style(self.comment.clone())
    }

    pub fn link_colored(&self, text: impl Into<TextBlobBuilder>, link: &str) -> TextBlobBuilder {
        text.into().style(self.link.clone()).link_to(
            LinkToBuilder::new(link)
                .link_style(self.link.clone())
                .separator_style(self.link_separator.clone())
                .into(),
        )
    }

    #[allow(unused)]
    pub fn link(&self, text: impl Into<TextBlobBuilder>, link: &str) -> TextBlobBuilder {
        text.into().link_to(
            LinkToBuilder::new(link)
                .link_style(self.link.clone())
                .separator_style(self.link_separator.clone())
                .into(),
        )
    }

    fn label_text(&self, title: &str) -> Object {
        TextBlobBuilder::new(format!("{}: ", title))
            .style(self.label.clone())
            .into()
    }

    pub fn label(&self, title: &str, data: Vec<Object>) -> Vec<Object> {
        let mut output: Vec<Object> = vec![self.label_text(title)];

        output.extend(data);

        output
    }
}
