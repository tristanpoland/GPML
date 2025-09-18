use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, relative, AnyElement, App, DefiniteLength, Edges, EdgesRefinement, Entity,
    InteractiveElement as _, IntoElement, IsZero, MouseButton, ParentElement as _, Pixels, Rems,
    RenderOnce, StyleRefinement, Styled, Window,
};

use crate::button::{Button, ButtonVariants as _};
use crate::indicator::Indicator;
use crate::input::clear_button;
use crate::input::element::{LINE_NUMBER_RIGHT_MARGIN, RIGHT_MARGIN};
use crate::scroll::Scrollbar;
use crate::{h_flex, StyledExt};
use crate::{v_flex, ActiveTheme};
use crate::{IconName, Size};
use crate::{Sizable, StyleSized};

use super::InputState;

#[derive(IntoElement)]
pub struct TextInput {
    state: Entity<InputState>,
    style: StyleRefinement,
    size: Size,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    height: Option<DefiniteLength>,
    appearance: bool,
    cleanable: bool,
    mask_toggle: bool,
    disabled: bool,
    bordered: bool,
    focus_bordered: bool,
}

impl Sizable for TextInput {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl TextInput {
    /// Create a new [`TextInput`] element bind to the [`InputState`].
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            size: Size::default(),
            style: StyleRefinement::default(),
            prefix: None,
            suffix: None,
            height: None,
            appearance: true,
            cleanable: false,
            mask_toggle: false,
            disabled: false,
            bordered: true,
            focus_bordered: true,
        }
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    /// Set full height of the input (Multi-line only).
    pub fn h_full(mut self) -> Self {
        self.height = Some(relative(1.));
        self
    }

    /// Set height of the input (Multi-line only).
    pub fn h(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Set the appearance of the input field, if false the input field will no border, background.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    /// Set the bordered for the input, default: true
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Set focus border for the input, default is true.
    pub fn focus_bordered(mut self, bordered: bool) -> Self {
        self.focus_bordered = bordered;
        self
    }

    /// Set true to show the clear button when the input field is not empty.
    pub fn cleanable(mut self) -> Self {
        self.cleanable = true;
        self
    }

    /// Set to enable toggle button for password mask state.
    pub fn mask_toggle(mut self) -> Self {
        self.mask_toggle = true;
        self
    }

    /// Set to disable the input field.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn render_toggle_mask_button(state: Entity<InputState>) -> impl IntoElement {
        Button::new("toggle-mask")
            .icon(IconName::Eye)
            .xsmall()
            .ghost()
            .on_mouse_down(MouseButton::Left, {
                let state = state.clone();
                move |_, window, cx| {
                    state.update(cx, |state, cx| {
                        state.set_masked(false, window, cx);
                    })
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let state = state.clone();
                move |_, window, cx| {
                    state.update(cx, |state, cx| {
                        state.set_masked(true, window, cx);
                    })
                }
            })
    }

    /// This method must after the refine_style.
    fn render_editor(
        paddings: EdgesRefinement<DefiniteLength>,
        input_state: &Entity<InputState>,
        state: &InputState,
        window: &Window,
        _cx: &App,
    ) -> impl IntoElement {
        let base_size = window.text_style().font_size;
        let rem_size = window.rem_size();

        let paddings = Edges {
            left: paddings
                .left
                .map(|v| v.to_pixels(base_size, rem_size))
                .unwrap_or(px(0.)),
            right: paddings
                .right
                .map(|v| v.to_pixels(base_size, rem_size))
                .unwrap_or(px(0.)),
            top: paddings
                .top
                .map(|v| v.to_pixels(base_size, rem_size))
                .unwrap_or(px(0.)),
            bottom: paddings
                .bottom
                .map(|v| v.to_pixels(base_size, rem_size))
                .unwrap_or(px(0.)),
        };

        const MIN_SCROLL_PADDING: Pixels = px(2.0);

        v_flex()
            .size_full()
            .children(state.search_panel.clone())
            .child(div().flex_1().child(input_state.clone()).map(|this| {
                if let Some(last_layout) = state.last_layout.as_ref() {
                    let left = if last_layout.line_number_width.is_zero() {
                        px(0.)
                    } else {
                        // Align left edge to the Line number.
                        paddings.left + last_layout.line_number_width - LINE_NUMBER_RIGHT_MARGIN
                    };

                    let scroll_size = gpui::Size {
                        width: state.scroll_size.width - left + paddings.right + RIGHT_MARGIN,
                        height: state.scroll_size.height,
                    };

                    let scrollbar = if !state.soft_wrap {
                        Scrollbar::both(&state.scroll_state, &state.scroll_handle)
                    } else {
                        Scrollbar::vertical(&state.scroll_state, &state.scroll_handle)
                    };

                    this.relative().child(
                        div()
                            .absolute()
                            .top(-paddings.top + MIN_SCROLL_PADDING)
                            .left(left)
                            .right(-paddings.right + MIN_SCROLL_PADDING)
                            .bottom(-paddings.bottom + MIN_SCROLL_PADDING)
                            .child(scrollbar.scroll_size(scroll_size)),
                    )
                } else {
                    this
                }
            }))
    }
}

impl Styled for TextInput {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for TextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        const LINE_HEIGHT: Rems = Rems(1.25);
        let font = window.text_style().font();
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());

        self.state.update(cx, |state, cx| {
            state.text_wrapper.set_font(font, font_size, cx);
            state.disabled = self.disabled;
        });

        let state = self.state.read(cx);
        let focused = state.focus_handle.is_focused(window);
        let gap_x = match self.size {
            Size::Small => px(4.),
            Size::Large => px(8.),
            _ => px(4.),
        };

        let bg = if state.disabled {
            cx.theme().muted
        } else {
            cx.theme().background
        };

        let prefix = self.prefix;
        let suffix = self.suffix;
        let show_clear_button =
            self.cleanable && !state.loading && state.text.len() > 0 && state.mode.is_single_line();
        let has_suffix = suffix.is_some() || state.loading || self.mask_toggle || show_clear_button;

        div()
            .id(("input", self.state.entity_id()))
            .flex()
            .key_context(crate::input::CONTEXT)
            .track_focus(&state.focus_handle)
            .when(!state.disabled, |this| {
                this.on_action(window.listener_for(&self.state, InputState::backspace))
                    .on_action(window.listener_for(&self.state, InputState::delete))
                    .on_action(
                        window.listener_for(&self.state, InputState::delete_to_beginning_of_line),
                    )
                    .on_action(window.listener_for(&self.state, InputState::delete_to_end_of_line))
                    .on_action(window.listener_for(&self.state, InputState::delete_previous_word))
                    .on_action(window.listener_for(&self.state, InputState::delete_next_word))
                    .on_action(window.listener_for(&self.state, InputState::enter))
                    .on_action(window.listener_for(&self.state, InputState::escape))
                    .on_action(window.listener_for(&self.state, InputState::paste))
                    .on_action(window.listener_for(&self.state, InputState::cut))
                    .on_action(window.listener_for(&self.state, InputState::undo))
                    .on_action(window.listener_for(&self.state, InputState::redo))
                    .when(state.mode.is_multi_line(), |this| {
                        this.on_action(window.listener_for(&self.state, InputState::indent_inline))
                            .on_action(window.listener_for(&self.state, InputState::outdent_inline))
                            .on_action(window.listener_for(&self.state, InputState::indent_block))
                            .on_action(window.listener_for(&self.state, InputState::outdent_block))
                    })
                    .on_action(window.listener_for(&self.state, InputState::toggle_code_actions))
            })
            .on_action(window.listener_for(&self.state, InputState::left))
            .on_action(window.listener_for(&self.state, InputState::right))
            .on_action(window.listener_for(&self.state, InputState::select_left))
            .on_action(window.listener_for(&self.state, InputState::select_right))
            .when(state.mode.is_multi_line(), |this| {
                this.on_action(window.listener_for(&self.state, InputState::up))
                    .on_action(window.listener_for(&self.state, InputState::down))
                    .on_action(window.listener_for(&self.state, InputState::select_up))
                    .on_action(window.listener_for(&self.state, InputState::select_down))
                    .on_action(window.listener_for(&self.state, InputState::page_up))
                    .on_action(window.listener_for(&self.state, InputState::page_down))
            })
            .on_action(window.listener_for(&self.state, InputState::select_all))
            .on_action(window.listener_for(&self.state, InputState::select_to_start_of_line))
            .on_action(window.listener_for(&self.state, InputState::select_to_end_of_line))
            .on_action(window.listener_for(&self.state, InputState::select_to_previous_word))
            .on_action(window.listener_for(&self.state, InputState::select_to_next_word))
            .on_action(window.listener_for(&self.state, InputState::home))
            .on_action(window.listener_for(&self.state, InputState::end))
            .on_action(window.listener_for(&self.state, InputState::move_to_start))
            .on_action(window.listener_for(&self.state, InputState::move_to_end))
            .on_action(window.listener_for(&self.state, InputState::move_to_previous_word))
            .on_action(window.listener_for(&self.state, InputState::move_to_next_word))
            .on_action(window.listener_for(&self.state, InputState::select_to_start))
            .on_action(window.listener_for(&self.state, InputState::select_to_end))
            .on_action(window.listener_for(&self.state, InputState::show_character_palette))
            .on_action(window.listener_for(&self.state, InputState::copy))
            .on_action(window.listener_for(&self.state, InputState::on_action_search))
            .on_key_down(window.listener_for(&self.state, InputState::on_key_down))
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&self.state, InputState::on_mouse_down),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&self.state, InputState::on_mouse_up),
            )
            .on_mouse_move(window.listener_for(&self.state, InputState::on_mouse_move))
            .on_scroll_wheel(window.listener_for(&self.state, InputState::on_scroll_wheel))
            .size_full()
            .line_height(LINE_HEIGHT)
            .input_px(self.size)
            .input_py(self.size)
            .input_h(self.size)
            .cursor_text()
            .text_size(font_size)
            .items_center()
            .when(state.mode.is_multi_line(), |this| {
                this.h_auto()
                    .when_some(self.height, |this, height| this.h(height))
            })
            .when(self.appearance, |this| {
                this.bg(bg)
                    .rounded(cx.theme().radius)
                    .when(self.bordered, |this| {
                        this.border_color(cx.theme().input)
                            .border_1()
                            .when(cx.theme().shadow, |this| this.shadow_xs())
                            .when(focused && self.focus_bordered, |this| {
                                this.focused_border(cx)
                            })
                    })
            })
            .items_center()
            .gap(gap_x)
            .refine_style(&self.style)
            .children(prefix)
            .when(state.mode.is_multi_line(), |mut this| {
                let paddings = this.style().padding.clone();
                this.child(Self::render_editor(
                    paddings,
                    &self.state,
                    &state,
                    window,
                    cx,
                ))
            })
            .when(!state.mode.is_multi_line(), |this| {
                this.child(self.state.clone())
            })
            .when(has_suffix, |this| {
                this.pr(self.size.input_px() / 2.).child(
                    h_flex()
                        .id("suffix")
                        .gap(gap_x)
                        .when(self.appearance, |this| this.bg(bg))
                        .items_center()
                        .when(state.loading, |this| {
                            this.child(Indicator::new().color(cx.theme().muted_foreground))
                        })
                        .when(self.mask_toggle, |this| {
                            this.child(Self::render_toggle_mask_button(self.state.clone()))
                        })
                        .when(show_clear_button, |this| {
                            this.child(clear_button(cx).on_click({
                                let state = self.state.clone();
                                move |_, window, cx| {
                                    state.update(cx, |state, cx| {
                                        state.clean(window, cx);
                                    })
                                }
                            }))
                        })
                        .children(suffix),
                )
            })
    }
}
