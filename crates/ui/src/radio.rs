use std::rc::Rc;

use crate::{
    checkbox::checkbox_check_icon, h_flex, text::Text, v_flex, ActiveTheme, AxisExt, Sizable, Size,
    StyledExt,
};
use gpui::{
    div, prelude::FluentBuilder, relative, rems, AnyElement, App, Axis, Div, ElementId,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, StyleRefinement, Styled, Window,
};

/// A Radio element.
///
/// This is not included the Radio group implementation, you can manage the group by yourself.
#[derive(IntoElement)]
pub struct Radio {
    base: Div,
    style: StyleRefinement,
    id: ElementId,
    label: Option<Text>,
    children: Vec<AnyElement>,
    checked: bool,
    disabled: bool,
    size: Size,
    on_click: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            base: div(),
            style: StyleRefinement::default(),
            label: None,
            children: Vec::new(),
            checked: false,
            disabled: false,
            size: Size::default(),
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<Text>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Sizable for Radio {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for Radio {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
impl InteractiveElement for Radio {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}
impl StatefulInteractiveElement for Radio {}

impl ParentElement for Radio {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Radio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;

        let (border_color, bg) = if checked {
            (cx.theme().primary, cx.theme().primary)
        } else {
            (cx.theme().input, cx.theme().input.opacity(0.3))
        };
        let (border_color, bg) = if disabled {
            (border_color.opacity(0.5), bg.opacity(0.5))
        } else {
            (border_color, bg)
        };

        // wrap a flex to patch for let Radio display inline
        div().child(
            self.base
                .h_flex()
                .id(self.id.clone())
                .gap_x_2()
                .text_color(cx.theme().foreground)
                .items_start()
                .line_height(relative(1.))
                .map(|this| match self.size {
                    Size::XSmall => this.text_xs(),
                    Size::Small => this.text_sm(),
                    Size::Medium => this.text_base(),
                    Size::Large => this.text_lg(),
                    _ => this,
                })
                .refine_style(&self.style)
                .child(
                    div()
                        .relative()
                        .map(|this| match self.size {
                            Size::XSmall => this.size_3(),
                            Size::Small => this.size_3p5(),
                            Size::Medium => this.size_4(),
                            Size::Large => this.size(rems(1.125)),
                            _ => this.size_4(),
                        })
                        .flex_shrink_0()
                        .rounded_full()
                        .border_1()
                        .border_color(border_color)
                        .when(cx.theme().shadow && !disabled, |this| this.shadow_xs())
                        .map(|this| match self.checked {
                            false => this.bg(cx.theme().background),
                            _ => this.bg(bg),
                        })
                        .child(checkbox_check_icon(
                            self.id, self.size, checked, disabled, window, cx,
                        )),
                )
                .child(
                    v_flex()
                        .w_full()
                        .line_height(relative(1.2))
                        .gap_1()
                        .when_some(self.label, |this, label| {
                            this.child(
                                div()
                                    .size_full()
                                    .overflow_hidden()
                                    .line_height(relative(1.))
                                    .when(self.disabled, |this| {
                                        this.text_color(cx.theme().muted_foreground)
                                    })
                                    .child(label),
                            )
                        })
                        .children(self.children),
                )
                .when_some(
                    self.on_click.filter(|_| !self.disabled),
                    |this, on_click| {
                        this.on_click(move |_event, window, cx| {
                            on_click(&!self.checked, window, cx);
                        })
                    },
                ),
        )
    }
}

/// A Radio group element.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    style: StyleRefinement,
    radios: Vec<Radio>,
    layout: Axis,
    selected_index: Option<usize>,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {
    fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default().flex_1(),
            on_change: None,
            layout: Axis::Vertical,
            selected_index: None,
            disabled: false,
            radios: vec![],
        }
    }

    /// Create a new Radio group with default Vertical layout.
    pub fn vertical(id: impl Into<ElementId>) -> Self {
        Self::new(id)
    }

    /// Create a new Radio group with Horizontal layout.
    pub fn horizontal(id: impl Into<ElementId>) -> Self {
        Self::new(id).layout(Axis::Horizontal)
    }

    /// Set the layout of the Radio group. Default is `Axis::Vertical`.
    pub fn layout(mut self, layout: Axis) -> Self {
        self.layout = layout;
        self
    }

    /// Listen to the change event.
    pub fn on_change(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Set the selected index.
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add a child Radio element.
    pub fn child(mut self, child: impl Into<Radio>) -> Self {
        self.radios.push(child.into());
        self
    }

    /// Add multiple child Radio elements.
    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<Radio>>) -> Self {
        self.radios.extend(children.into_iter().map(Into::into));
        self
    }
}

impl Styled for RadioGroup {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl From<&'static str> for Radio {
    fn from(label: &'static str) -> Self {
        Self::new(label).label(label)
    }
}

impl From<SharedString> for Radio {
    fn from(label: SharedString) -> Self {
        Self::new(label.clone()).label(label)
    }
}

impl From<String> for Radio {
    fn from(label: String) -> Self {
        Self::new(SharedString::from(label.clone())).label(SharedString::from(label))
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_change = self.on_change;
        let disabled = self.disabled;
        let selected_ix = self.selected_index;

        let base = if self.layout.is_vertical() {
            v_flex()
        } else {
            h_flex().w_full().flex_wrap()
        };

        let mut container = div().id(self.id);
        *container.style() = self.style;

        container.child(
            base.gap_3()
                .children(self.radios.into_iter().enumerate().map(|(ix, mut radio)| {
                    let checked = selected_ix == Some(ix);

                    radio.id = ix.into();
                    radio.disabled(disabled).checked(checked).when_some(
                        on_change.clone(),
                        |this, on_change| {
                            this.on_click(move |_, window, cx| {
                                on_change(&ix, window, cx);
                            })
                        },
                    )
                })),
        )
    }
}
