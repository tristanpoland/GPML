use std::sync::Arc;

use crate::button::{Button, ButtonVariants as _};
use crate::popup_menu::PopupMenuExt as _;
use crate::{h_flex, ActiveTheme, IconName, Selectable, Sizable, Size, StyledExt};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, Action, AnyElement, App, Corner, Div, Edges, ElementId, IntoElement, ParentElement,
    Pixels, RenderOnce, ScrollHandle, Stateful, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window,
};
use gpui::{px, InteractiveElement};
use smallvec::SmallVec;

use super::{Tab, TabVariant};

#[derive(Action, Debug, Clone, Copy, PartialEq, Eq)]
#[action(namespace = tab_bar, no_json)]
pub struct SelectTab(usize);

#[derive(IntoElement)]
pub struct TabBar {
    base: Stateful<Div>,
    style: StyleRefinement,
    scroll_handle: Option<ScrollHandle>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    children: SmallVec<[Tab; 2]>,
    last_empty_space: AnyElement,
    selected_index: Option<usize>,
    variant: TabVariant,
    size: Size,
    menu: bool,
    on_click: Option<Arc<dyn Fn(&usize, &mut Window, &mut App) + 'static>>,
    /// Special for internal TabPanel to remove the top border.
    tab_item_top_offset: Pixels,
}

impl TabBar {
    /// Create a new TabBar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id).px(px(-1.)),
            style: StyleRefinement::default(),
            children: SmallVec::new(),
            scroll_handle: None,
            prefix: None,
            suffix: None,
            variant: TabVariant::default(),
            size: Size::default(),
            last_empty_space: div().w_3().into_any_element(),
            selected_index: None,
            on_click: None,
            menu: false,
            tab_item_top_offset: px(0.),
        }
    }

    /// Set the Tab variant, all children will inherit the variant.
    pub fn with_variant(mut self, variant: TabVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the Tab variant to Pill, all children will inherit the variant.
    pub fn pill(mut self) -> Self {
        self.variant = TabVariant::Pill;
        self
    }

    /// Set the Tab variant to Outline, all children will inherit the variant.
    pub fn outline(mut self) -> Self {
        self.variant = TabVariant::Outline;
        self
    }

    /// Set the Tab variant to Segmented, all children will inherit the variant.
    pub fn segmented(mut self) -> Self {
        self.variant = TabVariant::Segmented;
        self
    }

    /// Set the Tab variant to Underline, all children will inherit the variant.
    pub fn underline(mut self) -> Self {
        self.variant = TabVariant::Underline;
        self
    }

    /// Enable or disable the popup menu for the TabBar
    pub fn with_menu(mut self, menu: bool) -> Self {
        self.menu = menu;
        self
    }

    /// Track the scroll of the TabBar
    pub fn track_scroll(mut self, scroll_handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(scroll_handle.clone());
        self
    }

    /// Set the prefix element of the TabBar
    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    /// Set the suffix element of the TabBar
    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    /// Add children of the TabBar, all children will inherit the variant.
    ///
    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<Tab>>) -> Self {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    /// Add child of the TabBar, tab will inherit the variant.
    pub fn child(mut self, child: impl Into<Tab>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set the selected index of the TabBar.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Set the last empty space element of the TabBar
    pub fn last_empty_space(mut self, last_empty_space: impl IntoElement) -> Self {
        self.last_empty_space = last_empty_space.into_any_element();
        self
    }

    /// Set the on_click callback of the TabBar, the first parameter is the index of the clicked tab.
    ///
    /// When this is set, the children's on_click will be ignored.
    pub fn on_click(mut self, on_click: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Arc::new(on_click));
        self
    }

    pub(crate) fn tab_item_top_offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.tab_item_top_offset = offset.into();
        self
    }
}

impl Styled for TabBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Sizable for TabBar {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl RenderOnce for TabBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let default_gap = match self.size {
            Size::Small | Size::XSmall => px(8.),
            Size::Large => px(16.),
            _ => px(12.),
        };
        let (bg, paddings, gap) = match self.variant {
            TabVariant::Tab => {
                let padding = Edges::all(px(0.));
                (cx.theme().tab_bar, padding, px(0.))
            }
            TabVariant::Outline => {
                let padding = Edges::all(px(0.));
                (cx.theme().transparent, padding, default_gap)
            }
            TabVariant::Pill => {
                let padding = Edges::all(px(0.));
                (cx.theme().transparent, padding, px(4.))
            }
            TabVariant::Segmented => {
                let padding_x = match self.size {
                    Size::XSmall => px(2.),
                    Size::Small => px(2.),
                    Size::Large => px(6.),
                    _ => px(5.),
                };
                let padding = Edges {
                    left: padding_x,
                    right: padding_x,
                    ..Default::default()
                };

                (cx.theme().tab_bar_segmented, padding, px(2.))
            }
            TabVariant::Underline => {
                // This gap is same as the tab inner_paddings
                let gap = match self.size {
                    Size::XSmall => px(10.),
                    Size::Small => px(12.),
                    Size::Large => px(20.),
                    _ => px(16.),
                };

                (cx.theme().transparent, Edges::all(px(0.)), gap)
            }
        };

        let mut item_labels = Vec::new();
        let selected_index = self.selected_index;

        self.base
            .group("tab-bar")
            .on_action({
                let on_click = self.on_click.clone();
                move |action: &SelectTab, window: &mut Window, cx: &mut App| {
                    if let Some(on_click) = on_click.clone() {
                        on_click(&action.0, window, cx);
                    }
                }
            })
            .relative()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(cx.theme().tab_foreground)
            .when(
                self.variant == TabVariant::Underline || self.variant == TabVariant::Tab,
                |this| {
                    this.child(
                        div()
                            .id("border-b")
                            .absolute()
                            .left_0()
                            .bottom_0()
                            .size_full()
                            .border_b_1()
                            .border_color(cx.theme().border),
                    )
                },
            )
            .when(
                self.variant == TabVariant::Pill || self.variant == TabVariant::Segmented,
                |this| this.rounded(cx.theme().radius),
            )
            .paddings(paddings)
            .refine_style(&self.style)
            .when_some(self.prefix, |this, prefix| this.child(prefix))
            .child(
                h_flex()
                    .id("tabs")
                    .flex_1()
                    .overflow_x_scroll()
                    .when_some(self.scroll_handle, |this, scroll_handle| {
                        this.track_scroll(&scroll_handle)
                    })
                    .gap(gap)
                    .children(self.children.into_iter().enumerate().map(|(ix, child)| {
                        item_labels.push((child.label.clone(), child.disabled));
                        child
                            .id(ix)
                            .mt(self.tab_item_top_offset)
                            .with_variant(self.variant)
                            .with_size(self.size)
                            .when_some(self.selected_index, |this, selected_ix| {
                                this.selected(selected_ix == ix)
                            })
                            .when_some(self.on_click.clone(), move |this, on_click| {
                                this.on_click(move |_, window, cx| on_click(&ix, window, cx))
                            })
                    }))
                    .when(self.suffix.is_some() || self.menu, |this| {
                        this.child(self.last_empty_space)
                    }),
            )
            .when(self.menu, |this| {
                this.child(
                    Button::new("more")
                        .xsmall()
                        .ghost()
                        .icon(IconName::ChevronDown)
                        .popup_menu(move |mut this, _, _| {
                            this = this.scrollable();
                            for (ix, (label, disabled)) in item_labels.iter().enumerate() {
                                this = this.menu_with_check_and_disabled(
                                    label.clone().unwrap_or_default(),
                                    selected_index == Some(ix),
                                    Box::new(SelectTab(ix)),
                                    *disabled,
                                );
                            }

                            this
                        })
                        .anchor(Corner::TopRight),
                )
            })
            .when_some(self.suffix, |this, suffix| this.child(suffix))
    }
}
