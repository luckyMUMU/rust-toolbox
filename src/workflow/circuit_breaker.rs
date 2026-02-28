//! 熔断器实现
//!
//! 提供熔断保护机制，防止级联故障

use chrono::{DateTime, Utc};
use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 熔断器配置
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（触发熔断的失败次数）
    pub failure_threshold: u32,
    /// 成功阈值（恢复关闭状态的成功次数）
    pub success_threshold: u32,
    /// 熔断超时时间
    pub timeout: Duration,
    /// 半开状态测试请求数
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

/// 熔断器状态
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircuitState {
    /// 关闭状态 - 正常处理请求
    Closed,
    /// 打开状态 - 拒绝请求，直接失败
    Open { opened_at: DateTime<Utc> },
    /// 半开状态 - 允许部分请求测试恢复
    HalfOpen { test_calls: u32 },
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open { opened_at } => write!(f, "Open (since {})", opened_at),
            CircuitState::HalfOpen { test_calls } => {
                write!(f, "HalfOpen (test_calls: {})", test_calls)
            }
        }
    }
}

/// 熔断器指标
pub struct CircuitBreakerMetrics {
    success_count: AtomicU32,
    failure_count: AtomicU32,
    /// 连续失败次数（成功后重置）
    consecutive_failures: AtomicU32,
    state_change_count: AtomicU32,
}

impl CircuitBreakerMetrics {
    pub fn new() -> Self {
        Self {
            success_count: AtomicU32::new(0),
            failure_count: AtomicU32::new(0),
            consecutive_failures: AtomicU32::new(0),
            state_change_count: AtomicU32::new(0),
        }
    }

    /// 记录成功，同时重置连续失败计数
    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }

    /// 记录失败，同时增加连续失败计数
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_state_change(&self) {
        self.state_change_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn success_count(&self) -> u32 {
        self.success_count.load(Ordering::Relaxed)
    }

    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Relaxed)
    }

    pub fn state_change_count(&self) -> u32 {
        self.state_change_count.load(Ordering::Relaxed)
    }

    /// 获取当前连续失败次数（用于判断是否达到熔断阈值）
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// 重置连续失败计数
    pub fn reset_consecutive_failures(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
    }
}

impl Default for CircuitBreakerMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 熔断器错误
#[derive(Debug, Clone)]
pub enum CircuitBreakerError {
    /// 熔断器打开，拒绝请求
    CircuitOpen,
    /// 操作执行失败
    OperationFailed(String),
    /// 配置错误
    InvalidConfig(String),
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            CircuitBreakerError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for CircuitBreakerError {}

/// 熔断器
///
/// 防止级联故障，保护系统稳定性
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    metrics: Arc<CircuitBreakerMetrics>,
    name: String,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let name = name.into();
        info!(
            "Creating circuit breaker '{}' with config: {:?}",
            name, config
        );

        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            metrics: Arc::new(CircuitBreakerMetrics::new()),
            name,
        }
    }

    /// 使用默认配置创建熔断器
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// 获取熔断器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取当前状态
    pub async fn current_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// 获取指标
    pub fn metrics(&self) -> &CircuitBreakerMetrics {
        &self.metrics
    }

    /// 执行受保护的操作
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        // 检查当前状态
        self.check_state().await?;

        // 执行操作
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                let error_msg = error.to_string();
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error_msg))
            }
        }
    }

    /// 检查当前状态，决定是否允许执行
    async fn check_state(&self) -> Result<(), CircuitBreakerError> {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => {
                // 关闭状态，允许执行
                Ok(())
            }
            CircuitState::Open { opened_at } => {
                // 检查是否到达恢复时间
                let elapsed = Utc::now() - opened_at;
                if elapsed > chrono::Duration::from_std(self.config.timeout).unwrap() {
                    // 切换到半开状态
                    info!(
                        "Circuit breaker '{}' transitioning from Open to HalfOpen",
                        self.name
                    );
                    *state = CircuitState::HalfOpen { test_calls: 0 };
                    self.metrics.record_state_change();
                    Ok(())
                } else {
                    // 仍在熔断期，拒绝请求
                    debug!(
                        "Circuit breaker '{}' is open, rejecting request (elapsed: {:?})",
                        self.name, elapsed
                    );
                    Err(CircuitBreakerError::CircuitOpen)
                }
            }
            CircuitState::HalfOpen { test_calls } => {
                // 检查是否超过测试请求数
                if test_calls >= self.config.half_open_max_calls {
                    debug!(
                        "Circuit breaker '{}' HalfOpen limit reached, rejecting request",
                        self.name
                    );
                    Err(CircuitBreakerError::CircuitOpen)
                } else {
                    // 增加测试调用计数
                    *state = CircuitState::HalfOpen {
                        test_calls: test_calls + 1,
                    };
                    Ok(())
                }
            }
        }
    }

    /// 成功回调
    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::HalfOpen { test_calls: _ } => {
                // 半开状态下，检查是否达到成功阈值
                self.metrics.record_success();
                let success_count = self.metrics.success_count();

                if success_count >= self.config.success_threshold {
                    info!(
                        "Circuit breaker '{}' transitioning from HalfOpen to Closed (success: {})",
                        self.name, success_count
                    );
                    *state = CircuitState::Closed;
                    self.metrics.record_state_change();
                    // 重置计数
                    self.reset_counts().await;
                }
                // 否则保持在半开状态
            }
            CircuitState::Closed => {
                // 关闭状态下，记录成功
                self.metrics.record_success();
            }
            _ => {}
        }
    }

    /// 失败回调
    async fn on_failure(&self) {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => {
                // 关闭状态下，检查是否达到失败阈值
                self.metrics.record_failure();
                let failure_count = self.metrics.consecutive_failures();

                if failure_count >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker '{}' transitioning from Closed to Open (failures: {})",
                        self.name, failure_count
                    );
                    *state = CircuitState::Open {
                        opened_at: Utc::now(),
                    };
                    self.metrics.record_state_change();
                }
            }
            CircuitState::HalfOpen { .. } => {
                // 半开状态下失败，立即熔断
                error!(
                    "Circuit breaker '{}' transitioning from HalfOpen to Open (test failed)",
                    self.name
                );
                *state = CircuitState::Open {
                    opened_at: Utc::now(),
                };
                self.metrics.record_state_change();
            }
            _ => {}
        }
    }

    /// 重置计数
    async fn reset_counts(&self) {
        self.metrics.reset_consecutive_failures();
    }

    /// 手动重置熔断器到关闭状态
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        info!(
            "Manually resetting circuit breaker '{}' to Closed",
            self.name
        );
        *state = CircuitState::Closed;
        self.metrics.record_state_change();
    }

    /// 强制打开熔断器
    pub async fn force_open(&self) {
        let mut state = self.state.write().await;
        warn!("Force opening circuit breaker '{}'", self.name);
        *state = CircuitState::Open {
            opened_at: Utc::now(),
        };
        self.metrics.record_state_change();
    }
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("name", &self.name)
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::with_defaults("test");

        // 初始状态应该是关闭
        assert_eq!(cb.current_state().await, CircuitState::Closed);

        // 成功执行应该正常工作
        let result = cb
            .call(|| async { Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42) })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 2,
        };
        let cb = CircuitBreaker::new("test", config);

        // 连续失败 3 次
        for _ in 0..3 {
            let result = cb
                .call(|| async {
                    Err::<i32, Box<dyn std::error::Error + Send + Sync>>("test error".into())
                })
                .await;
            assert!(result.is_err());
        }

        // 熔断器应该打开
        assert!(matches!(
            cb.current_state().await,
            CircuitState::Open { .. }
        ));

        // 再次请求应该被拒绝
        let result = cb
            .call(|| async { Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42) })
            .await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 1,
        };
        let cb = CircuitBreaker::new("test", config);

        // 触发熔断
        let _ = cb
            .call(|| async { Err::<i32, Box<dyn std::error::Error + Send + Sync>>("error".into()) })
            .await;

        assert!(matches!(
            cb.current_state().await,
            CircuitState::Open { .. }
        ));

        // 手动重置
        cb.reset().await;
        assert_eq!(cb.current_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_consecutive_failures_resets_on_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 2,
        };
        let cb = CircuitBreaker::new("test", config);

        // 连续失败 3 次
        for _ in 0..3 {
            let _ = cb
                .call(|| async {
                    Err::<i32, Box<dyn std::error::Error + Send + Sync>>("error".into())
                })
                .await;
        }

        // 验证连续失败计数为 3
        assert_eq!(cb.metrics().consecutive_failures(), 3);
        assert_eq!(cb.metrics().failure_count(), 3);

        // 成功一次
        let _ = cb
            .call(|| async { Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42) })
            .await;

        // 连续失败计数应该重置为 0，但总失败计数保持不变
        assert_eq!(cb.metrics().consecutive_failures(), 0);
        assert_eq!(cb.metrics().failure_count(), 3);
        assert_eq!(cb.metrics().success_count(), 1);

        // 熔断器应该仍然关闭（因为连续失败未达到阈值）
        assert_eq!(cb.current_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_consecutive_failures_triggers_circuit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_calls: 2,
        };
        let cb = CircuitBreaker::new("test", config);

        // 失败 2 次
        for _ in 0..2 {
            let _ = cb
                .call(|| async {
                    Err::<i32, Box<dyn std::error::Error + Send + Sync>>("error".into())
                })
                .await;
        }

        // 连续失败计数为 2，熔断器仍关闭
        assert_eq!(cb.metrics().consecutive_failures(), 2);
        assert_eq!(cb.current_state().await, CircuitState::Closed);

        // 成功一次，重置连续失败计数
        let _ = cb
            .call(|| async { Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42) })
            .await;
        assert_eq!(cb.metrics().consecutive_failures(), 0);

        // 再失败 3 次，触发熔断
        for _ in 0..3 {
            let _ = cb
                .call(|| async {
                    Err::<i32, Box<dyn std::error::Error + Send + Sync>>("error".into())
                })
                .await;
        }

        // 熔断器应该打开
        assert!(matches!(
            cb.current_state().await,
            CircuitState::Open { .. }
        ));
    }
}
