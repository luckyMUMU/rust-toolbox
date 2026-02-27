//! WASM 资源限制实现
//!
//! 提供内存、CPU、执行时间等资源限制机制

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// WASM 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmResourceLimits {
    /// 最大内存使用量（字节）
    pub max_memory: u64,
    /// 最大堆内存（字节）
    pub max_heap_memory: u64,
    /// 最大栈大小（字节）
    pub max_stack_size: u64,
    /// 最大表大小
    pub max_table_size: u32,
    /// 最大执行时间
    pub max_execution_time: Duration,
    /// 最大燃料（指令计数）
    pub max_fuel: Option<u64>,
    /// 最大线程数
    pub max_threads: u32,
    /// 最大打开文件数
    pub max_open_files: u32,
    /// 最大网络连接数
    pub max_network_connections: u32,
    /// CPU 时间限制（毫秒）
    pub max_cpu_time_ms: u64,
    /// 输出缓冲区大小限制
    pub max_output_size: u64,
}

impl Default for WasmResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024,      // 64MB
            max_heap_memory: 32 * 1024 * 1024, // 32MB
            max_stack_size: 1024 * 1024,       // 1MB
            max_table_size: 10000,
            max_execution_time: Duration::from_secs(30),
            max_fuel: Some(1_000_000_000),
            max_threads: 1,
            max_open_files: 10,
            max_network_connections: 5,
            max_cpu_time_ms: 5000,
            max_output_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl WasmResourceLimits {
    /// 创建无限制配置（仅用于信任模块）
    pub fn unrestricted() -> Self {
        Self {
            max_memory: u64::MAX,
            max_heap_memory: u64::MAX,
            max_stack_size: u64::MAX,
            max_table_size: u32::MAX,
            max_execution_time: Duration::from_secs(u64::MAX),
            max_fuel: None,
            max_threads: u32::MAX,
            max_open_files: u32::MAX,
            max_network_connections: u32::MAX,
            max_cpu_time_ms: u64::MAX,
            max_output_size: u64::MAX,
        }
    }

    /// 创建严格限制配置
    pub fn strict() -> Self {
        Self {
            max_memory: 16 * 1024 * 1024,     // 16MB
            max_heap_memory: 8 * 1024 * 1024, // 8MB
            max_stack_size: 256 * 1024,       // 256KB
            max_table_size: 1000,
            max_execution_time: Duration::from_secs(10),
            max_fuel: Some(100_000_000),
            max_threads: 1,
            max_open_files: 5,
            max_network_connections: 0,
            max_cpu_time_ms: 2000,
            max_output_size: 1024 * 1024, // 1MB
        }
    }

    /// 创建高性能配置
    pub fn high_performance() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024,      // 256MB
            max_heap_memory: 128 * 1024 * 1024, // 128MB
            max_stack_size: 4 * 1024 * 1024,    // 4MB
            max_table_size: 100000,
            max_execution_time: Duration::from_secs(300),
            max_fuel: Some(10_000_000_000),
            max_threads: 4,
            max_open_files: 100,
            max_network_connections: 50,
            max_cpu_time_ms: 60000,
            max_output_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// 资源使用统计
#[derive(Debug, Default)]
pub struct ResourceUsage {
    /// 已使用内存
    pub memory_used: AtomicU64,
    /// 已使用堆内存
    pub heap_memory_used: AtomicU64,
    /// 已使用燃料
    pub fuel_used: AtomicU64,
    /// 已执行时间（毫秒）
    pub execution_time_ms: AtomicU64,
    /// 已打开文件数
    pub open_files: AtomicU32,
    /// 已建立网络连接数
    pub network_connections: AtomicU32,
}

impl ResourceUsage {
    /// 创建新的资源使用统计
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置统计
    pub fn reset(&self) {
        self.memory_used.store(0, Ordering::SeqCst);
        self.heap_memory_used.store(0, Ordering::SeqCst);
        self.fuel_used.store(0, Ordering::SeqCst);
        self.execution_time_ms.store(0, Ordering::SeqCst);
        self.open_files.store(0, Ordering::SeqCst);
        self.network_connections.store(0, Ordering::SeqCst);
    }

    /// 获取快照
    pub fn snapshot(&self) -> ResourceUsageSnapshot {
        ResourceUsageSnapshot {
            memory_used: self.memory_used.load(Ordering::SeqCst),
            heap_memory_used: self.heap_memory_used.load(Ordering::SeqCst),
            fuel_used: self.fuel_used.load(Ordering::SeqCst),
            execution_time_ms: self.execution_time_ms.load(Ordering::SeqCst),
            open_files: self.open_files.load(Ordering::SeqCst),
            network_connections: self.network_connections.load(Ordering::SeqCst),
        }
    }
}

/// 资源使用快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    pub memory_used: u64,
    pub heap_memory_used: u64,
    pub fuel_used: u64,
    pub execution_time_ms: u64,
    pub open_files: u32,
    pub network_connections: u32,
}

/// 资源限制错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum ResourceLimitError {
    #[error("内存使用超出限制: 已使用 {used} 字节，限制 {limit} 字节")]
    MemoryExceeded { used: u64, limit: u64 },

    #[error("执行时间超出限制: 已执行 {used} 毫秒，限制 {limit} 毫秒")]
    ExecutionTimeExceeded { used: u64, limit: u64 },

    #[error("燃料耗尽: 已使用 {used}，限制 {limit}")]
    FuelExhausted { used: u64, limit: u64 },

    #[error("打开文件数超出限制: 已打开 {used}，限制 {limit}")]
    OpenFilesExceeded { used: u32, limit: u32 },

    #[error("网络连接数超出限制: 已建立 {used}，限制 {limit}")]
    NetworkConnectionsExceeded { used: u32, limit: u32 },

    #[error("输出大小超出限制: 已输出 {used} 字节，限制 {limit} 字节")]
    OutputSizeExceeded { used: u64, limit: u64 },
}

/// 资源限制监控器
pub struct ResourceLimiter {
    limits: WasmResourceLimits,
    usage: Arc<ResourceUsage>,
    start_time: Option<Instant>,
}

impl ResourceLimiter {
    /// 创建新的资源限制监控器
    pub fn new(limits: WasmResourceLimits) -> Self {
        Self {
            limits,
            usage: Arc::new(ResourceUsage::new()),
            start_time: None,
        }
    }

    /// 获取资源限制配置
    pub fn limits(&self) -> &WasmResourceLimits {
        &self.limits
    }

    /// 获取资源使用统计
    pub fn usage(&self) -> &Arc<ResourceUsage> {
        &self.usage
    }

    /// 开始执行
    pub fn start_execution(&mut self) {
        self.usage.reset();
        self.start_time = Some(Instant::now());
        debug!("WASM 执行开始，资源限制: {:?}", self.limits);
    }

    /// 结束执行
    pub fn end_execution(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_millis() as u64;
            self.usage
                .execution_time_ms
                .store(elapsed, Ordering::SeqCst);
            debug!("WASM 执行结束，耗时: {}ms", elapsed);
        }
        self.start_time = None;
    }

    /// 检查内存限制
    pub fn check_memory(&self, requested: u64) -> Result<(), ResourceLimitError> {
        let current = self.usage.memory_used.load(Ordering::SeqCst);
        let new_total = current.saturating_add(requested);

        if new_total > self.limits.max_memory {
            warn!("内存限制超出: {} > {}", new_total, self.limits.max_memory);
            return Err(ResourceLimitError::MemoryExceeded {
                used: new_total,
                limit: self.limits.max_memory,
            });
        }

        self.usage.memory_used.store(new_total, Ordering::SeqCst);
        Ok(())
    }

    /// 检查执行时间
    pub fn check_execution_time(&self) -> Result<(), ResourceLimitError> {
        if let Some(start) = self.start_time {
            let elapsed_ms = start.elapsed().as_millis() as u64;

            if elapsed_ms > self.limits.max_execution_time.as_millis() as u64 {
                warn!("执行时间超出限制: {}ms", elapsed_ms);
                return Err(ResourceLimitError::ExecutionTimeExceeded {
                    used: elapsed_ms,
                    limit: self.limits.max_execution_time.as_millis() as u64,
                });
            }
        }
        Ok(())
    }

    /// 检查燃料限制
    pub fn check_fuel(&self, consumed: u64) -> Result<(), ResourceLimitError> {
        if let Some(max_fuel) = self.limits.max_fuel {
            let current = self.usage.fuel_used.load(Ordering::SeqCst);
            let new_total = current.saturating_add(consumed);

            if new_total > max_fuel {
                warn!("燃料耗尽: {} > {}", new_total, max_fuel);
                return Err(ResourceLimitError::FuelExhausted {
                    used: new_total,
                    limit: max_fuel,
                });
            }

            self.usage.fuel_used.store(new_total, Ordering::SeqCst);
        }
        Ok(())
    }

    /// 检查文件打开
    pub fn check_open_file(&self) -> Result<(), ResourceLimitError> {
        let current = self.usage.open_files.load(Ordering::SeqCst);

        if current >= self.limits.max_open_files {
            warn!("打开文件数超出限制: {}", current);
            return Err(ResourceLimitError::OpenFilesExceeded {
                used: current,
                limit: self.limits.max_open_files,
            });
        }

        self.usage.open_files.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// 释放文件
    pub fn release_file(&self) {
        self.usage.open_files.fetch_sub(1, Ordering::SeqCst);
    }

    /// 检查网络连接
    pub fn check_network_connection(&self) -> Result<(), ResourceLimitError> {
        let current = self.usage.network_connections.load(Ordering::SeqCst);

        if current >= self.limits.max_network_connections {
            warn!("网络连接数超出限制: {}", current);
            return Err(ResourceLimitError::NetworkConnectionsExceeded {
                used: current,
                limit: self.limits.max_network_connections,
            });
        }

        self.usage
            .network_connections
            .fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// 释放网络连接
    pub fn release_network_connection(&self) {
        self.usage
            .network_connections
            .fetch_sub(1, Ordering::SeqCst);
    }

    /// 检查输出大小
    pub fn check_output_size(&self, size: u64) -> Result<(), ResourceLimitError> {
        if size > self.limits.max_output_size {
            warn!(
                "输出大小超出限制: {} > {}",
                size, self.limits.max_output_size
            );
            return Err(ResourceLimitError::OutputSizeExceeded {
                used: size,
                limit: self.limits.max_output_size,
            });
        }
        Ok(())
    }

    /// 获取资源使用报告
    pub fn report(&self) -> ResourceReport {
        let snapshot = self.usage.snapshot();
        let snapshot_for_utilization = snapshot.clone();
        ResourceReport {
            limits: self.limits.clone(),
            usage: snapshot,
            utilization: ResourceUtilization {
                memory_percent: if self.limits.max_memory > 0 {
                    (snapshot_for_utilization.memory_used as f64 / self.limits.max_memory as f64)
                        * 100.0
                } else {
                    0.0
                },
                fuel_percent: if let Some(max_fuel) = self.limits.max_fuel {
                    if max_fuel > 0 {
                        (snapshot_for_utilization.fuel_used as f64 / max_fuel as f64) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                },
                time_percent: if self.limits.max_execution_time.as_millis() > 0 {
                    (snapshot_for_utilization.execution_time_ms as f64
                        / self.limits.max_execution_time.as_millis() as f64)
                        * 100.0
                } else {
                    0.0
                },
            },
        }
    }
}

/// 资源使用报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReport {
    pub limits: WasmResourceLimits,
    pub usage: ResourceUsageSnapshot,
    pub utilization: ResourceUtilization,
}

/// 资源利用率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub memory_percent: f64,
    pub fuel_percent: f64,
    pub time_percent: f64,
}

/// 资源限制构建器
pub struct ResourceLimiterBuilder {
    limits: WasmResourceLimits,
}

impl ResourceLimiterBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            limits: WasmResourceLimits::default(),
        }
    }

    /// 设置最大内存
    pub fn max_memory(mut self, bytes: u64) -> Self {
        self.limits.max_memory = bytes;
        self
    }

    /// 设置最大堆内存
    pub fn max_heap_memory(mut self, bytes: u64) -> Self {
        self.limits.max_heap_memory = bytes;
        self
    }

    /// 设置最大执行时间
    pub fn max_execution_time(mut self, duration: Duration) -> Self {
        self.limits.max_execution_time = duration;
        self
    }

    /// 设置最大燃料
    pub fn max_fuel(mut self, fuel: u64) -> Self {
        self.limits.max_fuel = Some(fuel);
        self
    }

    /// 设置最大打开文件数
    pub fn max_open_files(mut self, count: u32) -> Self {
        self.limits.max_open_files = count;
        self
    }

    /// 设置最大网络连接数
    pub fn max_network_connections(mut self, count: u32) -> Self {
        self.limits.max_network_connections = count;
        self
    }

    /// 设置最大输出大小
    pub fn max_output_size(mut self, bytes: u64) -> Self {
        self.limits.max_output_size = bytes;
        self
    }

    /// 使用预设配置
    pub fn strict(mut self) -> Self {
        self.limits = WasmResourceLimits::strict();
        self
    }

    /// 使用高性能配置
    pub fn high_performance(mut self) -> Self {
        self.limits = WasmResourceLimits::high_performance();
        self
    }

    /// 构建资源限制器
    pub fn build(self) -> ResourceLimiter {
        ResourceLimiter::new(self.limits)
    }
}

impl Default for ResourceLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = WasmResourceLimits::default();
        assert_eq!(limits.max_memory, 64 * 1024 * 1024);
        assert_eq!(limits.max_fuel, Some(1_000_000_000));
    }

    #[test]
    fn test_resource_limits_strict() {
        let limits = WasmResourceLimits::strict();
        assert_eq!(limits.max_memory, 16 * 1024 * 1024);
        assert_eq!(limits.max_network_connections, 0);
    }

    #[test]
    fn test_resource_limiter_memory() {
        let mut limiter = ResourceLimiter::new(WasmResourceLimits::default());
        limiter.start_execution();

        assert!(limiter.check_memory(1024).is_ok());
        assert!(limiter.check_memory(100 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_resource_limiter_fuel() {
        let limiter = ResourceLimiter::new(WasmResourceLimits::strict());

        assert!(limiter.check_fuel(50_000_000).is_ok());
        assert!(limiter.check_fuel(100_000_000).is_err());
    }

    #[test]
    fn test_resource_limiter_builder() {
        let limiter = ResourceLimiterBuilder::new()
            .max_memory(32 * 1024 * 1024)
            .max_fuel(500_000_000)
            .build();

        assert_eq!(limiter.limits().max_memory, 32 * 1024 * 1024);
        assert_eq!(limiter.limits().max_fuel, Some(500_000_000));
    }

    #[test]
    fn test_resource_report() {
        let mut limiter = ResourceLimiter::new(WasmResourceLimits::default());
        limiter.start_execution();
        limiter.check_memory(1024 * 1024).unwrap();

        let report = limiter.report();
        assert!(report.usage.memory_used > 0);
        assert!(report.utilization.memory_percent > 0.0);
    }
}
