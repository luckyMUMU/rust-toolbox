//! Workflow engine and related components

pub mod definition;
pub mod validator;
pub mod scheduler;
pub mod engine;
pub mod execution;
pub mod execution_manager;
pub mod audit;
pub mod result_cache;

#[cfg(test)]
pub mod retry_tests;

#[cfg(test)]
pub mod execution_manager_simple_test;

#[cfg(test)]
pub mod audit_tests;

pub use definition::{
    WorkflowDefinition, WorkflowNode, WorkflowEdge, NodeType,
    WorkflowTemplate, TemplateParameter, ParameterType
};
pub use validator::{WorkflowValidator, ValidationResult};
pub use scheduler::{DagScheduler, SchedulingResult, NodeExecutionInfo, ExecutionStats};
pub use engine::WorkflowEngine;
pub use execution::{
    WorkflowExecution, WorkflowState, ExecutionRecord, 
    Checkpoint, NodeExecutionState
};
pub use execution_manager::{
    ExecutionManager, DefaultExecutionManager, ExecutionHandle, 
    ExecutionResult, TaskExecutionRequest
};
pub use audit::{
    AuditLogger, AuditEvent, AuditEventType, AuditSeverity,
    ExecutionLogEntry, LogLevel, AuditQueryCriteria, AuditReport,
    ErrorDetails, ErrorSummary
};
pub use result_cache::{
    ResultCache, CacheKey, CachedResult, InvalidationStrategy,
    CacheConfig, CacheStats
};