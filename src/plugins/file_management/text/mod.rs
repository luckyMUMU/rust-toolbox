//! Text processing module for file management plugin
//!
//! Contains tools for text analysis and transformation.

pub mod text_processor_tool;

// Re-export text components
pub use text_processor_tool::{
    ChineseProcessingConfig, TextOperation, TextOutputFormat, TextProcessorParams,
    TextProcessorResult, TextProcessorTool,
};
