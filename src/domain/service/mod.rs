//! 领域服务模块
//!
//! 提供跨聚合的业务逻辑服务

pub mod execution_calculator;
pub mod workflow_validator;

pub use execution_calculator::{
    ExecutionStateCalculator, ExecutionStateResult, ExecutionTimelineEntry, NodeStateStats,
    ProgressDetails,
};
pub use workflow_validator::{
    DomainValidationResult, DomainWorkflowValidator, ValidationError, ValidationRule,
    ValidationWarning,
};
