use crate::webserver::render::color::{ColorTrait, rgb::Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bit24Color(Option<Rgb>);

// https://github.com/kovidgoyal/kitty/blob/a17b2df580d7df9004d116c66b4c9c365125784a/kitty/colors.c#L20
impl Bit24Color {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const RED: Self = Self::new(205, 0, 0);
    pub const GREEN: Self = Self::new(0, 205, 0);
    pub const YELLOW: Self = Self::new(205, 205, 0);
    pub const BLUE: Self = Self::new(0, 0, 238);
    pub const MAGENTA: Self = Self::new(205, 0, 205);
    pub const CYAN: Self = Self::new(0, 205, 205);
    pub const WHITE: Self = Self::new(229, 229, 229);

    pub const BRIGHT_BLACK: Self = Self::new(127, 127, 127);
    pub const BRIGHT_RED: Self = Self::new(255, 0, 0);
    pub const BRIGHT_GREEN: Self = Self::new(0, 255, 0);
    pub const BRIGHT_YELLOW: Self = Self::new(255, 255, 0);
    pub const BRIGHT_BLUE: Self = Self::new(92, 92, 255);
    pub const BRIGHT_MAGENTA: Self = Self::new(255, 0, 255);
    pub const BRIGHT_CYAN: Self = Self::new(0, 255, 255);
    pub const BRIGHT_WHITE: Self = Self::new(255, 255, 255);

    pub const DEFAULT: Self = Self(None);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self(Some(Rgb::new(red, green, blue)))
    }
}

impl ColorTrait for Bit24Color {
    fn is_default(&self) -> bool {
        self.0.is_none()
    }

    fn hex(&self) -> Option<String> {
        self.0
            .map(|rgb| format!("#{:02X}{:02X}{:02X}", rgb.red(), rgb.green(), rgb.blue()))
    }

    fn ansi(&self, fg: bool) -> Option<String> {
        if !self.is_default()
            && let Some(rgb) = self.0
        {
            let code = if fg { 38 } else { 48 };
            Some(format!(
                "\x1b[{code};2;{};{};{}m",
                rgb.red(),
                rgb.green(),
                rgb.blue()
            ))
        } else {
            None
        }
    }
}

impl Default for Bit24Color {
    fn default() -> Self {
        Self::DEFAULT
    }
}
