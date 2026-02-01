//! # Tool System
//!
//! NOTE: This module is undergoing radical refactoring. The old trait-based system
//! is being replaced with an enum-based system for better performance and type safety.
//!
//! ## Migration Status
//!
//! - [x] ToolRegistry trait - REMOVED (use compat::ToolRegistry for temp compatibility)
//! - [x] ToolNode trait - REMOVED (use compat::ToolNode for temp compatibility)
//! - [x] ToolExecutor trait - REMOVED (use compat::ToolExecutor for temp compatibility)
//! - [x] ComposableTool trait - REMOVED (use compat::ComposableTool for temp compatibility)
//! - [x] Tool enum - DONE (Task 1.2)
//! - [x] New registry implementation - DONE (Task 1.3)
//! - [x] Tool node implementations - DONE (Task 1.4)
//! - [x] Middleware system - DONE (Task 2.1, 2.2, 2.3)
//! - [x] Typed tool system - DONE (Task 3.1, 3.2)
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

pub mod algo;
pub mod compat;  // COMPAT: Old trait compatibility layer
pub mod composable;
pub mod middleware;  // NEW: Middleware system (Task 2.1)
pub mod node;
pub mod registry;
pub mod template;
pub mod types;  // NEW: Enum-based tool system
pub mod version;

// COMPATIBILITY EXPORTS: Old trait system (temporary)
pub use compat::{ComposableTool, ToolExecutor, ToolNode, ToolRegistry as ToolRegistryTrait, BasicToolRegistry};

// Backward compatibility: ToolRegistry trait is now in compat module
// This allows old code using `use crate::tools::ToolRegistry` to continue working
pub use compat::ToolRegistry;

// NEW SYSTEM EXPORTS: Enum-based tool system
pub use composable::{
    ComposableToolAdapter, ConditionalTool, ParallelTools, ToolChain,
    ToolComposer, ToolCompositionBuilder,
};
pub use middleware::{
    CacheMiddleware, CircuitBreakerMiddleware, ExecutionMetadata, LoggingMiddleware,
    MetricsMiddleware, Middleware, MiddlewareContext, MiddlewareStack, MiddlewareStackBuilder,
    Next, RetryMiddleware, TimeoutMiddleware, TimingMiddleware,
};
pub use node::{
    AsyncFunctionExecutor, BasicTool, BasicToolBuilder, FunctionExecutor,
};
pub use registry::{ToolRegistryBuilder, ToolRegistry as ToolRegistryStruct};
pub use template::{ParameterTemplate, TemplateContext, TemplateEngine, TemplateFn};
pub use types::{
    CompositionType, ComposedTool, DockerTool, InputSchema, NativeTool, NativeToolBuilder, NodeJsTool, OutputSchema, 
    PythonTool, ResourceRequirements, Tool, ToolExample, ToolId, ToolInput, ToolInputConvert, 
    ToolKind, ToolMetadata, ToolOutput, ToolOutputConvert, WasmTool,
};
pub use version::{
    DependencyResolver, ResolutionResult, ToolDependency, ToolVersion, Version, VersionConflict,
    VersionRequirement,
};

// Compatibility type aliases for migration period
pub type ToolEnum = Tool;
pub type ToolInputStruct = ToolInput;
pub type ToolOutputStruct = ToolOutput;
