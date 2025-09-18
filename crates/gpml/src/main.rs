use gpml::*;
use gpui::*;
use gpui_component::*;
use std::collections::HashMap;

/// Example showing how to use the GPML Canvas component
fn main() {
    // Initialize tracing for debugging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting GPML application");
    
    let app = Application::new();
    app.run(|cx| {
        tracing::info!("Initializing GPUI application context");
        
        // Initialize GPUI component system and themes
        gpui_component::init(cx);
        tracing::info!("GPUI component system initialized");
        
        let options = WindowOptions::default();
        tracing::info!("Opening main window");
        
        let window = cx.open_window(options, |window, cx| {
            tracing::info!("Creating GPMLExample entity");
            let entity = cx.new(|cx| GPMLExample::new(cx));
            entity.into()
        });
        
        tracing::info!("Application setup complete");
    });
}

struct GPMLExample {
    canvas: Entity<GPMLCanvas>,
    focus_handle: FocusHandle,
}

impl GPMLExample {
    fn new(cx: &mut Context<Self>) -> Self {
        tracing::info!("GPMLExample::new called");
        
        // Create runtime variables
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), AttributeValue::Literal("My App".to_string()));
        variables.insert("subtitle".to_string(), AttributeValue::Literal("Built with GPML".to_string()));
        variables.insert("button_count".to_string(), AttributeValue::Number(3.0));

        tracing::info!("Runtime variables created: {:?}", variables);

        // Load the card component example
        let canvas_path = "examples/card-component/App.gpml";
        tracing::info!("Creating GPML canvas with path: {}", canvas_path);
        
        let canvas = cx.new(|_cx| {
            let mut canvas = GPMLCanvas::new(canvas_path).with_variables(variables);
            
            // Try to load the file
            tracing::info!("Attempting to load GPML file");
            if let Err(e) = canvas.load() {
                tracing::error!("Failed to load GPML file: {}", e);
            } else {
                tracing::info!("GPML file loaded successfully in GPMLExample::new");
            }
            
            canvas
        });

        let focus_handle = cx.focus_handle();
        tracing::info!("GPMLExample created successfully");

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
                    .bg(cx.theme().background)
                    .flex()
                    .items_center()
                    .px_4()
                    .border_b_1()
                    .border_color(cx.theme().colors.border)
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
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
