use pulsar_engine::*;
use gpui::*;
use gpui_component::*;
use std::collections::HashMap;

/// Example showing how to use the GPML Canvas component
fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                bounds: WindowBounds::Windowed(Bounds {
                    origin: point(px(100.0), px(100.0)),
                    size: size(px(800.0), px(600.0)),
                }),
                ..Default::default()
            },
            |cx| {
                cx.new(|cx| GPMLExample::new(cx))
            },
        )
        .unwrap();
    });
}

struct GPMLExample {
    canvas: Entity<GPMLCanvas>,
}

impl GPMLExample {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create runtime variables
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), AttributeValue::Literal("My App".to_string()));
        variables.insert("subtitle".to_string(), AttributeValue::Literal("Built with GPML".to_string()));
        variables.insert("button_count".to_string(), AttributeValue::Number(3.0));

        // Load the card component example
        let canvas = create_gpml_canvas_with_vars(
            "examples/card-component/App.gpml",
            variables,
            cx,
        );

        Self { canvas }
    }
}

impl Render for GPMLExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(
                // Title bar
                div()
                    .h(px(60.0))
                    .w_full()
                    .bg(cx.theme().surface)
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child("GPML Canvas Example")
                    )
            )
            .child(
                // Main content area
                div()
                    .flex_1()
                    .p_4()
                    .child(self.canvas.clone())
            )
    }
}

impl Focusable for GPMLExample {
    fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
        self.canvas.read(cx).focus_handle(cx)
    }
}
