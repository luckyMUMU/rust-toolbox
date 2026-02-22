//! 领域服务模块
//!
//! 提供跨聚合的业务逻辑服务

pub mod execution_calculator;
pub mod workflow_validator;

pub use execution_calculator::{
    ExecutionStateCalculator, ExecutionStateResult, NodeStateStats, ProgressDetails,
    ExecutionTimelineEntry,
};
pub use workflow_validator::{
    DomainWorkflowValidator, DomainValidationResult, ValidationRule, ValidationError, ValidationWarning,
};
