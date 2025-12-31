//! File Management Plugin Module
//! 
//! This module contains all components for the file management plugin including
//! tools, utilities, and error handling.

pub mod error;
pub mod plugin;
pub mod registry;
pub mod utils;

// Re-export main plugin components
pub use plugin::{
    FileManagementPlugin, FileManagementPluginBuilder, FileManagementConfig,
};
pub use error::{FileManagementError, FileManagementResult};
pub use registry::FileManagementToolRegistry;
pub use utils::{
    FileOperationManager, TextProcessor, PathUtils, ValidationUtils,
    ExperimentalMode, HumanDecisionContext,
};