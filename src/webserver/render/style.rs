use crate::webserver::render::Color;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
	pub fg: Color,
	pub bg: Color,
	pub bold: bool,
	pub dim: bool,
}

impl Style {
	pub fn new() -> Self {
		Self {
			fg: Color::DEFAULT,
			bg: Color::DEFAULT,
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
	pub fn ansi_code(self) -> String {
		let reset = "\x1b[0m";
		let bold = if self.bold { "\x1b[2m" } else { "" };
		let dim = if self.dim { "\x1b[3m" } else { "" };
		let fg = self.fg.ansi_fg();
		let bg = self.bg.ansi_bg();

		format!("{reset}{bold}{dim}{fg}{bg}")
	}
}

// HTML
impl Style {
	pub fn html_style(&self) -> String {
		let mut styles = vec![
			format!("color:{}", self.fg.html()),
			format!("background-color:{}", self.bg.html()),
		];
		if self.bold {
			styles.push("font-weight:bold".into());
		}
		if self.dim {
			styles.push("opacity:0.6".into());
		}
		styles.join("; ")
	}
}
