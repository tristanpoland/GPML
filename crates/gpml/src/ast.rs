use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// GPML Abstract Syntax Tree node types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GPMLNode {
    /// Root node containing all top-level declarations
    Document {
        imports: Vec<Import>,
        components: Vec<ComponentDef>,
        root: Option<Element>,
    },
    /// Import statement
    Import(Import),
    /// Component definition
    ComponentDef(ComponentDef),
    /// XML-like element
    Element(Element),
    /// Text content
    Text(String),
    /// Interpolated expression like ${variable}
    Expression(String),
}

/// Import statement: import ./path.gpml as Name
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub alias: String,
}

/// Component definition: def ComponentName(param1, param2) { ... }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Element,
}

/// XML-like element: <tag attr="value">children</tag>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub tag: String,
    pub attributes: HashMap<String, AttributeValue>,
    pub children: Vec<GPMLNode>,
    pub self_closing: bool,
}

/// Attribute value which can be a literal or expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    /// String literal: "value"
    Literal(String),
    /// Expression: ${expression}
    Expression(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
}

impl AttributeValue {
    pub fn as_string(&self) -> String {
        match self {
            AttributeValue::Literal(s) => s.clone(),
            AttributeValue::Expression(expr) => format!("${{{}}}", expr),
            AttributeValue::Number(n) => n.to_string(),
            AttributeValue::Boolean(b) => b.to_string(),
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            AttributeValue::Literal(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Boolean(b) => Some(*b),
            AttributeValue::Literal(s) => s.parse().ok(),
            _ => None,
        }
    }
}

impl Element {
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
            self_closing: false,
        }
    }

    pub fn with_attribute(mut self, name: String, value: AttributeValue) -> Self {
        self.attributes.insert(name, value);
        self
    }

    pub fn with_child(mut self, child: GPMLNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn get_attribute(&self, name: &str) -> Option<&AttributeValue> {
        self.attributes.get(name)
    }

    pub fn get_text_content(&self) -> String {
        let mut content = String::new();
        for child in &self.children {
            match child {
                GPMLNode::Text(text) => content.push_str(text),
                GPMLNode::Element(element) => content.push_str(&element.get_text_content()),
                _ => {}
            }
        }
        content
    }
}

impl GPMLNode {
    pub fn is_element(&self) -> bool {
        matches!(self, GPMLNode::Element(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, GPMLNode::Text(_))
    }

    pub fn as_element(&self) -> Option<&Element> {
        match self {
            GPMLNode::Element(element) => Some(element),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            GPMLNode::Text(text) => Some(text),
            _ => None,
        }
    }
}

// Type alias for backward compatibility 
pub type GPMLElement = Element;
