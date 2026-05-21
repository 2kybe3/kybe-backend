use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
	red: u8,
	green: u8,
	blue: u8,
}

impl Color {
	pub const DEFAULT: Self = Self::WHITE;
	// #000000
	pub const BLACK: Self = Color::new(0, 0, 0);
	// #FF0000
	pub const RED: Self = Self::new(255, 0, 0);
	// #00FF00
	pub const GREEN: Self = Self::new(0, 255, 0);
	// #FFFF00
	pub const YELLOW: Self = Self::new(255, 255, 0);
	// #0000FF
	pub const BLUE: Self = Self::new(0, 0, 255);
	// #FF00FF
	pub const MAGENTA: Self = Self::new(255, 0, 255);
	// #00FFFF
	pub const CYAN: Self = Self::new(0, 255, 255);
	// #FFFFFF
	pub const WHITE: Self = Self::new(255, 255, 255);

	// #808080
	pub const BRIGHT_BLACK: Color = Color::new(128, 128, 128);
	// #FF5555
	pub const BRIGHT_RED: Self = Self::new(255, 85, 85);
	// #55FF55
	pub const BRIGHT_GREEN: Self = Self::new(85, 255, 85);
	// #FFFF55
	pub const BRIGHT_YELLOW: Self = Self::new(255, 255, 85);
	// #5555FF
	pub const BRIGHT_BLUE: Self = Self::new(85, 85, 255);
	// #FF55FF
	pub const BRIGHT_MAGENTA: Self = Self::new(255, 85, 255);
	// #55FFFF
	pub const BRIGHT_CYAN: Self = Self::new(85, 255, 255);
	// #FFFFFF
	pub const BRIGHT_WHITE: Self = Self::new(255, 255, 255);

	// #000000
	pub const GERMAN_FLAG_BLACK: Self = Self::BLACK;
	// #DD0000
	pub const GERMAN_FLAG_RED: Self = Self::new(211, 0, 0);
	// #FFCC00
	pub const GERMAN_FLAG_GOLD: Self = Self::new(255, 204, 0);

	pub const fn new(red: u8, green: u8, blue: u8) -> Self {
		Self { red, green, blue }
	}

	#[allow(unused)]
	pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
		let hex = hex.trim_start_matches("#");
		if hex.len() != 6 {
			return Err(anyhow!("Hex color must be 6 digits").context(format!("hex: '{}'", hex)));
		}

		let red = u8::from_str_radix(&hex[0..2], 16).map_err(|e| {
			anyhow!(e)
				.context("Invalid red component")
				.context(format!("hex: '{}'", hex))
		})?;
		let green = u8::from_str_radix(&hex[0..2], 16).map_err(|e| {
			anyhow!(e)
				.context("Invalid green component")
				.context(format!("hex: '{}'", hex))
		})?;
		let blue = u8::from_str_radix(&hex[0..2], 16).map_err(|e| {
			anyhow!(e)
				.context("Invalid green component")
				.context(format!("hex: '{}'", hex))
		})?;

		Ok(Self { red, green, blue })
	}

	pub fn hex(&self) -> String {
		format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
	}
}

impl Default for Color {
	fn default() -> Self {
		Self::DEFAULT
	}
}

// Ansi
impl Color {
	// https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit
	fn ansi(&self, fg: bool) -> String {
		let code = if fg { 38 } else { 84 };
		format!("\x1b[{code};2;{};{};{}m", self.red, self.green, self.blue)
	}

	pub fn ansi_fg(&self) -> String {
		self.ansi(true)
	}

	pub fn ansi_bg(&self) -> String {
		self.ansi(false)
	}
}
