use crate::ast::*;
use crate::error::*;
use gpui::*;
use super::{ElementRenderer, extract_text_content, default_text_color, muted_text_color, apply_text_styles};

pub struct LinkElement;
pub struct ImgElement;
pub struct ImageElement;
pub struct IconElement;
pub struct AvatarElement;
pub struct BadgeElement;

impl ElementRenderer for LinkElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        let href = element.get_attribute("href")
            .map(|v| v.as_string())
            .unwrap_or_default();

        let mut link = div()
            .text_color(cx.theme().primary)
            .underline()
            .cursor_pointer()
            .child(text_content);

        if !href.is_empty() {
            link = link.hover(|style| style.text_color(cx.theme().primary.opacity(0.8)));
        }

        link = apply_text_styles(link, element, cx);
        Ok(link.into_any_element())
    }
}

impl ElementRenderer for ImgElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        ImageElement::render(element, cx)
    }
}

impl ElementRenderer for ImageElement {
    fn render<T>(element: &GPMLElement, _cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        // HARDCODED TEST: Render exact same image as story that works
        tracing::info!("=== HARDCODED IMAGE TEST ===");
        let hardcoded_test = div()
            .flex()
            .items_center()
            .justify_center()
            .child(img("https://pub.lbkrs.com/files/202503/vEnnmgUM6bo362ya/sdk.svg").h_24())
            .into_any_element();

        Ok(hardcoded_test)

        // Original dynamic code below (temporarily disabled)
        /*
        if let Some(src_attr) = element.get_attribute("src") {
            let src_str = src_attr.as_string();
            tracing::info!("Rendering image with src: '{}', length: {}", src_str, src_str.len());

            let mut img_el = img(src_str);

            if let Some(w) = element.get_attribute("width").and_then(|v| v.as_number()) {
                img_el = img_el.w(px(w as f32));
            }

            if let Some(h) = element.get_attribute("height").and_then(|v| v.as_number()) {
                img_el = img_el.h(px(h as f32));
            }

            if element.get_attribute("width").is_none() && element.get_attribute("height").is_none() {
                tracing::info!("No width/height specified, using h_24().flex_grow() like story");
                img_el = img_el.h_24().flex_grow();
            } else {
                tracing::info!("Using specified dimensions");
            }

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

            tracing::info!("Image element created successfully");

            Ok(div()
                .flex()
                .items_center()
                .justify_center()
                .child(img_el)
                .into_any_element())
        } else {
            tracing::warn!("Image element missing src attribute, rendering placeholder");
            Ok(div()
                .w(px(300.0))
                .h(px(200.0))
                .bg(gpui::rgb(0x333333))
                .border_1()
                .border_color(gpui::rgb(0x666666))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_color(muted_text_color())
                        .child("No image source")
                )
                .into_any_element())
        }
        */
    }
}

impl ElementRenderer for IconElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let icon_name = element.get_attribute("name")
            .map(|v| v.as_string())
            .unwrap_or_else(|| extract_text_content(element));

        let size = element.get_attribute("size")
            .and_then(|v| v.as_number())
            .unwrap_or(16.0);

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
            _ => IconName::CircleX,
        };

        Ok(Icon::new(icon_name_enum)
            .size(px(size as f32))
            .into_any_element())
    }
}

impl ElementRenderer for AvatarElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        // Placeholder implementation
        Ok(div()
            .w_8()
            .h_8()
            .rounded_full()
            .bg(cx.theme().secondary)
            .into_any_element())
    }
}

impl ElementRenderer for BadgeElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let text_content = extract_text_content(element);
        Ok(div()
            .px_2()
            .py_1()
            .bg(cx.theme().primary)
            .rounded_md()
            .text_xs()
            .text_color(default_text_color())
            .child(text_content)
            .into_any_element())
    }
}