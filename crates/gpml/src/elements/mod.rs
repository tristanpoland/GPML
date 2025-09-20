pub mod layout;
pub mod text;
pub mod interactive;
pub mod semantic;
pub mod formatting;
pub mod list;
pub mod media;
pub mod table;
pub mod form;
pub mod quote;
pub mod misc;

use crate::ast::*;
use crate::error::*;
use gpui::*;

pub trait ElementRenderer {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static;
}

pub(crate) fn render_child<T>(child: &GPMLNode, cx: &mut Context<T>) -> GPMLResult<AnyElement>
where
    T: 'static,
{
    match child {
        GPMLNode::Element(element) => crate::renderer::GPMLRenderer::render_resolved_element_direct(element, cx),
        GPMLNode::Text(text) => Ok(div().child(text.clone()).into_any_element()),
        _ => Ok(div().into_any_element()),
    }
}

pub(crate) fn extract_text_content(element: &GPMLElement) -> String {
    element.get_text_content()
}

pub(crate) fn default_text_color() -> Hsla {
    gpui::white()
}

pub(crate) fn muted_text_color() -> Hsla {
    gpui::rgb(0xcccccc).into()
}

pub(crate) fn apply_common_styles<T: Styled>(styled_el: T, _element: &GPMLElement) -> T {
    styled_el
}

pub(crate) fn apply_flex_styles<T: ParentElement + Styled>(flex_el: T, _element: &GPMLElement) -> T {
    flex_el
}

pub(crate) fn apply_text_styles<T, U>(text_el: T, _element: &GPMLElement, _cx: &mut Context<U>) -> T
where
    T: Styled,
    U: 'static,
{
    text_el
}

pub(crate) fn parse_color(color_str: &str) -> Option<Hsla> {
    match color_str {
        "red" => Some(gpui::red()),
        "green" => Some(gpui::green()),
        "blue" => Some(gpui::blue()),
        "yellow" => Some(gpui::yellow()),
        "black" => Some(gpui::black()),
        "white" => Some(gpui::white()),
        "gray" | "grey" => Some(gpui::rgb(0x808080).into()),
        "transparent" => Some(gpui::rgba(0x00000000).into()),
        _ => {
            if color_str.starts_with('#') {
                let hex = &color_str[1..];
                if hex.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                    ) {
                        let a: u8 = 0xFF;
                        let hex_value = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);
                        return Some(gpui::rgba(hex_value).into());
                    }
                } else if hex.len() == 8 {
                    if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                        u8::from_str_radix(&hex[0..2], 16),
                        u8::from_str_radix(&hex[2..4], 16),
                        u8::from_str_radix(&hex[4..6], 16),
                        u8::from_str_radix(&hex[6..8], 16),
                    ) {
                        let hex_value = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);
                        return Some(gpui::rgba(hex_value).into());
                    }
                }
            }
            None
        }
    }
}