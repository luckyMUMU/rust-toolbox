//! Runtime Manager for Plugin System
//!
//! This module provides runtime management for different plugin types including:
//! - Process pool management for Python and Node.js plugins
//! - Container lifecycle management for Docker plugins
//! - Resource monitoring and limits enforcement
//! - Runtime health checking and recovery

use crate::core::PluginType;
use crate::error::{Result, WorkflowError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Child;
use tracing::{debug, error, info, warn};

/// Runtime state for a managed process or container
#[derive(Debug, Clone)]
pub enum RuntimeState {
    /// Runtime is being initialized
    Initializing,
    /// Runtime is ready and idle
    Ready,
    /// Runtime is currently executing a task
    Running { task_id: String, started_at: Instant },
    /// Runtime encountered an error
    Error { message: String, recoverable: bool },
    /// Runtime is being shut down
    ShuttingDown,
    /// Runtime has been shut down
    Shutdown,
}

/// Resource usage statistics for a runtime
#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Number of tasks executed
    pub tasks_executed: u64,
    /// Total execution time
    pub total_execution_time: Duration,
}

/// Configuration for runtime pool
#[derive(Debug, Clone)]
pub struct RuntimePoolConfig {
    /// Maximum number of concurrent runtimes
    pub max_runtimes: usize,
    /// Minimum number of idle runtimes to maintain
    pub min_idle_runtimes: usize,
    /// Maximum tasks per runtime before recycling
    pub max_tasks_per_runtime: u64,
    /// Maximum memory per runtime in bytes
    pub max_memory_per_runtime: u64,
    /// Maximum execution time per task
    pub max_task_execution_time: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Whether to enable automatic recovery
    pub auto_recovery: bool,
}

impl Default for RuntimePoolConfig {
    fn default() -> Self {
        Self {
            max_runtimes: 4,
            min_idle_runtimes: 1,
            max_tasks_per_runtime: 100,
            max_memory_per_runtime: 1024 * 1024 * 512, // 512MB
            max_task_execution_time: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            auto_recovery: true,
        }
    }
}

/// Managed runtime entry
struct ManagedRuntime {
    /// Plugin name this runtime is for
    plugin_name: String,
    /// Plugin type
    plugin_type: PluginType,
    /// Current state
    state: RuntimeState,
    /// Resource usage statistics
    stats: ResourceStats,
    /// Process handle (for Python/Node.js)
    process: Option<Child>,
    /// Container ID (for Docker)
    container_id: Option<String>,
    /// When the runtime was created
    created_at: Instant,
    /// Last activity time
    last_activity: Instant,
}

/// Runtime pool for a specific plugin type
struct RuntimePool {
    /// Pool configuration
    config: RuntimePoolConfig,
    /// Active runtimes
    runtimes: Vec<ManagedRuntime>,
    /// Pool statistics
    total_tasks: u64,
    total_errors: u64,
}

/// Runtime Manager for different plugin types
pub struct RuntimeManager {
    /// Configuration for all runtime pools
    config: RuntimePoolConfig,
    /// Runtime pools by plugin name
    pools: Arc<Mutex<HashMap<String, RuntimePool>>>,
    /// Health check task handle
    health_check_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RuntimeManager {
    /// Create a new runtime manager with default configuration
    pub fn new() -> Self {
        Self {
            config: RuntimePoolConfig::default(),
            pools: Arc::new(Mutex::new(HashMap::new())),
            health_check_handle: None,
        }
    }

    /// Create a new runtime manager with custom configuration
    pub fn with_config(config: RuntimePoolConfig) -> Self {
        Self {
            config,
            pools: Arc::new(Mutex::new(HashMap::new())),
            health_check_handle: None,
        }
    }

    /// Create a runtime environment for a plugin (async version)
    ///
    /// This initializes the necessary runtime infrastructure:
    /// - For Python: Creates virtual environment if needed
    /// - For Node.js: Installs dependencies if needed
    /// - For Docker: Pulls image and creates container
    pub async fn create_runtime(
        &self,
        plugin_name: &str,
        plugin_type: PluginType,
    ) -> Result<()> {
        // For now, just delegate to sync version
        // In the future, this will do actual async setup (venv creation, docker pull, etc.)
        self.create_runtime_sync(plugin_name, plugin_type)
    }

    /// Create a runtime environment for a plugin (sync version for backward compatibility)
    pub fn create_runtime_sync(
        &self,
        plugin_name: &str,
        _plugin_type: PluginType,
    ) -> Result<()> {
        info!("Creating runtime pool for plugin '{}'", plugin_name);

        let mut pools = self.pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        // Check if pool already exists
        if pools.contains_key(plugin_name) {
            debug!("Runtime pool for plugin '{}' already exists", plugin_name);
            return Ok(());
        }

        // Create new pool for this plugin
        let pool = RuntimePool {
            config: self.config.clone(),
            runtimes: Vec::new(),
            total_tasks: 0,
            total_errors: 0,
        };

        pools.insert(plugin_name.to_string(), pool);
        info!("Created runtime pool for plugin '{}'", plugin_name);

        // Start health check task if not already running
        drop(pools); // Release lock before async operation
        // Note: We can't .await in a sync function or spawn tasks that borrow self
        // The health check task should be started by the async version (create_runtime)
        // This sync version just creates the pool infrastructure

        Ok(())
    }

    /// Get an available runtime from the pool
    ///
    /// Returns a runtime that is ready to execute tasks. If no runtime is available,
    /// may create a new one if under the limit, or wait for one to become available.
    pub async fn acquire_runtime(&self, plugin_name: &str) -> Result<RuntimeHandle> {
        self.acquire_runtime_internal(plugin_name, 0).await
    }

    /// Internal method with retry limit to avoid infinite recursion
    async fn acquire_runtime_internal(&self, plugin_name: &str, retry_count: usize) -> Result<RuntimeHandle> {
        let pools = self.pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        let pool = pools.get(plugin_name).ok_or_else(|| {
            WorkflowError::ValidationError(format!(
                "No runtime pool found for plugin '{}'",
                plugin_name
            ))
        })?;

        // Find an available runtime
        for (idx, runtime) in pool.runtimes.iter().enumerate() {
            if matches!(runtime.state, RuntimeState::Ready) {
                return Ok(RuntimeHandle {
                    plugin_name: plugin_name.to_string(),
                    runtime_idx: idx,
                    pools: Arc::clone(&self.pools),
                });
            }
        }

        // No runtime available - check if we can create more
        if pool.runtimes.len() < pool.config.max_runtimes && retry_count < 3 {
            // Create new runtime
            drop(pools); // Release lock
            self.spawn_runtime(plugin_name).await?;
            
            // Try again with incremented retry count (avoiding recursion)
            Box::pin(self.acquire_runtime_internal(plugin_name, retry_count + 1)).await
        } else {
            Err(WorkflowError::ResourceExhausted)
        }
    }

    /// Spawn a new runtime for a plugin
    async fn spawn_runtime(&self, plugin_name: &str) -> Result<()> {
        let mut pools = self.pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        let pool = pools.get_mut(plugin_name).ok_or_else(|| {
            WorkflowError::ValidationError(format!(
                "No runtime pool found for plugin '{}'",
                plugin_name
            ))
        })?;

        let runtime = ManagedRuntime {
            plugin_name: plugin_name.to_string(),
            plugin_type: PluginType::Native, // Will be set properly when plugin loads
            state: RuntimeState::Initializing,
            stats: ResourceStats::default(),
            process: None,
            container_id: None,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        };

        pool.runtimes.push(runtime);
        info!(
            "Spawned new runtime for plugin '{}'. Total: {}",
            plugin_name,
            pool.runtimes.len()
        );

        Ok(())
    }

    /// Cleanup runtime for a plugin
    ///
    /// Shuts down all runtimes for the specified plugin and removes the pool.
    pub async fn cleanup_runtime(&self, plugin_name: &str) -> Result<()> {
        info!("Cleaning up runtime for plugin '{}'", plugin_name);

        let mut pools = self.pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        if let Some(pool) = pools.remove(plugin_name) {
            // Shutdown all runtimes in the pool
            for runtime in pool.runtimes {
                if let Err(e) = self.shutdown_runtime(runtime).await {
                    error!("Error shutting down runtime: {}", e);
                }
            }
            info!("Cleaned up runtime pool for plugin '{}'", plugin_name);
        }

        Ok(())
    }

    /// Shutdown a single runtime
    async fn shutdown_runtime(&self, mut runtime: ManagedRuntime) -> Result<()> {
        runtime.state = RuntimeState::ShuttingDown;

        // Kill process if running
        if let Some(mut process) = runtime.process {
            if let Err(e) = process.kill().await {
                warn!("Error killing process: {}", e);
            }
        }

        // Stop container if Docker
        if let Some(container_id) = runtime.container_id {
            // Docker container cleanup would go here
            debug!("Would stop Docker container: {}", container_id);
        }

        runtime.state = RuntimeState::Shutdown;
        debug!("Runtime for plugin '{}' shut down", runtime.plugin_name);

        Ok(())
    }

    /// Start health check background task
    async fn start_health_checks(&self) {
        if self.health_check_handle.is_some() {
            return; // Already running
        }

        let pools = Arc::clone(&self.pools);
        let interval = self.config.health_check_interval;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::perform_health_checks(&pools).await {
                    error!("Health check error: {}", e);
                }
            }
        });

        // Note: We can't store the handle due to async lifetime issues
        // In production, you'd use a different approach
        drop(handle);
    }

    /// Perform health checks on all runtimes
    async fn perform_health_checks(
        pools: &Arc<Mutex<HashMap<String, RuntimePool>>>,
    ) -> Result<()> {
        let mut pools = pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        for (plugin_name, pool) in pools.iter_mut() {
            for runtime in &mut pool.runtimes {
                // Check for runtimes that have been running too long
                if let RuntimeState::Running { started_at, .. } = runtime.state {
                    let elapsed = started_at.elapsed();
                    if elapsed > Duration::from_secs(300) {
                        warn!(
                            "Runtime for plugin '{}' has been running for {:?}, may be stuck",
                            plugin_name,
                            elapsed
                        );
                    }
                }

                // Check memory usage
                if runtime.stats.memory_bytes > pool.config.max_memory_per_runtime {
                    warn!(
                        "Runtime for plugin '{}' exceeded memory limit: {} bytes",
                        plugin_name,
                        runtime.stats.memory_bytes
                    );
                }
            }
        }

        Ok(())
    }

    /// Get runtime statistics for a plugin
    pub fn get_runtime_stats(&self, plugin_name: &str) -> Result<Option<RuntimeStats>> {
        let pools = self.pools.lock().map_err(|_| {
            WorkflowError::ConcurrentAccess {
                message: "Failed to lock runtime pools".to_string(),
            }
        })?;

        Ok(pools.get(plugin_name).map(|pool| RuntimeStats {
            total_runtimes: pool.runtimes.len(),
            active_runtimes: pool
                .runtimes
                .iter()
                .filter(|r| matches!(r.state, RuntimeState::Running { .. }))
                .count(),
            ready_runtimes: pool
                .runtimes
                .iter()
                .filter(|r| matches!(r.state, RuntimeState::Ready))
                .count(),
            total_tasks: pool.total_tasks,
            total_errors: pool.total_errors,
        }))
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about runtime usage
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Total number of runtimes
    pub total_runtimes: usize,
    /// Number of active (running) runtimes
    pub active_runtimes: usize,
    /// Number of ready (idle) runtimes
    pub ready_runtimes: usize,
    /// Total tasks executed
    pub total_tasks: u64,
    /// Total errors encountered
    pub total_errors: u64,
}

/// Handle to an acquired runtime
pub struct RuntimeHandle {
    plugin_name: String,
    runtime_idx: usize,
    pools: Arc<Mutex<HashMap<String, RuntimePool>>>,
}

impl RuntimeHandle {
    /// Release the runtime back to the pool
    pub fn release(self) {
        // Runtime is automatically released when handle is dropped
        // This could be extended to update state, metrics, etc.
    }

    /// Get the plugin name
    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        // Mark runtime as ready again
        if let Ok(mut pools) = self.pools.lock() {
            if let Some(pool) = pools.get_mut(&self.plugin_name) {
                if let Some(runtime) = pool.runtimes.get_mut(self.runtime_idx) {
                    runtime.state = RuntimeState::Ready;
                    runtime.last_activity = Instant::now();
                }
            }
        }
    }
}
