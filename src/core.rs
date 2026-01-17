//! Core types and traits for the workflow toolkit.
//!
//! This module defines the fundamental data structures and types used throughout the system,
//! including:
//! - [`ExecutionContext`]: Context passed to tools and workflows during execution.
//! - [`WorkflowConfig`]: Configuration for workflow execution behavior.
//! - [`ExecutionStatus`]: State machine for workflow and task lifecycles.
//! - [`SystemStatus`]: Metrics for system monitoring.
//!
//! # Examples
//!
//! Creating a new execution context:
//! ```rust
//! use rust_tool_v2::core::ExecutionContext;
//!
//! let context = ExecutionContext::new()
//!     .with_user_id("user_123");
//! ```

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

// Helper function to deserialize duration from seconds
fn deserialize_duration_from_secs<'de, D>(
    deserializer: D,
) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

// Helper function to serialize duration as seconds
fn serialize_duration_as_secs<S>(
    duration: &Duration,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

/// Unique identifier for workflows (UUID v4)
pub type WorkflowId = Uuid;

/// Unique identifier for tool nodes (String)
pub type ToolId = String;

/// Unique identifier for plugins (String)
pub type PluginId = String;

/// Execution context passed to tools and workflows.
///
/// Contains runtime information such as the current workflow ID, execution ID,
/// user context, and global variables accessible to all nodes in the workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// ID of the workflow being executed, if any
    pub workflow_id: Option<WorkflowId>,
    /// Unique ID for this specific execution run
    pub execution_id: String,
    /// ID of the user initiating the execution
    pub user_id: Option<String>,
    /// Session ID for grouping executions
    pub session_id: Option<String>,
    /// Global variables accessible to all nodes
    pub global_variables: HashMap<String, Value>,
    /// Timestamp when execution started
    pub started_at: DateTime<Utc>,
}

impl ExecutionContext {
    /// Create a new execution context with a generated execution ID and current timestamp.
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

    /// Set the workflow ID for this context.
    pub fn with_workflow_id(mut self, workflow_id: WorkflowId) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Set the user ID for this context.
    pub fn with_user_id<S: Into<String>>(mut self, user_id: S) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set a global variable.
    pub fn set_variable<K: Into<String>>(&mut self, key: K, value: Value) {
        self.global_variables.insert(key.into(), value);
    }

    /// Get a global variable by key.
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.global_variables.get(key)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry policy for task execution.
///
/// Defines how a task should be retried in case of failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Strategy to use for calculating delay between retries
    pub strategy: RetryStrategy,
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay for the first retry
    pub base_delay: Duration,
    /// Maximum delay allowed between retries
    pub max_delay: Option<Duration>,
    /// Multiplier for exponential backoff (e.g., 2.0 for doubling delay)
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

/// Retry strategy enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// No delay between retries
    None,
    /// Fixed delay between retries
    FixedInterval,
    /// Exponentially increasing delay
    ExponentialBackoff,
    /// Linearly increasing delay
    LinearBackoff,
    /// Custom strategy (implementation specific)
    Custom(String),
}

/// Execution status for workflows and tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum ExecutionStatus {
    /// Queued for execution but not yet started
    Pending,
    /// Currently executing
    Running,
    /// Execution paused by user or system
    Paused,
    /// Successfully completed
    Completed,
    /// Execution failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
    /// Execution timed out
    Timeout,
}

impl ExecutionStatus {
    /// Check if the status represents a terminal state (cannot transition further).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Timeout
        )
    }

    /// Check if the execution is currently active (running).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the execution can be paused.
    pub fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the execution can be resumed.
    pub fn can_resume(&self) -> bool {
        matches!(self, Self::Paused)
    }

    /// Check if the execution can be stopped/cancelled.
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

/// Configuration for workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Global timeout for the entire workflow
    pub timeout: Option<Duration>,
    /// Default retry policy for nodes that don't specify one
    pub retry_policy: RetryPolicy,
    /// Maximum number of parallel tasks allowed
    pub parallel_limit: Option<usize>,
    /// Interval for saving execution state checkpoints
    pub checkpoint_interval: Option<Duration>,
    /// Whether to enable result caching
    pub enable_caching: bool,
    /// Execution mode (Sync/Async)
    pub execution_mode: ExecutionMode,
    /// Concurrency settings
    pub concurrency_config: ConcurrencyConfig,
    /// Additional metadata
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
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
            max_memory_bytes: Some(1024 * 1024 * 1024),         // 1GB
            max_cpu_time: Some(Duration::from_secs(300)),       // 5 minutes
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

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub active_workflows: u32,
    pub system_health: SystemHealth,
    pub uptime: Duration,
    pub network_status: NetworkStatus,
    pub disk_usage: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub process_count: u32,
    pub thread_count: u32,
    pub load_average: f64,
}

/// System health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy,
    Warning,
    Critical,
}

/// Network connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkStatus {
    Connected,
    Disconnected,
    Limited,
}

/// Comprehensive system health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthAssessment {
    pub overall_health: SystemHealth,
    pub overall_score: u8, // 0-100
    pub cpu_health_score: f64,
    pub memory_health_score: f64,
    pub disk_health_score: f64,
    pub network_health_score: f64,
    pub recommendations: Vec<String>,
    pub load_assessment: String,
    pub last_updated: DateTime<Utc>,
}

/// Connection status for external services
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

impl Default for SystemStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
            active_workflows: 0,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(0),
            network_status: NetworkStatus::Disconnected,
            disk_usage: 0.0,
            disk_total: 0,
            disk_used: 0,
            process_count: 0,
            thread_count: 0,
            load_average: 0.0,
        }
    }
}
