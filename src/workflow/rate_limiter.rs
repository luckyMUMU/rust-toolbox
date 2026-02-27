//! 限流器实现
//!
//! 使用令牌桶算法控制请求速率

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, trace, warn};

/// 限流器错误
#[derive(Debug, Clone)]
pub enum RateLimiterError {
    /// 超过速率限制
    RateLimitExceeded,
    /// 等待超时
    WaitTimeout,
    /// 配置错误
    InvalidConfig(String),
}

impl std::fmt::Display for RateLimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimiterError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            RateLimiterError::WaitTimeout => write!(f, "Wait for rate limit timeout"),
            RateLimiterError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for RateLimiterError {}

/// 限流器
///
/// 使用令牌桶算法控制请求速率
pub struct RateLimiter {
    /// 令牌桶
    tokens: Arc<Mutex<f64>>,
    /// 令牌生成速率（每秒）
    rate: f64,
    /// 桶容量
    capacity: f64,
    /// 上次更新时间
    last_update: Arc<Mutex<Instant>>,
    /// 名称（用于日志）
    name: String,
}

impl RateLimiter {
    /// 创建新的限流器
    ///
    /// # 参数
    /// - `rate`: 令牌生成速率（每秒）
    /// - `capacity`: 桶容量（最大令牌数）
    ///
    /// # 示例
    /// ```
    /// let limiter = RateLimiter::new("api", 100.0, 200.0);
    /// ```
    pub fn new(name: impl Into<String>, rate: f64, capacity: f64) -> Self {
        let name = name.into();

        if rate <= 0.0 {
            panic!("Rate must be positive");
        }
        if capacity <= 0.0 {
            panic!("Capacity must be positive");
        }

        debug!(
            "Creating rate limiter '{}' with rate: {}, capacity: {}",
            name, rate, capacity
        );

        Self {
            tokens: Arc::new(Mutex::new(capacity)),
            rate,
            capacity,
            last_update: Arc::new(Mutex::new(Instant::now())),
            name,
        }
    }

    /// 使用默认配置创建限流器
    ///
    /// 默认：rate = 100.0, capacity = 200.0
    pub fn with_defaults(name: impl Into<String>) -> Self {
        Self::new(name, 100.0, 200.0)
    }

    /// 获取限流器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取当前令牌数
    pub async fn current_tokens(&self) -> f64 {
        self.update_tokens().await;
        *self.tokens.lock().await
    }

    /// 更新令牌数量
    async fn update_tokens(&self) {
        let mut last_update = self.last_update.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_update).as_secs_f64();

        let mut tokens = self.tokens.lock().await;
        // 添加新产生的令牌，但不超过容量
        *tokens = (*tokens + elapsed * self.rate).min(self.capacity);
        *last_update = now;

        trace!(
            "Rate limiter '{}' updated tokens: {} (elapsed: {:.3}s)",
            self.name,
            *tokens,
            elapsed
        );
    }

    /// 尝试获取许可（非阻塞）
    ///
    /// # 参数
    /// - `tokens`: 需要获取的令牌数
    ///
    /// # 返回
    /// - `Ok(())`: 成功获取许可
    /// - `Err(RateLimiterError::RateLimitExceeded)`: 超过速率限制
    pub async fn acquire(&self, tokens: f64) -> Result<(), RateLimiterError> {
        if tokens <= 0.0 {
            return Ok(());
        }

        if tokens > self.capacity {
            return Err(RateLimiterError::InvalidConfig(format!(
                "Requested tokens ({}) exceeds capacity ({})",
                tokens, self.capacity
            )));
        }

        self.update_tokens().await;

        let mut current = self.tokens.lock().await;
        if *current >= tokens {
            *current -= tokens;
            trace!(
                "Rate limiter '{}' acquired {} tokens, remaining: {}",
                self.name,
                tokens,
                *current
            );
            Ok(())
        } else {
            debug!(
                "Rate limiter '{}' rate limit exceeded (requested: {}, available: {})",
                self.name, tokens, *current
            );
            Err(RateLimiterError::RateLimitExceeded)
        }
    }

    /// 尝试获取单个许可（非阻塞）
    pub async fn acquire_one(&self) -> Result<(), RateLimiterError> {
        self.acquire(1.0).await
    }

    /// 等待获取许可（阻塞）
    ///
    /// # 参数
    /// - `tokens`: 需要获取的令牌数
    ///
    /// 会一直等待直到获取到许可
    pub async fn acquire_with_wait(&self, tokens: f64) -> Result<(), RateLimiterError> {
        if tokens <= 0.0 {
            return Ok(());
        }

        if tokens > self.capacity {
            return Err(RateLimiterError::InvalidConfig(format!(
                "Requested tokens ({}) exceeds capacity ({})",
                tokens, self.capacity
            )));
        }

        loop {
            match self.acquire(tokens).await {
                Ok(()) => return Ok(()),
                Err(RateLimiterError::RateLimitExceeded) => {
                    // 计算需要等待的时间
                    let wait_time = self.calculate_wait_time(tokens).await;
                    trace!(
                        "Rate limiter '{}' waiting for {:.3}s",
                        self.name,
                        wait_time.as_secs_f64()
                    );
                    sleep(wait_time).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 等待获取许可（带超时）
    ///
    /// # 参数
    /// - `tokens`: 需要获取的令牌数
    /// - `timeout`: 最大等待时间
    pub async fn acquire_with_timeout(
        &self,
        tokens: f64,
        timeout: Duration,
    ) -> Result<(), RateLimiterError> {
        if tokens <= 0.0 {
            return Ok(());
        }

        let start = Instant::now();

        loop {
            match self.acquire(tokens).await {
                Ok(()) => return Ok(()),
                Err(RateLimiterError::RateLimitExceeded) => {
                    // 检查是否超时
                    if start.elapsed() >= timeout {
                        warn!(
                            "Rate limiter '{}' acquire timeout after {:?}",
                            self.name, timeout
                        );
                        return Err(RateLimiterError::WaitTimeout);
                    }

                    // 计算需要等待的时间
                    let wait_time = self.calculate_wait_time(tokens).await;
                    let remaining = timeout - start.elapsed();
                    let sleep_time = wait_time.min(remaining);

                    sleep(sleep_time).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 计算需要等待的时间
    async fn calculate_wait_time(&self, tokens: f64) -> Duration {
        let current = self.current_tokens().await;
        if current >= tokens {
            return Duration::from_secs(0);
        }

        let needed = tokens - current;
        let wait_secs = needed / self.rate;
        Duration::from_secs_f64(wait_secs.max(0.01)) // 至少等待 10ms
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("name", &self.name)
            .field("rate", &self.rate)
            .field("capacity", &self.capacity)
            .finish()
    }
}

/// 工作流限流配置
#[derive(Clone, Debug)]
pub struct WorkflowRateLimitConfig {
    /// 每秒最大执行数
    pub max_executions_per_second: u32,
    /// 并发执行限制
    pub max_concurrent: usize,
    /// 是否启用限流
    pub enabled: bool,
}

impl Default for WorkflowRateLimitConfig {
    fn default() -> Self {
        Self {
            max_executions_per_second: 100,
            max_concurrent: 10,
            enabled: true,
        }
    }
}

/// 工作流限流器管理器
pub struct WorkflowRateLimiter {
    /// 全局限流器
    global_limiter: RateLimiter,
    /// 按工作流类型的限流器
    workflow_limiters: std::collections::HashMap<String, RateLimiter>,
    /// 配置
    config: WorkflowRateLimitConfig,
}

impl WorkflowRateLimiter {
    /// 创建新的限流器管理器
    pub fn new(config: WorkflowRateLimitConfig) -> Self {
        let global_limiter = RateLimiter::new(
            "global",
            config.max_executions_per_second as f64,
            (config.max_executions_per_second * 2) as f64,
        );

        Self {
            global_limiter,
            workflow_limiters: std::collections::HashMap::new(),
            config,
        }
    }

    /// 获取全局限流器
    pub fn global(&self) -> &RateLimiter {
        &self.global_limiter
    }

    /// 获取或创建工作流限流器
    pub fn get_workflow_limiter(&mut self, workflow_id: &str) -> &RateLimiter {
        self.workflow_limiters
            .entry(workflow_id.to_string())
            .or_insert_with(|| {
                RateLimiter::new(
                    format!("workflow:{}", workflow_id),
                    self.config.max_executions_per_second as f64 / 10.0, // 每个工作流限制为全局的 1/10
                    self.config.max_executions_per_second as f64 / 5.0,
                )
            })
    }

    /// 检查是否启用限流
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new("test", 10.0, 10.0);

        // 应该能获取 10 个令牌
        for i in 0..10 {
            let result = limiter.acquire(1.0).await;
            assert!(result.is_ok(), "Failed at iteration {}", i);
        }

        // 第 11 个应该失败
        let result = limiter.acquire(1.0).await;
        assert!(matches!(result, Err(RateLimiterError::RateLimitExceeded)));
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new("test", 100.0, 10.0);

        // 消耗所有令牌
        for _ in 0..10 {
            limiter.acquire(1.0).await.unwrap();
        }

        // 立即再次获取应该失败
        assert!(limiter.acquire(1.0).await.is_err());

        // 等待 100ms，应该产生 10 个令牌
        sleep(Duration::from_millis(100)).await;

        // 现在应该能获取令牌
        assert!(limiter.acquire(1.0).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let limiter = RateLimiter::new("test", 100.0, 1.0);

        // 消耗唯一令牌
        limiter.acquire(1.0).await.unwrap();

        // 使用等待模式
        let start = Instant::now();
        let result = limiter.acquire_with_wait(1.0).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 应该等待了至少 10ms（产生 1 个令牌需要 10ms）
        assert!(elapsed >= Duration::from_millis(5));
    }

    #[tokio::test]
    async fn test_rate_limiter_timeout() {
        let limiter = RateLimiter::new("test", 1.0, 1.0);

        // 消耗唯一令牌
        limiter.acquire(1.0).await.unwrap();

        // 使用超时模式，超时 50ms
        let result = limiter
            .acquire_with_timeout(1.0, Duration::from_millis(50))
            .await;

        // 应该超时（需要 1000ms 才能产生 1 个令牌）
        assert!(matches!(result, Err(RateLimiterError::WaitTimeout)));
    }

    #[tokio::test]
    async fn test_rate_limiter_invalid_config() {
        let limiter = RateLimiter::new("test", 10.0, 5.0);

        // 请求超过容量的令牌数
        let result = limiter.acquire(10.0).await;
        assert!(matches!(result, Err(RateLimiterError::InvalidConfig(_))));
    }
}
