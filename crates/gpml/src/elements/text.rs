use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::*;
use super::{ElementRenderer, extract_text_content, default_text_color, apply_text_styles, parse_color};

pub struct HeadingElement;
pub struct ParagraphElement;
pub struct TextElement;
pub struct LabelElement;
pub struct SpanElement;

#[derive(Debug, Clone, Copy)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingElement {
    pub fn render<T>(element: &GPMLElement, cx: &mut Context<T>, level: HeadingLevel) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);

        let font_size = match level {
            HeadingLevel::H1 => px(32.0),
            HeadingLevel::H2 => px(24.0),
            HeadingLevel::H3 => px(20.0),
            HeadingLevel::H4 => px(16.0),
            HeadingLevel::H5 => px(14.0),
            HeadingLevel::H6 => px(12.0),
        };

        let mut heading = div()
            .text_size(font_size)
            .font_weight(FontWeight::BOLD)
            .text_color(default_text_color())
            .child(text_content);

        heading = apply_text_styles(heading, element, cx);

        Ok(heading.into_any_element())
    }
}

impl ElementRenderer for ParagraphElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(14.0);

        let mut p = div()
            .text_size(px(size as f32))
            .text_color(default_text_color())
            .child(text_content);

        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = parse_color(&color_attr.as_string()) {
                p = p.text_color(color);
            }
        }

        Ok(p.into_any_element())
    }
}

impl ElementRenderer for TextElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = element.get_attribute("text")
            .map(|v| v.as_string())
            .unwrap_or_else(|| extract_text_content(element));

        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(14.0);

        let mut text_el = div()
            .text_size(px(size as f32))
            .text_color(default_text_color())
            .child(text_content);

        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = parse_color(&color_attr.as_string()) {
                text_el = text_el.text_color(color);
            }
        }

        Ok(text_el.into_any_element())
    }
}

impl ElementRenderer for LabelElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(label::Label::new(text_content).into_any_element())
    }
}

impl ElementRenderer for SpanElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut span = div()
            .text_color(default_text_color())
            .child(text_content);

        span = apply_text_styles(span, element, cx);
        Ok(span.into_any_element())
    }
}