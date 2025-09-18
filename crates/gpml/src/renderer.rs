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

        let mut button = button::Button::new("gpml-button")
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

        let mut checkbox = checkbox::Checkbox::new(
            "gpml-checkbox",
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

        let radio = radio::Radio::new(
            "gpml-radio",
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

        let mut switch = switch::Switch::new(
            "gpml-switch",
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
        crate::style::Style::apply_common_to_div(div_el, element)
    }

    fn apply_flex_styles<T: ParentElement + Styled>(flex_el: T, element: &GPMLElement) -> T {
        crate::style::Style::apply_flex_to_container(flex_el, element)
    }

    fn apply_text_styles<T, U>(text_el: T, element: &GPMLElement, cx: &mut Context<U>) -> T
    where
        T: Styled,
        U: 'static,
    {
        crate::style::Style::apply_text_to(text_el, element, cx)
    }

    pub(crate) fn parse_color(color_str: &str) -> Option<Hsla> {
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
                // Try to parse hex color. gpui::rgba expects a u32 in the format 0xRRGGBBAA
                if color_str.starts_with('#') {
                    let hex = &color_str[1..];
                    // Support #RRGGBB and #RRGGBBAA
                    if hex.len() == 6 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            u8::from_str_radix(&hex[0..2], 16),
                            u8::from_str_radix(&hex[2..4], 16),
                            u8::from_str_radix(&hex[4..6], 16),
                        ) {
                            let a: u8 = 0xFF;
                            let hex_value = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);
                            return Some(rgba(hex_value).into());
                        }
                    } else if hex.len() == 8 {
                        if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                            u8::from_str_radix(&hex[0..2], 16),
                            u8::from_str_radix(&hex[2..4], 16),
                            u8::from_str_radix(&hex[4..6], 16),
                            u8::from_str_radix(&hex[6..8], 16),
                        ) {
                            let hex_value = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32);
                            return Some(rgba(hex_value).into());
                        }
                    }
                }
                None
            }
        }
    }

    /// Parse an inline CSS style string into a map of property -> value
    // parse helpers moved to `style` module

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
        // Try to read src/alt/width/height attributes and construct a gpui image
        // src is required for meaningful image, fall back to empty div otherwise
        if let Some(src_attr) = element.get_attribute("src") {
            let src_str = src_attr.as_string();

            // Convert to ImageSource via Into<ImageSource> by using the img builder directly
            // Use string clone - img accepts Into<ImageSource>
            let mut img_el = img(src_str.clone());

            // width/height attributes (numeric)
            if let Some(w) = element.get_attribute("width").and_then(|v| v.as_number()) {
                img_el = img_el.w(px(w as f32));
            }
            if let Some(h) = element.get_attribute("height").and_then(|v| v.as_number()) {
                img_el = img_el.h(px(h as f32));
            }

            // object-fit/style: allow inline style to set object-fit via style attr
            if let Some(style_attr) = element.get_attribute("style") {
                let style = crate::style::Style::from_inline(&style_attr.as_string());
                if let Some(of) = style.get("object-fit") {
                    match of.as_str() {
                        "cover" => img_el = img_el.object_fit(ObjectFit::Cover),
                        "contain" => img_el = img_el.object_fit(ObjectFit::Contain),
                        _ => {}
                    }
                }
            }

            // Apply common styles (padding/margin/background etc.) by refining into a wrapper div
            // We will return the image as AnyElement but also allow common props via wrapper if needed
            // For now apply common styles directly to the image via refine_style when available.
            // Note: gpui::Image supports Styled API, so we can apply StyleRefinement via refine_style if present.

            // Apply remaining common style props (width/height precedence handled above)
            // If background specified via attribute/style, try to apply as background on a wrapper div
            let mut outer = div().child(img_el.into_any_element());
            outer = Self::apply_common_styles(outer, element);
            outer.into_any_element()
        } else {
            // no src -> fallback to empty div
            Self::render_div(element, cx)
        }
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
