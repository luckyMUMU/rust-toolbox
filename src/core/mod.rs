//! Core domain models and types.

pub mod version;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a workflow execution
pub type WorkflowId = Uuid;

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    
    // Schema fields
    #[serde(rename = "parameters")]
    pub parameters_schema: Value, // Renamed to match usage parameters_schema
    pub return_schema: Value,
    
    // Metadata
    pub category: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    // Plugin integration
    pub plugin_name: Option<String>,
    pub version_requirements: HashMap<String, String>, // Tool dependencies with version reqs
    
    // Timestamps
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Wasm,
    Docker,
    Go,
}

/// Information about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub plugin_type: PluginType,
    pub metadata: HashMap<String, Value>,
}

/// Context for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub session_id: Option<String>, // Added
    pub global_variables: HashMap<String, Value>,
    pub step_results: HashMap<String, Value>,
    #[serde(default = "Utc::now")]
    pub started_at: DateTime<Utc>, // Added
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
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of a workflow or node execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
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

/// Configuration for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_max_concurrent_steps")]
    pub max_concurrent_steps: usize,
    pub default_timeout: Option<u64>, // in seconds
    pub checkpoint_interval: Option<Duration>, // in seconds (Added)
    pub retry_policy: Option<RetryPolicy>,
}

fn default_max_concurrent_steps() -> usize {
    10
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_concurrent_steps: default_max_concurrent_steps(),
            default_timeout: None,
            checkpoint_interval: None,
            retry_policy: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    None,
    FixedInterval,
    ExponentialBackoff,
    LinearBackoff,
    Custom(Value),
}

/// Retry policy for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration, // Renamed from delay_ms (and type changed)
    pub max_delay: Option<Duration>, // Added
    pub backoff_multiplier: f64, // Renamed from multiplier
    pub strategy: RetryStrategy, // Added
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Some(Duration::from_secs(60)),
            backoff_multiplier: 2.0,
            strategy: RetryStrategy::ExponentialBackoff,
        }
    }
}

// --- Additional Types for compatibility ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub enabled: bool,
    pub token: Option<String>,
    pub jwt_secret: Option<String>,
    pub token_expiry: Duration,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests: u32,
    pub window_ms: u64,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetworkStatus {
    #[default]
    Connected,
    Disconnected,
    Degraded,
    Limited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SystemHealth {
    #[default]
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthAssessment {
    pub overall_health: SystemHealth, // Renamed from status to match usage
    pub message: String,
    pub overall_score: f64, // Changed to f64
    pub cpu_health_score: f64, // Changed to f64
    pub memory_health_score: f64, // Changed to f64
    pub disk_health_score: f64, // Changed to f64
    pub network_health_score: f64, // Changed to f64
    pub recommendations: Vec<String>,
    pub load_assessment: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_usage: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub uptime: Duration,
    pub load_average: f64,
    pub process_count: usize,
    pub thread_count: usize,
    pub active_workflows: usize,
    pub network_status: NetworkStatus,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub active_executions: usize,
    pub queued_executions: usize,
    pub total_capacity: usize,
    pub queue_capacity: usize,
    pub available_permits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Sync,
    Async,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_time: Option<Duration>,
    pub max_execution_time: Option<Duration>,
    // Kept for compatibility if needed, otherwise can be removed
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}
