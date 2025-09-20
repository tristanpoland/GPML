use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::*;
use super::{ElementRenderer, extract_text_content, default_text_color, muted_text_color};

pub struct ButtonElement;
pub struct InputElement;
pub struct CheckboxElement;
pub struct RadioElement;
pub struct SwitchElement;
pub struct SliderElement;

impl ElementRenderer for ButtonElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = element.get_attribute("text")
            .map(|v| v.as_string())
            .unwrap_or_else(|| extract_text_content(element));

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut button = button::Button::new("gpml-button")
            .label(text_content);

        if disabled {
            button = button.disabled(true);
        }

        Ok(button.into_any_element())
    }
}

impl ElementRenderer for InputElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let placeholder = element.get_attribute("placeholder")
            .map(|v| v.as_string())
            .unwrap_or_default();

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut input_div = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .px_3()
            .py_2()
            .bg(cx.theme().background)
            .text_color(default_text_color());

        if !placeholder.is_empty() {
            input_div = input_div.child(format!("[{}]", placeholder));
        } else {
            input_div = input_div.child("[Input]");
        }

        if disabled {
            input_div = input_div
                .opacity(0.5)
                .text_color(muted_text_color());
        }

        Ok(input_div.into_any_element())
    }
}

impl ElementRenderer for CheckboxElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let checked = element.get_attribute("checked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let label_text = element.get_attribute("label")
            .map(|v| v.as_string())
            .unwrap_or_else(|| extract_text_content(element));

        let mut checkbox = checkbox::Checkbox::new("gpml-checkbox")
            .checked(checked);

        if disabled {
            checkbox = checkbox.disabled(true);
        }

        if !label_text.is_empty() {
            Ok(h_flex()
                .items_center()
                .gap_2()
                .child(checkbox)
                .child(label::Label::new(label_text))
                .into_any_element())
        } else {
            Ok(checkbox.into_any_element())
        }
    }
}

impl ElementRenderer for RadioElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let selected = element.get_attribute("selected")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let value = element.get_attribute("value")
            .map(|v| v.as_string())
            .unwrap_or_default();

        let label_text = element.get_attribute("label")
            .map(|v| v.as_string())
            .unwrap_or_else(|| extract_text_content(element));

        let radio = radio::Radio::new("gpml-radio");

        if !label_text.is_empty() {
            Ok(h_flex()
                .items_center()
                .gap_2()
                .child(radio)
                .child(label::Label::new(label_text))
                .into_any_element())
        } else {
            Ok(radio.into_any_element())
        }
    }
}

impl ElementRenderer for SwitchElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let checked = element.get_attribute("checked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut switch = switch::Switch::new("gpml-switch")
            .checked(checked);

        if disabled {
            switch = switch.disabled(true);
        }

        Ok(switch.into_any_element())
    }
}

impl ElementRenderer for SliderElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let value = element.get_attribute("value")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0);

        let min = element.get_attribute("min")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0);

        let max = element.get_attribute("max")
            .and_then(|v| v.as_number())
            .unwrap_or(100.0);

        let step = element.get_attribute("step")
            .and_then(|v| v.as_number())
            .unwrap_or(1.0);

        Ok(div()
            .h_8()
            .w_full()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_full()
            .bg(cx.theme().secondary)
            .child(
                div()
                    .h_full()
                    .w(px(((value - min) / (max - min) * 100.0) as f32))
                    .bg(cx.theme().primary)
                    .rounded_full()
            )
            .into_any_element())
    }
}