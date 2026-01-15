use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::webserver::render::{Color, ColorMapping, Object};

pub static COLOR_MAPPING: Lazy<ColorMapping> = Lazy::new(|| {
	let mut map = HashMap::new();
	map.insert("B".into(), Color::Black);
	map.insert("R".into(), Color::Red);
	map.insert("G".into(), Color::Green);
	map.insert("Y".into(), Color::Yellow);
	map.insert("BL".into(), Color::Blue);
	map.insert("M".into(), Color::Magenta);
	map.insert("C".into(), Color::Cyan);
	map.insert("W".into(), Color::White);

	map.insert("BrB".into(), Color::BrightBlack);
	map.insert("BrR".into(), Color::BrightRed);
	map.insert("BrG".into(), Color::BrightGreen);
	map.insert("BrY".into(), Color::BrightYellow);
	map.insert("BrBL".into(), Color::BrightBlue);
	map.insert("BrM".into(), Color::BrightMagenta);
	map.insert("BrC".into(), Color::BrightCyan);
	map.insert("BrW".into(), Color::BrightWhite);

	map
});

pub struct CanvasBuilder {
	data: String,
	color_mapping: ColorMapping,
}

impl CanvasBuilder {
	pub fn new(data: impl Into<String>) -> CanvasBuilder {
		CanvasBuilder {
			data: data.into(),
			color_mapping: COLOR_MAPPING.clone(),
		}
	}
}

impl From<CanvasBuilder> for Object {
	fn from(c: CanvasBuilder) -> Self {
		Object::Canvas {
			data: c.data,
			color_mapping: c.color_mapping,
		}
	}
}
