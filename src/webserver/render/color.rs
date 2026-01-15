#[allow(unused)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Color {
	#[default]
	Default,

	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Magenta,
	Cyan,
	White,

	BrightBlack,
	BrightRed,
	BrightGreen,
	BrightYellow,
	BrightBlue,
	BrightMagenta,
	BrightCyan,
	BrightWhite,
}

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
			fg: Color::default(),
			bg: Color::default(),
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
