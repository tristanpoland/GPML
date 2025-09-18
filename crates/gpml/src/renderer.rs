use crate::ast::*;
use crate::error::*;
use crate::component::*;
use gpui::*;
use gpui_component::*;
use gpui_component::scroll::ScrollbarAxis;
use std::collections::HashMap;

/// GPML renderer that converts GPML AST to GPUI elements
pub struct GPMLRenderer;

impl GPMLRenderer {
    /// Render a GPML element to a GPUI element
    pub fn render_element<T>(
        element: &GPMLElement,
        context: &GPMLContext,
        resolver: &ComponentResolver,
        cx: &mut Context<T>,
    ) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        // First resolve any custom components
        let resolved_element = resolve_element(element, context, resolver)?;
        
        // Then render to GPUI
        Self::render_resolved_element(&resolved_element, cx)
    }

    fn render_resolved_element<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        match element.tag.as_str() {
            // Layout containers
            "div" => Ok(Self::render_div(element, cx)),
            "flex" => Ok(Self::render_flex(element, cx)),
            "root" => Ok(Self::render_root(element, cx)),
            
            // Text elements
            "h1" => Ok(Self::render_heading(element, cx, HeadingLevel::H1)),
            "h2" => Ok(Self::render_heading(element, cx, HeadingLevel::H2)),
            "h3" => Ok(Self::render_heading(element, cx, HeadingLevel::H3)),
            "h4" => Ok(Self::render_heading(element, cx, HeadingLevel::H4)),
            "h5" => Ok(Self::render_heading(element, cx, HeadingLevel::H5)),
            "h6" => Ok(Self::render_heading(element, cx, HeadingLevel::H6)),
            "p" => Ok(Self::render_paragraph(element, cx)),
            "text" => Ok(Self::render_text(element, cx)),
            "label" => Ok(Self::render_label(element, cx)),
            
            // Interactive elements
            "button" => Ok(Self::render_button(element, cx)),
            "input" => Ok(Self::render_input(element, cx)),
            "checkbox" => Ok(Self::render_checkbox(element, cx)),
            "radio" => Ok(Self::render_radio(element, cx)),
            "slider" => Ok(Self::render_slider(element, cx)),
            "switch" => Ok(Self::render_switch(element, cx)),
            
            // Layout and structure
            "modal" => Ok(Self::render_modal(element, cx)),
            "popover" => Ok(Self::render_popover(element, cx)),
            "tooltip" => Ok(Self::render_tooltip(element, cx)),
            "scroll" => Ok(Self::render_scroll(element, cx)),
            "resizable" => Ok(Self::render_resizable(element, cx)),
            
            // Display elements
            "icon" => Ok(Self::render_icon(element, cx)),
            "image" => Ok(Self::render_image(element, cx)),
            "badge" => Ok(Self::render_badge(element, cx)),
            "avatar" => Ok(Self::render_avatar(element, cx)),
            
            // Lists and data
            "list" => Ok(Self::render_list(element, cx)),
            "table" => Ok(Self::render_table(element, cx)),
            "tree" => Ok(Self::render_tree(element, cx)),
            
            // Unknown tag - render as div with warning
            _ => {
                tracing::warn!("Unknown GPML tag: {}", element.tag);
                Ok(Self::render_div(element, cx))
            }
        }
    }

    fn render_div<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let mut div_el = div();
        
        // Apply common styling
        div_el = Self::apply_common_styles(div_el, element);
        
        // Render children
        for child in &element.children {
            if let Ok(child_element) = Self::render_child(child, cx) {
                div_el = div_el.child(child_element);
            }
        }
        
        div_el.into_any_element()
    }

    fn render_flex<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
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

        // Apply common styling
        container = Self::apply_flex_styles(container, element);

        // Render children
        for child in &element.children {
            if let Ok(child_element) = Self::render_child(child, cx) {
                container = container.child(child_element);
            }
        }

        container.into_any_element()
    }

    fn render_root<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let mut root = div()
            .w_full()
            .h_full()
            .bg(cx.theme().background);

        // Apply common styling
        root = Self::apply_common_styles(root, element);

        // Render children
        for child in &element.children {
            if let Ok(child_element) = Self::render_child(child, cx) {
                root = root.child(child_element);
            }
        }

        root.into_any_element()
    }

    fn render_heading<T>(element: &GPMLElement, cx: &mut Context<T>, level: HeadingLevel) -> AnyElement
    where
        T: 'static,
    {
        let text_content = Self::extract_text_content(element);
        
        // Determine font size based on heading level
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
            .text_color(cx.theme().foreground)
            .child(text_content);
        
        // Apply text styling
        heading = Self::apply_text_styles(heading, element, cx);
        
        heading.into_any_element()
    }

    fn render_paragraph<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let text_content = Self::extract_text_content(element);
        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(14.0);
            
        let mut p = div()
            .text_size(px(size as f32))
            .text_color(cx.theme().foreground)
            .child(text_content);
            
        // Apply color if specified
        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = Self::parse_color(&color_attr.as_string()) {
                p = p.text_color(color);
            }
        }
        
        p.into_any_element()
    }

    fn render_text<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let text_content = element.get_attribute("text")
            .map(|v| v.as_string())
            .unwrap_or_else(|| Self::extract_text_content(element));
            
        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(14.0);

        let mut text_el = div()
            .text_size(px(size as f32))
            .text_color(cx.theme().foreground)
            .child(text_content);

        // Apply color if specified
        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = Self::parse_color(&color_attr.as_string()) {
                text_el = text_el.text_color(color);
            }
        }

        text_el.into_any_element()
    }

    fn render_label<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let text_content = Self::extract_text_content(element);
        label::Label::new(text_content).into_any_element()
    }

    fn render_button<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let text_content = element.get_attribute("text")
            .map(|v| v.as_string())
            .unwrap_or_else(|| Self::extract_text_content(element));

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let button_id = format!("gpml-button-{}", uuid::Uuid::new_v4());
        let mut button = button::Button::new(&*button_id)
            .label(text_content);

        if disabled {
            button = button.disabled(true);
        }

        button.into_any_element()
    }

    fn render_input<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let placeholder = element.get_attribute("placeholder")
            .map(|v| v.as_string())
            .unwrap_or_default();

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Placeholder implementation - creates a styled div that looks like an input
        let mut input_div = div()
            .border_1()
            .border_color(cx.theme().border)
            .rounded_md()
            .px_3()
            .py_2()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground);

        if !placeholder.is_empty() {
            input_div = input_div.child(format!("[{}]", placeholder));
        } else {
            input_div = input_div.child("[Input]");
        }

        if disabled {
            input_div = input_div
                .opacity(0.5)
                .text_color(cx.theme().muted_foreground);
        }

        input_div.into_any_element()
    }

    fn render_checkbox<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
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
            .unwrap_or_else(|| Self::extract_text_content(element));

        let checkbox_id = format!("gpml-checkbox-{}", uuid::Uuid::new_v4());
        let mut checkbox = checkbox::Checkbox::new(
            &*checkbox_id,
        )
        .checked(checked);

        if disabled {
            checkbox = checkbox.disabled(true);
        }

        if !label_text.is_empty() {
            h_flex()
                .items_center()
                .gap_2()
                .child(checkbox)
                .child(label::Label::new(label_text))
                .into_any_element()
        } else {
            checkbox.into_any_element()
        }
    }

    fn render_radio<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
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
            .unwrap_or_else(|| Self::extract_text_content(element));

        let radio_id = format!("gpml-radio-{}", uuid::Uuid::new_v4());
        let radio = radio::Radio::new(
            &*radio_id,
        );

        if !label_text.is_empty() {
            h_flex()
                .items_center()
                .gap_2()
                .child(radio)
                .child(label::Label::new(label_text))
                .into_any_element()
        } else {
            radio.into_any_element()
        }
    }

    fn render_switch<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let checked = element.get_attribute("checked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let disabled = element.get_attribute("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let switch_id = format!("gpml-switch-{}", uuid::Uuid::new_v4());
        let mut switch = switch::Switch::new(
            &*switch_id,
        )
        .checked(checked);

        if disabled {
            switch = switch.disabled(true);
        }

        switch.into_any_element()
    }

    fn render_slider<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
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

        // Placeholder implementation - create a simple div that represents a slider
        div()
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
            .into_any_element()
    }

    fn render_icon<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let icon_name = element.get_attribute("name")
            .map(|v| v.as_string())
            .unwrap_or_else(|| Self::extract_text_content(element));

        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(16.0);

        // Convert string to IconName - this is a simplified approach
        // In practice, you'd want a proper mapping
        let icon_name_enum = match icon_name.as_str() {
            "check" => IconName::Check,
            "close" => IconName::Close,
            "arrow-left" => IconName::ArrowLeft,
            "arrow-right" => IconName::ArrowRight,
            "arrow-up" => IconName::ArrowUp,
            "arrow-down" => IconName::ArrowDown,
            "settings" => IconName::Settings,
            "user" => IconName::User,
            "globe" => IconName::Globe,
            "star" => IconName::Star,
            "heart" => IconName::Heart,
            _ => IconName::CircleX, // Default fallback
        };

        Icon::new(icon_name_enum)
            .size(px(size as f32))
            .into_any_element()
    }

    fn render_modal<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        // For now, render modal content inline
        // In a real implementation, you'd integrate with the modal system
        Self::render_div(element, cx)
    }

    fn render_scroll<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let mut scroll_el = div().scrollable(ScrollbarAxis::Both);

        for child in &element.children {
            if let Ok(child_element) = Self::render_child(child, cx) {
                scroll_el = scroll_el.child(child_element);
            }
        }

        scroll_el.into_any_element()
    }

    fn render_list<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        let mut list_items = Vec::new();

        for child in &element.children {
            if let Ok(child_element) = Self::render_child(child, cx) {
                list_items.push(child_element);
            }
        }

        v_flex()
            .gap_1()
            .children(list_items)
            .into_any_element()
    }

    // Helper methods

    fn render_child<T>(child: &GPMLNode, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        match child {
            GPMLNode::Element(element) => Self::render_resolved_element(element, cx),
            GPMLNode::Text(text) => Ok(div().child(text.clone()).into_any_element()),
            _ => Ok(div().into_any_element()),
        }
    }

    fn extract_text_content(element: &GPMLElement) -> String {
        element.get_text_content()
    }

    fn apply_common_styles(div_el: Div, element: &GPMLElement) -> Div {
        let mut styled = div_el;

        // Width and height
        if let Some(width) = element.get_attribute("width").and_then(|v| v.as_number()) {
            styled = styled.w(px(width as f32));
        }
        if let Some(height) = element.get_attribute("height").and_then(|v| v.as_number()) {
            styled = styled.h(px(height as f32));
        }

        // Padding and margin
        if let Some(padding) = element.get_attribute("padding").and_then(|v| v.as_number()) {
            styled = styled.p(px(padding as f32));
        }
        if let Some(margin) = element.get_attribute("margin").and_then(|v| v.as_number()) {
            styled = styled.m(px(margin as f32));
        }

        // Background color
        if let Some(bg_color) = element.get_attribute("background") {
            if let Some(color) = Self::parse_color(&bg_color.as_string()) {
                styled = styled.bg(color);
            }
        }

        styled
    }

    fn apply_flex_styles<T: ParentElement + Styled>(flex_el: T, element: &GPMLElement) -> T {
        let mut styled = flex_el;

        // Justify content
        if let Some(justify) = element.get_attribute("justify") {
            match justify.as_string().as_str() {
                "start" => styled = styled.justify_start(),
                "end" => styled = styled.justify_end(),
                "center" => styled = styled.justify_center(),
                "between" => styled = styled.justify_between(),
                "around" => styled = styled.justify_around(),
                "evenly" => styled = styled.justify_between(),
                _ => {}
            }
        }

        // Align items
        if let Some(align) = element.get_attribute("align") {
            match align.as_string().as_str() {
                "start" => styled = styled.items_start(),
                "end" => styled = styled.items_end(),
                "center" => styled = styled.items_center(),
                "stretch" => styled = styled.items_start(),
                _ => {}
            }
        }

        styled
    }

    fn apply_text_styles<T, U>(text_el: T, element: &GPMLElement, cx: &mut Context<U>) -> T
    where
        T: Styled,
        U: 'static,
    {
        let mut styled = text_el;

        // Font size
        if let Some(size) = element.get_attribute("size").and_then(|v| v.as_number()) {
            styled = styled.text_size(px(size as f32));
        }

        // Text color
        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = Self::parse_color(&color_attr.as_string()) {
                styled = styled.text_color(color);
            }
        }

        // Font weight
        if let Some(weight) = element.get_attribute("weight") {
            match weight.as_string().as_str() {
                "bold" => styled = styled.font_weight(FontWeight::BOLD),
                "normal" => styled = styled.font_weight(FontWeight::NORMAL),
                _ => {}
            }
        }

        styled
    }

    fn parse_color(color_str: &str) -> Option<Hsla> {
        match color_str {
            "red" => Some(gpui::red()),
            "green" => Some(gpui::green()),
            "blue" => Some(gpui::blue()),
            "yellow" => Some(gpui::yellow()),
            "black" => Some(gpui::black()),
            "white" => Some(gpui::white()),
            "gray" | "grey" => Some(gray(500)),
            "transparent" => Some(transparent_black()),
            _ => {
                // Try to parse hex color
                if color_str.starts_with('#') && color_str.len() == 7 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&color_str[1..3], 16),
                        u8::from_str_radix(&color_str[3..5], 16),
                        u8::from_str_radix(&color_str[5..7], 16),
                    ) {
                        let hex_value = (r << 16) | (g << 8) | b;
                        return Some(rgba(hex_value as u32).into());
                    }
                }
                None
            }
        }
    }

    // Placeholder implementations for missing components
    fn render_badge<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_avatar<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_image<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_table<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_tree<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_popover<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_tooltip<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }

    fn render_resizable<T>(element: &GPMLElement, cx: &mut Context<T>) -> AnyElement
    where
        T: 'static,
    {
        Self::render_div(element, cx)
    }
}

// Helper type for heading levels
#[derive(Debug, Clone, Copy)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}
