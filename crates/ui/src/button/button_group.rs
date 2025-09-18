use gpui::{
    div, prelude::FluentBuilder as _, App, Corners, Edges, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window,
};
use std::{cell::Cell, rc::Rc};

use crate::{
    button::{Button, ButtonVariant, ButtonVariants},
    Disableable, Sizable, Size, StyledExt,
};

/// A ButtonGroup element, to wrap multiple buttons in a group.
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    style: StyleRefinement,
    children: Vec<Button>,
    pub(super) multiple: bool,
    pub(super) disabled: bool,

    // The button props
    pub(super) compact: bool,
    pub(super) outline: bool,
    pub(super) variant: Option<ButtonVariant>,
    pub(super) size: Option<Size>,

    on_click: Option<Box<dyn Fn(&Vec<usize>, &mut Window, &mut App) + 'static>>,
}

impl Disableable for ButtonGroup {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl ButtonGroup {
    /// Creates a new ButtonGroup.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            children: Vec::new(),
            variant: None,
            size: None,
            compact: false,
            outline: false,
            multiple: false,
            disabled: false,
            on_click: None,
        }
    }

    /// Adds a button as a child to the ButtonGroup.
    pub fn child(mut self, child: Button) -> Self {
        self.children.push(child.disabled(self.disabled));
        self
    }

    /// Adds multiple buttons as children to the ButtonGroup.
    pub fn children(mut self, children: impl IntoIterator<Item = Button>) -> Self {
        self.children.extend(children);
        self
    }

    /// With the multiple selection mode.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// With the compact mode for the ButtonGroup.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// With the outline mode for the ButtonGroup.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    /// Sets the on_click handler for the ButtonGroup.
    ///
    /// The handler first argument is a vector of the selected button indices.
    ///
    /// The `&Vec<usize>` is the indices of the clicked (selected in `multiple` mode) buttons.
    /// For example: `[0, 2, 3]` is means the first, third and fourth buttons are clicked.
    ///
    /// ```rust
    /// ButtonGroup::new("size-button")
    ///    .child(Button::new("large").label("Large").selected(self.size == Size::Large))
    ///    .child(Button::new("medium").label("Medium").selected(self.size == Size::Medium))
    ///    .child(Button::new("small").label("Small").selected(self.size == Size::Small))
    ///    .on_click(cx.listener(|view, clicks: &Vec<usize>, _, cx| {
    ///        if clicks.contains(&0) {
    ///            view.size = Size::Large;
    ///        } else if clicks.contains(&1) {
    ///            view.size = Size::Medium;
    ///        } else if clicks.contains(&2) {
    ///            view.size = Size::Small;
    ///        }
    ///        cx.notify();
    ///    }))
    /// ```
    pub fn on_click(
        mut self,
        handler: impl Fn(&Vec<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl Sizable for ButtonGroup {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl Styled for ButtonGroup {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl ButtonVariants for ButtonGroup {
    fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = Some(variant);
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let children_len = self.children.len();
        let mut selected_ixs: Vec<usize> = Vec::new();
        let state = Rc::new(Cell::new(None));

        for (ix, child) in self.children.iter().enumerate() {
            if child.selected {
                selected_ixs.push(ix);
            }
        }

        div()
            .id(self.id)
            .flex()
            .items_center()
            .refine_style(&self.style)
            .children(
                self.children
                    .into_iter()
                    .enumerate()
                    .map(|(child_index, child)| {
                        let state = Rc::clone(&state);
                        let child = if children_len == 1 {
                            child
                        } else if child_index == 0 {
                            // First
                            child
                                .border_corners(Corners {
                                    top_left: true,
                                    top_right: false,
                                    bottom_left: true,
                                    bottom_right: false,
                                })
                                .border_edges(Edges {
                                    left: true,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                        } else if child_index == children_len - 1 {
                            // Last
                            child
                                .border_edges(Edges {
                                    left: false,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                                .border_corners(Corners {
                                    top_left: false,
                                    top_right: true,
                                    bottom_left: false,
                                    bottom_right: true,
                                })
                        } else {
                            // Middle
                            child
                                .border_corners(Corners::all(false))
                                .border_edges(Edges {
                                    left: false,
                                    top: true,
                                    right: true,
                                    bottom: true,
                                })
                        }
                        .stop_propagation(false)
                        .when_some(self.size, |this, size| this.with_size(size))
                        .when_some(self.variant, |this, variant| this.with_variant(variant))
                        .when(self.compact, |this| this.compact())
                        .when(self.outline, |this| this.outline())
                        .on_click(move |_, _, _| {
                            state.set(Some(child_index));
                        });

                        child
                    }),
            )
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                move |this, on_click| {
                    this.on_click(move |_, window, cx| {
                        let mut selected_ixs = selected_ixs.clone();
                        if let Some(ix) = state.get() {
                            if self.multiple {
                                if let Some(pos) = selected_ixs.iter().position(|&i| i == ix) {
                                    selected_ixs.remove(pos);
                                } else {
                                    selected_ixs.push(ix);
                                }
                            } else {
                                selected_ixs.clear();
                                selected_ixs.push(ix);
                            }
                        }

                        on_click(&selected_ixs, window, cx);
                    })
                },
            )
    }
}
