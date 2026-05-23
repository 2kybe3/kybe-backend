use crate::webserver::render::color::{ColorTrait, bit24::Bit24Color};

impl Bit4Code {
	pub fn fg(&self) -> u8 {
		let start = if self.0 <= 7 { 30 } else { 90 - 8 };
		start + self.0
	}

	pub fn bg(&self) -> u8 {
		let start = if self.0 <= 7 { 40 } else { 100 - 8 };
		start + self.0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bit4Code(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bit4Color(Option<(Bit4Code, Bit24Color)>);

impl Bit4Color {
	pub const BLACK: Self = Self::new(0, Bit24Color::BLACK);
	pub const RED: Self = Self::new(1, Bit24Color::RED);
	pub const GREEN: Self = Self::new(2, Bit24Color::GREEN);
	pub const YELLOW: Self = Self::new(3, Bit24Color::YELLOW);
	pub const BLUE: Self = Self::new(4, Bit24Color::BLUE);
	pub const MAGENTA: Self = Self::new(5, Bit24Color::MAGENTA);
	pub const CYAN: Self = Self::new(6, Bit24Color::CYAN);
	pub const WHITE: Self = Self::new(7, Bit24Color::WHITE);

	pub const BRIGHT_BLACK: Self = Self::new(8, Bit24Color::BRIGHT_BLACK);
	pub const BRIGHT_RED: Self = Self::new(9, Bit24Color::BRIGHT_RED);
	pub const BRIGHT_GREEN: Self = Self::new(10, Bit24Color::BRIGHT_GREEN);
	pub const BRIGHT_YELLOW: Self = Self::new(11, Bit24Color::BRIGHT_YELLOW);
	pub const BRIGHT_BLUE: Self = Self::new(12, Bit24Color::BRIGHT_BLUE);
	pub const BRIGHT_MAGENTA: Self = Self::new(13, Bit24Color::BRIGHT_MAGENTA);
	pub const BRIGHT_CYAN: Self = Self::new(14, Bit24Color::BRIGHT_CYAN);
	pub const BRIGHT_WHITE: Self = Self::new(15, Bit24Color::BRIGHT_WHITE);

	pub const DEFAULT: Self = Self(None);

	const fn new(code: u8, bit24: Bit24Color) -> Self {
		Self(Some((Bit4Code(code), bit24)))
	}
}

impl ColorTrait for Bit4Color {
	fn is_default(&self) -> bool {
		self.0.is_none()
	}

	fn hex(&self) -> Option<String> {
		if let Some((_code, bit24)) = self.0 {
			bit24.hex()
		} else {
			None
		}
	}

	fn ansi(&self, fg: bool) -> Option<String> {
		if !self.is_default()
			&& let Some((code, _bit24)) = self.0
		{
			Some(format!("\x1b[{}m", if fg { code.fg() } else { code.bg() }))
		} else {
			None
		}
	}
}

impl Default for Bit4Color {
	fn default() -> Self {
		Self::DEFAULT
	}
}
