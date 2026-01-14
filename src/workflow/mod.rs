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
//! # Execution Flow
//!
//! 1. Define a workflow using [`WorkflowDefinition`].
//! 2. Submit it to the [`WorkflowEngine`].
//! 3. The engine validates the definition.
//! 4. The scheduler determines the execution order.
//! 5. Nodes are executed (possibly in parallel) using the tool registry.

pub mod audit;
pub mod definition;
pub mod engine;
pub mod execution;
pub mod execution_manager;
pub mod result_cache;
pub mod scheduler;
pub mod validator;

#[cfg(test)]
pub mod retry_tests;

#[cfg(test)]
pub mod execution_manager_simple_test;

#[cfg(test)]
pub mod audit_tests;

pub use audit::{
    AuditEvent, AuditEventType, AuditLogger, AuditQueryCriteria, AuditReport, AuditSeverity,
    ErrorDetails, ErrorSummary, ExecutionLogEntry, LogLevel,
};
pub use definition::{
    NodeType, ParameterType, TemplateParameter, WorkflowDefinition, WorkflowEdge, WorkflowNode,
    WorkflowTemplate,
};
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
