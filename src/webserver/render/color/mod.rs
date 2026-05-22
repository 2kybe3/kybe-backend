use crate::webserver::render::color::{bit4::Bit4Color, bit24::Bit24Color};

pub mod bit24;
pub mod bit4;
pub mod rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[enum_delegate::implement(ColorTrait)]
pub enum Color {
	Bit24Color(Bit24Color),
	Bit4(Bit4Color),
}

impl Default for Color {
	fn default() -> Self {
		Color::Bit4(Bit4Color::DEFAULT)
	}
}

#[enum_delegate::register]
pub trait ColorTrait {
	fn is_default(&self) -> bool;

	fn html(&self) -> Option<String> {
		if self.is_default() { None } else { self.hex() }
	}

	fn ansi(&self, fg: bool) -> Option<String>;

	fn hex(&self) -> Option<String>;

	fn ansi_fg(&self) -> Option<String> {
		self.ansi(true)
	}

	fn ansi_bg(&self) -> Option<String> {
		self.ansi(false)
	}
}
