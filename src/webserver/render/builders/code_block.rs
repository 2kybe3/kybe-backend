use std::marker::PhantomData;

use crate::webserver::render::Object;

pub struct HasTitle;
pub struct NoTitle;

pub struct HasLanguage;
pub struct NoLanguage;

pub struct CodeBlockBuilder<'a, T, L> {
	title: Option<&'a str>,
	language: Option<&'a str>,
	code: &'a str,
	_state: PhantomData<(T, L)>,
}

impl<'a> CodeBlockBuilder<'a, NoTitle, NoLanguage> {
	pub fn new(code: &'a str) -> CodeBlockBuilder<'a, NoTitle, NoLanguage> {
		CodeBlockBuilder {
			title: None,
			language: None,
			code,
			_state: PhantomData::<(NoTitle, NoLanguage)>,
		}
	}
}

impl<'a, T> CodeBlockBuilder<'a, T, NoLanguage> {
	pub fn language(self, language: &'a str) -> CodeBlockBuilder<'a, T, HasLanguage> {
		CodeBlockBuilder {
			title: self.title,
			language: Some(language),
			code: self.code,
			_state: PhantomData::<(T, HasLanguage)>,
		}
	}
}

impl<'a, L> CodeBlockBuilder<'a, NoTitle, L> {
	pub fn title(self, title: &'a str) -> CodeBlockBuilder<'a, HasTitle, L> {
		CodeBlockBuilder {
			title: Some(title),
			language: self.language,
			code: self.code,
			_state: PhantomData::<(HasTitle, L)>,
		}
	}
}

impl<'a, T, L> CodeBlockBuilder<'a, T, L> {
	pub fn build(self) -> Object<'a> {
		Object::CodeBlock {
			title: self.title,
			language: self.language,
			code: self.code,
		}
	}
}
