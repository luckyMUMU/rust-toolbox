//! 领域模型 - 执行相关

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// 工作流执行ID
pub type WorkflowId = Uuid;

/// 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub global_variables: HashMap<String, Value>,
    pub step_results: HashMap<String, Value>,
    #[serde(default = "Utc::now")]
    pub started_at: DateTime<Utc>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            workflow_id: None,
            user_id: None,
            session_id: None,
            global_variables: HashMap::new(),
            step_results: HashMap::new(),
            started_at: Utc::now(),
        }
    }

    pub fn with_workflow_id(mut self, id: Uuid) -> Self {
        self.workflow_id = Some(id);
        self
    }

    pub fn set_input_params(&mut self, params: Value) -> crate::error::Result<()> {
        if let Value::Object(map) = params {
            for (k, v) in map {
                self.global_variables.insert(k, v);
            }
        }
        Ok(())
    }

    pub fn set_variable(&mut self, key: &str, value: Value) {
        self.global_variables.insert(key.to_string(), value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.global_variables.get(key)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

impl ExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExecutionStatus::Completed
                | ExecutionStatus::Failed
                | ExecutionStatus::Cancelled
                | ExecutionStatus::Timeout
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionStatus::Completed)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, ExecutionStatus::Failed | ExecutionStatus::Timeout)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, ExecutionStatus::Running)
    }

    pub fn can_resume(&self) -> bool {
        matches!(self, ExecutionStatus::Paused)
    }

    pub fn can_stop(&self) -> bool {
        !self.is_terminal()
    }
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Pending => write!(f, "Pending"),
            ExecutionStatus::Running => write!(f, "Running"),
            ExecutionStatus::Paused => write!(f, "Paused"),
            ExecutionStatus::Completed => write!(f, "Completed"),
            ExecutionStatus::Failed => write!(f, "Failed"),
            ExecutionStatus::Cancelled => write!(f, "Cancelled"),
            ExecutionStatus::Timeout => write!(f, "Timeout"),
        }
    }
}

/// 执行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Sync,
    Async,
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}
