use crate::webserver::render::Object;

pub struct CodeBlockBuilder {
	title: Option<String>,
	language: Option<String>,
	code: Vec<Object>,
}

impl CodeBlockBuilder {
	pub fn new(code: Vec<Object>) -> CodeBlockBuilder {
		CodeBlockBuilder {
			title: None,
			language: None,
			code: code.into_iter().collect(),
		}
	}

	#[allow(unused)]
	pub fn language(self, language: impl Into<String>) -> CodeBlockBuilder {
		CodeBlockBuilder {
			title: self.title,
			language: Some(language.into()),
			code: self.code,
		}
	}

	pub fn title(self, title: impl Into<String>) -> CodeBlockBuilder {
		CodeBlockBuilder {
			title: Some(title.into()),
			language: self.language,
			code: self.code,
		}
	}
}

impl From<CodeBlockBuilder> for Object {
	fn from(b: CodeBlockBuilder) -> Self {
		Object::CodeBlock {
			title: b.title,
			language: b.language,
			code: b.code,
		}
	}
}
