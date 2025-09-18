use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum GPMLError {
    #[error("Parse error: {message} at line {line}, column {column}")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Component '{name}' not found")]
    ComponentNotFound { name: String },

    #[error("Import error: {message}")]
    ImportError { message: String },

    #[error("Render error: {message}")]
    RenderError { message: String },

    #[error("Invalid attribute value: {message}")]
    InvalidAttributeValue { message: String },

    #[error("Parameter mismatch: expected {expected}, got {actual}")]
    ParameterMismatch { expected: usize, actual: usize },

    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },

    #[error("Syntax error: {message}")]
    SyntaxError { message: String },

    #[error("Type error: {message}")]
    TypeError { message: String },
}

pub type GPMLResult<T> = Result<T, GPMLError>;
