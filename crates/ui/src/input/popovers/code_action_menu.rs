use std::rc::Rc;

use gpui::{
    canvas, deferred, div, prelude::FluentBuilder, px, relative, Action, AnyElement, App,
    AppContext, Bounds, Context, DismissEvent, Empty, Entity, EventEmitter,
    InteractiveElement as _, IntoElement, ParentElement, Pixels, Point, Render, RenderOnce,
    SharedString, Styled, StyledText, Subscription, Window,
};
use lsp_types::CodeAction;

const MAX_MENU_WIDTH: Pixels = px(320.);
const MAX_MENU_HEIGHT: Pixels = px(480.);

use crate::{
    actions, h_flex,
    input::{self, InputState},
    list::{List, ListDelegate, ListEvent},
    ActiveTheme, IndexPath, Selectable,
};

#[derive(Debug, Clone)]
pub(crate) struct CodeActionItem {
    /// The `id` of the `CodeActionProvider` that provided this item.
    pub(crate) provider_id: SharedString,
    pub(crate) action: CodeAction,
}

struct MenuDelegate {
    menu: Entity<CodeActionMenu>,
    items: Vec<Rc<CodeActionItem>>,
    selected_ix: usize,
}

impl MenuDelegate {
    fn set_items(&mut self, items: Vec<CodeActionItem>) {
        self.items = items.into_iter().map(Rc::new).collect();
        self.selected_ix = 0;
    }

    fn selected_item(&self) -> Option<&Rc<CodeActionItem>> {
        self.items.get(self.selected_ix)
    }
}

#[derive(IntoElement)]
struct MenuItem {
    ix: usize,
    item: Rc<CodeActionItem>,
    children: Vec<AnyElement>,
    selected: bool,
}

impl MenuItem {
    fn new(ix: usize, item: Rc<CodeActionItem>) -> Self {
        Self {
            ix,
            item,
            children: vec![],
            selected: false,
        }
    }
}
impl Selectable for MenuItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl ParentElement for MenuItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}
impl RenderOnce for MenuItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let item = self.item;

        let highlights = vec![];

        h_flex()
            .id(self.ix)
            .gap_2()
            .p_1()
            .text_xs()
            .line_height(relative(1.))
            .rounded_sm()
            .hover(|this| this.bg(cx.theme().accent.opacity(0.8)))
            .when(self.selected, |this| {
                this.bg(cx.theme().accent)
                    .text_color(cx.theme().accent_foreground)
            })
            .child(
                div().child(StyledText::new(item.action.title.clone()).with_highlights(highlights)),
            )
            .children(self.children)
    }
}

impl EventEmitter<DismissEvent> for MenuDelegate {}

impl ListDelegate for MenuDelegate {
    type Item = MenuItem;

    fn items_count(&self, _: usize, _: &gpui::App) -> usize {
        self.items.len()
    }

    fn render_item(
        &self,
        ix: crate::IndexPath,
        _: &mut Window,
        _: &mut Context<List<Self>>,
    ) -> Option<Self::Item> {
        let item = self.items.get(ix.row)?;
        Some(MenuItem::new(ix.row, item.clone()))
    }

    fn set_selected_index(
        &mut self,
        ix: Option<crate::IndexPath>,
        _: &mut Window,
        cx: &mut Context<List<Self>>,
    ) {
        self.selected_ix = ix.map(|i| i.row).unwrap_or(0);
        cx.notify();
    }

    fn confirm(&mut self, _: bool, window: &mut Window, cx: &mut Context<List<Self>>) {
        let Some(item) = self.selected_item() else {
            return;
        };

        self.menu.update(cx, |this, cx| {
            this.select_item(&item, window, cx);
        });
    }
}

/// A context menu for code completions and code actions.
pub struct CodeActionMenu {
    offset: usize,
    state: Entity<InputState>,
    list: Entity<List<MenuDelegate>>,
    open: bool,
    bounds: Bounds<Pixels>,

    _subscriptions: Vec<Subscription>,
}

impl CodeActionMenu {
    /// Creates a new `CompletionMenu` with the given offset and completion items.
    ///
    /// NOTE: This element should not call from InputState::new, unless that will stack overflow.
    pub(crate) fn new(
        state: Entity<InputState>,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let view = cx.entity();
            let menu = MenuDelegate {
                menu: view,
                items: vec![],
                selected_ix: 0,
            };

            let list = cx.new(|cx| {
                List::new(menu, window, cx)
                    .no_query()
                    .max_h(MAX_MENU_HEIGHT)
            });

            let _subscriptions =
                vec![
                    cx.subscribe(&list, |this: &mut Self, _, ev: &ListEvent, cx| {
                        match ev {
                            ListEvent::Confirm(_) => {
                                this.hide(cx);
                            }
                            _ => {}
                        }
                        cx.notify();
                    }),
                ];

            Self {
                offset: 0,
                state,
                list,
                open: false,
                bounds: Bounds::default(),
                _subscriptions,
            }
        })
    }

    fn select_item(&mut self, item: &CodeActionItem, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let item = item.clone();

        cx.spawn_in(window, {
            async move |_, cx| {
                state.update_in(cx, |state, window, cx| {
                    state.perform_code_action(&item, window, cx);
                })
            }
        })
        .detach();

        self.hide(cx);
    }

    pub(crate) fn handle_action(
        &mut self,
        action: Box<dyn Action>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.open {
            return false;
        }

        cx.propagate();
        if action.partial_eq(&input::Enter { secondary: false }) {
            self.on_action_enter(window, cx);
        } else if action.partial_eq(&input::Escape) {
            self.on_action_escape(window, cx);
        } else if action.partial_eq(&input::MoveUp) {
            self.on_action_up(window, cx);
        } else if action.partial_eq(&input::MoveDown) {
            self.on_action_down(window, cx);
        } else {
            return false;
        }

        true
    }

    fn on_action_enter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.list.read(cx).delegate().selected_item().cloned() else {
            return;
        };
        self.select_item(&item, window, cx);
    }

    fn on_action_escape(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.hide(cx);
    }

    fn on_action_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |this, cx| {
            this.on_action_select_prev(&actions::SelectPrev, window, cx)
        });
    }

    fn on_action_down(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |this, cx| {
            this.on_action_select_next(&actions::SelectNext, window, cx)
        });
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    /// Hide the completion menu and reset the trigger start offset.
    pub(crate) fn hide(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    pub(crate) fn show(
        &mut self,
        offset: usize,
        items: impl Into<Vec<CodeActionItem>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let items = items.into();
        self.offset = offset;
        self.open = true;
        self.list.update(cx, |this, cx| {
            this.delegate_mut().set_items(items);
            this.set_selected_index(Some(IndexPath::new(0)), window, cx);
        });

        cx.notify();
    }

    fn origin(&self, cx: &App) -> Option<Point<Pixels>> {
        let state = self.state.read(cx);
        let Some(last_layout) = state.last_layout.as_ref() else {
            return None;
        };
        let Some(cursor_origin) = last_layout.cursor_bounds.map(|b| b.origin) else {
            return None;
        };

        let scroll_origin = self.state.read(cx).scroll_handle.offset();

        Some(
            scroll_origin + cursor_origin - state.input_bounds.origin
                + Point::new(-px(4.), last_layout.line_height + px(4.)),
        )
    }
}

impl Render for CodeActionMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return Empty.into_any_element();
        }

        if self.list.read(cx).delegate().items.is_empty() {
            self.open = false;
            return Empty.into_any_element();
        }

        let view = cx.entity();

        let Some(pos) = self.origin(cx) else {
            return Empty.into_any_element();
        };

        let max_width = MAX_MENU_WIDTH.min(window.bounds().size.width - pos.x);

        deferred(
            div()
                .id("code-action-menu")
                .absolute()
                .left(pos.x)
                .top(pos.y)
                .flex_none()
                .occlude()
                .p_1()
                .text_xs()
                .text_color(cx.theme().popover_foreground)
                .bg(cx.theme().popover)
                .border_1()
                .border_color(cx.theme().border)
                .rounded(cx.theme().radius)
                .shadow_md()
                .max_w(max_width)
                .min_w(px(120.))
                .child(self.list.clone())
                .child(
                    canvas(
                        move |bounds, _, cx| view.update(cx, |r, _| r.bounds = bounds),
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.hide(cx);
                })),
        )
        .into_any_element()
    }
}
