use pulsar_engine::*;
use gpui::*;
use gpui_component::*;
use std::collections::HashMap;

/// Example showing how to use the GPML Canvas component
fn main() {
    App::new().run(move |cx: &mut AppContext| {
        let options = WindowOptions::default();
        cx.open_window(options, |cx| cx.new_view(|cx| GPMLExample::new(cx)))
            .unwrap();
    });
}

struct GPMLExample {
    canvas: Entity<GPMLCanvas>,
    focus_handle: FocusHandle,
}

impl GPMLExample {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create runtime variables
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), AttributeValue::Literal("My App".to_string()));
        variables.insert("subtitle".to_string(), AttributeValue::Literal("Built with GPML".to_string()));
        variables.insert("button_count".to_string(), AttributeValue::Number(3.0));

        // Load the card component example
        let canvas = cx.new(|cx| {
            let mut canvas = GPMLCanvas::new("examples/card-component/App.gpml").with_variables(variables);
            
            // Try to load the file
            if let Err(e) = canvas.load() {
                tracing::error!("Failed to load GPML file: {}", e);
            }
            
            canvas
        });

        let focus_handle = cx.focus_handle();

        Self { canvas, focus_handle }
    }
}

impl Render for GPMLExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().colors.background)
            .child(
                // Title bar
                div()
                    .h(px(60.0))
                    .w_full()
                    .bg(cx.theme().colors.elevated_surface_background)
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(cx.theme().colors.border)
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().colors.text)
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
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
