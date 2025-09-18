use gpui::{
    App, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled, Window,
};
use gpui_component::{button::Button, h_flex, progress::Progress, v_flex, IconName, Sizable};

use crate::section;

pub struct ProgressStory {
    focus_handle: gpui::FocusHandle,
    value: f32,
}

impl super::Story for ProgressStory {
    fn title() -> &'static str {
        "Progress"
    }

    fn description() -> &'static str {
        "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar."
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl ProgressStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            value: 50.,
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}

impl Focusable for ProgressStory {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ProgressStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().items_center().gap_y_3().child(
            section("Progress Bar").max_w_md().child(
                v_flex()
                    .w_full()
                    .gap_3()
                    .justify_center()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("button-1").small().label("0%").on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value(0.);
                                }),
                            ))
                            .child(Button::new("button-2").small().label("25%").on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value(25.);
                                }),
                            ))
                            .child(Button::new("button-3").small().label("75%").on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value(75.);
                                }),
                            ))
                            .child(Button::new("button-4").small().label("100%").on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value(100.);
                                }),
                            )),
                    )
                    .child(Progress::new().value(self.value))
                    .child(
                        h_flex()
                            .gap_x_2()
                            .child(Button::new("button-5").icon(IconName::Minus).on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value((this.value - 1.).max(0.));
                                }),
                            ))
                            .child(Button::new("button-6").icon(IconName::Plus).on_click(
                                cx.listener(|this, _, _, _| {
                                    this.set_value((this.value + 1.).min(100.));
                                }),
                            )),
                    ),
            ),
        )
    }
}
