use aho_corasick::AhoCorasick;
use rust_i18n::t;
use std::{ops::Range, rc::Rc};

use gpui::{
    actions, div, prelude::FluentBuilder as _, App, AppContext as _, Context, Empty, Entity,
    EntityInputHandler, FocusHandle, Focusable, Half, InteractiveElement as _, IntoElement,
    KeyBinding, ParentElement as _, Render, Styled, Subscription, Window,
};
use rope::Rope;

use crate::{
    actions::SelectPrev,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Enter, Escape, IndentInline, InputEvent, InputState, RopeExt, Search, TextInput},
    v_flex, ActiveTheme, IconName, Selectable, Sizable,
};

const KEY_CONTEXT: &'static str = "SearchPanel";

actions!(input, [Tab]);

pub(super) fn init(cx: &mut App) {
    cx.bind_keys(vec![KeyBinding::new(
        "shift-enter",
        SelectPrev,
        Some(KEY_CONTEXT),
    )]);
}

#[derive(Debug, Clone)]
pub struct SearchMatcher {
    text: Rope,
    pub query: Option<AhoCorasick>,

    pub(super) matched_ranges: Rc<Vec<Range<usize>>>,
    pub(super) current_match_ix: usize,
    /// Is in replacing mode, if true, the next update will not reset the current match index.
    replacing: bool,
}

impl SearchMatcher {
    pub fn new() -> Self {
        Self {
            text: "".into(),
            query: None,
            matched_ranges: Rc::new(Vec::new()),
            current_match_ix: 0,
            replacing: false,
        }
    }

    /// Update source text and re-match
    pub(crate) fn update(&mut self, text: &Rope) {
        if self.text.eq(text) {
            return;
        }

        self.text = text.clone();
        self.update_matches();
    }

    fn update_matches(&mut self) {
        let mut new_ranges = Vec::new();
        if let Some(query) = &self.query {
            let matches = query.stream_find_iter(self.text.bytes_in_range(0..self.text.len()));

            for query_match in matches.into_iter() {
                let query_match = query_match.expect("query match for select all action");
                new_ranges.push(query_match.range());
            }
        }
        self.matched_ranges = Rc::new(new_ranges);
        if !self.replacing {
            self.current_match_ix = 0;
            self.replacing = false;
        }
    }

    /// Update the search query and reset the current match index.
    pub fn update_query(&mut self, query: &str, case_insensitive: bool) {
        if query.len() > 0 {
            self.query = Some(
                AhoCorasick::builder()
                    .ascii_case_insensitive(case_insensitive)
                    .build(&[query.to_string()])
                    .expect("failed to build AhoCorasick query in SearchMatcher"),
            );
        } else {
            self.query = None;
        }
        self.update_matches();
    }

    /// Returns the number of matches found.
    #[allow(unused)]
    #[inline]
    fn len(&self) -> usize {
        self.matched_ranges.len()
    }

    fn peek(&self) -> Option<Range<usize>> {
        self.matched_ranges.get(self.current_match_ix + 1).cloned()
    }
}

impl Iterator for SearchMatcher {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.matched_ranges.is_empty() {
            return None;
        }

        if self.current_match_ix < self.matched_ranges.len().saturating_sub(1) {
            self.current_match_ix += 1;
        } else {
            self.current_match_ix = 0;
        }

        self.matched_ranges.get(self.current_match_ix).cloned()
    }
}

impl DoubleEndedIterator for SearchMatcher {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.matched_ranges.is_empty() {
            return None;
        }

        if self.current_match_ix == 0 {
            self.current_match_ix = self.matched_ranges.len();
        }

        self.current_match_ix -= 1;
        let item = self.matched_ranges[self.current_match_ix].clone();

        Some(item)
    }
}

pub(super) struct SearchPanel {
    text_state: Entity<InputState>,
    search_input: Entity<InputState>,
    replace_input: Entity<InputState>,
    case_insensitive: bool,
    replace_mode: bool,
    matcher: SearchMatcher,

    open: bool,
    _subscriptions: Vec<Subscription>,
}

impl InputState {
    /// Update the search matcher when text changes.
    pub(super) fn update_search(&mut self, cx: &mut App) {
        let Some(search_panel) = self.search_panel.as_ref() else {
            return;
        };

        let text = self.text.clone();
        search_panel.update(cx, |this, _| {
            this.matcher.update(&text);
        });
    }

    pub(super) fn on_action_search(
        &mut self,
        _: &Search,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.searchable {
            return;
        }

        let search_panel = match self.search_panel.as_ref() {
            Some(panel) => panel.clone(),
            None => SearchPanel::new(cx.entity(), window, cx),
        };

        let text = self.text.clone();
        let text_state = cx.entity();
        let selected_text = self.selected_text();
        search_panel.update(cx, |this, cx| {
            this.text_state = text_state;
            this.matcher.update(&text);
            this.show(&selected_text, window, cx);
        });
        self.search_panel = Some(search_panel);
        cx.notify();
    }
}

impl SearchPanel {
    pub fn new(text_state: Entity<InputState>, window: &mut Window, cx: &mut App) -> Entity<Self> {
        let search_input = cx.new(|cx| InputState::new(window, cx));
        let replace_input = cx.new(|cx| InputState::new(window, cx));

        cx.new(|cx| {
            let _subscriptions = vec![cx.subscribe(
                &search_input,
                |this: &mut Self, search_input, ev: &InputEvent, cx| {
                    // Handle search input changes
                    match ev {
                        InputEvent::Change => {
                            let value = search_input.read(cx).value();
                            this.matcher
                                .update_query(value.as_str(), this.case_insensitive);
                        }
                        _ => {}
                    }
                },
            )];

            Self {
                text_state,
                search_input,
                replace_input,
                case_insensitive: true,
                replace_mode: false,
                matcher: SearchMatcher::new(),
                open: true,
                _subscriptions,
            }
        })
    }

    pub(super) fn show(
        &mut self,
        selected_text: &Rope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open = true;
        self.search_input.read(cx).focus_handle.focus(window);

        self.search_input.update(cx, |this, cx| {
            if selected_text.len() > 0 {
                this.set_value(selected_text.to_string(), window, cx);
            }
            this.select_all(&super::SelectAll, window, cx);
        });
        self.update_search(cx);
        cx.notify();
    }

    fn update_search(&mut self, cx: &mut Context<Self>) {
        let query = self.search_input.read(cx).value();
        self.matcher
            .update_query(query.as_str(), self.case_insensitive);
        self.update_text_selection(cx);
    }

    pub(super) fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        self.text_state.read(cx).focus_handle.focus(window);
        cx.notify();
    }

    fn on_action_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.prev(window, cx);
    }

    fn on_action_next(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        self.next(window, cx);
    }

    fn on_action_escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>) {
        self.hide(window, cx);
    }

    fn on_action_tab(&mut self, _: &IndentInline, window: &mut Window, cx: &mut Context<Self>) {
        self.text_state.focus_handle(cx).focus(window);
    }

    fn update_text_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(range) = self
            .matcher
            .matched_ranges
            .get(self.matcher.current_match_ix)
            .cloned()
        {
            let state = self.text_state.clone();
            cx.spawn(async move |_, cx| {
                _ = cx.update(|cx| {
                    state.update(cx, |state, cx| {
                        state.selected_range = range.into();
                        cx.notify();
                    });
                });
            })
            .detach();
        }
    }

    fn prev(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.matcher.next_back() {
            self.text_state.update(cx, |state, cx| {
                state.scroll_to(range.start, cx);
            });
        }
    }

    fn next(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.matcher.next() {
            self.text_state.update(cx, |state, cx| {
                state.scroll_to(range.end, cx);
            });
        }
    }

    pub(super) fn matcher(&self) -> Option<&SearchMatcher> {
        if !self.open {
            return None;
        }

        Some(&self.matcher)
    }

    fn replace_next(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_text = self.replace_input.read(cx).value();
        self.matcher.replacing = true;
        if let Some(range) = self
            .matcher
            .matched_ranges
            .get(self.matcher.current_match_ix)
            .cloned()
        {
            let text_state = self.text_state.clone();

            let next_range = self.matcher.peek().unwrap_or(range.clone());
            cx.spawn_in(window, async move |_, cx| {
                cx.update(|window, cx| {
                    text_state.update(cx, |state, cx| {
                        let range_utf16 = state.range_to_utf16(&range);
                        state.scroll_to(next_range.end, cx);
                        state.replace_text_in_range(
                            Some(range_utf16),
                            new_text.as_str(),
                            window,
                            cx,
                        );
                    });
                })
            })
            .detach();
        }
    }

    fn replace_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_text = self.replace_input.read(cx).value();
        self.matcher.replacing = true;
        let ranges = self.matcher.matched_ranges.clone();
        if ranges.is_empty() {
            return;
        }

        let text_state = self.text_state.clone();
        cx.spawn_in(window, async move |_, cx| {
            cx.update(|window, cx| {
                text_state.update(cx, |state, cx| {
                    // Replace from the end to avoid messing up the ranges.
                    let mut rope = state.text.clone();
                    for range in ranges.iter().rev() {
                        rope.replace(range.clone(), new_text.as_str());
                    }
                    state.replace_text_in_range(
                        Some(0..state.text.len()),
                        &rope.to_string(),
                        window,
                        cx,
                    );
                    state.scroll_to(0, cx);
                });
            })
        })
        .detach();
    }
}

impl Focusable for SearchPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.search_input.read(cx).focus_handle.clone()
    }
}

impl Render for SearchPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return Empty.into_any_element();
        }

        v_flex()
            .id("search-panel")
            .occlude()
            .track_focus(&self.focus_handle(cx))
            .key_context(KEY_CONTEXT)
            .on_action(cx.listener(Self::on_action_prev))
            .on_action(cx.listener(Self::on_action_next))
            .on_action(cx.listener(Self::on_action_escape))
            .on_action(cx.listener(Self::on_action_tab))
            .font_family(".SystemUIFont")
            .items_center()
            .py_2()
            .px_3()
            .w_full()
            .gap_1()
            .bg(cx.theme().popover)
            .border_b_1()
            .rounded(cx.theme().radius.half())
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        div().flex_1().gap_1().child(
                            TextInput::new(&self.search_input)
                                .focus_bordered(false)
                                .suffix(
                                    Button::new("case-insensitive")
                                        .selected(!self.case_insensitive)
                                        .xsmall()
                                        .compact()
                                        .ghost()
                                        .icon(IconName::CaseSensitive)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.case_insensitive = !this.case_insensitive;
                                            this.update_search(cx);
                                            cx.notify();
                                        })),
                                )
                                .small()
                                .w_full()
                                .cleanable()
                                .shadow_none(),
                        ),
                    )
                    .child(
                        Button::new("replace-mode")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Replace)
                            .selected(self.replace_mode)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.replace_mode = !this.replace_mode;
                                this.replace_input.read(cx).focus_handle.focus(window);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("prev")
                            .xsmall()
                            .ghost()
                            .icon(IconName::ChevronLeft)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.prev(window, cx);
                            })),
                    )
                    .child(
                        Button::new("next")
                            .xsmall()
                            .ghost()
                            .icon(IconName::ChevronRight)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.next(window, cx);
                            })),
                    )
                    .child(div().w_5())
                    .child(
                        Button::new("close")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Close)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.on_action_escape(&Escape, window, cx);
                            })),
                    ),
            )
            .when(self.replace_mode, |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(
                            TextInput::new(&self.replace_input)
                                .focus_bordered(false)
                                .small()
                                .w_full()
                                .shadow_none(),
                        )
                        .child(
                            Button::new("replace-one")
                                .small()
                                .label(t!("Input.Replace"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.replace_next(window, cx);
                                })),
                        )
                        .child(
                            Button::new("replace-all")
                                .small()
                                .label(t!("Input.Replace All"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.replace_all(window, cx);
                                })),
                        ),
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let mut search = SearchMatcher::new();
        search.update(&Rope::from("Hello 世界 this is a Is test string."));
        search.update_query("Is", true);

        assert_eq!(search.len(), 3);
        let mut matches = search.clone().into_iter();
        assert_eq!(matches.current_match_ix, 0);
        assert_eq!(matches.next(), Some(18..20));
        assert_eq!(matches.next(), Some(23..25));
        assert_eq!(matches.current_match_ix, 2);
        assert_eq!(matches.next(), Some(15..17));
        assert_eq!(matches.current_match_ix, 0);
        assert_eq!(matches.next_back(), Some(23..25));
        assert_eq!(matches.current_match_ix, 2);
        assert_eq!(matches.next_back(), Some(18..20));
        assert_eq!(matches.current_match_ix, 1);
        assert_eq!(matches.next_back(), Some(15..17));
        assert_eq!(matches.current_match_ix, 0);
        assert_eq!(matches.next_back(), Some(23..25));

        search.update_query("IS", false);
        assert_eq!(search.len(), 0);
        assert_eq!(search.next(), None);
        assert_eq!(search.next_back(), None);
    }
}
