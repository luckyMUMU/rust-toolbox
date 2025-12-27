//! Error handling for the workflow toolkit

use thiserror::Error;

/// Result type alias for the workflow toolkit
pub type Result<T> = std::result::Result<T, WorkflowError>;

/// Main error type for the workflow toolkit
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Workflow validation error: {message}")]
    WorkflowValidation { message: String },
    
    #[error("Workflow execution error: {message}")]
    WorkflowExecution { message: String },
    
    #[error("Duplicate node ID: {0}")]
    DuplicateNodeId(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Invalid workflow name: {0}")]
    InvalidWorkflowName(String),
    
    #[error("Invalid workflow version: {0}")]
    InvalidWorkflowVersion(String),
    
    #[error("Empty workflow: workflow must contain at least one node")]
    EmptyWorkflow,
    
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    
    #[error("Missing tool name for node: {0}")]
    MissingToolName(String),
    
    #[error("Circular dependency detected in workflow")]
    CircularDependency,
    
    #[error("Invalid edge: {0}")]
    InvalidEdge(String),
    
    #[error("Tool error: {message}")]
    Tool { message: String },
    
    #[error("Plugin error: {message}")]
    Plugin { message: String },
    
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Timeout error: operation timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },
    
    #[error("Concurrent access error: {message}")]
    ConcurrentAccess { message: String },
    
    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

impl WorkflowError {
    /// Create a new workflow validation error
    pub fn workflow_validation<S: Into<String>>(message: S) -> Self {
        Self::WorkflowValidation {
            message: message.into(),
        }
    }
    
    /// Create a new workflow execution error
    pub fn workflow_execution<S: Into<String>>(message: S) -> Self {
        Self::WorkflowExecution {
            message: message.into(),
        }
    }
    
    /// Create a new tool error
    pub fn tool<S: Into<String>>(message: S) -> Self {
        Self::Tool {
            message: message.into(),
        }
    }
    
    /// Create a new plugin error
    pub fn plugin<S: Into<String>>(message: S) -> Self {
        Self::Plugin {
            message: message.into(),
        }
    }
    
    /// Create a new storage error
    pub fn storage<S: Into<String>>(message: S) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }
}