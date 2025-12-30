//! Core types and traits for the workflow toolkit

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// Helper function to deserialize duration from seconds
fn deserialize_duration_from_secs<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

/// Unique identifier for workflows
pub type WorkflowId = Uuid;

/// Unique identifier for tool nodes
pub type ToolId = String;

/// Unique identifier for plugins
pub type PluginId = String;

/// Execution context passed to tools and workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub workflow_id: Option<WorkflowId>,
    pub execution_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub global_variables: HashMap<String, Value>,
    pub started_at: DateTime<Utc>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            workflow_id: None,
            execution_id: Uuid::new_v4().to_string(),
            user_id: None,
            session_id: None,
            global_variables: HashMap::new(),
            started_at: Utc::now(),
        }
    }
    
    pub fn with_workflow_id(mut self, workflow_id: WorkflowId) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }
    
    pub fn with_user_id<S: Into<String>>(mut self, user_id: S) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    pub fn set_variable<K: Into<String>>(&mut self, key: K, value: Value) {
        self.global_variables.insert(key.into(), value);
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

/// Retry policy for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub strategy: RetryStrategy,
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Option<Duration>,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            strategy: RetryStrategy::ExponentialBackoff,
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Some(Duration::from_secs(60)),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    None,
    FixedInterval,
    ExponentialBackoff,
    LinearBackoff,
    Custom(String),
}

/// Execution status for workflows and tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
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
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled | Self::Timeout)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }
    
    pub fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }
    
    pub fn can_resume(&self) -> bool {
        matches!(self, Self::Paused)
    }
    
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

/// Configuration for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub timeout: Option<Duration>,
    pub retry_policy: RetryPolicy,
    pub parallel_limit: Option<usize>,
    pub checkpoint_interval: Option<Duration>,
    pub enable_caching: bool,
    pub execution_mode: ExecutionMode,
    pub concurrency_config: ConcurrencyConfig,
    pub metadata: HashMap<String, Value>,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(3600)), // 1 hour default
            retry_policy: RetryPolicy::default(),
            parallel_limit: Some(10),
            checkpoint_interval: Some(Duration::from_secs(300)), // 5 minutes
            enable_caching: true,
            execution_mode: ExecutionMode::default(),
            concurrency_config: ConcurrencyConfig::default(),
            metadata: HashMap::new(),
        }
    }
}

/// Tool information for registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub parameters_schema: Value,
    pub return_schema: Value,
    pub plugin_name: Option<String>,
    pub dependencies: Vec<String>, // Tool names this tool depends on
    pub version_requirements: HashMap<String, String>, // Tool name -> version requirement
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub description: Option<String>,
    pub author: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Plugin type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Go,
    Docker,
    Wasm,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            enabled: true,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: Option<String>,
    #[serde(deserialize_with = "deserialize_duration_from_secs")]
    pub token_expiry: Duration,
    pub allowed_origins: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwt_secret: None,
            token_expiry: Duration::from_secs(3600), // 1 hour
            allowed_origins: vec!["*".to_string()],
        }
    }
}

/// Execution mode for workflows and tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum ExecutionMode {
    /// Synchronous execution - blocks until completion
    Sync,
    /// Asynchronous execution - returns immediately with execution handle
    Async,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Async
    }
}

/// Concurrency control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Maximum number of concurrent workflows
    pub max_concurrent_workflows: usize,
    /// Task queue size limit
    pub task_queue_size: usize,
    /// Enable task prioritization
    pub enable_prioritization: bool,
    /// Resource limits per task
    pub resource_limits: ResourceLimits,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            max_concurrent_workflows: 10,
            task_queue_size: 1000,
            enable_prioritization: false,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in seconds
    pub max_cpu_time: Option<Duration>,
    /// Maximum execution time
    pub max_execution_time: Option<Duration>,
    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_cpu_time: Some(Duration::from_secs(300)), // 5 minutes
            max_execution_time: Some(Duration::from_secs(600)), // 10 minutes
            max_file_descriptors: Some(1024),
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Execution metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub active_executions: usize,
    pub queued_executions: usize,
    pub total_capacity: usize,
    pub queue_capacity: usize,
    pub available_permits: usize,
}