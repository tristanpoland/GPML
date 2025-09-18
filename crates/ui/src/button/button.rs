use std::rc::Rc;

use crate::{
    h_flex, indicator::Indicator, tooltip::Tooltip, ActiveTheme, Colorize as _, Disableable, Icon,
    Selectable, Sizable, Size, StyleSized, StyledExt,
};
use gpui::{
    div, prelude::FluentBuilder as _, relative, Action, AnyElement, App, ClickEvent, Corners, Div,
    Edges, ElementId, Hsla, InteractiveElement, Interactivity, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, SharedString, Stateful, StatefulInteractiveElement as _,
    StyleRefinement, Styled, Window,
};

#[derive(Default, Clone, Copy)]
pub enum ButtonRounded {
    None,
    Small,
    #[default]
    Medium,
    Large,
    Size(Pixels),
}

impl From<Pixels> for ButtonRounded {
    fn from(px: Pixels) -> Self {
        ButtonRounded::Size(px)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ButtonCustomVariant {
    color: Hsla,
    foreground: Hsla,
    border: Hsla,
    shadow: bool,
    hover: Hsla,
    active: Hsla,
}

pub trait ButtonVariants: Sized {
    fn with_variant(self, variant: ButtonVariant) -> Self;

    /// With the primary style for the Button.
    fn primary(self) -> Self {
        self.with_variant(ButtonVariant::Primary)
    }

    /// With the danger style for the Button.
    fn danger(self) -> Self {
        self.with_variant(ButtonVariant::Danger)
    }

    /// With the warning style for the Button.
    fn warning(self) -> Self {
        self.with_variant(ButtonVariant::Warning)
    }

    /// With the success style for the Button.
    fn success(self) -> Self {
        self.with_variant(ButtonVariant::Success)
    }

    /// With the info style for the Button.
    fn info(self) -> Self {
        self.with_variant(ButtonVariant::Info)
    }

    /// With the ghost style for the Button.
    fn ghost(self) -> Self {
        self.with_variant(ButtonVariant::Ghost)
    }

    /// With the link style for the Button.
    fn link(self) -> Self {
        self.with_variant(ButtonVariant::Link)
    }

    /// With the text style for the Button, it will no padding look like a normal text.
    fn text(self) -> Self {
        self.with_variant(ButtonVariant::Text)
    }

    /// With the custom style for the Button.
    fn custom(self, style: ButtonCustomVariant) -> Self {
        self.with_variant(ButtonVariant::Custom(style))
    }
}

impl ButtonCustomVariant {
    pub fn new(cx: &App) -> Self {
        Self {
            color: cx.theme().transparent,
            foreground: cx.theme().foreground,
            border: cx.theme().transparent,
            hover: cx.theme().transparent,
            active: cx.theme().transparent,
            shadow: false,
        }
    }

    /// Set background color, default is transparent.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    /// Set foreground color, default is theme foreground.
    pub fn foreground(mut self, color: Hsla) -> Self {
        self.foreground = color;
        self
    }

    /// Set border color, default is transparent.
    pub fn border(mut self, color: Hsla) -> Self {
        self.border = color;
        self
    }

    /// Set hover background color, default is transparent.
    pub fn hover(mut self, color: Hsla) -> Self {
        self.hover = color;
        self
    }

    /// Set active background color, default is transparent.
    pub fn active(mut self, color: Hsla) -> Self {
        self.active = color;
        self
    }

    /// Set shadow, default is false.
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }
}

/// The veriant of the Button.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Info,
    Success,
    Warning,
    Ghost,
    Link,
    Text,
    Custom(ButtonCustomVariant),
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Secondary
    }
}

impl ButtonVariant {
    #[inline]
    pub fn is_link(&self) -> bool {
        matches!(self, Self::Link)
    }

    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text)
    }

    #[inline]
    pub fn is_ghost(&self) -> bool {
        matches!(self, Self::Ghost)
    }

    #[inline]
    fn no_padding(&self) -> bool {
        self.is_link() || self.is_text()
    }
}

/// A Button element.
#[derive(IntoElement)]
pub struct Button {
    base: Stateful<Div>,
    style: StyleRefinement,
    icon: Option<Icon>,
    label: Option<SharedString>,
    children: Vec<AnyElement>,
    disabled: bool,
    pub(crate) selected: bool,
    variant: ButtonVariant,
    rounded: ButtonRounded,
    outline: bool,
    border_corners: Corners<bool>,
    border_edges: Edges<bool>,
    size: Size,
    compact: bool,
    tooltip: Option<(
        SharedString,
        Option<(Rc<Box<dyn Action>>, Option<SharedString>)>,
    )>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    pub(crate) stop_propagation: bool,
    loading: bool,
    loading_icon: Option<Icon>,
}

impl From<Button> for AnyElement {
    fn from(button: Button) -> Self {
        button.into_any_element()
    }
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id.into()).flex_shrink_0(),
            style: StyleRefinement::default(),
            icon: None,
            label: None,
            disabled: false,
            selected: false,
            variant: ButtonVariant::default(),
            rounded: ButtonRounded::Medium,
            border_corners: Corners::all(true),
            border_edges: Edges::all(true),
            size: Size::Medium,
            tooltip: None,
            on_click: None,
            stop_propagation: true,
            loading: false,
            compact: false,
            outline: false,
            children: Vec::new(),
            loading_icon: None,
        }
    }

    /// Set the outline style of the Button.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    /// Set the border radius of the Button.
    pub fn rounded(mut self, rounded: impl Into<ButtonRounded>) -> Self {
        self.rounded = rounded.into();
        self
    }

    /// Set the border corners side of the Button.
    pub(crate) fn border_corners(mut self, corners: impl Into<Corners<bool>>) -> Self {
        self.border_corners = corners.into();
        self
    }

    /// Set the border edges of the Button.
    pub(crate) fn border_edges(mut self, edges: impl Into<Edges<bool>>) -> Self {
        self.border_edges = edges.into();
        self
    }

    /// Set label to the Button, if no label is set, the button will be in Icon Button mode.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the icon of the button, if the Button have no label, the button well in Icon Button mode.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the tooltip of the button.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some((tooltip.into(), None));
        self
    }

    pub fn tooltip_with_action(
        mut self,
        tooltip: impl Into<SharedString>,
        action: &dyn Action,
        context: Option<&str>,
    ) -> Self {
        self.tooltip = Some((
            tooltip.into(),
            Some((
                Rc::new(action.boxed_clone()),
                context.map(|c| c.to_string().into()),
            )),
        ));
        self
    }

    /// Set true to show the loading indicator.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set the button to compact mode, then padding will be reduced.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn stop_propagation(mut self, val: bool) -> Self {
        self.stop_propagation = val;
        self
    }

    pub fn loading_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.loading_icon = Some(icon.into());
        self
    }
}

impl Disableable for Button {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Selectable for Button {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Sizable for Button {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl ButtonVariants for Button {
    fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style: ButtonVariant = self.variant;
        let normal_style = style.normal(self.outline, cx);
        let icon_size = match self.size {
            Size::Size(v) => Size::Size(v * 0.75),
            _ => self.size,
        };

        self.base
            .flex_shrink_0()
            .cursor_default()
            .flex()
            .items_center()
            .justify_center()
            .when(self.variant.is_link(), |this| this.cursor_pointer())
            .overflow_hidden()
            .when(cx.theme().shadow && normal_style.shadow, |this| {
                this.shadow_xs()
            })
            .when(!style.no_padding(), |this| {
                if self.label.is_none() && self.children.is_empty() {
                    // Icon Button
                    match self.size {
                        Size::Size(px) => this.size(px),
                        Size::XSmall => this.size_5(),
                        Size::Small => this.size_6(),
                        Size::Large | Size::Medium => this.size_8(),
                    }
                } else {
                    // Normal Button
                    match self.size {
                        Size::Size(size) => this.px(size * 0.2),
                        Size::XSmall => this.h_5().px_1(),
                        Size::Small => this.h_6().px_3().when(self.compact, |this| this.px_1p5()),
                        _ => this.h_8().px_4().when(self.compact, |this| this.px_2()),
                    }
                }
            })
            .when(
                self.border_corners.top_left && self.border_corners.bottom_left,
                |this| match self.rounded {
                    ButtonRounded::Small => this.rounded_l(cx.theme().radius * 0.5),
                    ButtonRounded::Medium => this.rounded_l(cx.theme().radius),
                    ButtonRounded::Large => this.rounded_l(cx.theme().radius * 2.0),
                    ButtonRounded::Size(px) => this.rounded_l(px),
                    ButtonRounded::None => this.rounded_none(),
                },
            )
            .when(
                self.border_corners.top_right && self.border_corners.bottom_right,
                |this| match self.rounded {
                    ButtonRounded::Small => this.rounded_r(cx.theme().radius * 0.5),
                    ButtonRounded::Medium => this.rounded_r(cx.theme().radius),
                    ButtonRounded::Large => this.rounded_r(cx.theme().radius * 2.0),
                    ButtonRounded::Size(px) => this.rounded_r(px),
                    ButtonRounded::None => this.rounded_none(),
                },
            )
            .when(self.border_edges.left, |this| this.border_l_1())
            .when(self.border_edges.right, |this| this.border_r_1())
            .when(self.border_edges.top, |this| this.border_t_1())
            .when(self.border_edges.bottom, |this| this.border_b_1())
            .text_color(normal_style.fg)
            .when(self.selected, |this| {
                let selected_style = style.selected(self.outline, cx);
                this.bg(selected_style.bg)
                    .border_color(selected_style.border)
                    .text_color(selected_style.fg)
            })
            .when(!self.disabled && !self.selected, |this| {
                this.border_color(normal_style.border)
                    .bg(normal_style.bg)
                    .when(normal_style.underline, |this| this.text_decoration_1())
                    .hover(|this| {
                        let hover_style = style.hovered(self.outline, cx);
                        this.bg(hover_style.bg)
                            .border_color(hover_style.border)
                            .text_color(crate::red_400())
                    })
                    .active(|this| {
                        let active_style = style.active(self.outline, cx);
                        this.bg(active_style.bg)
                            .border_color(active_style.border)
                            .text_color(active_style.fg)
                    })
            })
            .when(self.disabled, |this| {
                let disabled_style = style.disabled(self.outline, cx);
                this.bg(disabled_style.bg)
                    .text_color(disabled_style.fg)
                    .border_color(disabled_style.border)
                    .shadow_none()
            })
            .refine_style(&self.style)
            .when_some(
                self.on_click.filter(|_| !self.disabled && !self.loading),
                |this, on_click| {
                    let stop_propagation = self.stop_propagation;
                    this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        window.prevent_default();
                        if stop_propagation {
                            cx.stop_propagation();
                        }
                    })
                    .on_click(move |event, window, cx| {
                        (on_click)(event, window, cx);
                    })
                },
            )
            .child({
                h_flex()
                    .id("label")
                    .items_center()
                    .justify_center()
                    .button_text_size(self.size)
                    .map(|this| match self.size {
                        Size::XSmall => this.gap_1(),
                        Size::Small => this.gap_1(),
                        _ => this.gap_2(),
                    })
                    .when(!self.loading, |this| {
                        this.when_some(self.icon, |this, icon| {
                            this.child(icon.with_size(icon_size))
                        })
                    })
                    .when(self.loading, |this| {
                        this.child(
                            Indicator::new()
                                .with_size(self.size)
                                .when_some(self.loading_icon, |this, icon| this.icon(icon)),
                        )
                    })
                    .when_some(self.label, |this, label| {
                        this.child(div().flex_none().line_height(relative(1.)).child(label))
                    })
                    .children(self.children)
            })
            .when(self.loading && !self.disabled, |this| {
                this.bg(normal_style.bg.opacity(0.8))
                    .border_color(normal_style.border.opacity(0.8))
                    .text_color(normal_style.fg.opacity(0.8))
            })
            .when_some(self.tooltip, |this, (tooltip, action)| {
                this.tooltip(move |window, cx| {
                    Tooltip::new(tooltip.clone())
                        .when_some(action.clone(), |this, (action, context)| {
                            this.action(
                                action.boxed_clone().as_ref(),
                                context.as_ref().map(|c| c.as_ref()),
                            )
                        })
                        .build(window, cx)
                })
            })
    }
}

struct ButtonVariantStyle {
    bg: Hsla,
    border: Hsla,
    fg: Hsla,
    underline: bool,
    shadow: bool,
}

impl ButtonVariant {
    fn bg_color(&self, outline: bool, cx: &mut App) -> Hsla {
        if outline {
            return cx.theme().background;
        }

        match self {
            ButtonVariant::Primary => cx.theme().primary,
            ButtonVariant::Secondary => cx.theme().secondary,
            ButtonVariant::Danger => cx.theme().danger,
            ButtonVariant::Warning => cx.theme().warning,
            ButtonVariant::Success => cx.theme().success,
            ButtonVariant::Info => cx.theme().info,
            ButtonVariant::Ghost | ButtonVariant::Link | ButtonVariant::Text => {
                cx.theme().transparent
            }
            ButtonVariant::Custom(colors) => colors.color,
        }
    }

    fn text_color(&self, outline: bool, cx: &mut App) -> Hsla {
        match self {
            ButtonVariant::Primary => {
                if outline {
                    cx.theme().primary
                } else {
                    cx.theme().primary_foreground
                }
            }
            ButtonVariant::Secondary | ButtonVariant::Ghost => cx.theme().secondary_foreground,
            ButtonVariant::Danger => {
                if outline {
                    cx.theme().danger
                } else {
                    cx.theme().danger_foreground
                }
            }
            ButtonVariant::Warning => {
                if outline {
                    cx.theme().warning
                } else {
                    cx.theme().warning_foreground
                }
            }
            ButtonVariant::Success => {
                if outline {
                    cx.theme().success
                } else {
                    cx.theme().success_foreground
                }
            }
            ButtonVariant::Info => {
                if outline {
                    cx.theme().info
                } else {
                    cx.theme().info_foreground
                }
            }
            ButtonVariant::Link => cx.theme().link,
            ButtonVariant::Text => cx.theme().foreground,
            ButtonVariant::Custom(colors) => {
                if outline {
                    colors.color
                } else {
                    colors.foreground
                }
            }
        }
    }

    fn border_color(&self, bg: Hsla, outline: bool, cx: &mut App) -> Hsla {
        match self {
            ButtonVariant::Secondary => {
                if outline {
                    cx.theme().border
                } else {
                    bg
                }
            }
            ButtonVariant::Primary => {
                if outline {
                    cx.theme().primary
                } else {
                    bg
                }
            }
            ButtonVariant::Danger => {
                if outline {
                    cx.theme().danger
                } else {
                    bg
                }
            }
            ButtonVariant::Info => {
                if outline {
                    cx.theme().info
                } else {
                    bg
                }
            }
            ButtonVariant::Warning => {
                if outline {
                    cx.theme().warning
                } else {
                    bg
                }
            }
            ButtonVariant::Success => {
                if outline {
                    cx.theme().success
                } else {
                    bg
                }
            }
            ButtonVariant::Ghost | ButtonVariant::Link | ButtonVariant::Text => {
                cx.theme().transparent
            }
            ButtonVariant::Custom(colors) => colors.border,
        }
    }

    fn underline(&self, _: &App) -> bool {
        match self {
            ButtonVariant::Link => true,
            _ => false,
        }
    }

    fn shadow(&self, outline: bool, _: &App) -> bool {
        match self {
            ButtonVariant::Primary | ButtonVariant::Secondary | ButtonVariant::Danger => outline,
            ButtonVariant::Custom(c) => c.shadow,
            _ => false,
        }
    }

    fn normal(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = self.bg_color(outline, cx);
        let border = self.border_color(bg, outline, cx);
        let fg = self.text_color(outline, cx);
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn hovered(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = match self {
            ButtonVariant::Primary => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    cx.theme().primary_hover
                }
            }
            ButtonVariant::Secondary => cx.theme().secondary_hover,
            ButtonVariant::Danger => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    cx.theme().danger_hover
                }
            }
            ButtonVariant::Warning => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    cx.theme().warning_hover
                }
            }
            ButtonVariant::Success => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    cx.theme().success_hover
                }
            }
            ButtonVariant::Info => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    cx.theme().info_hover
                }
            }
            ButtonVariant::Ghost => {
                if cx.theme().mode.is_dark() {
                    cx.theme().secondary.lighten(0.1).opacity(0.8)
                } else {
                    cx.theme().secondary.darken(0.1).opacity(0.8)
                }
            }
            ButtonVariant::Link => cx.theme().transparent,
            ButtonVariant::Text => cx.theme().transparent,
            ButtonVariant::Custom(colors) => {
                if outline {
                    cx.theme().secondary_hover
                } else {
                    colors.hover
                }
            }
        };

        let border = self.border_color(bg, outline, cx);
        let fg = match self {
            ButtonVariant::Link => cx.theme().link_hover,
            _ => self.text_color(outline, cx),
        };

        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn active(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = match self {
            ButtonVariant::Primary => {
                if outline {
                    cx.theme().primary_active.opacity(0.1)
                } else {
                    cx.theme().primary_active
                }
            }
            ButtonVariant::Secondary => cx.theme().secondary_active,
            ButtonVariant::Ghost => {
                if cx.theme().mode.is_dark() {
                    cx.theme().secondary.lighten(0.2).opacity(0.8)
                } else {
                    cx.theme().secondary.darken(0.2).opacity(0.8)
                }
            }
            ButtonVariant::Danger => {
                if outline {
                    cx.theme().danger_active.opacity(0.1)
                } else {
                    cx.theme().danger_active
                }
            }
            ButtonVariant::Warning => {
                if outline {
                    cx.theme().warning_active.opacity(0.1)
                } else {
                    cx.theme().warning_active
                }
            }
            ButtonVariant::Success => {
                if outline {
                    cx.theme().success_active.opacity(0.1)
                } else {
                    cx.theme().success_active
                }
            }
            ButtonVariant::Info => {
                if outline {
                    cx.theme().info_active.opacity(0.1)
                } else {
                    cx.theme().info_active
                }
            }
            ButtonVariant::Link => cx.theme().transparent,
            ButtonVariant::Text => cx.theme().transparent,
            ButtonVariant::Custom(colors) => {
                if outline {
                    colors.active.opacity(0.1)
                } else {
                    colors.active
                }
            }
        };
        let border = self.border_color(bg, outline, cx);
        let fg = match self {
            ButtonVariant::Link => cx.theme().link_active,
            ButtonVariant::Text => cx.theme().foreground.opacity(0.7),
            _ => self.text_color(outline, cx),
        };
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn selected(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = match self {
            ButtonVariant::Primary => cx.theme().primary_active,
            ButtonVariant::Secondary | ButtonVariant::Ghost => cx.theme().secondary_active,
            ButtonVariant::Danger => cx.theme().danger_active,
            ButtonVariant::Warning => cx.theme().warning_active,
            ButtonVariant::Success => cx.theme().success_active,
            ButtonVariant::Info => cx.theme().info_active,
            ButtonVariant::Link => cx.theme().transparent,
            ButtonVariant::Text => cx.theme().transparent,
            ButtonVariant::Custom(colors) => colors.active,
        };

        let border = self.border_color(bg, outline, cx);
        let fg = match self {
            ButtonVariant::Link => cx.theme().link_active,
            ButtonVariant::Text => cx.theme().foreground.opacity(0.7),
            _ => self.text_color(false, cx),
        };
        let underline = self.underline(cx);
        let shadow = self.shadow(outline, cx);

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }

    fn disabled(&self, outline: bool, cx: &mut App) -> ButtonVariantStyle {
        let bg = match self {
            ButtonVariant::Link | ButtonVariant::Ghost | ButtonVariant::Text => {
                cx.theme().transparent
            }
            ButtonVariant::Primary => cx.theme().primary.opacity(0.15),
            ButtonVariant::Danger => cx.theme().danger.opacity(0.15),
            ButtonVariant::Warning => cx.theme().warning.opacity(0.15),
            ButtonVariant::Success => cx.theme().success.opacity(0.15),
            ButtonVariant::Info => cx.theme().info.opacity(0.15),
            ButtonVariant::Secondary => cx.theme().secondary.opacity(1.5),
            ButtonVariant::Custom(style) => style.color.opacity(0.15),
        };
        let fg = cx.theme().muted_foreground.opacity(0.5);
        let (bg, border) = if outline {
            (cx.theme().transparent, cx.theme().border.opacity(0.5))
        } else {
            (bg, bg)
        };

        let underline = self.underline(cx);
        let shadow = false;

        ButtonVariantStyle {
            bg,
            border,
            fg,
            underline,
            shadow,
        }
    }
}
