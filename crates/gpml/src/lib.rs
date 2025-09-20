pub mod ast;
pub mod component;
pub mod elements;
pub mod error;
pub mod parser;
pub mod renderer;
pub mod style;
pub mod hot_reload;
pub mod canvas;

// Re-export main types for convenience
pub use ast::*;
pub use component::*;
pub use error::*;
pub use parser::*;
pub use renderer::*;
pub use style::*;
pub use hot_reload::*;
pub use canvas::*;

// Re-export for backward compatibility
use gpui::*;
use gpui::{
    div, AnyElement, ClickEvent, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
};
use gpui_component::{ActiveTheme as _, StyledExt};