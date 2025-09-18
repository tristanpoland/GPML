use gpui::{
    px, App, AppContext as _, ClickEvent, Context, Entity, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, ParentElement as _, Render, Styled, Window,
};

use crate::{section, Tab, TabPrev};
use gpui_component::{
    button::Button,
    h_flex,
    input::{InputState, TextInput},
    v_flex, FocusableCycle, Sizable,
};

const CONTEXT: &str = "TextareaStory";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("shift-tab", TabPrev, Some(CONTEXT)),
        KeyBinding::new("tab", Tab, Some(CONTEXT)),
    ])
}

pub struct TextareaStory {
    textarea: Entity<InputState>,
    textarea_auto_grow: Entity<InputState>,
    textarea_no_wrap: Entity<InputState>,
}

impl super::Story for TextareaStory {
    fn title() -> &'static str {
        "Textarea"
    }

    fn description() -> &'static str {
        "TextInput with multi-line mode."
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl TextareaStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let textarea = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .rows(10)
                .placeholder("Enter text here...")
                .searchable(true)
                .default_value(
                    unindent::unindent(
                        r#"Hello 世界，this is GPUI component.

                    The GPUI Component is a collection of UI components for GPUI framework, including.

                    Button, Input, Checkbox, Radio, Dropdown, Tab, and more...

                    Here is an application that is built by using GPUI Component.

                    > This application is still under development, not published yet.

                    ![image](https://github.com/user-attachments/assets/559a648d-19df-4b5a-b563-b78cc79c8894)

                    ![image](https://github.com/user-attachments/assets/5e06ad5d-7ea0-43db-8d13-86a240da4c8d)

                    ## Demo

                    If you want to see the demo, here is a some demo applications.
                    "#,
                    )
                )
        });

        let textarea_auto_grow = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(1, 5)
                .placeholder("Enter text here...")
                .default_value("Hello 世界，this is GPUI component.")
        });

        let textarea_no_wrap = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line()
                .rows(6)
                .soft_wrap(false)
                .default_value("This is a very long line of text to test if the horizontal scrolling function is working properly, and it should not wrap automatically but display a horizontal scrollbar.\nThe second line is also very long text, used to test the horizontal scrolling effect under multiple lines, and you can input more content to test.\nThe third line: Here you can input other long text content that requires horizontal scrolling.\n")
        });

        Self {
            textarea,
            textarea_auto_grow,
            textarea_no_wrap,
        }
    }

    fn tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(true, window, cx);
    }

    fn tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.cycle_focus(false, window, cx);
    }

    fn on_insert_text_to_textarea(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.textarea.update(cx, |input, cx| {
            input.insert("Hello 你好", window, cx);
        });
    }

    fn on_replace_text_to_textarea(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.textarea.update(cx, |input, cx| {
            input.replace("Hello 你好", window, cx);
        });
    }
}

impl FocusableCycle for TextareaStory {
    fn cycle_focus_handles(&self, _: &mut Window, _: &mut App) -> Vec<FocusHandle> {
        [].to_vec()
    }
}
impl Focusable for TextareaStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.textarea.focus_handle(cx)
    }
}

impl Render for TextareaStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let loc = self.textarea.read(cx).cursor_position();

        v_flex()
            .key_context(CONTEXT)
            .id("textarea-story")
            .on_action(cx.listener(Self::tab))
            .on_action(cx.listener(Self::tab_prev))
            .gap_3()
            .child(
                section("Textarea").child(
                    v_flex()
                        .gap_2()
                        .w_full()
                        .child(TextInput::new(&self.textarea).h(px(320.)))
                        .child(
                            h_flex()
                                .justify_between()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("btn-insert-text")
                                                .outline()
                                                .xsmall()
                                                .label("Insert Text")
                                                .on_click(
                                                    cx.listener(Self::on_insert_text_to_textarea),
                                                ),
                                        )
                                        .child(
                                            Button::new("btn-replace-text")
                                                .outline()
                                                .xsmall()
                                                .label("Replace Text")
                                                .on_click(
                                                    cx.listener(Self::on_replace_text_to_textarea),
                                                ),
                                        ),
                                )
                                .child(format!("{}:{}", loc.line, loc.character)),
                        ),
                ),
            )
            .child(section("Textarea Auto Grow").child(TextInput::new(&self.textarea_auto_grow)))
            .child(
                section("No Wrap")
                    .max_w_md()
                    .child(TextInput::new(&self.textarea_no_wrap).h(px(200.))),
            )
    }
}
