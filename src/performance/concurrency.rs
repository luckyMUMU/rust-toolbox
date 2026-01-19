//! Concurrency performance optimization and tuning

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Maximum number of concurrent workflows
    pub max_concurrent_workflows: usize,

    /// Maximum number of concurrent tools per workflow
    pub max_concurrent_tools: usize,

    /// Thread pool size for CPU-intensive tasks
    pub cpu_thread_pool_size: usize,

    /// Thread pool size for I/O tasks
    pub io_thread_pool_size: usize,

    /// Task queue size
    pub task_queue_size: usize,

    /// Enable work stealing
    pub enable_work_stealing: bool,

    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,

    /// Adaptive concurrency settings
    pub adaptive_concurrency: AdaptiveConcurrencyConfig,

    /// Backpressure configuration
    pub backpressure: BackpressureConfig,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: num_cpus::get() * 2,
            max_concurrent_tools: num_cpus::get() * 4,
            cpu_thread_pool_size: num_cpus::get(),
            io_thread_pool_size: num_cpus::get() * 2,
            task_queue_size: 10000,
            enable_work_stealing: true,
            load_balancing: LoadBalancingStrategy::RoundRobin,
            adaptive_concurrency: AdaptiveConcurrencyConfig::default(),
            backpressure: BackpressureConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ResourceBased,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConcurrencyConfig {
    pub enabled: bool,
    pub min_concurrency: usize,
    pub max_concurrency: usize,
    pub adjustment_interval: Duration,
    pub target_latency: Duration,
    pub latency_tolerance: f64,
}

impl Default for AdaptiveConcurrencyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_concurrency: 1,
            max_concurrency: num_cpus::get() * 8,
            adjustment_interval: Duration::from_secs(30),
            target_latency: Duration::from_millis(100),
            latency_tolerance: 0.2, // 20%
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub enabled: bool,
    pub queue_size_threshold: usize,
    pub latency_threshold: Duration,
    pub rejection_strategy: RejectionStrategy,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            queue_size_threshold: 1000,
            latency_threshold: Duration::from_secs(5),
            rejection_strategy: RejectionStrategy::DropOldest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RejectionStrategy {
    DropOldest,
    DropNewest,
    Reject,
}

/// Concurrency statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcurrencyStats {
    pub active_workflows: usize,
    pub active_tools: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub rejected_tasks: u64,
    pub average_latency: Duration,
    pub throughput: f64,
    pub cpu_utilization: f64,
    pub thread_pool_utilization: f64,
}

/// Concurrency optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyOptimization {
    pub optimization_type: ConcurrencyOptimizationType,
    pub description: String,
    pub estimated_improvement: f64,
    pub priority: super::OptimizationPriority,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcurrencyOptimizationType {
    IncreaseThreadPoolSize,
    DecreaseThreadPoolSize,
    AdjustQueueSize,
    EnableWorkStealing,
    ChangeLoadBalancing,
    OptimizeTaskBatching,
    ReduceContention,
    ImproveLocality,
}

/// Concurrency manager for optimization and monitoring
pub struct ConcurrencyManager {
    config: Arc<RwLock<ConcurrencyConfig>>,

    // Semaphores for controlling concurrency
    workflow_semaphore: Arc<Semaphore>,
    tool_semaphore: Arc<Semaphore>,

    // Thread pools
    cpu_pool: Arc<tokio::runtime::Handle>,
    io_pool: Arc<tokio::runtime::Handle>,

    // Statistics and monitoring
    stats: Arc<RwLock<ConcurrencyStats>>,
    latency_history: Arc<DashMap<String, Vec<Duration>>>,
    throughput_history: Arc<DashMap<String, Vec<f64>>>,

    // Load balancer
    load_balancer: Arc<LoadBalancer>,

    // Adaptive concurrency controller
    adaptive_controller: Arc<AdaptiveConcurrencyController>,

    // Backpressure manager
    backpressure_manager: Arc<BackpressureManager>,
}

impl ConcurrencyManager {
    /// Create a new concurrency manager
    pub fn new(config: ConcurrencyConfig) -> Self {
        let workflow_semaphore = Arc::new(Semaphore::new(config.max_concurrent_workflows));
        let tool_semaphore = Arc::new(Semaphore::new(config.max_concurrent_tools));

        // Use current runtime handle for thread pools
        let cpu_pool = Arc::new(tokio::runtime::Handle::current());
        let io_pool = Arc::new(tokio::runtime::Handle::current());

        let stats = Arc::new(RwLock::new(ConcurrencyStats::default()));
        let latency_history = Arc::new(DashMap::new());
        let throughput_history = Arc::new(DashMap::new());

        let load_balancer = Arc::new(LoadBalancer::new(config.load_balancing.clone()));

        let adaptive_controller = Arc::new(AdaptiveConcurrencyController::new(
            config.adaptive_concurrency.clone(),
            workflow_semaphore.clone(),
            tool_semaphore.clone(),
        ));

        let backpressure_manager = Arc::new(BackpressureManager::new(config.backpressure.clone()));

        let config = Arc::new(RwLock::new(config));

        Self {
            config,
            workflow_semaphore,
            tool_semaphore,
            cpu_pool,
            io_pool,
            stats,
            latency_history,
            throughput_history,
            load_balancer,
            adaptive_controller,
            backpressure_manager,
        }
    }

    /// Acquire workflow concurrency permit
    pub async fn acquire_workflow_permit(&self) -> crate::Result<WorkflowPermit> {
        // Check backpressure
        self.backpressure_manager.check_backpressure().await?;

        let permit = self
            .workflow_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| crate::WorkflowError::concurrency("Failed to acquire workflow permit"))?;

        Ok(WorkflowPermit {
            _permit: permit,
            manager: Arc::new(self.clone()),
            acquired_at: Instant::now(),
        })
    }

    /// Acquire tool concurrency permit
    pub async fn acquire_tool_permit(&self) -> crate::Result<ToolPermit> {
        // Check backpressure
        self.backpressure_manager.check_backpressure().await?;

        let permit = self
            .tool_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| crate::WorkflowError::concurrency("Failed to acquire tool permit"))?;

        Ok(ToolPermit {
            _permit: permit,
            manager: Arc::new(self.clone()),
            acquired_at: Instant::now(),
        })
    }

    /// Execute a CPU-intensive task
    pub async fn execute_cpu_task<F, T>(&self, task: F) -> crate::Result<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let start_time = Instant::now();

        let result = tokio::task::spawn_blocking(task)
            .await
            .map_err(|e| crate::WorkflowError::concurrency(format!("CPU task failed: {}", e)))?;

        let duration = start_time.elapsed();
        self.record_task_completion("cpu_task", duration).await;

        Ok(result)
    }

    /// Execute an I/O task
    pub async fn execute_io_task<F, T>(&self, task: F) -> crate::Result<T>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let start_time = Instant::now();

        let result = task.await;

        let duration = start_time.elapsed();
        self.record_task_completion("io_task", duration).await;

        Ok(result)
    }

    /// Record task completion for statistics
    pub async fn record_task_completion(&self, task_type: &str, duration: Duration) {
        // Update latency history
        let mut history = self
            .latency_history
            .entry(task_type.to_string())
            .or_default();
        history.push(duration);

        // Keep only recent history
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.completed_tasks += 1;

        // Calculate average latency
        if let Some(history) = self.latency_history.get(task_type) {
            let total_duration: Duration = history.iter().sum();
            stats.average_latency = total_duration / history.len() as u32;
        }

        // Notify adaptive controller
        self.adaptive_controller.record_latency(duration).await;
    }

    /// Get current concurrency statistics
    pub async fn get_statistics(&self) -> ConcurrencyStats {
        let mut stats = self.stats.read().await.clone();

        // Update current active counts
        stats.active_workflows = self.workflow_semaphore.available_permits();
        stats.active_tools = self.tool_semaphore.available_permits();

        // Calculate throughput
        stats.throughput = self.calculate_throughput().await;

        // Get CPU utilization (mock for now)
        stats.cpu_utilization = self.get_cpu_utilization().await;

        stats
    }

    /// Optimize concurrency settings
    pub async fn optimize(&self) -> crate::Result<Vec<ConcurrencyOptimization>> {
        let mut optimizations = Vec::new();
        let stats = self.get_statistics().await;

        // Analyze latency patterns
        optimizations.extend(self.analyze_latency_patterns().await);

        // Analyze throughput patterns
        optimizations.extend(self.analyze_throughput_patterns().await);

        // Check for contention
        if self.detect_contention().await {
            optimizations.push(ConcurrencyOptimization {
                optimization_type: ConcurrencyOptimizationType::ReduceContention,
                description: "High contention detected - consider reducing shared state access"
                    .to_string(),
                estimated_improvement: 0.2, // 20% improvement
                priority: super::OptimizationPriority::High,
                component: "concurrency".to_string(),
            });
        }

        // Check CPU utilization
        if stats.cpu_utilization < 0.5 {
            optimizations.push(ConcurrencyOptimization {
                optimization_type: ConcurrencyOptimizationType::IncreaseThreadPoolSize,
                description: "Low CPU utilization - consider increasing thread pool size"
                    .to_string(),
                estimated_improvement: 0.3, // 30% improvement
                priority: super::OptimizationPriority::Medium,
                component: "thread_pool".to_string(),
            });
        } else if stats.cpu_utilization > 0.9 {
            optimizations.push(ConcurrencyOptimization {
                optimization_type: ConcurrencyOptimizationType::DecreaseThreadPoolSize,
                description: "High CPU utilization - consider decreasing thread pool size"
                    .to_string(),
                estimated_improvement: 0.1, // 10% improvement
                priority: super::OptimizationPriority::Low,
                component: "thread_pool".to_string(),
            });
        }

        Ok(optimizations)
    }

    /// Adjust concurrency limits dynamically
    pub async fn adjust_limits(
        &self,
        workflow_limit: Option<usize>,
        tool_limit: Option<usize>,
    ) -> crate::Result<()> {
        let mut config = self.config.write().await;

        if let Some(limit) = workflow_limit {
            config.max_concurrent_workflows = limit;
            // Note: We can't actually change semaphore permits at runtime in this implementation
            // In a real system, we'd need a more sophisticated approach
        }

        if let Some(limit) = tool_limit {
            config.max_concurrent_tools = limit;
        }

        Ok(())
    }

    async fn analyze_latency_patterns(&self) -> Vec<ConcurrencyOptimization> {
        let mut optimizations = Vec::new();

        for entry in self.latency_history.iter() {
            let task_type = entry.key();
            let history = entry.value();

            if history.len() < 10 {
                continue;
            }

            let avg_latency = history.iter().sum::<Duration>() / history.len() as u32;
            let config = self.config.read().await;

            if avg_latency > config.adaptive_concurrency.target_latency * 2 {
                optimizations.push(ConcurrencyOptimization {
                    optimization_type: ConcurrencyOptimizationType::OptimizeTaskBatching,
                    description: format!(
                        "High latency detected for {}: {:?}",
                        task_type, avg_latency
                    ),
                    estimated_improvement: 0.25, // 25% improvement
                    priority: super::OptimizationPriority::High,
                    component: task_type.clone(),
                });
            }
        }

        optimizations
    }

    async fn analyze_throughput_patterns(&self) -> Vec<ConcurrencyOptimization> {
        let mut optimizations = Vec::new();

        for entry in self.throughput_history.iter() {
            let component = entry.key();
            let history = entry.value();

            if history.len() < 5 {
                continue;
            }

            let recent_throughput = history[history.len() - 3..].iter().sum::<f64>() / 3.0;
            let older_throughput = history[history.len() - 5..history.len() - 3]
                .iter()
                .sum::<f64>()
                / 2.0;

            if recent_throughput < older_throughput * 0.8 {
                optimizations.push(ConcurrencyOptimization {
                    optimization_type: ConcurrencyOptimizationType::ImproveLocality,
                    description: format!("Throughput degradation detected for {}", component),
                    estimated_improvement: 0.15, // 15% improvement
                    priority: super::OptimizationPriority::Medium,
                    component: component.clone(),
                });
            }
        }

        optimizations
    }

    async fn detect_contention(&self) -> bool {
        let stats = self.get_statistics().await;

        // Simple heuristic: if we have many queued tasks but low CPU utilization,
        // it might indicate contention
        stats.queued_tasks > 100 && stats.cpu_utilization < 0.6
    }

    async fn calculate_throughput(&self) -> f64 {
        let stats = self.stats.read().await;

        // Simple throughput calculation (tasks per second)
        // In a real implementation, this would be more sophisticated
        stats.completed_tasks as f64 / 60.0 // Assume 1-minute window
    }

    async fn get_cpu_utilization(&self) -> f64 {
        // Mock CPU utilization
        // In a real implementation, this would query actual system metrics
        0.7 // 70% utilization
    }
}

impl Clone for ConcurrencyManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            workflow_semaphore: self.workflow_semaphore.clone(),
            tool_semaphore: self.tool_semaphore.clone(),
            cpu_pool: self.cpu_pool.clone(),
            io_pool: self.io_pool.clone(),
            stats: self.stats.clone(),
            latency_history: self.latency_history.clone(),
            throughput_history: self.throughput_history.clone(),
            load_balancer: self.load_balancer.clone(),
            adaptive_controller: self.adaptive_controller.clone(),
            backpressure_manager: self.backpressure_manager.clone(),
        }
    }
}

/// Workflow concurrency permit
pub struct WorkflowPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    manager: Arc<ConcurrencyManager>,
    acquired_at: Instant,
}

impl Drop for WorkflowPermit {
    fn drop(&mut self) {
        let duration = self.acquired_at.elapsed();
        let manager = self.manager.clone();

        tokio::spawn(async move {
            manager.record_task_completion("workflow", duration).await;
        });
    }
}

/// Tool concurrency permit
pub struct ToolPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    manager: Arc<ConcurrencyManager>,
    acquired_at: Instant,
}

impl Drop for ToolPermit {
    fn drop(&mut self) {
        let duration = self.acquired_at.elapsed();
        let manager = self.manager.clone();

        tokio::spawn(async move {
            manager.record_task_completion("tool", duration).await;
        });
    }
}

/// Load balancer for distributing work
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    worker_stats: Arc<DashMap<String, WorkerStats>>,
    round_robin_counter: Arc<Mutex<usize>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            worker_stats: Arc::new(DashMap::new()),
            round_robin_counter: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn select_worker(&self, workers: &[String]) -> Option<String> {
        if workers.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let mut counter = self.round_robin_counter.lock().await;
                let index = *counter % workers.len();
                *counter = (*counter + 1) % workers.len();
                Some(workers[index].clone())
            }
            LoadBalancingStrategy::LeastConnections => {
                let mut min_connections = usize::MAX;
                let mut selected_worker = None;

                for worker in workers {
                    let connections = self
                        .worker_stats
                        .get(worker)
                        .map(|stats| stats.active_connections)
                        .unwrap_or(0);

                    if connections < min_connections {
                        min_connections = connections;
                        selected_worker = Some(worker.clone());
                    }
                }

                selected_worker
            }
            _ => {
                // Default to round robin for other strategies
                let mut counter = self.round_robin_counter.lock().await;
                let index = *counter % workers.len();
                *counter = (*counter + 1) % workers.len();
                Some(workers[index].clone())
            }
        }
    }

    pub async fn record_worker_activity(&self, worker: &str, active: bool) {
        let mut stats = self
            .worker_stats
            .entry(worker.to_string())
            .or_default();

        if active {
            stats.active_connections += 1;
        } else {
            stats.active_connections = stats.active_connections.saturating_sub(1);
        }
    }
}

/// Adaptive concurrency controller
pub struct AdaptiveConcurrencyController {
    config: AdaptiveConcurrencyConfig,
    #[allow(dead_code)]
    workflow_semaphore: Arc<Semaphore>,
    #[allow(dead_code)]
    tool_semaphore: Arc<Semaphore>,
    recent_latencies: Arc<Mutex<Vec<Duration>>>,
    last_adjustment: Arc<Mutex<Instant>>,
}

impl AdaptiveConcurrencyController {
    pub fn new(
        config: AdaptiveConcurrencyConfig,
        workflow_semaphore: Arc<Semaphore>,
        tool_semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            config,
            workflow_semaphore,
            tool_semaphore,
            recent_latencies: Arc::new(Mutex::new(Vec::new())),
            last_adjustment: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub async fn record_latency(&self, latency: Duration) {
        if !self.config.enabled {
            return;
        }

        let mut latencies = self.recent_latencies.lock().await;
        latencies.push(latency);

        // Keep only recent latencies
        if latencies.len() > 100 {
            let len = latencies.len();
            latencies.drain(0..len - 100);
        }

        // Check if it's time to adjust
        let mut last_adjustment = self.last_adjustment.lock().await;
        if last_adjustment.elapsed() >= self.config.adjustment_interval {
            self.adjust_concurrency(&latencies).await;
            *last_adjustment = Instant::now();
        }
    }

    async fn adjust_concurrency(&self, latencies: &[Duration]) {
        if latencies.len() < 10 {
            return;
        }

        let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let target_latency = self.config.target_latency;
        let tolerance = self.config.latency_tolerance;

        let latency_ratio = avg_latency.as_secs_f64() / target_latency.as_secs_f64();

        if latency_ratio > 1.0 + tolerance {
            // Latency too high, reduce concurrency
            tracing::info!(
                "Reducing concurrency due to high latency: {:?} > {:?}",
                avg_latency,
                target_latency
            );
            // In a real implementation, we'd adjust semaphore permits
        } else if latency_ratio < 1.0 - tolerance {
            // Latency acceptable, try increasing concurrency
            tracing::info!(
                "Increasing concurrency due to low latency: {:?} < {:?}",
                avg_latency,
                target_latency
            );
            // In a real implementation, we'd adjust semaphore permits
        }
    }
}

/// Backpressure manager
pub struct BackpressureManager {
    config: BackpressureConfig,
    queue_size: Arc<Mutex<usize>>,
    recent_latencies: Arc<Mutex<Vec<Duration>>>,
}

impl BackpressureManager {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            config,
            queue_size: Arc::new(Mutex::new(0)),
            recent_latencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn check_backpressure(&self) -> crate::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let queue_size = *self.queue_size.lock().await;
        let latencies = self.recent_latencies.lock().await;

        // Check queue size threshold
        if queue_size > self.config.queue_size_threshold {
            return Err(crate::WorkflowError::backpressure(
                "Queue size threshold exceeded",
            ));
        }

        // Check latency threshold
        if !latencies.is_empty() {
            let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
            if avg_latency > self.config.latency_threshold {
                return Err(crate::WorkflowError::backpressure(
                    "Latency threshold exceeded",
                ));
            }
        }

        Ok(())
    }

    pub async fn increment_queue_size(&self) {
        let mut queue_size = self.queue_size.lock().await;
        *queue_size += 1;
    }

    pub async fn decrement_queue_size(&self) {
        let mut queue_size = self.queue_size.lock().await;
        *queue_size = queue_size.saturating_sub(1);
    }
}

// Supporting types

#[derive(Debug, Clone, Default)]
pub struct WorkerStats {
    pub active_connections: usize,
    pub total_requests: u64,
    pub average_response_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_manager_creation() {
        let config = ConcurrencyConfig::default();
        let manager = ConcurrencyManager::new(config);

        let stats = manager.get_statistics().await;
        assert_eq!(stats.completed_tasks, 0);
    }

    #[tokio::test]
    async fn test_workflow_permit_acquisition() {
        let config = ConcurrencyConfig {
            max_concurrent_workflows: 2,
            ..Default::default()
        };
        let manager = ConcurrencyManager::new(config);

        let permit1 = manager.acquire_workflow_permit().await.unwrap();
        let permit2 = manager.acquire_workflow_permit().await.unwrap();

        // Third permit should be available but would block in real scenario
        // For testing, we just verify the permits were acquired
        drop(permit1);
        drop(permit2);
    }

    #[tokio::test]
    async fn test_load_balancer() {
        let balancer = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);
        let workers = vec![
            "worker1".to_string(),
            "worker2".to_string(),
            "worker3".to_string(),
        ];

        let selected1 = balancer.select_worker(&workers).await.unwrap();
        let selected2 = balancer.select_worker(&workers).await.unwrap();
        let selected3 = balancer.select_worker(&workers).await.unwrap();
        let selected4 = balancer.select_worker(&workers).await.unwrap();

        // Should cycle through workers
        assert_eq!(selected1, "worker1");
        assert_eq!(selected2, "worker2");
        assert_eq!(selected3, "worker3");
        assert_eq!(selected4, "worker1"); // Back to first
    }

    #[tokio::test]
    async fn test_backpressure_manager() {
        let config = BackpressureConfig {
            enabled: true,
            queue_size_threshold: 5,
            latency_threshold: Duration::from_millis(100),
            rejection_strategy: RejectionStrategy::Reject,
        };

        let manager = BackpressureManager::new(config);

        // Should pass initially
        assert!(manager.check_backpressure().await.is_ok());

        // Simulate queue growth
        for _ in 0..6 {
            manager.increment_queue_size().await;
        }

        // Should fail due to queue size
        assert!(manager.check_backpressure().await.is_err());
    }
}
