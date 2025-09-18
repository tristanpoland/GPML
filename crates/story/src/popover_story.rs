use gpui::{
    actions, div, px, Action, App, AppContext, Context, Corner, DismissEvent, Element, Entity,
    EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement, KeyBinding, MouseButton,
    ParentElement as _, Render, Styled as _, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants as _},
    divider::Divider,
    h_flex,
    input::{InputState, TextInput},
    popover::{Popover, PopoverContent},
    v_flex, ContextModal, Sizable,
};
use serde::Deserialize;

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = popover_story, no_json)]
struct Info(usize);

actions!(popover_story, [Copy, Paste, Cut, SearchAll, ToggleCheck]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-c", Copy, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-c", Copy, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-v", Paste, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-v", Paste, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-x", Cut, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-x", Cut, None),
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-shift-f", SearchAll, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-shift-f", SearchAll, None),
    ])
}

struct Form {
    input1: Entity<InputState>,
}

impl Form {
    fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            input1: cx.new(|cx| InputState::new(window, cx)),
        })
    }
}

impl Focusable for Form {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input1.focus_handle(cx)
    }
}

impl EventEmitter<DismissEvent> for Form {}

impl Render for Form {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .p_4()
            .size_full()
            .child("This is a form container.")
            .child(TextInput::new(&self.input1))
            .child(
                Button::new("submit")
                    .label("Submit")
                    .primary()
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
            )
    }
}

pub struct PopoverStory {
    focus_handle: FocusHandle,
    form: Entity<Form>,
    checked: bool,
    message: String,
}

impl super::Story for PopoverStory {
    fn title() -> &'static str {
        "Popover"
    }

    fn description() -> &'static str {
        "A popup displays content on top of the main page."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl PopoverStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let form = Form::new(window, cx);

        cx.focus_self(window);

        Self {
            form,
            checked: true,
            focus_handle: cx.focus_handle(),
            message: "".to_string(),
        }
    }

    fn on_copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.message = "You have clicked copy".to_string();
        cx.notify()
    }

    fn on_cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        self.message = "You have clicked cut".to_string();
        cx.notify()
    }

    fn on_paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        self.message = "You have clicked paste".to_string();
        cx.notify()
    }

    fn on_search_all(&mut self, _: &SearchAll, _: &mut Window, cx: &mut Context<Self>) {
        self.message = "You have clicked search all".to_string();
        cx.notify()
    }

    fn on_action_info(&mut self, info: &Info, _: &mut Window, cx: &mut Context<Self>) {
        self.message = format!("You have clicked info: {}", info.0);
        cx.notify()
    }

    fn on_action_toggle_check(&mut self, _: &ToggleCheck, _: &mut Window, cx: &mut Context<Self>) {
        self.checked = !self.checked;
        self.message = format!("You have clicked toggle check: {}", self.checked);
        cx.notify()
    }
}

impl Focusable for PopoverStory {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let form = self.form.clone();

        v_flex()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_copy))
            .on_action(cx.listener(Self::on_cut))
            .on_action(cx.listener(Self::on_paste))
            .on_action(cx.listener(Self::on_search_all))
            .on_action(cx.listener(Self::on_action_info))
            .on_action(cx.listener(Self::on_action_toggle_check))
            .size_full()
            .min_h(px(400.))
            .gap_6()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex().gap_4().child(
                            Popover::new("info-top-left")
                                .trigger(Button::new("info-top-left").outline().label("Top Left"))
                                .content(|window, cx| {
                                    cx.new(|cx| {
                                        PopoverContent::new(window, cx, |_, _| {
                                            v_flex()
                                                .gap_4()
                                                .child("Hello, this is a Popover.")
                                                .w(px(400.))
                                                .child(Divider::horizontal())
                                                .child(
                                                    Button::new("info1")
                                                        .primary()
                                                        .label("Ok")
                                                        .w(px(80.))
                                                        .small(),
                                                )
                                                .into_any()
                                        })
                                        .p_4()
                                        .max_w(px(600.))
                                    })
                                }),
                        ),
                    )
                    .child(
                        Popover::new("info-top-right")
                            .anchor(Corner::TopRight)
                            .trigger(Button::new("info-top-right").outline().label("Top Right"))
                            .content(|window, cx| {
                                cx.new(|cx| {
                                    PopoverContent::new(window, cx, |_, _| {
                                        v_flex()
                                            .gap_4()
                                            .w_96()
                                            .child("Hello, this is a Popover on the Top Right.")
                                            .child(Divider::horizontal())
                                            .child(
                                                Button::new("info1")
                                                    .primary()
                                                    .label("Ok")
                                                    .w(px(80.))
                                                    .small(),
                                            )
                                            .into_any()
                                    })
                                    .p_4()
                                })
                            }),
                    ),
            )
            .child(
                div().absolute().bottom_4().left_0().w_full().h_10().child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            Popover::new("info-bottom-left")
                                .anchor(Corner::BottomLeft)
                                .trigger(
                                    Button::new("pop")
                                        .outline()
                                        .label("Popup with Form")
                                        .w(px(300.)),
                                )
                                .content(move |_, _| form.clone()),
                        )
                        .child(
                            Popover::new("info-bottom-right")
                                .anchor(Corner::BottomRight)
                                .mouse_button(MouseButton::Right)
                                .trigger(
                                    Button::new("pop")
                                        .outline()
                                        .label("Mouse Right Click")
                                        .w(px(300.)),
                                )
                                .content(|window, cx| {
                                    cx.new(|cx| {
                                        PopoverContent::new(window, cx, |_, cx| {
                                            v_flex()
                                                .gap_2()
                                                .child(
                                                    "Hello, this is a Popover on the Bottom Right.",
                                                )
                                                .child(Divider::horizontal())
                                                .child(
                                                    h_flex()
                                                        .gap_2()
                                                        .justify_end()
                                                        .child(
                                                            Button::new("info1")
                                                                .primary()
                                                                .label("Ok")
                                                                .w(px(80.))
                                                                .small()
                                                                .on_click(cx.listener(
                                                                    |_, _, window, cx| {
                                                                        window.push_notification(
                                                                            "You have clicked Ok.",
                                                                            cx,
                                                                        );
                                                                        cx.emit(DismissEvent);
                                                                    },
                                                                )),
                                                        )
                                                        .child(
                                                            Button::new("close")
                                                                .label("Cancel")
                                                                .small()
                                                                .on_click(cx.listener(
                                                                    |_, _, _, cx| {
                                                                        cx.emit(DismissEvent);
                                                                    },
                                                                )),
                                                        ),
                                                )
                                                .into_any()
                                        })
                                        .p_4()
                                    })
                                }),
                        ),
                ),
            )
    }
}
