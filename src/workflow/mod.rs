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
pub mod component;
pub mod context;
pub mod converter;
pub mod definition;
pub mod engine;
pub mod engine_v2;
pub mod execution;
pub mod execution_manager;
pub mod executor;
pub mod flow_node;
pub mod result_cache;
pub mod scheduler;
pub mod state;
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

// New architecture exports (LiteFlow-inspired)
pub use component::{
    Component, ComponentOutput, ComponentRegistry, ComponentStatus, ComponentType,
    ParallelComponent, ToolComponent, WaitStrategy,
};
pub use context::{DataContext, SlotValue};
pub use converter::WorkflowConverter;
pub use executor::{
    AuditExecutor, BasicExecutor, BoxedExecutor, CacheExecutor, Executor, ExecutorChainBuilder,
    RetryExecutor,
};
pub use state::{CheckpointManager, ControlSignals, ExecutionTracker};

// Refactored engine (v2)
pub use engine_v2::RefactoredWorkflowEngine;
pub use flow_node::FlowNode;
