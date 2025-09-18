use gpui::{
    div, prelude::FluentBuilder as _, px, App, AppContext, Axis, Context, Entity, Focusable,
    InteractiveElement, IntoElement, ParentElement as _, Render, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonGroup},
    checkbox::Checkbox,
    color_picker::{ColorPicker, ColorPickerState},
    date_picker::{DatePicker, DatePickerState},
    divider::Divider,
    dropdown::{Dropdown, DropdownState},
    form::{form_field, v_form},
    h_flex,
    input::{InputState, TextInput},
    switch::Switch,
    v_flex, ActiveTheme, AxisExt, FocusableCycle, IndexPath, Selectable, Sizable, Size,
};

pub struct FormStory {
    name_prefix_state: Entity<DropdownState<Vec<String>>>,
    name_input: Entity<InputState>,
    email_input: Entity<InputState>,
    bio_input: Entity<InputState>,
    color_state: Entity<ColorPickerState>,
    subscribe_email: bool,
    date: Entity<DatePickerState>,
    layout: Axis,
    size: Size,
    column: u16,
}

impl super::Story for FormStory {
    fn title() -> &'static str {
        "Form"
    }

    fn description() -> &'static str {
        "Form to collect multiple inputs."
    }

    fn closable() -> bool {
        false
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        Self::view(window, cx)
    }
}

impl FormStory {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }

    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name_prefix_state = cx.new(|cx| {
            DropdownState::new(
                vec![
                    "Mr.".to_string(),
                    "Mrs.".to_string(),
                    "Ms.".to_string(),
                    "Dr.".to_string(),
                ],
                Some(IndexPath::default()),
                window,
                cx,
            )
        });

        let name_input = cx.new(|cx| InputState::new(window, cx).default_value("Jason Lee"));
        let color_state = cx.new(|cx| ColorPickerState::new(window, cx));

        let email_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Enter text here..."));
        let bio_input = cx.new(|cx| {
            InputState::new(window, cx)
                .auto_grow(5, 20)
                .placeholder("Enter text here...")
                .default_value("Hello 世界，this is GPUI component.")
        });
        let date = cx.new(|cx| DatePickerState::new(window, cx));

        Self {
            name_prefix_state,
            name_input,
            email_input,
            bio_input,
            date,
            color_state,
            subscribe_email: false,
            layout: Axis::Vertical,
            size: Size::default(),
            column: 1,
        }
    }
}

impl FocusableCycle for FormStory {
    fn cycle_focus_handles(&self, _: &mut Window, cx: &mut App) -> Vec<gpui::FocusHandle>
    where
        Self: Sized,
    {
        vec![
            self.name_input.focus_handle(cx),
            self.email_input.focus_handle(cx),
            self.bio_input.focus_handle(cx),
        ]
    }
}

impl Focusable for FormStory {
    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.name_input.focus_handle(cx)
    }
}

impl Render for FormStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_multi_column = self.column > 1;
        let is_horizontal = self.layout.is_horizontal();

        v_flex()
            .id("form-story")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .flex_wrap()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_x_3()
                            .child(
                                Switch::new("layout")
                                    .checked(self.layout.is_horizontal())
                                    .label("Horizontal")
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        if *checked {
                                            this.layout = Axis::Horizontal;
                                        } else {
                                            this.layout = Axis::Vertical;
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("column")
                                    .checked(self.column > 1)
                                    .label("Multi Column")
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        if *checked {
                                            this.column = 2;
                                        } else {
                                            this.column = 1;
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        ButtonGroup::new("size")
                            .outline()
                            .small()
                            .child(
                                Button::new("large")
                                    .selected(self.size == Size::Large)
                                    .child("Large"),
                            )
                            .child(
                                Button::new("medium")
                                    .child("Medium")
                                    .selected(self.size == Size::Medium),
                            )
                            .child(
                                Button::new("small")
                                    .child("Small")
                                    .selected(self.size == Size::Small),
                            )
                            .on_click(cx.listener(|this, selecteds: &Vec<usize>, _, cx| {
                                if selecteds.contains(&0) {
                                    this.size = Size::Large;
                                } else if selecteds.contains(&1) {
                                    this.size = Size::Medium;
                                } else if selecteds.contains(&2) {
                                    this.size = Size::Small;
                                }
                                cx.notify();
                            })),
                    ),
            )
            .child(Divider::horizontal())
            .child(
                v_form()
                    .layout(self.layout)
                    .with_size(self.size)
                    .column(self.column)
                    .label_width(px(if is_multi_column { 100. } else { 140. }))
                    .child(
                        form_field().label_fn(|_, _| "Name").child(
                            h_flex()
                                .gap_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded(cx.theme().radius)
                                .child(
                                    div().w(px(90.)).child(
                                        Dropdown::new(&self.name_prefix_state)
                                            .pr_0()
                                            .appearance(false),
                                    ),
                                )
                                .child(div().flex_1().child(
                                    TextInput::new(&self.name_input).pl_0().appearance(false),
                                )),
                        ),
                    )
                    .child(
                        form_field()
                            .label("Email")
                            .child(TextInput::new(&self.email_input))
                            .required(true),
                    )
                    .child(
                        form_field()
                            .label("Bio")
                            .when(self.layout.is_vertical(), |this| this.items_start())
                            .child(TextInput::new(&self.bio_input))
                            .description_fn(|_, _| {
                                div().child("Use at most 100 words to describe yourself.")
                            }),
                    )
                    .child(
                        form_field()
                            .no_label_indent()
                            .when(is_multi_column, |this| this.col_span(2))
                            .child("This is a full width form field."),
                    )
                    .child(
                        form_field()
                            .label("Please select your birthday")
                            .description("Select your birthday, we will send you a gift.")
                            .child(DatePicker::new(&self.date)),
                    )
                    .child(
                        form_field()
                            .when(is_horizontal && is_multi_column, |this| {
                                this.no_label_indent()
                            })
                            .when(is_multi_column, |this| this.col_start(1))
                            .child(
                                Switch::new("subscribe-newsletter")
                                    .label("Subscribe our newsletter")
                                    .checked(self.subscribe_email)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.subscribe_email = *checked;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        form_field()
                            .when(is_horizontal && is_multi_column, |this| {
                                this.no_label_indent()
                            })
                            .child(
                                ColorPicker::new(&self.color_state)
                                    .small()
                                    .label("Theme color"),
                            ),
                    )
                    .child(
                        form_field()
                            .when(is_horizontal && is_multi_column, |this| {
                                this.no_label_indent()
                            })
                            .child(
                                Checkbox::new("use-vertical-layout")
                                    .label("Vertical layout")
                                    .checked(self.layout.is_vertical())
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.layout = if *checked {
                                            Axis::Vertical
                                        } else {
                                            Axis::Horizontal
                                        };
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
    }
}
