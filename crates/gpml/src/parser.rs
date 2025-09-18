use crate::ast::*;
use crate::error::*;

pub struct GPMLParser {
    line: usize,
    column: usize,
}

impl GPMLParser {
    pub fn new() -> Self {
        Self { line: 1, column: 1 }
    }

    pub fn parse(&mut self, input: &str) -> GPMLResult<GPMLNode> {
        // For now, return a simple document structure to get compilation working
        Ok(GPMLNode::Document {
            imports: vec![],
            components: vec![],
            root: Some(Element::new("div".to_string())),
        })
    }
}

impl Default for GPMLParser {
    fn default() -> Self {
        Self::new()
    }
}
