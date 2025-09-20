use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::v_flex;
use super::{ElementRenderer, render_child, apply_common_styles, extract_text_content, default_text_color};

pub struct FormElement;
pub struct FieldsetElement;
pub struct LegendElement;
pub struct TextareaElement;

impl ElementRenderer for FormElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut form = v_flex()
            .gap_4()
            .p_4()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md();

        form = apply_common_styles(form, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                form = form.child(child_element);
            }
        }

        Ok(form.into_any_element())
    }
}

impl ElementRenderer for FieldsetElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut fieldset = v_flex()
            .gap_2()
            .p_4()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md();

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                fieldset = fieldset.child(child_element);
            }
        }

        Ok(fieldset.into_any_element())
    }
}

impl ElementRenderer for LegendElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(div()
            .font_weight(FontWeight::BOLD)
            .text_color(default_text_color())
            .mb_2()
            .child(text_content)
            .into_any_element())
    }
}

impl ElementRenderer for TextareaElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let placeholder = element.get_attribute("placeholder")
            .map(|v| v.as_string())
            .unwrap_or_default();

        let rows = element.get_attribute("rows")
            .and_then(|v| v.as_number())
            .unwrap_or(3.0) as f32;

        let mut textarea = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .p_3()
            .bg(cx.theme().background)
            .text_color(default_text_color())
            .min_h(px(rows * 24.0));

        if !placeholder.is_empty() {
            textarea = textarea.child(format!("[Textarea: {}]", placeholder));
        } else {
            textarea = textarea.child("[Textarea]");
        }

        Ok(textarea.into_any_element())
    }
}