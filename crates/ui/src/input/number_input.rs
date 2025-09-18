use gpui::{
    actions, prelude::FluentBuilder as _, px, AnyElement, App, Context, Entity, EventEmitter,
    FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};

use crate::{
    button::{Button, ButtonVariants as _},
    h_flex, ActiveTheme, Disableable, IconName, Sizable, Size, StyleSized, StyledExt as _,
};

use super::{InputState, TextInput};

actions!(number_input, [Increment, Decrement]);

const KEY_CONTENT: &str = "NumberInput";

pub fn init(cx: &mut App) {
    cx.bind_keys(vec![
        KeyBinding::new("up", Increment, Some(KEY_CONTENT)),
        KeyBinding::new("down", Decrement, Some(KEY_CONTENT)),
    ]);
}

#[derive(IntoElement)]
pub struct NumberInput {
    state: Entity<InputState>,
    placeholder: SharedString,
    size: Size,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    appearance: bool,
    disabled: bool,
}

impl NumberInput {
    /// Create a new [`NumberInput`] element bind to the [`InputState`].
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            size: Size::default(),
            placeholder: SharedString::default(),
            prefix: None,
            suffix: None,
            appearance: true,
            disabled: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    pub fn increment(state: &Entity<InputState>, window: &mut Window, cx: &mut App) {
        state.update(cx, |state, cx| {
            state.on_action_increment(&Increment, window, cx);
        })
    }

    pub fn decrement(state: &Entity<InputState>, window: &mut Window, cx: &mut App) {
        state.update(cx, |state, cx| {
            state.on_action_decrement(&Decrement, window, cx);
        })
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    /// Set the appearance of the number input, if false will no border and background.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }
}

impl Disableable for NumberInput {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl InputState {
    fn on_action_increment(&mut self, _: &Increment, window: &mut Window, cx: &mut Context<Self>) {
        self.on_number_input_step(StepAction::Increment, window, cx);
    }

    fn on_action_decrement(&mut self, _: &Decrement, window: &mut Window, cx: &mut Context<Self>) {
        self.on_number_input_step(StepAction::Decrement, window, cx);
    }

    fn on_number_input_step(&mut self, action: StepAction, _: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }

        cx.emit(NumberInputEvent::Step(action));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepAction {
    Decrement,
    Increment,
}
pub enum NumberInputEvent {
    Step(StepAction),
}
impl EventEmitter<NumberInputEvent> for InputState {}

impl Focusable for NumberInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Sizable for NumberInput {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}
impl RenderOnce for NumberInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focused = self.state.focus_handle(cx).is_focused(window);

        h_flex()
            .id(("number-input", self.state.entity_id()))
            .key_context(KEY_CONTENT)
            .on_action(window.listener_for(&self.state, InputState::on_action_increment))
            .on_action(window.listener_for(&self.state, InputState::on_action_decrement))
            .flex_1()
            .input_size(self.size)
            .px(self.size.input_px() / 2.)
            .when(self.appearance, |this| {
                this.bg(cx.theme().background)
                    .border_color(cx.theme().input)
                    .border_1()
                    .rounded(cx.theme().radius)
            })
            .when(self.disabled, |this| this.bg(cx.theme().muted))
            .when(focused, |this| this.focused_border(cx))
            .child(
                Button::new("minus")
                    .ghost()
                    .with_size(self.size.smaller())
                    .icon(IconName::Minus)
                    .compact()
                    .disabled(self.disabled)
                    .on_click({
                        let state = self.state.clone();
                        move |_, window, cx| {
                            Self::decrement(&state, window, cx);
                        }
                    }),
            )
            .child(
                TextInput::new(&self.state)
                    .appearance(false)
                    .disabled(self.disabled)
                    .px(px(2.))
                    .gap_0()
                    .when_some(self.prefix, |this, prefix| this.prefix(prefix))
                    .when_some(self.suffix, |this, suffix| this.suffix(suffix)),
            )
            .child(
                Button::new("plus")
                    .ghost()
                    .with_size(self.size.smaller())
                    .icon(IconName::Plus)
                    .compact()
                    .disabled(self.disabled)
                    .on_click({
                        let state = self.state.clone();
                        move |_, window, cx| {
                            Self::increment(&state, window, cx);
                        }
                    }),
            )
    }
}
