use crate::ast::*;
use crate::error::*;
use gpui::*;
use gpui_component::ActiveTheme;
use super::{ElementRenderer, render_child, apply_common_styles};

pub struct ArticleElement;
pub struct SectionElement;
pub struct AsideElement;
pub struct NavElement;
pub struct HeaderElement;
pub struct FooterElement;
pub struct MainElement;

impl ElementRenderer for ArticleElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut article = div()
            .id("gpml-article")
            .p_4()
            .mb_6();

        article = apply_common_styles(article, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                article = article.child(child_element);
            }
        }

        Ok(article.into_any_element())
    }
}

impl ElementRenderer for SectionElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut section = div()
            .id("gpml-section")
            .mb_4();

        section = apply_common_styles(section, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                section = section.child(child_element);
            }
        }

        Ok(section.into_any_element())
    }
}

impl ElementRenderer for AsideElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut aside = div()
            .id("gpml-aside")
            .border_l_2()
            .border_color(cx.theme().border)
            .pl_4()
            .ml_4();

        aside = apply_common_styles(aside, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                aside = aside.child(child_element);
            }
        }

        Ok(aside.into_any_element())
    }
}

impl ElementRenderer for NavElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut nav = div()
            .id("gpml-nav")
            .p_2()
            .bg(cx.theme().secondary.opacity(0.1));

        nav = apply_common_styles(nav, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                nav = nav.child(child_element);
            }
        }

        Ok(nav.into_any_element())
    }
}

impl ElementRenderer for HeaderElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut header = div()
            .id("gpml-header")
            .py_4()
            .border_b_1()
            .border_color(cx.theme().border)
            .mb_4();

        header = apply_common_styles(header, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                header = header.child(child_element);
            }
        }

        Ok(header.into_any_element())
    }
}

impl ElementRenderer for FooterElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut footer = div()
            .id("gpml-footer")
            .py_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .mt_4()
            .text_sm();

        footer = apply_common_styles(footer, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                footer = footer.child(child_element);
            }
        }

        Ok(footer.into_any_element())
    }
}

impl ElementRenderer for MainElement {
    fn render<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        let mut main = div()
            .id("gpml-main")
            .flex_1()
            .p_4();

        main = apply_common_styles(main, element);

        for child in &element.children {
            if let Ok(child_element) = render_child(child, cx) {
                main = main.child(child_element);
            }
        }

        Ok(main.into_any_element())
    }
}