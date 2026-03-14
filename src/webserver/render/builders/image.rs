use crate::webserver::render::Object;

pub struct ImageBuilder {
	url: String,
	alt: String,
	width: i64,
	height: i64,
}

impl ImageBuilder {
	pub fn new(url: impl Into<String>, alt: impl Into<String>, width: i64, height: i64) -> Self {
		Self {
			url: url.into(),
			alt: alt.into(),
			width,
			height,
		}
	}
}

impl From<ImageBuilder> for Object {
	fn from(b: ImageBuilder) -> Self {
		Object::Image {
			url: b.url,
			alt: b.alt,
			width: b.width,
			height: b.height,
		}
	}
}
