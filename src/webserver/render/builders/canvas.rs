use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::webserver::render::{Object, color::bit4::Bit4Color, object::ColorMapping};

pub static COLOR_MAPPING: Lazy<ColorMapping> = Lazy::new(|| {
	let mut map = HashMap::new();
	map.insert("D".into(), Bit4Color::DEFAULT.into());
	map.insert("BL".into(), Bit4Color::BLACK.into());
	map.insert("R".into(), Bit4Color::RED.into());
	map.insert("G".into(), Bit4Color::GREEN.into());
	map.insert("Y".into(), Bit4Color::YELLOW.into());
	map.insert("BU".into(), Bit4Color::BLUE.into());
	map.insert("M".into(), Bit4Color::MAGENTA.into());
	map.insert("C".into(), Bit4Color::CYAN.into());
	map.insert("W".into(), Bit4Color::WHITE.into());

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
