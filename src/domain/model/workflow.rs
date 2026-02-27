//! 领域模型 - 工作流相关

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_max_concurrent_steps")]
    pub max_concurrent_steps: usize,
    pub default_timeout: Option<u64>,
    #[serde(
        deserialize_with = "crate::core::deserialize_option_duration_from_secs",
        serialize_with = "crate::core::serialize_option_duration_as_secs",
        default
    )]
    pub checkpoint_interval: Option<Duration>,
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

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    None,
    FixedInterval,
    ExponentialBackoff,
    LinearBackoff,
    Custom(serde_json::Value),
}

/// 重试策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    #[serde(
        deserialize_with = "crate::core::deserialize_duration_from_secs",
        serialize_with = "crate::core::serialize_duration_as_secs"
    )]
    pub base_delay: Duration,
    #[serde(
        deserialize_with = "crate::core::deserialize_option_duration_from_secs",
        serialize_with = "crate::core::serialize_option_duration_as_secs",
        default
    )]
    pub max_delay: Option<Duration>,
    pub backoff_multiplier: f64,
    pub strategy: RetryStrategy,
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

/// 并发配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,
    pub max_concurrent_tasks: usize,
    pub task_queue_size: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 10,
            max_concurrent_tasks: 50,
            task_queue_size: 100,
        }
    }
}

/// 执行指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration: Option<Duration>,
    pub active_executions: usize,
    pub queued_executions: usize,
    pub total_capacity: usize,
    pub queue_capacity: usize,
    pub available_permits: usize,
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_time: Option<Duration>,
    pub max_execution_time: Option<Duration>,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
}
