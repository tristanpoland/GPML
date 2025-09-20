use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::{v_flex, h_flex};
use super::{ElementRenderer, render_child, extract_text_content, default_text_color};

pub struct TableElement;
pub struct TheadElement;
pub struct TbodyElement;
pub struct TfootElement;
pub struct TrElement;
pub struct TdElement;
pub struct ThElement;
pub struct CaptionElement;

impl ElementRenderer for TableElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut table = v_flex()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .overflow_hidden();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                table = table.child(child_element);
            }
        }

        Ok(table.into_any_element())
    }
}

impl ElementRenderer for TheadElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut thead = v_flex()
            .bg(cx.theme().secondary.opacity(0.1));

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                thead = thead.child(child_element);
            }
        }

        Ok(thead.into_any_element())
    }
}

impl ElementRenderer for TbodyElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut tbody = v_flex();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                tbody = tbody.child(child_element);
            }
        }

        Ok(tbody.into_any_element())
    }
}

impl ElementRenderer for TfootElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut tfoot = v_flex()
            .bg(cx.theme().secondary.opacity(0.1))
            .border_t_1()
            .border_color(cx.theme().border);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                tfoot = tfoot.child(child_element);
            }
        }

        Ok(tfoot.into_any_element())
    }
}

impl ElementRenderer for TrElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut tr = h_flex()
            .border_b_1()
            .border_color(cx.theme().border);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                tr = tr.child(child_element);
            }
        }

        Ok(tr.into_any_element())
    }
}

impl ElementRenderer for TdElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut td = div()
            .p_2()
            .border_r_1()
            .border_color(cx.theme().border)
            .flex_1();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                td = td.child(child_element);
            }
        }

        Ok(td.into_any_element())
    }
}

impl ElementRenderer for ThElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut th = div()
            .p_2()
            .border_r_1()
            .border_color(cx.theme().border)
            .font_weight(FontWeight::BOLD)
            .flex_1();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                th = th.child(child_element);
            }
        }

        Ok(th.into_any_element())
    }
}

impl ElementRenderer for CaptionElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(div()
            .p_2()
            .text_center()
            .font_weight(FontWeight::BOLD)
            .text_color(default_text_color())
            .child(text_content)
            .into_any_element())
    }
}