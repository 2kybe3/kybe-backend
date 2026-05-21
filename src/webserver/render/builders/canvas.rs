use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::webserver::render::{Color, ColorMapping, Object};

pub static COLOR_MAPPING: Lazy<ColorMapping> = Lazy::new(|| {
	let mut map = HashMap::new();
	map.insert("D".into(), Color::DEFAULT);
	map.insert("BL".into(), Color::BLACK);
	map.insert("R".into(), Color::RED);
	map.insert("G".into(), Color::GREEN);
	map.insert("Y".into(), Color::YELLOW);
	map.insert("BU".into(), Color::BLUE);
	map.insert("M".into(), Color::MAGENTA);
	map.insert("C".into(), Color::CYAN);
	map.insert("W".into(), Color::WHITE);

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
