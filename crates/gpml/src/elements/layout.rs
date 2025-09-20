use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::{h_flex, v_flex};
use super::{ElementRenderer, render_child, apply_common_styles, apply_flex_styles};

pub struct DivElement;
pub struct FlexElement;
pub struct RootElement;

impl ElementRenderer for DivElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut div_el = div();

        div_el = apply_common_styles(div_el, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                div_el = div_el.child(child_element);
            }
        }

        Ok(div_el.into_any_element())
    }
}

impl ElementRenderer for FlexElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let direction_attr = element.get_attribute("dir");
        let direction_string = direction_attr.map(|v| v.as_string());
        let direction = direction_string
            .as_deref()
            .unwrap_or("vertical");

        let spacing = element.get_attribute("spacing")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0);

        let mut container = match direction {
            "horizontal" | "row" => h_flex(),
            _ => v_flex(),
        };

        if spacing > 0.0 {
            container = container.gap(px(spacing as f32));
        }

        container = apply_flex_styles(container, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                container = container.child(child_element);
            }
        }

        Ok(container.into_any_element())
    }
}

impl ElementRenderer for RootElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut root = div()
            .w_full()
            .h_full()
            .bg(cx.theme().background);

        root = apply_common_styles(root, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                root = root.child(child_element);
            }
        }

        Ok(root.into_any_element())
    }
}