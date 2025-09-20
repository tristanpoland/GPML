use crate::ast::*;
use crate::error::*;
use gpui::*;
use super::{ElementRenderer, render_child, extract_text_content, muted_text_color};

pub struct BlockquoteElement;
pub struct QElement;

impl ElementRenderer for BlockquoteElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut blockquote = div()
            .border_l_4()
            .border_color(cx.theme().primary)
            .pl_4()
            .my_4()
            .italic()
            .text_color(muted_text_color());

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                blockquote = blockquote.child(child_element);
            }
        }

        Ok(blockquote.into_any_element())
    }
}

impl ElementRenderer for QElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(div()
            .italic()
            .text_color(muted_text_color())
            .child(format!("\"{}\"", text_content))
            .into_any_element())
    }
}