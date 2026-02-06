//! # Workflow Toolkit
//!
//! # Architecture Overview
//!
//! ## Core Components
//!
//! ### Workflow Engine
//! - **DefaultWorkflowEngine**: Main execution engine that orchestrates workflow runs
//! - **DagScheduler**: Handles DAG-based node scheduling and dependency resolution
//! - **ExecutionManager**: High-level interface for workflow lifecycle management
//!
//! ### Tool System
//! - **ToolRegistry**: Registry for all available tools and their implementations
//! - **ToolNode**: Individual tool execution units with retry and error handling
//! - **Template System**: Reusable tool configurations
//!
//! ### Plugin System
//! - **PluginManager**: Manages external plugins (Docker, Python, Node.js, WASM)
//! - **Native Plugins**: Rust-based plugins compiled into the system
//! - **External Process Plugins**: Sandboxed execution of external code
//!
//! ### Storage & State
//! - **StateManager**: Persists workflow state and execution records
//! - **ResultCache**: Caches execution results for performance
//! - **Checkpoint System**: Allows workflow pause/resume with state recovery
//!
//! ### Interfaces
//! - **CLI**: Command-line interface for workflow execution
//! - **TUI**: Terminal UI for interactive workflow management
//! - **MCP Server**: Model Context Protocol server (stub implementation)
//!
//! ## Execution Flow
//!
//! 1. **Workflow Definition**: User defines workflow with nodes and dependencies
//! 2. **Validation**: System validates DAG structure and tool availability
//! 3. **Scheduling**: DagScheduler determines execution order
//! 4. **Execution**: Engine executes nodes with retry logic and caching
//! 5. **Monitoring**: Real-time status updates and pause/resume capability
//! 6. **Persistence**: State saved to storage backend
//! 7. **Completion**: Final results recorded and cached
//!
//! ## Key Design Principles
//!
//! - **Async/Await**: All operations are non-blocking and concurrent
//! - **Type Safety**: Strong typing throughout with thiserror for errors
//! - **Observability**: Comprehensive tracing for all operations
//! - **Fault Tolerance**: Retry logic, checkpointing, and recovery mechanisms
//! - **Extensibility**: Plugin architecture for adding new capabilities
//!
//! ## Configuration
//!
//! Configuration is managed through Config struct with support for:
//! - Multiple storage backends (File, Memory, Redis)
//! - Parallel execution limits
//! - Caching strategies
//! - Plugin-specific settings
//!
//! ## Error Handling
//!
//! All public APIs return Result<T, WorkflowError> with:
//! - Detailed error variants for different failure modes
//! - Contextual error information
//! - Recovery suggestions where applicable
//!
//! ## Testing
//!
//! - Unit tests in module files
//! - Integration tests in tests/ directory
//! - Property-based tests for critical paths
//! - Mock implementations for isolation

//!
//! A multi-interface workflow execution system built with Rust.
//! Supports CLI, TUI, and MCP server interfaces for workflow management.

pub mod adapter;
pub mod application;
pub mod config;
pub mod core;
pub mod di;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod interfaces;
pub mod performance;
pub mod plugins;
pub mod storage;
pub mod tools;
pub mod workflow;

// Re-export proc-macros when the "macros" feature is enabled
#[cfg(feature = "macros")]
pub use workflow_toolkit_macros::{ToolInput, ToolOutput};

// Re-export commonly used types
pub use crate::config::{CliConfigOverrides, Config, ConfigManager};
pub use crate::core::*;
pub use crate::error::{Result, WorkflowError};
pub use crate::performance::{PerformanceConfig, PerformanceManager};
pub use crate::tools::{ToolNode, ToolRegistry};
pub use crate::workflow::{
    DefaultExecutionManager, ExecutionHandle, ExecutionManager, ExecutionResult,
    WorkflowDefinition, WorkflowEngine, WorkflowExecution,
};

/// Initialize the logging system
pub fn init_logging() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(e) => {
            // Log the parsing error in debug mode before falling back to default
            eprintln!(
                "Warning: Failed to parse RUST_LOG environment variable: {}",
                e
            );
            "workflow_toolkit=info".into()
        }
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.ws_port, 8081);
    }

    #[test]
    fn test_execution_context() {
        let mut context = ExecutionContext::new();
        context.set_variable(
            "test_key",
            serde_json::Value::String("test_value".to_string()),
        );

        assert!(context.get_variable("test_key").is_some());
        assert!(context.get_variable("nonexistent").is_none());
    }

    #[test]
    fn test_execution_status() {
        assert!(ExecutionStatus::Completed.is_terminal());
        assert!(ExecutionStatus::Failed.is_terminal());
        assert!(!ExecutionStatus::Running.is_terminal());

        assert!(ExecutionStatus::Running.can_pause());
        assert!(!ExecutionStatus::Completed.can_pause());

        assert!(ExecutionStatus::Paused.can_resume());
        assert!(!ExecutionStatus::Running.can_resume());
    }

    #[tokio::test]
    async fn test_storage_backends() -> Result<()> {
        use crate::storage::{CacheBackend, FileStorage, SimpleMemoryCache, StorageBackend};
        use tempfile::TempDir;

        // Test memory cache
        let cache = SimpleMemoryCache::new();
        cache
            .set("test_key", b"test_value".to_vec(), None)
            .await?;
        let value = cache.get("test_key").await;
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test file storage
        let temp_dir = TempDir::new().map_err(|e| WorkflowError::storage(format!("Failed to create temp dir: {}", e)))?;
        let storage = FileStorage::new(temp_dir.path())?;
        storage.save("test_key", b"test_value").await?;
        let value = storage.load("test_key").await?;
        assert_eq!(value, Some(b"test_value".to_vec()));
        Ok(())
    }
}
