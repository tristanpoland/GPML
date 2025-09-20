use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::{h_flex, v_flex};
use super::{ElementRenderer, render_child, muted_text_color, default_text_color, extract_text_content};

pub struct UlElement;
pub struct OlElement;
pub struct LiElement;
pub struct DlElement;
pub struct DtElement;
pub struct DdElement;
pub struct ListElement;

impl ElementRenderer for UlElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut ul = v_flex()
            .gap_1()
            .ml_4();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                ul = ul.child(child_element);
            }
        }

        Ok(ul.into_any_element())
    }
}

impl ElementRenderer for OlElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut ol = v_flex()
            .gap_1()
            .ml_4();

        for (index, child) in element.children.iter().enumerate() {
            if let Ok(child_element) = render_child(child, cx) {
                let list_item = h_flex()
                    .gap_2()
                    .child(div().text_color(muted_text_color()).child(format!("{}.", index + 1)))
                    .child(child_element);
                ol = ol.child(list_item.into_any_element());
            }
        }

        Ok(ol.into_any_element())
    }
}

impl ElementRenderer for LiElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut li = h_flex()
            .gap_2()
            .child(div()
                .text_color(muted_text_color())
                .child("•"));

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                li = li.child(child_element);
            }
        }

        Ok(li.into_any_element())
    }
}

impl ElementRenderer for DlElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut dl = v_flex().gap_2();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                dl = dl.child(child_element);
            }
        }

        Ok(dl.into_any_element())
    }
}

impl ElementRenderer for DtElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(div()
            .font_weight(FontWeight::BOLD)
            .text_color(default_text_color())
            .child(text_content)
            .into_any_element())
    }
}

impl ElementRenderer for DdElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut dd = div()
            .ml_4()
            .mb_2();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                dd = dd.child(child_element);
            }
        }

        Ok(dd.into_any_element())
    }
}

impl ElementRenderer for ListElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut list_items = Vec::new();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                list_items.push(child_element);
            }
        }

        Ok(v_flex()
            .gap_1()
            .children(list_items)
            .into_any_element())
    }
}