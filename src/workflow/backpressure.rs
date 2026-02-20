//! 背压机制
//!
//! 提供工作流执行时的流量控制和过载保护

use crate::error::{Result, WorkflowError};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

/// 背压策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureStrategy {
    /// 拒绝新请求
    Reject,
    /// 等待队列有空位
    Wait,
    /// 丢弃最旧的请求
    DropOldest,
    /// 丢弃最新的请求
    DropNewest,
}

/// 背压控制器配置
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// 最大并发数
    pub max_concurrent: usize,
    /// 最大队列长度
    pub max_queue_size: usize,
    /// 等待超时
    pub wait_timeout: Duration,
    /// 背压策略
    pub strategy: BackpressureStrategy,
    /// 高水位线（触发背压的阈值百分比）
    pub high_watermark: f32,
    /// 低水位线（解除背压的阈值百分比）
    pub low_watermark: f32,
    /// 采样窗口大小
    pub sample_window_size: usize,
    /// 自适应调整间隔
    pub adaptive_interval: Duration,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 100,
            max_queue_size: 1000,
            wait_timeout: Duration::from_secs(30),
            strategy: BackpressureStrategy::Wait,
            high_watermark: 0.8,
            low_watermark: 0.5,
            sample_window_size: 100,
            adaptive_interval: Duration::from_secs(10),
        }
    }
}

/// 背压状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureState {
    /// 正常状态
    Normal,
    /// 警告状态（接近阈值）
    Warning,
    /// 背压激活
    Active,
    /// 拒绝状态
    Rejecting,
}

/// 背压统计
#[derive(Debug, Clone, Default)]
pub struct BackpressureStats {
    /// 当前并发数
    pub current_concurrent: usize,
    /// 当前队列长度
    pub current_queue_size: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 被拒绝的请求数
    pub rejected_requests: u64,
    /// 被丢弃的请求数
    pub dropped_requests: u64,
    /// 超时请求数
    pub timed_out_requests: u64,
    /// 平均等待时间
    pub avg_wait_time: Duration,
    /// 当前状态
    pub state: BackpressureState,
}

/// 请求优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急优先级
    Critical = 3,
}

/// 排队请求
struct QueuedRequest {
    priority: RequestPriority,
    timestamp: Instant,
    id: u64,
}

/// 背压控制器
pub struct BackpressureController {
    /// 配置
    config: BackpressureConfig,
    /// 并发信号量
    semaphore: Arc<Semaphore>,
    /// 当前队列长度
    queue_size: AtomicUsize,
    /// 当前并发数
    concurrent_count: AtomicUsize,
    /// 总请求数
    total_requests: AtomicU64,
    /// 被拒绝请求数
    rejected_requests: AtomicU64,
    /// 被丢弃请求数
    dropped_requests: AtomicU64,
    /// 超时请求数
    timed_out_requests: AtomicU64,
    /// 当前状态
    state: std::sync::atomic::AtomicU8,
    /// 是否启用
    enabled: AtomicBool,
    /// 请求队列
    queue: Mutex<Vec<QueuedRequest>>,
    /// 请求 ID 计数器
    request_id_counter: AtomicU64,
}

impl BackpressureController {
    /// 创建新的背压控制器
    pub fn new(config: BackpressureConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        
        Self {
            semaphore,
            queue_size: AtomicUsize::new(0),
            concurrent_count: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            rejected_requests: AtomicU64::new(0),
            dropped_requests: AtomicU64::new(0),
            timed_out_requests: AtomicU64::new(0),
            state: std::sync::atomic::AtomicU8::new(BackpressureState::Normal as u8),
            enabled: AtomicBool::new(true),
            queue: Mutex::new(Vec::new()),
            request_id_counter: AtomicU64::new(0),
            config,
        }
    }

    /// 使用默认配置创建控制器
    pub fn with_defaults() -> Self {
        Self::new(BackpressureConfig::default())
    }

    /// 尝试获取执行许可
    pub async fn acquire(&self) -> Result<BackpressurePermit> {
        self.acquire_with_priority(RequestPriority::Normal).await
    }

    /// 使用指定优先级获取执行许可
    pub async fn acquire_with_priority(&self, priority: RequestPriority) -> Result<BackpressurePermit> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(BackpressurePermit {
                controller: self,
                _permit: None,
            });
        }

        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        let current_state = self.get_state();
        
        if current_state == BackpressureState::Rejecting {
            self.rejected_requests.fetch_add(1, Ordering::Relaxed);
            return Err(WorkflowError::ResourceExhausted);
        }

        match self.config.strategy {
            BackpressureStrategy::Reject => {
                if current_state == BackpressureState::Active {
                    self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                    return Err(WorkflowError::ResourceExhausted);
                }
                
                let permit = self.semaphore.try_acquire()
                    .map_err(|_| {
                        self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                        WorkflowError::ResourceExhausted
                    })?;
                
                self.concurrent_count.fetch_add(1, Ordering::Relaxed);
                self.update_state();
                
                Ok(BackpressurePermit {
                    controller: self,
                    _permit: Some(permit),
                })
            }
            
            BackpressureStrategy::Wait => {
                let result = tokio::time::timeout(
                    self.config.wait_timeout,
                    self.semaphore.acquire()
                ).await;
                
                match result {
                    Ok(Ok(permit)) => {
                        self.concurrent_count.fetch_add(1, Ordering::Relaxed);
                        self.update_state();
                        
                        Ok(BackpressurePermit {
                            controller: self,
                            _permit: Some(permit),
                        })
                    }
                    Ok(Err(_)) => {
                        self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                        Err(WorkflowError::ResourceExhausted)
                    }
                    Err(_) => {
                        self.timed_out_requests.fetch_add(1, Ordering::Relaxed);
                        Err(WorkflowError::execution(format!(
                            "获取执行许可超时（{}秒）",
                            self.config.wait_timeout.as_secs()
                        )))
                    }
                }
            }
            
            BackpressureStrategy::DropOldest => {
                self.enqueue_request(priority).await?;
                self.try_acquire_with_drop_oldest().await
            }
            
            BackpressureStrategy::DropNewest => {
                if current_state == BackpressureState::Active {
                    self.dropped_requests.fetch_add(1, Ordering::Relaxed);
                    return Err(WorkflowError::ResourceExhausted);
                }
                
                let permit = self.semaphore.acquire().await
                    .map_err(|_| WorkflowError::ResourceExhausted)?;
                
                self.concurrent_count.fetch_add(1, Ordering::Relaxed);
                self.update_state();
                
                Ok(BackpressurePermit {
                    controller: self,
                    _permit: Some(permit),
                })
            }
        }
    }

    /// 尝试非阻塞获取许可
    pub fn try_acquire(&self) -> Result<BackpressurePermit> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(BackpressurePermit {
                controller: self,
                _permit: None,
            });
        }

        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        let permit = self.semaphore.try_acquire()
            .map_err(|_| {
                self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                WorkflowError::ResourceExhausted
            })?;
        
        self.concurrent_count.fetch_add(1, Ordering::Relaxed);
        self.update_state();
        
        Ok(BackpressurePermit {
            controller: self,
            _permit: Some(permit),
        })
    }

    /// 释放许可
    fn release(&self) {
        self.concurrent_count.fetch_sub(1, Ordering::Relaxed);
        self.update_state();
    }

    /// 入队请求
    async fn enqueue_request(&self, priority: RequestPriority) -> Result<()> {
        let mut queue = self.queue.lock().await;
        
        if queue.len() >= self.config.max_queue_size {
            return Err(WorkflowError::ResourceExhausted);
        }
        
        let request_id = self.request_id_counter.fetch_add(1, Ordering::Relaxed);
        
        queue.push(QueuedRequest {
            priority,
            timestamp: Instant::now(),
            id: request_id,
        });
        
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        self.queue_size.store(queue.len(), Ordering::Relaxed);
        
        Ok(())
    }

    /// 尝试获取并丢弃最旧请求
    async fn try_acquire_with_drop_oldest(&self) -> Result<BackpressurePermit> {
        let mut queue = self.queue.lock().await;
        
        while queue.len() > self.config.max_queue_size {
            queue.pop();
            self.dropped_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        drop(queue);
        
        let result = tokio::time::timeout(
            self.config.wait_timeout,
            self.semaphore.acquire()
        ).await;
        
        match result {
            Ok(Ok(permit)) => {
                self.concurrent_count.fetch_add(1, Ordering::Relaxed);
                self.update_state();
                
                Ok(BackpressurePermit {
                    controller: self,
                    _permit: Some(permit),
                })
            }
            _ => {
                self.timed_out_requests.fetch_add(1, Ordering::Relaxed);
                Err(WorkflowError::execution("获取执行许可超时"))
            }
        }
    }

    /// 更新状态
    fn update_state(&self) {
        let concurrent = self.concurrent_count.load(Ordering::Relaxed);
        let ratio = concurrent as f32 / self.config.max_concurrent as f32;
        
        let new_state = if ratio >= 1.0 {
            BackpressureState::Rejecting
        } else if ratio >= self.config.high_watermark {
            BackpressureState::Active
        } else if ratio >= self.config.low_watermark {
            BackpressureState::Warning
        } else {
            BackpressureState::Normal
        };
        
        self.state.store(new_state as u8, Ordering::Relaxed);
        
        if new_state != BackpressureState::Normal {
            debug!(
                state = ?new_state,
                concurrent = concurrent,
                ratio = ratio,
                "背压状态更新"
            );
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> BackpressureState {
        match self.state.load(Ordering::Relaxed) {
            0 => BackpressureState::Normal,
            1 => BackpressureState::Warning,
            2 => BackpressureState::Active,
            3 => BackpressureState::Rejecting,
            _ => BackpressureState::Normal,
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> BackpressureStats {
        BackpressureStats {
            current_concurrent: self.concurrent_count.load(Ordering::Relaxed),
            current_queue_size: self.queue_size.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            dropped_requests: self.dropped_requests.load(Ordering::Relaxed),
            timed_out_requests: self.timed_out_requests.load(Ordering::Relaxed),
            avg_wait_time: Duration::from_millis(0),
            state: self.get_state(),
        }
    }

    /// 启用背压控制
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
        info!("背压控制已启用");
    }

    /// 禁用背压控制
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        info!("背压控制已禁用");
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.rejected_requests.store(0, Ordering::Relaxed);
        self.dropped_requests.store(0, Ordering::Relaxed);
        self.timed_out_requests.store(0, Ordering::Relaxed);
    }
}

/// 背压许可
pub struct BackpressurePermit<'a> {
    controller: &'a BackpressureController,
    _permit: Option<tokio::sync::SemaphorePermit<'a>>,
}

impl Drop for BackpressurePermit<'_> {
    fn drop(&mut self) {
        if self._permit.is_some() {
            self.controller.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.max_concurrent, 100);
        assert_eq!(config.strategy, BackpressureStrategy::Wait);
    }

    #[tokio::test]
    async fn test_backpressure_controller_creation() {
        let controller = BackpressureController::with_defaults();
        assert!(controller.is_enabled());
        assert_eq!(controller.get_state(), BackpressureState::Normal);
    }

    #[tokio::test]
    async fn test_acquire_release() {
        let config = BackpressureConfig {
            max_concurrent: 2,
            ..Default::default()
        };
        let controller = BackpressureController::new(config);
        
        {
            let _permit1 = controller.acquire().await.unwrap();
            let _permit2 = controller.acquire().await.unwrap();
            
            let stats = controller.stats();
            assert_eq!(stats.current_concurrent, 2);
        }
        
        let stats = controller.stats();
        assert_eq!(stats.current_concurrent, 0);
    }

    #[test]
    fn test_backpressure_state() {
        let controller = BackpressureController::with_defaults();
        assert_eq!(controller.get_state(), BackpressureState::Normal);
    }

    #[tokio::test]
    async fn test_enable_disable() {
        let controller = BackpressureController::with_defaults();
        
        controller.disable();
        assert!(!controller.is_enabled());
        
        controller.enable();
        assert!(controller.is_enabled());
    }

    #[test]
    fn test_request_priority_ordering() {
        assert!(RequestPriority::Critical > RequestPriority::High);
        assert!(RequestPriority::High > RequestPriority::Normal);
        assert!(RequestPriority::Normal > RequestPriority::Low);
    }
}
