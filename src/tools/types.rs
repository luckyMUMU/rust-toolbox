//! Tool type definitions for the new enum-based tool system.
//!
//! This module provides the core `Tool` enum that replaces the old dyn-trait system,
//! offering better performance, type safety, and developer experience.

use crate::core::{ExecutionContext, ToolInfo};
use crate::error::{Result, WorkflowError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

/// Unique identifier for tools
///
/// Using a newtype pattern for type safety and to prevent mixing up with other IDs
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ToolId(u64);

impl ToolId {
    /// Create a new tool ID
    pub fn new() -> Self {
        // Use a simple counter-based ID generation for now
        // In production, this could use UUID or atomic counter
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for ToolId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tool_{}", self.0)
    }
}

/// Categories of tools
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    /// Native Rust tools
    Native,
    /// Python script tools
    Python,
    /// Node.js tools
    NodeJs,
    /// Docker container tools
    Docker,
    /// WebAssembly tools
    Wasm,
    /// Composed tools (chains, conditionals, etc.)
    Composed,
}

impl fmt::Display for ToolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolKind::Native => write!(f, "native"),
            ToolKind::Python => write!(f, "python"),
            ToolKind::NodeJs => write!(f, "nodejs"),
            ToolKind::Docker => write!(f, "docker"),
            ToolKind::Wasm => write!(f, "wasm"),
            ToolKind::Composed => write!(f, "composed"),
        }
    }
}

/// Input to a tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInput {
    /// The input parameters
    pub params: Value,
    /// Additional metadata
    pub metadata: Option<Value>,
}

impl ToolInput {
    /// Create a new tool input
    pub fn new(params: Value) -> Self {
        Self {
            params,
            metadata: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(params: Value, metadata: Value) -> Self {
        Self {
            params,
            metadata: Some(metadata),
        }
    }
}

/// Output from a tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The execution result
    pub result: Value,
    /// Execution metadata (timing, resources used, etc.)
    pub metadata: Option<Value>,
    /// Whether the execution was successful
    pub success: bool,
}

impl ToolOutput {
    /// Create a successful output
    pub fn success(result: Value) -> Self {
        Self {
            result,
            metadata: None,
            success: true,
        }
    }

    /// Create a failed output
    pub fn failure(error: &str) -> Self {
        Self {
            result: Value::String(error.to_string()),
            metadata: None,
            success: false,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// The main Tool enum that replaces dyn ToolNode
///
/// This enum provides a unified interface for all tool types while maintaining
/// zero-cost abstraction through static dispatch.
#[derive(Clone)]
pub enum Tool {
    /// Native Rust tool
    Native(Arc<NativeTool>),
    /// Python script tool
    Python(Arc<PythonTool>),
    /// Node.js tool
    NodeJs(Arc<NodeJsTool>),
    /// Docker container tool
    Docker(Arc<DockerTool>),
    /// WebAssembly tool
    Wasm(Arc<WasmTool>),
    /// Composed tool (chains, conditionals, parallel)
    Composed(Arc<ComposedTool>),
}

impl Tool {
    /// Get the tool kind/category
    pub fn kind(&self) -> ToolKind {
        match self {
            Tool::Native(_) => ToolKind::Native,
            Tool::Python(_) => ToolKind::Python,
            Tool::NodeJs(_) => ToolKind::NodeJs,
            Tool::Docker(_) => ToolKind::Docker,
            Tool::Wasm(_) => ToolKind::Wasm,
            Tool::Composed(_) => ToolKind::Composed,
        }
    }

    /// Get tool metadata
    pub fn metadata(&self) -> ToolMetadata {
        match self {
            Tool::Native(t) => (*t.metadata).clone(),
            Tool::Python(t) => (*t.metadata).clone(),
            Tool::NodeJs(t) => (*t.metadata).clone(),
            Tool::Docker(t) => (*t.metadata).clone(),
            Tool::Wasm(t) => (*t.metadata).clone(),
            Tool::Composed(t) => (*t.metadata).clone(),
        }
    }

    /// Get tool ID
    pub fn id(&self) -> ToolId {
        match self {
            Tool::Native(t) => t.id,
            Tool::Python(t) => t.id,
            Tool::NodeJs(t) => t.id,
            Tool::Docker(t) => t.id,
            Tool::Wasm(t) => t.id,
            Tool::Composed(t) => t.id,
        }
    }

    /// Execute the tool
    /// 
    /// TODO: This is a placeholder implementation.
    /// Actual execution logic will be implemented in Task 1.4
    pub async fn execute(
        &self,
        _input: ToolInput,
        _ctx: ExecutionContext,
    ) -> crate::error::Result<ToolOutput> {
        // Placeholder - will be implemented with actual execution logic
        Ok(ToolOutput::success(serde_json::Value::Null))
    }
}

/// Native Rust tool implementation
#[derive(Clone)]
pub struct NativeTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    // TODO: Add executor field
}

/// Python script tool
#[derive(Clone)]
pub struct PythonTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub script_path: std::path::PathBuf,
    // TODO: Add interpreter configuration
}

/// Node.js tool
#[derive(Clone)]
pub struct NodeJsTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub script_path: std::path::PathBuf,
    // TODO: Add node configuration
}

/// Docker container tool
#[derive(Clone)]
pub struct DockerTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub image: String,
    // TODO: Add container configuration
}

/// WebAssembly tool
#[derive(Clone)]
pub struct WasmTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub wasm_path: std::path::PathBuf,
    // TODO: Add wasm runtime configuration
}

/// Composed tool (chains, conditionals, parallel execution)
#[derive(Clone)]
pub struct ComposedTool {
    pub id: ToolId,
    pub metadata: Arc<ToolMetadata>,
    pub composition_type: CompositionType,
    // TODO: Add composition configuration
}

/// Types of tool composition
#[derive(Clone, Debug)]
pub enum CompositionType {
    /// Sequential chain of tools
    Chain(Vec<ToolId>),
    /// Conditional execution
    Conditional {
        condition: String,
        then_tool: ToolId,
        else_tool: Option<ToolId>,
    },
    /// Parallel execution
    Parallel(Vec<ToolId>),
}

/// Rich metadata for tools
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Basic tool info
    pub info: ToolInfo,
    /// Tool category/kind
    pub kind: ToolKind,
    /// Input schema
    pub input_schema: Option<Value>,
    /// Output schema
    pub output_schema: Option<Value>,
    /// Usage examples
    pub examples: Vec<ToolExample>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Version information
    pub version: String,
}

/// Tool usage example
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolExample {
    pub title: String,
    pub description: String,
    pub input: Value,
    pub expected_output: Value,
}

/// Resource requirements for tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: u64,
    /// Recommended memory in MB
    pub recommended_memory_mb: u64,
    /// CPU intensity level (1-10)
    pub cpu_intensity: u8,
    /// Whether network access is required
    pub network_required: bool,
    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: u64,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_memory_mb: 64,
            recommended_memory_mb: 256,
            cpu_intensity: 5,
            network_required: false,
            estimated_duration_ms: 1000,
        }
    }
}

// TODO: Implement Tool methods (execute, metadata, etc.)
// This will be done in subsequent tasks as we build out the system

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_id_generation() {
        let id1 = ToolId::new();
        let id2 = ToolId::new();
        assert_ne!(id1, id2);
        assert!(id1.as_u64() > 0);
        assert!(id2.as_u64() > id1.as_u64());
    }

    #[test]
    fn test_tool_kind_display() {
        assert_eq!(ToolKind::Native.to_string(), "native");
        assert_eq!(ToolKind::Python.to_string(), "python");
        assert_eq!(ToolKind::Docker.to_string(), "docker");
    }

    #[test]
    fn test_tool_output() {
        let output = ToolOutput::success(Value::String("result".to_string()));
        assert!(output.success);
        assert_eq!(output.result, Value::String("result".to_string()));

        let error = ToolOutput::failure("error message");
        assert!(!error.success);
    }
}
