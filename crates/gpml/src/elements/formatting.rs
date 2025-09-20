use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::ActiveTheme;
use super::{ElementRenderer, extract_text_content, default_text_color, muted_text_color, apply_text_styles};

pub struct StrongElement;
pub struct EmElement;
pub struct UnderlineElement;
pub struct StrikethroughElement;
pub struct CodeElement;
pub struct PreElement;
pub struct CiteElement;
pub struct MarkElement;
pub struct SmallElement;
pub struct SubElement;
pub struct SupElement;

impl ElementRenderer for StrongElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut strong = div()
            .font_weight(FontWeight::BOLD)
            .text_color(default_text_color())
            .child(text_content);

        strong = apply_text_styles(strong, element, cx);
        Ok(strong.into_any_element())
    }
}

impl ElementRenderer for EmElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut em = div()
            .italic()
            .text_color(default_text_color())
            .child(text_content);

        em = apply_text_styles(em, element, cx);
        Ok(em.into_any_element())
    }
}

impl ElementRenderer for UnderlineElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut u = div()
            .underline()
            .text_color(default_text_color())
            .child(text_content);

        u = apply_text_styles(u, element, cx);
        Ok(u.into_any_element())
    }
}

impl ElementRenderer for StrikethroughElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut s = div()
            .line_through()
            .text_color(default_text_color())
            .child(text_content);

        s = apply_text_styles(s, element, cx);
        Ok(s.into_any_element())
    }
}

impl ElementRenderer for CodeElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut code = div()
            .font_family("monospace")
            .px_1()
            .bg(cx.theme().secondary)
            .rounded_sm()
            .text_color(default_text_color())
            .child(text_content);

        code = apply_text_styles(code, element, cx);
        Ok(code.into_any_element())
    }
}

impl ElementRenderer for PreElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut pre = div()
            .font_family("monospace")
            .p_4()
            .bg(cx.theme().secondary)
            .rounded_md()
            .text_color(default_text_color())
            .child(text_content);

        pre = apply_text_styles(pre, element, cx);
        Ok(pre.into_any_element())
    }
}

impl ElementRenderer for CiteElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut cite = div()
            .italic()
            .text_color(muted_text_color())
            .child(text_content);

        cite = apply_text_styles(cite, element, cx);
        Ok(cite.into_any_element())
    }
}

impl ElementRenderer for MarkElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut mark = div()
            .bg(gpui::yellow())
            .px_1()
            .child(text_content);

        mark = apply_text_styles(mark, element, cx);
        Ok(mark.into_any_element())
    }
}

impl ElementRenderer for SmallElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut small = div()
            .text_xs()
            .text_color(muted_text_color())
            .child(text_content);

        small = apply_text_styles(small, element, cx);
        Ok(small.into_any_element())
    }
}

impl ElementRenderer for SubElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut sub = div()
            .text_xs()
            .relative()
            .top(px(4.0))
            .child(text_content);

        sub = apply_text_styles(sub, element, cx);
        Ok(sub.into_any_element())
    }
}

impl ElementRenderer for SupElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let mut sup = div()
            .text_xs()
            .relative()
            .top(px(-4.0))
            .child(text_content);

        sup = apply_text_styles(sup, element, cx);
        Ok(sup.into_any_element())
    }
}