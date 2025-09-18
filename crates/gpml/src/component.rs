use crate::ast::*;
use crate::error::*;
use crate::parser::GPMLParser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Runtime context for GPML component evaluation
#[derive(Debug, Clone)]
pub struct GPMLContext {
    /// Component definitions available in this context
    pub components: HashMap<String, ComponentDef>,
    /// Variable bindings for interpolation
    pub variables: HashMap<String, AttributeValue>,
    /// Base path for resolving imports
    pub base_path: PathBuf,
}

impl GPMLContext {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            components: HashMap::new(),
            variables: HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn with_variable(mut self, name: String, value: AttributeValue) -> Self {
        self.variables.insert(name, value);
        self
    }

    pub fn add_component(&mut self, component: ComponentDef) {
        self.components.insert(component.name.clone(), component);
    }

    pub fn get_component(&self, name: &str) -> Option<&ComponentDef> {
        self.components.get(name)
    }

    pub fn get_variable(&self, name: &str) -> Option<&AttributeValue> {
        self.variables.get(name)
    }

    pub fn interpolate_string(&self, value: &str) -> String {
        if value.starts_with("${") && value.ends_with("}") {
            let var_name = &value[2..value.len()-1];
            if let Some(var_value) = self.get_variable(var_name) {
                var_value.as_string()
            } else {
                value.to_string()
            }
        } else {
            value.to_string()
        }
    }

    pub fn interpolate_attribute(&self, value: &AttributeValue) -> AttributeValue {
        match value {
            AttributeValue::Expression(expr) => {
                if let Some(var_value) = self.get_variable(expr) {
                    var_value.clone()
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }
}

/// Component resolver handles imports and component instantiation
pub struct ComponentResolver {
    cache: HashMap<PathBuf, GPMLNode>,
    loading: Vec<PathBuf>, // Track files currently being loaded to detect circular deps
}

impl ComponentResolver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            loading: Vec::new(),
        }
    }

    /// Load and parse a GPML file with all its dependencies
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> GPMLResult<GPMLContext> {
        let path = path.as_ref();
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        self.loading.clear();
        let document = self.load_document(&absolute_path)?;
        
        let mut context = GPMLContext::new(absolute_path.parent().unwrap_or(Path::new(".")));
        self.process_document(&document, &mut context)?;
        
        Ok(context)
    }

    fn load_document(&mut self, path: &Path) -> GPMLResult<GPMLNode> {
        // Check for circular dependencies
        if self.loading.contains(&path.to_path_buf()) {
            return Err(GPMLError::CircularDependency {
                path: path.display().to_string(),
            });
        }

        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        // Mark as loading
        self.loading.push(path.to_path_buf());

        // Read and parse the file
        let content = fs::read_to_string(path).map_err(|_| GPMLError::FileNotFound {
            path: path.display().to_string(),
        })?;

        let document = GPMLParser::parse_file(&content)
            .map_err(|e| GPMLError::ParseError { 
                message: e, 
                line: 0, 
                column: 0 
            })?;

        // Cache the result
        self.cache.insert(path.to_path_buf(), document.clone());
        
        // Remove from loading list
        self.loading.retain(|p| p != path);

        Ok(document)
    }

    fn process_document(&mut self, document: &GPMLNode, context: &mut GPMLContext) -> GPMLResult<()> {
        if let GPMLNode::Document { imports, components, .. } = document {
            // Process imports first
            for import in imports {
                self.process_import(import, context)?;
            }

            // Then add local component definitions
            for component in components {
                context.add_component(component.clone());
            }
        }

        Ok(())
    }

    fn process_import(&mut self, import: &Import, context: &mut GPMLContext) -> GPMLResult<()> {
        let import_path = context.base_path.join(&import.path);
        let imported_doc = self.load_document(&import_path)?;

        if let GPMLNode::Document { components, .. } = imported_doc {
            for component in components {
                // Add imported component with alias prefix if needed
                let component_name = if import.alias.is_empty() {
                    component.name.clone()
                } else {
                    format!("{}.{}", import.alias, component.name)
                };

                let mut aliased_component = component.clone();
                aliased_component.name = component_name;
                context.add_component(aliased_component);
            }
        }

        Ok(())
    }

    /// Instantiate a component with given parameters
    pub fn instantiate_component(
        &self,
        component_def: &ComponentDef,
        args: &HashMap<String, AttributeValue>,
        context: &GPMLContext,
    ) -> GPMLResult<Element> {
        // Validate parameter count
        if args.len() != component_def.parameters.len() {
            return Err(GPMLError::ParameterMismatch {
                expected: component_def.parameters.len(),
                actual: args.len(),
            });
        }

        // Create new context with parameter bindings
        let mut instance_context = context.clone();
        for (param, arg_name) in component_def.parameters.iter().enumerate() {
            if let Some(value) = args.get(arg_name) {
                instance_context.variables.insert(arg_name.clone(), value.clone());
            }
        }

        // Clone and interpolate the component body
        let mut instance_body = component_def.body.clone();
        self.interpolate_element(&mut instance_body, &instance_context)?;

        Ok(instance_body)
    }

    fn interpolate_element(&self, element: &mut Element, context: &GPMLContext) -> GPMLResult<()> {
        // Interpolate attributes
        for (_, value) in element.attributes.iter_mut() {
            *value = context.interpolate_attribute(value);
        }

        // Interpolate children
        for child in element.children.iter_mut() {
            match child {
                GPMLNode::Element(child_element) => {
                    self.interpolate_element(child_element, context)?;
                }
                GPMLNode::Text(text) => {
                    *text = context.interpolate_string(text);
                }
                GPMLNode::Expression(expr) => {
                    if let Some(value) = context.get_variable(expr) {
                        *child = GPMLNode::Text(value.as_string());
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_from_cache(&mut self, path: &Path) {
        self.cache.remove(path);
    }
}

impl Default for ComponentResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to resolve a complete GPML element tree with all components instantiated
pub fn resolve_element(
    element: &Element,
    context: &GPMLContext,
    resolver: &ComponentResolver,
) -> GPMLResult<Element> {
    // Check if this element refers to a custom component
    if let Some(component_def) = context.get_component(&element.tag) {
        // Convert attributes to argument map
        let mut args = HashMap::new();
        for (key, value) in &element.attributes {
            args.insert(key.clone(), context.interpolate_attribute(value));
        }

        // Instantiate the component
        resolver.instantiate_component(component_def, &args, context)
    } else {
        // This is a regular element, just resolve children
        let mut resolved = element.clone();
        let mut resolved_children = Vec::new();

        for child in &element.children {
            match child {
                GPMLNode::Element(child_element) => {
                    let resolved_child = resolve_element(child_element, context, resolver)?;
                    resolved_children.push(GPMLNode::Element(resolved_child));
                }
                GPMLNode::Text(text) => {
                    let interpolated_text = context.interpolate_string(text);
                    resolved_children.push(GPMLNode::Text(interpolated_text));
                }
                GPMLNode::Expression(expr) => {
                    if let Some(value) = context.get_variable(expr) {
                        resolved_children.push(GPMLNode::Text(value.as_string()));
                    } else {
                        resolved_children.push(child.clone());
                    }
                }
                _ => resolved_children.push(child.clone()),
            }
        }

        resolved.children = resolved_children;
        
        // Interpolate attributes
        for (_, value) in resolved.attributes.iter_mut() {
            *value = context.interpolate_attribute(value);
        }

        Ok(resolved)
    }
}
