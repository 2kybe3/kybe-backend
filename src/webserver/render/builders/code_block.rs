use crate::webserver::render::Object;
use std::marker::PhantomData;
pub struct HasTitle;
pub struct NoTitle;

pub struct HasLanguage;
pub struct NoLanguage;

pub struct CodeBlockBuilder<T, L> {
	title: Option<String>,
	language: Option<String>,
	code: String,
	_state: PhantomData<(T, L)>,
}

impl CodeBlockBuilder<NoTitle, NoLanguage> {
	pub fn new(code: impl Into<String>) -> CodeBlockBuilder<NoTitle, NoLanguage> {
		CodeBlockBuilder {
			title: None,
			language: None,
			code: code.into(),
			_state: PhantomData::<(NoTitle, NoLanguage)>,
		}
	}
}

impl<T> CodeBlockBuilder<T, NoLanguage> {
	pub fn language(self, language: impl Into<String>) -> CodeBlockBuilder<T, HasLanguage> {
		CodeBlockBuilder {
			title: self.title,
			language: Some(language.into()),
			code: self.code,
			_state: PhantomData::<(T, HasLanguage)>,
		}
	}
}

impl<L> CodeBlockBuilder<NoTitle, L> {
	pub fn title(self, title: impl Into<String>) -> CodeBlockBuilder<HasTitle, L> {
		CodeBlockBuilder {
			title: Some(title.into()),
			language: self.language,
			code: self.code,
			_state: PhantomData::<(HasTitle, L)>,
		}
	}
}

impl<T, L> From<CodeBlockBuilder<T, L>> for Object {
	fn from(b: CodeBlockBuilder<T, L>) -> Self {
		Object::CodeBlock {
			title: b.title,
			language: b.language,
			code: b.code,
		}
	}
}
