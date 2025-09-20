use crate::ast::*;
use crate::error::*;
use crate::component::*;
use crate::elements::*;
use gpui::*;

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

    /// Render an already resolved GPML element to a GPUI element (skips component resolution)
    pub fn render_resolved_element_direct<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        Self::render_resolved_element(element, cx)
    }

    fn render_resolved_element<T>(element: &GPMLElement, cx: &mut Context<T>) -> GPMLResult<AnyElement>
    where
        T: 'static,
    {
        match element.tag.as_str() {
            // Layout containers
            "div" => layout::DivElement::render(element, cx),
            "flex" => layout::FlexElement::render(element, cx),
            "root" => layout::RootElement::render(element, cx),

            // Semantic elements
            "article" => semantic::ArticleElement::render(element, cx),
            "section" => semantic::SectionElement::render(element, cx),
            "aside" => semantic::AsideElement::render(element, cx),
            "nav" => semantic::NavElement::render(element, cx),
            "header" => semantic::HeaderElement::render(element, cx),
            "footer" => semantic::FooterElement::render(element, cx),
            "main" => semantic::MainElement::render(element, cx),

            // Text elements
            "h1" => text::HeadingElement::render(element, cx, text::HeadingLevel::H1),
            "h2" => text::HeadingElement::render(element, cx, text::HeadingLevel::H2),
            "h3" => text::HeadingElement::render(element, cx, text::HeadingLevel::H3),
            "h4" => text::HeadingElement::render(element, cx, text::HeadingLevel::H4),
            "h5" => text::HeadingElement::render(element, cx, text::HeadingLevel::H5),
            "h6" => text::HeadingElement::render(element, cx, text::HeadingLevel::H6),
            "p" => text::ParagraphElement::render(element, cx),
            "text" => text::TextElement::render(element, cx),
            "label" => text::LabelElement::render(element, cx),
            "span" => text::SpanElement::render(element, cx),

            // Text formatting
            "strong" | "b" => formatting::StrongElement::render(element, cx),
            "em" | "i" => formatting::EmElement::render(element, cx),
            "u" => formatting::UnderlineElement::render(element, cx),
            "s" => formatting::StrikethroughElement::render(element, cx),
            "code" => formatting::CodeElement::render(element, cx),
            "pre" => formatting::PreElement::render(element, cx),
            "cite" => formatting::CiteElement::render(element, cx),
            "mark" => formatting::MarkElement::render(element, cx),
            "small" => formatting::SmallElement::render(element, cx),
            "sub" => formatting::SubElement::render(element, cx),
            "sup" => formatting::SupElement::render(element, cx),

            // Lists
            "ul" => list::UlElement::render(element, cx),
            "ol" => list::OlElement::render(element, cx),
            "li" => list::LiElement::render(element, cx),
            "dl" => list::DlElement::render(element, cx),
            "dt" => list::DtElement::render(element, cx),
            "dd" => list::DdElement::render(element, cx),

            // Links and media
            "a" => media::LinkElement::render(element, cx),
            "img" => media::ImgElement::render(element, cx),

            // Tables (HTML elements)
            "table" => table::TableElement::render(element, cx),
            "thead" => table::TheadElement::render(element, cx),
            "tbody" => table::TbodyElement::render(element, cx),
            "tfoot" => table::TfootElement::render(element, cx),
            "tr" => table::TrElement::render(element, cx),
            "td" => table::TdElement::render(element, cx),
            "th" => table::ThElement::render(element, cx),
            "caption" => table::CaptionElement::render(element, cx),

            // Forms
            "form" => form::FormElement::render(element, cx),
            "fieldset" => form::FieldsetElement::render(element, cx),
            "legend" => form::LegendElement::render(element, cx),
            "textarea" => form::TextareaElement::render(element, cx),

            // Quotes
            "blockquote" => quote::BlockquoteElement::render(element, cx),
            "q" => quote::QElement::render(element, cx),

            // Line breaks and separators
            "br" => misc::BrElement::render(element, cx),
            "hr" => misc::HrElement::render(element, cx),

            // Interactive elements
            "button" => interactive::ButtonElement::render(element, cx),
            "input" => interactive::InputElement::render(element, cx),
            "checkbox" => interactive::CheckboxElement::render(element, cx),
            "radio" => interactive::RadioElement::render(element, cx),
            "slider" => interactive::SliderElement::render(element, cx),
            "switch" => interactive::SwitchElement::render(element, cx),

            // Layout and structure
            "modal" => misc::ModalElement::render(element, cx),
            "popover" => misc::PopoverElement::render(element, cx),
            "tooltip" => misc::TooltipElement::render(element, cx),
            "scroll" => misc::ScrollElement::render(element, cx),
            "resizable" => misc::ResizableElement::render(element, cx),

            // Display elements
            "icon" => media::IconElement::render(element, cx),
            "image" => media::ImageElement::render(element, cx),
            "badge" => media::BadgeElement::render(element, cx),
            "avatar" => media::AvatarElement::render(element, cx),

            // Lists and data (GPML-specific)
            "list" => list::ListElement::render(element, cx),
            "tree" => misc::TreeElement::render(element, cx),

            // No-op elements (parse but don't render)
            "script" | "style" | "meta" | "link" | "base" => misc::NoopElement::render(element, cx),

            // Unknown tag - render as div with warning
            _ => {
                tracing::warn!("Unknown GPML tag: {}", element.tag);
                layout::DivElement::render(element, cx)
            }
        }
    }
}