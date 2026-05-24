mod ansi;
pub mod builders;
pub mod color;
mod html;
pub mod object;
mod style;
mod theme;

pub use color::Color;
pub use style::Style;
pub use theme::Theme;

use crate::{
    config::types::Config,
    webserver::render::{
        ansi::AnsiRenderer,
        html::HtmlRenderer,
        object::{ColorMapping, LinkTo, Object, Objects},
    },
};

pub struct Page<'a> {
    title: &'a str,
    config: &'a Config,
    objects: Vec<Object>,
}

pub struct RenderResult {
    data: String,
    content_type: String,
}

pub trait PageRenderer<'a> {
    fn render(page: &Page<'a>) -> String;
    fn render_object(obj: &Object) -> String;
    fn render_text_blob(text: &str, style: &Style, link_to: &Option<LinkTo>) -> String;
    fn render_code_block(title: &Option<String>, language: &Option<String>, code: &str) -> String;
    fn render_image(url: &str, alt: &str, width: &i64, height: &i64) -> String;
    fn render_canvas(data: &str, color_mapping: &ColorMapping) -> String;
}

impl RenderResult {
    pub fn new(page: Page, user_agent: &str) -> RenderResult {
        match user_agent_is_cli(user_agent) {
            true => RenderResult {
                data: AnsiRenderer::render(&page),
                content_type: "text/:-)".into(),
            },
            false => RenderResult {
                data: HtmlRenderer::render(&page),
                content_type: "text/html".into(),
            },
        }
    }

    pub fn take_content_type(&mut self) -> String {
        std::mem::take(&mut self.content_type)
    }

    pub fn take_data(&mut self) -> String {
        std::mem::take(&mut self.data)
    }
}

impl<'a> Page<'a> {
    pub fn new(title: &'a str, config: &'a Config, objects: Vec<Object>) -> Page<'a> {
        Page {
            title,
            config,
            objects,
        }
    }

    pub fn render(self, user_agent: &str) -> RenderResult {
        RenderResult::new(self, user_agent)
    }
}

impl<'a> Page<'a> {
    pub fn from_iter<I>(title: &'a str, config: &'a Config, iter: I) -> Self
    where
        I: IntoIterator<Item = Objects>,
    {
        let objects = iter.into_iter().flatten().collect();
        Self::new(title, config, objects)
    }
}

fn user_agent_is_cli(user_agent: &str) -> bool {
    user_agent.to_lowercase().trim().contains("curl")
}
