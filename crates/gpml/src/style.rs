use std::collections::HashMap;
use gpui::*;
use gpui_component::*;
use crate::ast::GPMLElement;

/// Lightweight style model parsed from inline `style` attribute.
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub props: HashMap<String, String>,
}

impl Style {
    /// Parse an inline CSS string into a Style
    pub fn from_inline(s: &str) -> Self {
        let mut props = HashMap::new();
        for part in s.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() { continue; }
            if let Some(idx) = trimmed.find(':') {
                let (prop, val) = trimmed.split_at(idx);
                let prop = prop.trim().to_lowercase();
                let val = val[1..].trim().to_string();
                props.insert(prop, val);
            }
        }
        Style { props }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.props.get(key)
    }

    pub fn parse_px(s: &str) -> Option<f32> {
        let s = s.trim();
        if s.ends_with("px") {
            s[..s.len()-2].trim().parse::<f32>().ok()
        } else {
            s.parse::<f32>().ok()
        }
    }

    /// Apply common style props (width/height/padding/margin/background) to a Div
    pub fn apply_common_to_div(div_el: Div, element: &GPMLElement) -> Div {
        Self::apply_common_to_styled(div_el, element)
    }

    /// Apply common style props to any Styled element
    pub fn apply_common_to_styled<T: Styled>(styled_el: T, element: &GPMLElement) -> T {
        let mut styled = styled_el;
        let style = element.get_attribute("style").map(|v| Style::from_inline(&v.as_string()));

        // Width/Height (attribute precedence)
        if let Some(width) = element.get_attribute("width").and_then(|v| v.as_number()) {
            styled = styled.w(px(width as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("width") {
                if let Some(pxv) = Style::parse_px(v) { styled = styled.w(px(pxv)); }
            }
        }

        if let Some(height) = element.get_attribute("height").and_then(|v| v.as_number()) {
            styled = styled.h(px(height as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("height") {
                if let Some(pxv) = Style::parse_px(v) { styled = styled.h(px(pxv)); }
            }
        }

        // padding/margin
        if let Some(padding) = element.get_attribute("padding").and_then(|v| v.as_number()) {
            styled = styled.p(px(padding as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("padding") {
                if let Some(pxv) = Style::parse_px(v) { styled = styled.p(px(pxv)); }
            }
        }

        if let Some(margin) = element.get_attribute("margin").and_then(|v| v.as_number()) {
            styled = styled.m(px(margin as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("margin") {
                if let Some(pxv) = Style::parse_px(v) { styled = styled.m(px(pxv)); }
            }
        }

        // background / background-color
        if let Some(bg) = element.get_attribute("background") {
            if let Some(color) = crate::renderer::GPMLRenderer::parse_color(&bg.as_string()) {
                styled = styled.bg(color);
            }
        } else if let Some(s) = &style {
            if let Some(v) = s.get("background") {
                if let Some(color) = crate::renderer::GPMLRenderer::parse_color(v) { styled = styled.bg(color); }
            } else if let Some(v) = s.get("background-color") {
                if let Some(color) = crate::renderer::GPMLRenderer::parse_color(v) { styled = styled.bg(color); }
            }
        }

        styled
    }

    /// Apply flex-related style props (gap, flex-direction) to a flex container
    pub fn apply_flex_to_container<T: ParentElement + Styled>(mut container: T, element: &GPMLElement) -> T {
        let style = element.get_attribute("style").map(|v| Style::from_inline(&v.as_string()));

        if let Some(spacing) = element.get_attribute("spacing").and_then(|v| v.as_number()) {
            container = container.gap(px(spacing as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("gap") {
                if let Some(pxv) = Style::parse_px(v) { container = container.gap(px(pxv)); }
            }
        }

        // flex-direction could be mapped, but direction is typically decided when creating container (h_flex/v_flex)
        container
    }

    /// Apply text style props to a Styled text element
    pub fn apply_text_to<T: Styled, U: 'static>(mut text_el: T, element: &GPMLElement, cx: &mut Context<U>) -> T {
        let style = element.get_attribute("style").map(|v| Style::from_inline(&v.as_string()));

        if let Some(size) = element.get_attribute("size").and_then(|v| v.as_number()) {
            text_el = text_el.text_size(px(size as f32));
        } else if let Some(s) = &style {
            if let Some(v) = s.get("font-size") {
                if let Some(pxv) = Style::parse_px(v) { text_el = text_el.text_size(px(pxv)); }
            }
        }

        if let Some(color_attr) = element.get_attribute("color") {
            if let Some(color) = crate::renderer::GPMLRenderer::parse_color(&color_attr.as_string()) {
                text_el = text_el.text_color(color);
            }
        } else if let Some(s) = &style {
            if let Some(v) = s.get("color") {
                if let Some(color) = crate::renderer::GPMLRenderer::parse_color(v) { text_el = text_el.text_color(color); }
            }
        }

        if let Some(weight) = element.get_attribute("weight") {
            match weight.as_string().as_str() {
                "bold" => text_el = text_el.font_weight(FontWeight::BOLD),
                "normal" => text_el = text_el.font_weight(FontWeight::NORMAL),
                _ => {}
            }
        } else if let Some(s) = &style {
            if let Some(v) = s.get("font-weight") {
                match v.as_str() {
                    "bold" => text_el = text_el.font_weight(FontWeight::BOLD),
                    "normal" => text_el = text_el.font_weight(FontWeight::NORMAL),
                    _ => {}
                }
            }
        }

        text_el
    }
}
