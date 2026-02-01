//! Plugin system errors
//!
//! This module provides unified error types for the plugin system.
//! These errors replace the generic WorkflowError for plugin-specific operations.

use thiserror::Error;

/// Result type alias for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;

/// Errors that can occur in the plugin system
#[derive(Error, Debug, Clone)]
pub enum PluginError {
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {message}")]
    InitializationError {
        /// Error message
        message: String,
        /// Plugin name
        plugin_name: String,
    },

    /// Plugin runtime error
    #[error("Plugin runtime error in '{plugin_name}': {message}")]
    RuntimeError {
        /// Error message
        message: String,
        /// Plugin name
        plugin_name: String,
    },

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin already loaded
    #[error("Plugin '{0}' is already loaded")]
    AlreadyLoaded(String),

    /// Invalid plugin configuration
    #[error("Invalid plugin configuration for '{plugin_name}': {message}")]
    InvalidConfiguration {
        /// Plugin name
        plugin_name: String,
        /// Error message
        message: String,
    },

    /// Security policy violation
    #[error("Security policy violation in '{plugin_name}': {violation}")]
    SecurityViolation {
        /// Plugin name
        plugin_name: String,
        /// Violation details
        violation: String,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded for '{plugin_name}': {resource} (limit: {limit})")]
    ResourceLimitExceeded {
        /// Plugin name
        plugin_name: String,
        /// Resource type (memory, cpu, etc.)
        resource: String,
        /// Limit that was exceeded
        limit: u64,
    },

    /// Dependency resolution failed
    #[error("Dependency resolution failed for '{plugin_name}': {message}")]
    DependencyError {
        /// Plugin name
        plugin_name: String,
        /// Error message
        message: String,
    },

    /// Runtime creation failed
    #[error("Failed to create runtime for '{plugin_name}' ({plugin_type:?}): {message}")]
    RuntimeCreationError {
        /// Plugin name
        plugin_name: String,
        /// Plugin type
        plugin_type: crate::core::PluginType,
        /// Error message
        message: String,
    },

    /// Runtime pool exhausted
    #[error("Runtime pool exhausted for '{0}' - all {1} runtimes are in use")]
    PoolExhausted(String, usize),

    /// Tool registration failed
    #[error("Failed to register tool '{tool_name}' from plugin '{plugin_name}': {message}")]
    ToolRegistrationError {
        /// Plugin name
        plugin_name: String,
        /// Tool name
        tool_name: String,
        /// Error message
        message: String,
    },

    /// Tool execution failed
    #[error("Tool execution failed: {message}")]
    ToolExecutionError {
        /// Error message
        message: String,
        /// Tool name
        tool_name: Option<String>,
    },

    /// I/O error during plugin operation
    #[error("I/O error: {0}")]
    IoError(String),

    /// Concurrent access error (e.g., mutex poisoned)
    #[error("Concurrent access error: {0}")]
    ConcurrentAccess(String),

    /// Unknown error
    #[error("Unknown plugin error: {0}")]
    Unknown(String),
}

impl PluginError {
    /// Create an initialization error
    pub fn initialization(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InitializationError {
            plugin_name: plugin_name.into(),
            message: message.into(),
        }
    }

    /// Create a runtime error
    pub fn runtime(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::RuntimeError {
            plugin_name: plugin_name.into(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }

    /// Create a configuration error
    pub fn configuration(plugin_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidConfiguration {
            plugin_name: plugin_name.into(),
            message: message.into(),
        }
    }

    /// Create a security violation error
    pub fn security(plugin_name: impl Into<String>, violation: impl Into<String>) -> Self {
        Self::SecurityViolation {
            plugin_name: plugin_name.into(),
            violation: violation.into(),
        }
    }

    /// Create a resource limit error
    pub fn resource_limit(
        plugin_name: impl Into<String>,
        resource: impl Into<String>,
        limit: u64,
    ) -> Self {
        Self::ResourceLimitExceeded {
            plugin_name: plugin_name.into(),
            resource: resource.into(),
            limit,
        }
    }

    /// Get the plugin name if available
    pub fn plugin_name(&self) -> Option<&str> {
        match self {
            Self::InitializationError { plugin_name, .. } => Some(plugin_name),
            Self::RuntimeError { plugin_name, .. } => Some(plugin_name),
            Self::NotFound(name) => Some(name),
            Self::AlreadyLoaded(name) => Some(name),
            Self::InvalidConfiguration { plugin_name, .. } => Some(plugin_name),
            Self::SecurityViolation { plugin_name, .. } => Some(plugin_name),
            Self::ResourceLimitExceeded { plugin_name, .. } => Some(plugin_name),
            Self::DependencyError { plugin_name, .. } => Some(plugin_name),
            Self::RuntimeCreationError { plugin_name, .. } => Some(plugin_name),
            Self::PoolExhausted(name, _) => Some(name),
            Self::ToolRegistrationError { plugin_name, .. } => Some(plugin_name),
            _ => None,
        }
    }
}

/// Extension trait for converting other errors to PluginError
pub trait IntoPluginError<T> {
    /// Convert to PluginError with context
    fn into_plugin_error(self, plugin_name: &str) -> Result<T>;
}

impl<T, E: std::fmt::Display> IntoPluginError<T> for std::result::Result<T, E> {
    fn into_plugin_error(self, plugin_name: &str) -> Result<T> {
        self.map_err(|e| PluginError::runtime(plugin_name, e.to_string()))
    }
}
