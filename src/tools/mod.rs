//! # Tool System
//!
//! The tool system provides a flexible framework for defining, registering, and executing
//! tools within the workflow engine. Tools are the executable units that perform actual work.
//!
//! ## Key Components
//!
//! - **ToolNode**: The core trait representing an executable tool.
//! - **ToolRegistry**: Manages tool registration, retrieval, and dependency resolution.
//! - **BasicTool**: A concrete implementation of `ToolNode` supporting async execution.
//! - **TemplateEngine**: Handles parameter substitution and dynamic values.
//! - **Version**: SemVer-compatible versioning for tools.
//!
//! ## Example
//!
//! ```rust
//! use std::sync::Arc;
//! use serde_json::{json, Value};
//! use rust_tool_v2::tools::{BasicTool, BasicToolRegistry, ToolRegistry, AsyncFunctionExecutor};
//! use rust_tool_v2::core::{ExecutionContext, WorkflowId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Create a registry
//! let mut registry = BasicToolRegistry::default();
//!
//! // 2. Define a simple tool
//! let echo_tool = BasicTool::builder()
//!     .name("echo")
//!     .version("1.0.0")
//!     .description("Echoes the input")
//!     .executor(Arc::new(AsyncFunctionExecutor::new(|params, _| Box::pin(async move {
//!         Ok(params)
//!     }))))
//!     .build()?;
//!
//! // 3. Register the tool
//! registry.register_tool(Arc::new(echo_tool))?;
//!
//! // 4. Execute the tool
//! let context = ExecutionContext::new(WorkflowId::new(), "test-node");
//! let result = registry.execute_tool("echo", json!({ "message": "hello" }), context).await?;
//! println!("Result: {:?}", result);
//! # Ok(())
//! # }
//! ```

pub mod data;
pub mod node;
pub mod registry;
pub mod template;
pub mod version;

pub use data::{DataCacheTool, DataTransformTool};
pub use node::{
    AsyncFunctionExecutor, BasicTool, BasicToolBuilder, FunctionExecutor, ToolExecutor, ToolNode,
};
pub use registry::{BasicToolRegistry, ToolRegistry, ToolRegistryBuilder};
pub use template::{ParameterTemplate, TemplateContext, TemplateEngine, TemplateFn};
pub use version::{
    DependencyResolver, ResolutionResult, ToolDependency, ToolVersion, Version, VersionConflict,
    VersionRequirement,
};
