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
	pub fn new_fg(fg: Color) -> Self {
		Self {
			fg,
			bg: Color::default(),
			bold: false,
			dim: false,
		}
	}

	#[allow(unused)]
	pub fn new_bg(bg: Color) -> Self {
		Self {
			fg: Color::default(),
			bg,
			bold: false,
			dim: false,
		}
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
