//! CLI-specific error types

use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific errors
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Invalid command arguments: {0}")]
    InvalidArguments(String),
    
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

impl From<CliError> for crate::WorkflowError {
    fn from(err: CliError) -> Self {
        match err {
            CliError::WorkflowNotFound(id) => crate::WorkflowError::NotFound {
                resource: format!("workflow '{}'", id),
            },
            CliError::ToolNotFound(name) => crate::WorkflowError::NotFound {
                resource: format!("tool '{}'", name),
            },
            CliError::ExecutionFailed(msg) => crate::WorkflowError::workflow_execution(&msg),
            CliError::ConfigError(msg) => crate::WorkflowError::workflow_execution(&msg),
            _ => crate::WorkflowError::workflow_execution(&err.to_string()),
        }
    }
}