//! # Tool System
//!
//! NOTE: This module is undergoing radical refactoring. The old trait-based system
//! is being replaced with an enum-based system for better performance and type safety.
//!
//! ## Migration Status
//!
//! - [x] ToolRegistry trait - REMOVED
//! - [x] ToolNode trait - REMOVED
//! - [x] ToolExecutor trait - REMOVED
//! - [x] ComposableTool trait - REMOVED
//! - [ ] Tool enum - TODO (Task 1.2)
//! - [ ] New registry implementation - TODO (Task 1.3)
//! - [ ] Typed tool system - TODO (Task 3)
//!
//! See [AGENTS.md](AGENTS.md) for detailed documentation.

pub mod algo;
pub mod composable;
pub mod node;
pub mod registry;
pub mod template;
pub mod types;  // NEW: Enum-based tool system
pub mod version;

// TODO: Remove old exports after migration
// These exports are temporarily kept for dependent modules
// Will be replaced with new enum-based exports

pub use composable::{
    ComposableToolAdapter, ConditionalTool, ParallelTools, ToolChain,
    ToolComposer, ToolCompositionBuilder,
};
pub use node::{
    AsyncFunctionExecutor, BasicTool, BasicToolBuilder, FunctionExecutor,
};
pub use registry::{BasicToolRegistry, ToolRegistryBuilder};
pub use template::{ParameterTemplate, TemplateContext, TemplateEngine, TemplateFn};
pub use types::{
    CompositionType, ComposedTool, DockerTool, NativeTool, NodeJsTool, PythonTool, ResourceRequirements,
    Tool, ToolExample, ToolId, ToolInput, ToolKind, ToolMetadata, ToolOutput, WasmTool,
};
pub use version::{
    DependencyResolver, ResolutionResult, ToolDependency, ToolVersion, Version, VersionConflict,
    VersionRequirement,
};
