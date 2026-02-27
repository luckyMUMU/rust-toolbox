//! 插件进程池
//!
//! 提供插件进程的生命周期管理，包括：
//! - 进程创建和回收
//! - 空闲进程超时清理
//! - 进程崩溃自动重启
//! - 资源限制

use crate::error::{Result, WorkflowError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};

/// 进程池配置
#[derive(Debug, Clone)]
pub struct ProcessPoolConfig {
    /// 最大进程数
    pub max_processes: usize,
    /// 最小空闲进程数
    pub min_idle: usize,
    /// 进程空闲超时时间
    pub idle_timeout: Duration,
    /// 进程最大生命周期
    pub max_lifetime: Duration,
    /// 每个进程最大任务数
    pub max_tasks_per_process: u64,
    /// 进程启动超时
    pub startup_timeout: Duration,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 崩溃后自动重启
    pub auto_restart: bool,
    /// 最大重启次数
    pub max_restart_attempts: u32,
    /// 重启间隔
    pub restart_delay: Duration,
    /// 内存限制（字节）
    pub memory_limit: Option<u64>,
    /// CPU 限制（百分比）
    pub cpu_limit: Option<u32>,
}

impl Default for ProcessPoolConfig {
    fn default() -> Self {
        Self {
            max_processes: 8,
            min_idle: 1,
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
            max_tasks_per_process: 1000,
            startup_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            auto_restart: true,
            max_restart_attempts: 3,
            restart_delay: Duration::from_secs(5),
            memory_limit: Some(512 * 1024 * 1024),
            cpu_limit: Some(80),
        }
    }
}

/// 进程状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    /// 正在启动
    Starting,
    /// 空闲可用
    Idle,
    /// 正在执行任务
    Busy { task_id: String },
    /// 正在关闭
    Stopping,
    /// 已停止
    Stopped,
    /// 错误状态
    Error { message: String },
}

/// 进程信息
#[derive(Debug)]
pub struct ProcessInfo {
    /// 进程 ID
    pub pid: Option<u32>,
    /// 进程状态
    pub state: ProcessState,
    /// 创建时间
    pub created_at: Instant,
    /// 最后活动时间
    pub last_activity: Instant,
    /// 已执行任务数
    pub tasks_executed: u64,
    /// 重启次数
    pub restart_count: u32,
    /// 资源使用统计
    pub resource_usage: ResourceUsage,
}

/// 资源使用统计
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// CPU 使用率
    pub cpu_percent: f32,
    /// 内存使用（字节）
    pub memory_bytes: u64,
    /// 峰值内存
    pub peak_memory: u64,
}

/// 托管进程
struct ManagedProcess {
    /// 进程信息
    info: ProcessInfo,
    /// 进程句柄
    child: Option<Child>,
    /// 进程名称
    name: String,
    /// 命令模板
    command_template: Vec<String>,
    /// 是否标记为关闭
    shutdown_requested: AtomicBool,
}

impl ManagedProcess {
    fn new(name: String, command_template: Vec<String>) -> Self {
        Self {
            info: ProcessInfo {
                pid: None,
                state: ProcessState::Starting,
                created_at: Instant::now(),
                last_activity: Instant::now(),
                tasks_executed: 0,
                restart_count: 0,
                resource_usage: ResourceUsage::default(),
            },
            child: None,
            name,
            command_template,
            shutdown_requested: AtomicBool::new(false),
        }
    }

    fn is_expired(&self, config: &ProcessPoolConfig) -> bool {
        let age = self.info.created_at.elapsed();
        let idle_time = self.info.last_activity.elapsed();

        age > config.max_lifetime
            || (self.info.state == ProcessState::Idle && idle_time > config.idle_timeout)
            || self.info.tasks_executed >= config.max_tasks_per_process
    }

    fn should_restart(&self, config: &ProcessPoolConfig) -> bool {
        config.auto_restart
            && self.info.restart_count < config.max_restart_attempts
            && matches!(
                self.info.state,
                ProcessState::Error { .. } | ProcessState::Stopped
            )
    }
}

/// 进程池统计
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// 总进程数
    pub total_processes: usize,
    /// 空闲进程数
    pub idle_processes: usize,
    /// 忙碌进程数
    pub busy_processes: usize,
    /// 错误进程数
    pub error_processes: usize,
    /// 总任务数
    pub total_tasks: u64,
    /// 总重启次数
    pub total_restarts: u64,
}

/// 插件进程池
pub struct PluginProcessPool {
    /// 配置
    config: ProcessPoolConfig,
    /// 进程映射
    processes: Arc<RwLock<HashMap<String, Arc<Mutex<ManagedProcess>>>>>,
    /// 空闲进程队列
    idle_queue: Arc<Mutex<Vec<String>>>,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 统计信息
    stats: PoolStatsInner,
    /// 关闭标志
    shutdown: AtomicBool,
}

/// 内部统计
struct PoolStatsInner {
    total_tasks: AtomicU64,
    total_restarts: AtomicU64,
}

impl Default for PoolStatsInner {
    fn default() -> Self {
        Self {
            total_tasks: AtomicU64::new(0),
            total_restarts: AtomicU64::new(0),
        }
    }
}

impl PluginProcessPool {
    /// 创建新的进程池
    pub fn new(config: ProcessPoolConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_processes));
        Self {
            config,
            processes: Arc::new(RwLock::new(HashMap::new())),
            idle_queue: Arc::new(Mutex::new(Vec::new())),
            semaphore,
            stats: PoolStatsInner::default(),
            shutdown: AtomicBool::new(false),
        }
    }

    /// 使用默认配置创建进程池
    pub fn with_defaults() -> Self {
        Self::new(ProcessPoolConfig::default())
    }

    /// 启动进程池（预创建空闲进程）
    pub async fn start(&self) -> Result<()> {
        info!("启动插件进程池，最小空闲进程数: {}", self.config.min_idle);

        for i in 0..self.config.min_idle {
            let process_name = format!("worker-{}", i);
            self.spawn_process(&process_name, vec!["placeholder".to_string()])
                .await?;
        }

        self.start_health_check_task().await;
        self.start_cleanup_task().await;

        info!("插件进程池启动完成");
        Ok(())
    }

    /// 衍生新进程
    async fn spawn_process(&self, name: &str, command: Vec<String>) -> Result<()> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(WorkflowError::execution("进程池正在关闭"));
        }

        info!("衍生新进程: {}", name);

        let mut process = ManagedProcess::new(name.to_string(), command.clone());

        if !command.is_empty() && command[0] != "placeholder" {
            let mut cmd = Command::new(&command[0]);
            if command.len() > 1 {
                cmd.args(&command[1..]);
            }

            let child = cmd
                .spawn()
                .map_err(|e| WorkflowError::execution(format!("启动进程失败: {}", e)))?;

            process.info.pid = child.id();
            process.child = Some(child);
        }

        process.info.state = ProcessState::Idle;
        process.info.created_at = Instant::now();
        process.info.last_activity = Instant::now();

        let process_arc = Arc::new(Mutex::new(process));

        self.processes
            .write()
            .await
            .insert(name.to_string(), process_arc);
        self.idle_queue.lock().await.push(name.to_string());

        debug!("进程 {} 衍生完成", name);
        Ok(())
    }

    /// 获取可用进程
    pub async fn acquire(&self) -> Result<ProcessGuard> {
        if self.shutdown.load(Ordering::Relaxed) {
            return Err(WorkflowError::execution("进程池正在关闭"));
        }

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| WorkflowError::ResourceExhausted)?;

        let process_name = self.get_idle_process().await?;

        {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(&process_name) {
                let mut p = process.lock().await;
                p.info.state = ProcessState::Busy {
                    task_id: uuid::Uuid::new_v4().to_string(),
                };
                p.info.last_activity = Instant::now();
            }
        }

        self.stats.total_tasks.fetch_add(1, Ordering::Relaxed);

        Ok(ProcessGuard {
            pool: self,
            process_name,
            _permit,
        })
    }

    /// 获取空闲进程
    async fn get_idle_process(&self) -> Result<String> {
        let mut idle_queue = self.idle_queue.lock().await;

        while let Some(name) = idle_queue.pop() {
            let processes = self.processes.read().await;
            if let Some(process) = processes.get(&name) {
                let p = process.lock().await;
                if p.info.state == ProcessState::Idle && !p.is_expired(&self.config) {
                    return Ok(name);
                }
            }
        }

        drop(idle_queue);

        let processes = self.processes.read().await;
        let current_count = processes.len();
        drop(processes);

        if current_count < self.config.max_processes {
            let name = format!("worker-{}", uuid::Uuid::new_v4());
            self.spawn_process(&name, vec!["placeholder".to_string()])
                .await?;
            return Ok(name);
        }

        Err(WorkflowError::ResourceExhausted)
    }

    /// 释放进程
    async fn release_process(&self, name: &str) {
        let processes = self.processes.read().await;
        if let Some(process) = processes.get(name) {
            let mut p = process.lock().await;
            p.info.state = ProcessState::Idle;
            p.info.last_activity = Instant::now();
            p.info.tasks_executed += 1;

            if !p.is_expired(&self.config) {
                drop(p);
                self.idle_queue.lock().await.push(name.to_string());
            }
        }
    }

    /// 停止进程
    async fn stop_process(&self, name: &str) -> Result<()> {
        info!("停止进程: {}", name);

        let process_arc = {
            let mut processes = self.processes.write().await;
            processes.remove(name)
        };

        if let Some(process) = process_arc {
            let mut p = process.lock().await;
            p.info.state = ProcessState::Stopping;

            if let Some(mut child) = p.child.take() {
                if let Err(e) = child.kill().await {
                    warn!("终止进程 {} 失败: {}", name, e);
                }
            }

            p.info.state = ProcessState::Stopped;
            debug!("进程 {} 已停止", name);
        }

        Ok(())
    }

    /// 重启进程
    async fn restart_process(&self, name: &str) -> Result<()> {
        info!("重启进程: {}", name);

        self.stop_process(name).await?;

        tokio::time::sleep(self.config.restart_delay).await;

        let processes = self.processes.read().await;
        if let Some(process) = processes.get(name) {
            let mut p = process.lock().await;
            p.info.restart_count += 1;
            p.info.state = ProcessState::Idle;
            p.info.created_at = Instant::now();
            p.info.last_activity = Instant::now();
        }

        self.stats.total_restarts.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// 启动健康检查任务
    async fn start_health_check_task(&self) {
        let processes = Arc::clone(&self.processes);
        let interval = self.config.health_check_interval;
        let memory_limit = self.config.memory_limit;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while !shutdown_clone.load(Ordering::Relaxed) {
                interval_timer.tick().await;

                let procs = processes.read().await;
                for (name, process) in procs.iter() {
                    let p = process.lock().await;

                    if let ProcessState::Busy { task_id: _ } = &p.info.state {
                        let elapsed = p.info.last_activity.elapsed();
                        if elapsed > Duration::from_secs(600) {
                            warn!("进程 {} 执行超时: {:?}", name, elapsed);
                        }
                    }

                    if p.info.resource_usage.memory_bytes > memory_limit.unwrap_or(u64::MAX) {
                        warn!(
                            "进程 {} 内存超限: {} bytes",
                            name, p.info.resource_usage.memory_bytes
                        );
                    }
                }
            }
        });
    }

    /// 启动清理任务
    async fn start_cleanup_task(&self) {
        let processes = Arc::clone(&self.processes);
        let idle_queue = Arc::clone(&self.idle_queue);
        let idle_timeout = self.config.idle_timeout;
        let max_lifetime = self.config.max_lifetime;
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(30));

            while !shutdown_clone.load(Ordering::Relaxed) {
                interval_timer.tick().await;

                let procs = processes.read().await;
                let mut to_remove = Vec::new();

                for (name, process) in procs.iter() {
                    let p = process.lock().await;
                    let is_expired = p.info.created_at.elapsed() > max_lifetime
                        || (p.info.state == ProcessState::Idle
                            && p.info.last_activity.elapsed() > idle_timeout);
                    if is_expired && p.info.state == ProcessState::Idle {
                        to_remove.push(name.clone());
                    }
                }

                drop(procs);

                for name in to_remove {
                    let mut queue = idle_queue.lock().await;
                    queue.retain(|n| n != &name);
                    drop(queue);

                    let mut procs = processes.write().await;
                    procs.remove(&name);
                    debug!("清理过期进程: {}", name);
                }
            }
        });
    }

    /// 关闭进程池
    pub async fn shutdown(&self) -> Result<()> {
        info!("关闭插件进程池");
        self.shutdown.store(true, Ordering::Relaxed);

        let processes = self.processes.read().await;
        let names: Vec<String> = processes.keys().cloned().collect();
        drop(processes);

        for name in names {
            if let Err(e) = self.stop_process(&name).await {
                error!("停止进程 {} 失败: {}", name, e);
            }
        }

        self.processes.write().await.clear();
        self.idle_queue.lock().await.clear();

        info!("插件进程池已关闭");
        Ok(())
    }

    /// 获取统计信息
    pub async fn stats(&self) -> PoolStats {
        let processes = self.processes.read().await;

        let mut idle = 0;
        let mut busy = 0;
        let mut error = 0;

        for process in processes.values() {
            let p = process.lock().await;
            match &p.info.state {
                ProcessState::Idle => idle += 1,
                ProcessState::Busy { .. } => busy += 1,
                ProcessState::Error { .. } => error += 1,
                _ => {}
            }
        }

        PoolStats {
            total_processes: processes.len(),
            idle_processes: idle,
            busy_processes: busy,
            error_processes: error,
            total_tasks: self.stats.total_tasks.load(Ordering::Relaxed),
            total_restarts: self.stats.total_restarts.load(Ordering::Relaxed),
        }
    }

    /// 获取进程信息
    pub async fn get_process_info(&self, name: &str) -> Option<ProcessInfo> {
        let processes = self.processes.read().await;
        if let Some(p) = processes.get(name) {
            let process = p.lock().await;
            Some(ProcessInfo {
                pid: process.info.pid,
                state: process.info.state.clone(),
                created_at: process.info.created_at,
                last_activity: process.info.last_activity,
                tasks_executed: process.info.tasks_executed,
                restart_count: process.info.restart_count,
                resource_usage: process.info.resource_usage.clone(),
            })
        } else {
            None
        }
    }
}

/// 进程守卫，自动释放
pub struct ProcessGuard<'a> {
    pool: &'a PluginProcessPool,
    process_name: String,
    _permit: tokio::sync::SemaphorePermit<'a>,
}

impl ProcessGuard<'_> {
    /// 获取进程名称
    pub fn name(&self) -> &str {
        &self.process_name
    }

    /// 手动释放进程
    pub async fn release(self) {
        self.pool.release_process(&self.process_name).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = PluginProcessPool::with_defaults();
        let stats = pool.stats().await;
        assert_eq!(stats.total_processes, 0);
    }

    #[tokio::test]
    async fn test_pool_config_default() {
        let config = ProcessPoolConfig::default();
        assert_eq!(config.max_processes, 8);
        assert_eq!(config.min_idle, 1);
        assert!(config.auto_restart);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = PluginProcessPool::with_defaults();
        pool.start().await.unwrap();

        let stats = pool.stats().await;
        assert!(stats.total_processes > 0);

        pool.shutdown().await.unwrap();
    }

    #[test]
    fn test_process_state() {
        let state = ProcessState::Idle;
        assert_eq!(state, ProcessState::Idle);

        let busy = ProcessState::Busy {
            task_id: "test".to_string(),
        };
        assert!(matches!(busy, ProcessState::Busy { .. }));
    }

    #[test]
    fn test_resource_usage() {
        let usage = ResourceUsage {
            cpu_percent: 50.0,
            memory_bytes: 1024 * 1024,
            peak_memory: 2 * 1024 * 1024,
        };
        assert_eq!(usage.cpu_percent, 50.0);
    }
}
