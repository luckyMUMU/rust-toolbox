//! Workflow engine and related components.
//!
//! This module provides the core functionality for defining, validating, scheduling, and executing workflows.
//!
//! # Key Components
//!
//! - [`WorkflowEngine`]: The main entry point for executing workflows.
//! - [`WorkflowDefinition`]: The structure defining a workflow's nodes and edges.
//! - [`DagScheduler`]: Handles topological sorting and task scheduling.
//! - [`WorkflowExecution`]: Tracks the runtime state of a workflow.
//!
//! # New Architecture (LiteFlow-inspired)
//!
//! - [`component`]: Component system for workflow nodes (Tool, Parallel, etc.)
//! - [`context`]: Data context and slot system for data passing
//! - [`executor`]: Executor chain for cross-cutting concerns (retry, cache, audit)
//! - [`state`]: Unified state management and checkpointing
//!
//! # Execution Flow
//!
//! 1. Define a workflow using [`WorkflowDefinition`].
//! 2. Submit it to the [`WorkflowEngine`].
//! 3. The engine validates the definition.
//! 4. The scheduler determines the execution order.
//! 5. Nodes are executed (possibly in parallel) using the tool registry.

pub mod audit;
pub mod backpressure;
pub mod circuit_breaker;
pub mod component;
pub mod config_validator;
pub mod context;
pub mod converter;
pub mod definition;
pub mod el_expression;
pub mod engine;
pub mod error_handler;
pub mod execution;
pub mod execution_manager;
pub mod executor;
pub mod flow_control;
pub mod flow_node;
pub mod metrics;
pub mod parallel_executor;
pub mod rate_limiter;
pub mod result_cache;
pub mod scheduler;
pub mod state;
pub mod validator;

// Re-export EL expression types
pub mod el {
    pub use super::el_expression::{ExpressionContext, ExpressionEngine};
}

// 注意：以下测试模块已移除，因为使用了已废弃的 ToolNode trait
// #[cfg(test)]
// pub mod retry_tests;
// #[cfg(test)]
// pub mod execution_manager_simple_test;

#[cfg(test)]
pub mod audit_tests;

pub use audit::{
    AuditEvent, AuditEventType, AuditLogger, AuditQueryCriteria, AuditReport, AuditSeverity,
    ErrorDetails, ErrorSummary, ExecutionLogEntry, LogLevel,
};
pub use backpressure::{
    BackpressureConfig, BackpressureController, BackpressurePermit, BackpressureState,
    BackpressureStats, BackpressureStrategy, RequestPriority,
};
pub use definition::{
    NodeType, ParameterType, TemplateParameter, WorkflowDefinition, WorkflowEdge, WorkflowNode,
    WorkflowTemplate,
};
// WorkflowEngine trait exported from engine module
pub use engine::WorkflowEngine;
pub use execution::{
    Checkpoint, ExecutionRecord, NodeExecutionState, WorkflowExecution, WorkflowState,
};
pub use execution_manager::{
    DefaultExecutionManager, ExecutionHandle, ExecutionManager, ExecutionResult,
    TaskExecutionRequest,
};
pub use result_cache::{
    CacheConfig, CacheKey, CacheStats, CachedResult, InvalidationStrategy, ResultCache,
};
pub use scheduler::{DagScheduler, ExecutionStats, NodeExecutionInfo, SchedulingResult};
pub use validator::{ValidationResult, WorkflowValidator};

// New architecture exports (LiteFlow-inspired)
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerMetrics, CircuitState,
};
pub use component::{
    Component, ComponentOutput, ComponentRegistry, ComponentStatus, ComponentType,
    ParallelComponent, ToolComponent, WaitStrategy,
};
pub use config_validator::{
    ConfigHotReloader, ConfigLoader, ConfigValidator, FileConfigLoader, ValidationReport,
};
pub use context::{DataContext, SlotValue};
pub use converter::WorkflowConverter;
pub use error_handler::{
    ErrorClassification, ErrorHandler, ErrorHandlingStrategy, ErrorSeverity,
    WorkflowErrorClassifier,
};
pub use executor::{
    AuditExecutor, BasicExecutor, BoxedExecutor, CacheExecutor, Executor, ExecutorChainBuilder,
    RetryExecutor,
};
pub use flow_control::FlowControlExecutor;
pub use metrics::{
    CompositeMetricsCollector, InMemoryMetricsCollector, MetricsCollector, MetricsSnapshot,
    PrometheusMetricsCollector, WorkflowStats,
};
pub use rate_limiter::{
    RateLimiter, RateLimiterError, WorkflowRateLimitConfig, WorkflowRateLimiter,
};
pub use state::{CheckpointManager, ControlSignals, ExecutionTracker};

// Refactored engine (now default)
pub use engine::{DefaultWorkflowEngine, RefactoredWorkflowEngine};
pub use flow_node::FlowNode;
