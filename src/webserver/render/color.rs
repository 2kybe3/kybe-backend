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

	pub fn fg(mut self, fg: Color) -> Self {
		self.fg = fg;
		self
	}

	pub fn bg(mut self, bg: Color) -> Self {
		self.bg = bg;
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
