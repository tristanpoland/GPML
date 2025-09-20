use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::{scroll::ScrollbarAxis, ActiveTheme, StyledExt};
use super::{ElementRenderer, render_child, apply_common_styles};

pub struct ModalElement;
pub struct PopoverElement;
pub struct TooltipElement;
pub struct ScrollElement;
pub struct ResizableElement;
pub struct BrElement;
pub struct HrElement;
pub struct NoopElement;
pub struct TreeElement;

impl ElementRenderer for ModalElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        // For now, render modal content inline
        // In a real implementation, you'd integrate with the modal system
        let mut modal = div();
        modal = apply_common_styles(modal, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                modal = modal.child(child_element);
            }
        }

        Ok(modal.into_any_element())
    }
}

impl ElementRenderer for PopoverElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut popover = div();
        popover = apply_common_styles(popover, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                popover = popover.child(child_element);
            }
        }

        Ok(popover.into_any_element())
    }
}

impl ElementRenderer for TooltipElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut tooltip = div();
        tooltip = apply_common_styles(tooltip, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                tooltip = tooltip.child(child_element);
            }
        }

        Ok(tooltip.into_any_element())
    }
}

impl ElementRenderer for ScrollElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut scroll_el = div().scrollable(ScrollbarAxis::Both);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                scroll_el = scroll_el.child(child_element);
            }
        }

        Ok(scroll_el.into_any_element())
    }
}

impl ElementRenderer for ResizableElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut resizable = div();
        resizable = apply_common_styles(resizable, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                resizable = resizable.child(child_element);
            }
        }

        Ok(resizable.into_any_element())
    }
}

impl ElementRenderer for BrElement {
    fn render<T>(_element: &GPMLElement, _cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        Ok(div()
            .h(px(1.0))
            .w_full()
            .into_any_element())
    }
}

impl ElementRenderer for HrElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut hr = div()
            .h(px(1.0))
            .w_full()
            .bg(cx.theme().border)
            .my_4();

        hr = apply_common_styles(hr, element);
        Ok(hr.into_any_element())
    }
}

impl ElementRenderer for NoopElement {
    fn render<T>(_element: &GPMLElement, _cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        Ok(div().into_any_element())
    }
}

impl ElementRenderer for TreeElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut tree = div();
        tree = apply_common_styles(tree, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                tree = tree.child(child_element);
            }
        }

        Ok(tree.into_any_element())
    }
}