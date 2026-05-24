use crate::webserver::render::{
    Color,
    color::{ColorTrait, bit4::Bit4Color},
};

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
}

impl Style {
    pub fn new() -> Self {
        Self {
            fg: Color::Bit4(Bit4Color::DEFAULT),
            bg: Color::Bit4(Bit4Color::DEFAULT),
            bold: false,
            dim: false,
        }
    }

    pub fn fg(mut self, fg: impl Into<Color>) -> Self {
        self.fg = fg.into();
        self
    }

    pub fn bg(mut self, bg: impl Into<Color>) -> Self {
        self.bg = bg.into();
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn dim(mut self, dim: bool) -> Self {
        self.dim = dim;
        self
    }
}

// Ansi
impl Style {
    pub fn ansi_code(&self) -> String {
        let reset = "\x1b[0m";
        let bold = if self.bold { "\x1b[2m" } else { "" };
        let dim = if self.dim { "\x1b[3m" } else { "" };

        let fg = self.fg.ansi_fg().unwrap_or_default();
        let bg = self.bg.ansi_bg().unwrap_or_default();

        format!("{reset}{bold}{dim}{fg}{bg}")
    }
}

// HTML
impl Style {
    pub fn html_style(&self) -> String {
        let fg_html = self.fg.html();
        let bg_html = self.bg.html();

        let mut styles = Vec::with_capacity(
            self.bold as usize
                + self.dim as usize
                + fg_html.is_some() as usize
                + bg_html.is_some() as usize,
        );

        if self.bold {
            styles.push("font-weight:bold".into());
        }

        if self.dim {
            styles.push("opacity:0.6".into());
        }

        if let Some(fg_html) = fg_html {
            styles.push(format!("color:{fg_html}"));
        }

        if let Some(bg_html) = bg_html {
            styles.push(format!("background-color:{bg_html}"));
        }

        styles.join(";")
    }
}
