use crate::ast::*;
use crate::component::*;
use crate::error::*;
use crate::hot_reload::*;
use crate::parser::GPMLParser;
use crate::renderer::GPMLRenderer;
use gpui::*;
use gpui_component::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Main GPML canvas component that loads and renders GPML files dynamically
pub struct GPMLCanvas {
    /// Path to the main GPML file
    root_path: PathBuf,
    /// Current parsed document
    current_document: Option<GPMLNode>,
    /// Component resolution context
    context: Option<GPMLContext>,
    /// Component resolver for handling imports
    resolver: ComponentResolver,
    /// Hot reload manager
    hot_reload_manager: HotReloadManager,
    /// Error state
    error: Option<String>,
    /// Loading state
    is_loading: bool,
    /// Runtime variables that can be injected
    runtime_vars: HashMap<String, AttributeValue>,
}

impl GPMLCanvas {
    /// Create a new GPML canvas with the given root file
    pub fn new(root_path: impl AsRef<Path>) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        
        Self {
            root_path,
            current_document: None,
            context: None,
            resolver: ComponentResolver::new(),
            hot_reload_manager: HotReloadManager::new(),
            error: None,
            is_loading: false,
            runtime_vars: HashMap::new(),
        }
    }

    /// Create a new GPML canvas with runtime variables
    pub fn with_variables(mut self, vars: HashMap<String, AttributeValue>) -> Self {
        self.runtime_vars = vars;
        self
    }

    /// Add a runtime variable
    pub fn add_variable(&mut self, name: String, value: AttributeValue) {
        self.runtime_vars.insert(name, value);
    }

    /// Load the GPML file and all its dependencies
    pub fn load(&mut self) -> GPMLResult<()> {
        self.is_loading = true;
        self.error = None;

        match self.load_internal() {
            Ok(()) => {
                self.is_loading = false;
                Ok(())
            }
            Err(e) => {
                self.error = Some(format!("{}", e));
                self.is_loading = false;
                Err(e)
            }
        }
    }

    fn load_internal(&mut self) -> GPMLResult<()> {
        // Load the context with all components and imports
        let mut context = self.resolver.load_file(&self.root_path)?;
        
        // Add runtime variables to context
        for (name, value) in &self.runtime_vars {
            context.variables.insert(name.clone(), value.clone());
        }
        
        self.context = Some(context);

        // Parse the main document
        let content = std::fs::read_to_string(&self.root_path)
            .map_err(|_| GPMLError::FileNotFound {
                path: self.root_path.display().to_string(),
            })?;

        let document = GPMLParser::parse_file(&content)
            .map_err(|e| GPMLError::ParseError { message: e })?;
        self.current_document = Some(document);

        Ok(())
    }

    /// Start hot reloading for this canvas
    pub fn start_hot_reload(&mut self) -> GPMLResult<()> {
        self.hot_reload_manager.start_watching(&self.root_path)?;
        
        // Also watch the directory for new files
        if let Some(parent) = self.root_path.parent() {
            self.hot_reload_manager.add_watched_file(parent);
        }
        
        Ok(())
    }

    /// Check for changes and reload if necessary
    pub fn check_and_reload(&mut self) -> GPMLResult<bool> {
        let changes = self.hot_reload_manager.check_for_changes();
        
        if !changes.is_empty() {
            tracing::debug!("GPML files changed: {:?}", changes);
            
            // Clear resolver cache for changed files
            for changed_path in &changes {
                self.resolver.remove_from_cache(changed_path);
            }
            
            // Reload everything
            self.load()?;
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Force reload the canvas
    pub fn reload(&mut self) -> GPMLResult<()> {
        self.resolver.clear_cache();
        self.load()
    }

    /// Get the current error if any
    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }

    /// Check if the canvas is currently loading
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Check if the canvas has loaded content
    pub fn is_loaded(&self) -> bool {
        self.current_document.is_some() && self.context.is_some()
    }

    /// Get the root element from the document
    pub fn get_root_element(&self) -> Option<&GPMLElement> {
        if let Some(GPMLNode::Document { root: Some(root), .. }) = &self.current_document {
            Some(root)
        } else {
            None
        }
    }

    /// Load GPML from a string instead of a file
    pub fn load_from_string(&mut self, content: &str, base_path: Option<&Path>) -> GPMLResult<()> {
        self.is_loading = true;
        self.error = None;

        let base_path = base_path.unwrap_or_else(|| Path::new("."));
        let mut context = GPMLContext::new(base_path);
        
        // Add runtime variables
        for (name, value) in &self.runtime_vars {
            context.variables.insert(name.clone(), value.clone());
        }

        let mut parser = GPMLParser::new();
        let document = parser.parse(content)?;

        // Process imports and components from the document
        self.resolver.clear_cache();
        
        if let GPMLNode::Document { imports, components, .. } = &document {
            for component in components {
                context.add_component(component.clone());
            }
            
            // Note: imports won't work with string content unless base_path is set properly
            if !imports.is_empty() && base_path == Path::new(".") {
                tracing::warn!("GPML imports found but no base path set - imports will not resolve");
            }
        }

        self.current_document = Some(document);
        self.context = Some(context);
        self.is_loading = false;

        Ok(())
    }

    /// Update a runtime variable and trigger re-render if canvas is loaded
    pub fn update_variable(&mut self, name: String, value: AttributeValue) -> bool {
        self.runtime_vars.insert(name.clone(), value.clone());
        
        if let Some(ref mut context) = self.context {
            context.variables.insert(name, value);
            true
        } else {
            false
        }
    }

    /// Get current runtime variables
    pub fn get_variables(&self) -> &HashMap<String, AttributeValue> {
        &self.runtime_vars
    }

    /// Clear all runtime variables
    pub fn clear_variables(&mut self) {
        self.runtime_vars.clear();
        if let Some(ref mut context) = self.context {
            // Keep only the original variables from the document
            context.variables.retain(|k, _| !self.runtime_vars.contains_key(k));
        }
    }
}

impl Render for GPMLCanvas {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check for hot reload changes
        if let Ok(reloaded) = self.check_and_reload() {
            if reloaded {
                cx.notify();
            }
        }

        // Handle different states
        if self.is_loading {
            return self.render_loading_state(window, cx);
        }

        if let Some(error) = &self.error {
            return self.render_error_state(error, window, cx);
        }

        if let (Some(root_element), Some(context)) = (self.get_root_element(), &self.context) {
            match GPMLRenderer::render_element(root_element, context, &self.resolver, cx) {
                Ok(element) => element,
                Err(e) => {
                    tracing::error!("GPML render error: {}", e);
                    self.render_error_state(&format!("{}", e), window, cx)
                }
            }
        } else {
            self.render_empty_state(window, cx)
        }
    }
}

impl GPMLCanvas {
    fn render_loading_state(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .child(
                Icon::new(IconName::Loader)
                    .size(px(24.0))
                    .text_color(cx.theme().muted_foreground)
            )
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(cx.theme().muted_foreground)
                    .child("Loading GPML...")
            )
            .into_any_element()
    }

    fn render_error_state(&self, error: &String, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .p_4()
            .child(
                Icon::new(IconName::TriangleAlert)
                    .size(px(24.0))
                    .text_color(gpui::red())
            )
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(gpui::red())
                    .child("GPML Error")
            )
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(cx.theme().muted_foreground)
                    //TODO:.text_wrap()
                    .max_w(px(600.0))
                    .child(error.clone())
            )
            .child(
                button::Button::new("reload-button")
                    .child("Reload")
                    .on_click(cx.listener(|canvas, _event, _cx| {
                        if let Err(e) = canvas.reload() {
                            tracing::error!("Failed to reload GPML: {}", e);
                        }
                    }))
            )
            .into_any_element()
    }

    fn render_empty_state(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .items_center()
            .justify_center()
            .size_full()
            .gap_4()
            .child(
                Icon::new(IconName::Folder)
                    .size(px(24.0))
                    .text_color(cx.theme().muted_foreground)
            )
            .child(
                div()
                    .text_size(px(14.0))
                    .text_color(cx.theme().muted_foreground)
                    .child("No GPML content loaded")
            )
            .into_any_element()
    }
}

/// Create a GPML canvas view entity
pub fn create_gpml_canvas<V>(
    root_path: impl AsRef<Path>,
    cx: &mut Context<V>,
) -> Entity<GPMLCanvas>
where
    V: Render + 'static,
{
    cx.new_entity(|_cx| {
        let mut canvas = GPMLCanvas::new(root_path);
        
        // Try to load the file
        if let Err(e) = canvas.load() {
            tracing::error!("Failed to load GPML file: {}", e);
        }
        
        // Start hot reload
        if let Err(e) = canvas.start_hot_reload() {
            tracing::error!("Failed to start hot reload: {}", e);
        }
        
        canvas
    })
}

/// Create a GPML canvas view with runtime variables
pub fn create_gpml_canvas_with_vars<V>(
    root_path: impl AsRef<Path>,
    variables: HashMap<String, AttributeValue>,
    cx: &mut Context<V>,
) -> Entity<GPMLCanvas>
where
    V: Render + 'static,
{
    cx.new_entity(|_cx| {
        let mut canvas = GPMLCanvas::new(root_path).with_variables(variables);
        
        if let Err(e) = canvas.load() {
            tracing::error!("Failed to load GPML file: {}", e);
        }
        
        if let Err(e) = canvas.start_hot_reload() {
            tracing::error!("Failed to start hot reload: {}", e);
        }
        
        canvas
    })
}
