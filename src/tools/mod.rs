//! # Tool System
//!
//! A high-performance, enum-based tool system for workflow execution.
//!
//! ## Architecture
//!
//! - **Tool**: Core enum representing all tool types (Native, Python, Node.js, Docker, WASM, Composed)
//! - **ToolRegistry**: High-performance registry with O(1) lookups
//! - **Middleware**: Composable middleware stack for cross-cutting concerns
//! - **Composable**: Tool composition (chains, conditionals, parallel execution)
//! - **External**: External tool executors for Python, Node.js, Docker

pub mod algo;
pub mod composable;
pub mod composed_executor;
pub mod composition;
pub mod executor;
pub mod external;
pub mod middleware;
pub mod registry;
pub mod schema_validator;
pub mod template;
pub mod types;
pub mod version;
pub mod versioned_registry;

// Core exports
pub use composable::{
    ComposableToolAdapter, ConditionalTool, ParallelTools, ToolChain, ToolComposer,
    ToolCompositionBuilder,
};
pub use composition::{
    CompositionConfig, CompositionExecutor, CompositionExecutorBuilder, CompositionResult,
    CompositionStrategy, FailureStrategy, ResultMergeStrategy, ToolExecutionRecord,
};
pub use executor::{ToolExecutor, ToolExecutorBuilder, ToolExecutorConfig};
pub use external::{
    DockerExecutor, DockerExecutorConfig, DockerMount, ExecutionResult, ExecutorFactory,
    ExecutorType, ExternalExecutor, ExternalExecutorConfig, NodeJsExecutor, PythonExecutor,
};
pub use middleware::{
    CacheMiddleware, CircuitBreakerMiddleware, ExecutionMetadata, LoggingMiddleware,
    MetricsMiddleware, Middleware, MiddlewareContext, MiddlewareStack, MiddlewareStackBuilder,
    Next, RetryMiddleware, TimeoutMiddleware, TimingMiddleware,
};
pub use registry::{ToolRegistry, ToolRegistryBuilder};
pub use schema_validator::{
    SchemaErrorType, SchemaRegistry, SchemaValidationError, SchemaValidationResult,
    SchemaValidator, SchemaValidatorBuilder, SchemaValidatorConfig, ToolSchema,
};
pub use template::{ParameterTemplate, TemplateContext, TemplateEngine, TemplateFn};
pub use types::{
    ComposedTool, CompositionType, DataFlowMapping, DockerTool, ErrorPropagationStrategy,
    InputSchema, NativeTool, NativeToolBuilder, NodeJsTool, OutputSchema, PythonTool,
    ResourceRequirements, Tool, ToolExample, ToolId, ToolInput, ToolInputConvert, ToolKind,
    ToolMetadata, ToolOutput, ToolOutputConvert, WasmTool,
};
pub use composed_executor::ComposedToolExecutor;
pub use version::{
    DependencyResolver, ResolutionResult, ToolDependency, ToolVersion, Version, VersionConflict,
    VersionRequirement,
};
pub use versioned_registry::{
    VersionSelectionStrategy, VersionedToolRegistry, VersionedToolRegistryBuilder,
};
