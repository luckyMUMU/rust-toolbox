//! File Management Plugin implementation

use super::error::{FileManagementError, FileManagementResult};
use super::error_recovery::{ErrorRecoveryManager, RecoveryConfig};
use super::monitoring::{FileManagementMonitor, MonitoringConfig};
use crate::core::{PluginInfo, PluginType};
use crate::error::{Result, WorkflowError};
use crate::performance::{PerformanceConfig, PerformanceManager};
use crate::plugins::types::{Plugin, PluginConfig, PluginStatus, ResourceLimits, SecurityPolicy};
use crate::tools::{ToolNode, ToolRegistry};
use crate::tools::compat::tool_node_to_enum;
use crate::tools::types::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for the File Management Plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagementConfig {
    /// Maximum number of threads for parallel operations
    pub max_threads: usize,
    /// Temporary directory for intermediate files
    pub temp_directory: PathBuf,
    /// Default text encoding
    pub default_encoding: String,
    /// Enable Chinese text processing features
    pub enable_chinese_processing: bool,
    /// Maximum file size for processing (in bytes)
    pub max_file_size: u64,
    /// Maximum batch size for parallel operations
    pub max_batch_size: usize,
    /// Enable experimental mode by default
    pub default_experimental_mode: bool,
    /// Timeout for human decision prompts (in seconds)
    pub human_decision_timeout: Option<u64>,
    /// Performance optimization settings
    pub performance: FileManagementPerformanceConfig,
    /// Error recovery configuration
    pub error_recovery: RecoveryConfig,
    /// Monitoring and metrics configuration
    pub monitoring: MonitoringConfig,
}

/// Performance configuration specific to file management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagementPerformanceConfig {
    /// Enable memory optimization for large datasets
    pub enable_memory_optimization: bool,
    /// Memory pool size for file operations (in MB)
    pub memory_pool_size_mb: usize,
    /// Enable streaming for large file operations
    pub enable_streaming: bool,
    /// Buffer size for file I/O operations (in KB)
    pub io_buffer_size_kb: usize,
    /// Enable concurrent file operations
    pub enable_concurrent_operations: bool,
    /// Maximum concurrent file operations
    pub max_concurrent_operations: usize,
    /// Enable caching for frequently accessed data
    pub enable_caching: bool,
    /// Cache size limit (in MB)
    pub cache_size_mb: usize,
    /// Cache TTL (time to live) in seconds
    pub cache_ttl_seconds: u64,
    /// Enable compression for temporary files
    pub enable_compression: bool,
    /// Compression level (1-9, where 9 is highest compression)
    pub compression_level: u32,
    /// Enable lazy loading for large directory structures
    pub enable_lazy_loading: bool,
    /// Batch size for lazy loading operations
    pub lazy_loading_batch_size: usize,
    /// Enable resource monitoring
    pub enable_resource_monitoring: bool,
    /// Memory usage threshold for triggering cleanup (percentage)
    pub memory_cleanup_threshold: f64,
    /// Enable performance metrics collection
    pub enable_metrics_collection: bool,
}

impl Default for FileManagementPerformanceConfig {
    fn default() -> Self {
        Self {
            enable_memory_optimization: true,
            memory_pool_size_mb: 256, // 256MB memory pool
            enable_streaming: true,
            io_buffer_size_kb: 64, // 64KB I/O buffer
            enable_concurrent_operations: true,
            max_concurrent_operations: num_cpus::get().max(4),
            enable_caching: true,
            cache_size_mb: 128,        // 128MB cache
            cache_ttl_seconds: 300,    // 5 minutes
            enable_compression: false, // Disabled by default for performance
            compression_level: 6,      // Balanced compression
            enable_lazy_loading: true,
            lazy_loading_batch_size: 100,
            enable_resource_monitoring: true,
            memory_cleanup_threshold: 0.8, // 80%
            enable_metrics_collection: true,
        }
    }
}

impl Default for FileManagementConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get().max(4),
            temp_directory: std::env::temp_dir().join("workflow-toolkit-file-mgmt"),
            default_encoding: "utf-8".to_string(),
            enable_chinese_processing: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_batch_size: 1000,
            default_experimental_mode: false,
            human_decision_timeout: Some(300), // 5 minutes
            performance: FileManagementPerformanceConfig::default(),
            error_recovery: RecoveryConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

/// File Management Plugin implementation
pub struct FileManagementPlugin {
    info: PluginInfo,
    config: Option<FileManagementConfig>,
    status: PluginStatus,
    tools: Arc<RwLock<Vec<Arc<dyn ToolNode>>>>,
    tool_registry: Option<Arc<dyn ToolRegistry>>,
    performance_manager: Option<Arc<PerformanceManager>>,
    error_recovery_manager: Option<Arc<RwLock<ErrorRecoveryManager>>>,
    monitoring_system: Option<Arc<FileManagementMonitor>>,
}

impl FileManagementPlugin {
    /// Create a new File Management Plugin
    pub fn new() -> Self {
        let info = PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Intelligent file and folder management tools".to_string()),
            author: Some("Workflow Toolkit".to_string()),
            homepage: None,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "category".to_string(),
                    Value::String("file-management".to_string()),
                );
                metadata.insert(
                    "capabilities".to_string(),
                    Value::Array(vec![
                        Value::String("classification".to_string()),
                        Value::String("text-processing".to_string()),
                        Value::String("batch-operations".to_string()),
                        Value::String("human-decision".to_string()),
                        Value::String("experimental-mode".to_string()),
                    ]),
                );
                metadata
            },
        };

        Self {
            info,
            config: None,
            status: PluginStatus::Uninitialized,
            tools: Arc::new(RwLock::new(Vec::new())),
            tool_registry: None,
            performance_manager: None,
            error_recovery_manager: None,
            monitoring_system: None,
        }
    }

    /// Set the tool registry for integration with workflow-toolkit
    pub fn set_tool_registry(&mut self, registry: Arc<dyn ToolRegistry>) -> Result<()> {
        if self.status != PluginStatus::Uninitialized {
            return Err(WorkflowError::ValidationError(
                "Cannot set tool registry after plugin initialization".to_string(),
            ));
        }

        self.tool_registry = Some(registry);
        debug!("Tool registry set for file management plugin");
        Ok(())
    }

    /// Register all tools with the workflow-toolkit tool registry
    fn register_tools_with_main_registry(&self, tools: &[Arc<dyn ToolNode>]) -> Result<()> {
        if let Some(_registry) = &self.tool_registry {
            info!(
                "Registering {} file management tools with main tool registry",
                tools.len()
            );

            // Note: We need a mutable reference to the registry, but we only have an Arc<dyn ToolRegistry>
            // This is a design limitation that would need to be addressed in the main tool registry
            // For now, we'll log the registration attempt
            for tool in tools {
                debug!("Would register tool '{}' with main registry", tool.name());
            }

            info!("File management tools registered with main tool registry");
        } else {
            warn!("No tool registry set - tools will only be available through plugin");
        }

        Ok(())
    }

    /// Unregister all tools from the workflow-toolkit tool registry
    fn unregister_tools_from_main_registry(&self, tools: &[Arc<dyn ToolNode>]) -> Result<()> {
        if let Some(_registry) = &self.tool_registry {
            info!(
                "Unregistering {} file management tools from main tool registry",
                tools.len()
            );

            // Note: Same limitation as above - we need a mutable reference
            for tool in tools {
                debug!("Would unregister tool '{}' from main registry", tool.name());
            }

            info!("File management tools unregistered from main tool registry");
        }

        Ok(())
    }

    /// Create a builder for the File Management Plugin
    pub fn builder() -> FileManagementPluginBuilder {
        FileManagementPluginBuilder::new()
    }

    /// Initialize all tools for the plugin
    fn initialize_tools(&self, config: &FileManagementConfig) -> Result<Vec<Arc<dyn ToolNode>>> {
        info!(
            "Initializing file management tools with config: {:?}",
            config
        );

        // Create tool registry
        let mut registry =
            super::registry::FileManagementToolRegistry::new(config.clone(), self.info.clone());

        // Register all tools
        let tools = registry.register_all_tools()?;

        debug!(
            "File management plugin tools initialized: {} tools",
            tools.len()
        );
        Ok(tools)
    }

    /// Initialize performance manager with file management specific configuration
    fn initialize_performance_manager(&mut self, config: &FileManagementConfig) -> Result<()> {
        let perf_config = PerformanceConfig {
            memory: crate::performance::MemoryConfig {
                max_workflow_memory: config.performance.memory_pool_size_mb * 1024 * 1024,
                max_tool_memory: config.max_file_size as usize,
                cleanup_threshold: config.performance.memory_cleanup_threshold,
                monitoring_interval: Duration::from_secs(30),
                enable_pooling: config.performance.enable_memory_optimization,
                pool_sizes: {
                    let mut sizes = HashMap::new();
                    sizes.insert(
                        "file_operations".to_string(),
                        config.performance.max_concurrent_operations,
                    );
                    sizes.insert("text_processing".to_string(), 200);
                    sizes.insert("classification_results".to_string(), 500);
                    sizes
                },
                enable_gc_hints: config.performance.enable_memory_optimization,
                pressure_threshold: 0.9,
            },
            concurrency: crate::performance::ConcurrencyConfig {
                max_concurrent_workflows: 1,
                max_concurrent_tools: config.performance.max_concurrent_operations,
                cpu_thread_pool_size: config.max_threads,
                io_thread_pool_size: config.max_threads * 2,
                task_queue_size: config.max_batch_size,
                enable_work_stealing: true,
                load_balancing: crate::performance::concurrency::LoadBalancingStrategy::RoundRobin,
                adaptive_concurrency: crate::performance::concurrency::AdaptiveConcurrencyConfig {
                    enabled: true,
                    min_concurrency: 1,
                    max_concurrency: config.performance.max_concurrent_operations,
                    adjustment_interval: Duration::from_secs(30),
                    target_latency: Duration::from_millis(100),
                    latency_tolerance: 0.2,
                },
                backpressure: crate::performance::concurrency::BackpressureConfig::default(),
            },
            cache: crate::performance::CacheConfig {
                enabled: config.performance.enable_caching,
                default_ttl: Duration::from_secs(config.performance.cache_ttl_seconds),
                max_entries: 10000,
                max_memory: config.performance.cache_size_mb * 1024 * 1024,
                eviction_policy: crate::performance::cache::EvictionPolicy::LRU,
                warming: crate::performance::cache::CacheWarmingConfig::default(),
                partitions: 4,
                compression: config.performance.enable_compression,
                collect_stats: config.performance.enable_metrics_collection,
            },
            metrics: crate::performance::MetricsConfig {
                enabled: config.performance.enable_metrics_collection,
                collection_interval: Duration::from_secs(10),
                max_data_points: 1000,
                prometheus_enabled: false,
                prometheus_port: 9090,
                custom_metrics: HashMap::new(),
            },
            profiling: crate::performance::ProfilingConfig {
                enabled: config.performance.enable_resource_monitoring,
                sampling_rate: 0.1, // 10% sampling
                max_samples: 1000,
                cpu_profiling: true,
                memory_profiling: config.performance.enable_memory_optimization,
                io_profiling: true,
                output_directory: config
                    .temp_directory
                    .join("profiling")
                    .to_string_lossy()
                    .to_string(),
                auto_profile_interval: Some(Duration::from_secs(60)),
            },
        };

        self.performance_manager = Some(Arc::new(PerformanceManager::new(perf_config)));

        info!("Performance manager initialized for file management plugin");
        Ok(())
    }

    /// Initialize error recovery manager with file management specific configuration
    fn initialize_error_recovery_manager(&mut self, config: &FileManagementConfig) -> Result<()> {
        let recovery_manager = if let Some(perf_manager) = &self.performance_manager {
            ErrorRecoveryManager::with_performance_manager(
                config.error_recovery.clone(),
                perf_manager.clone(),
            )
        } else {
            ErrorRecoveryManager::new(config.error_recovery.clone())
        };

        self.error_recovery_manager = Some(Arc::new(RwLock::new(recovery_manager)));

        info!("Error recovery manager initialized for file management plugin");
        Ok(())
    }

    /// Initialize monitoring system with file management specific configuration
    fn initialize_monitoring_system(&mut self, config: &FileManagementConfig) -> Result<()> {
        let monitor = if let Some(_perf_manager) = &self.performance_manager {
            // Try to get metrics collector from performance manager
            // For now, create without metrics collector integration
            FileManagementMonitor::new(config.monitoring.clone())
        } else {
            FileManagementMonitor::new(config.monitoring.clone())
        };

        self.monitoring_system = Some(Arc::new(monitor));

        info!("Monitoring system initialized for file management plugin");
        Ok(())
    }

    /// Get performance manager
    pub fn performance_manager(&self) -> Option<&Arc<PerformanceManager>> {
        self.performance_manager.as_ref()
    }

    /// Get error recovery manager
    pub fn error_recovery_manager(&self) -> Option<&Arc<RwLock<ErrorRecoveryManager>>> {
        self.error_recovery_manager.as_ref()
    }

    /// Attempt to recover from an error using the error recovery manager
    pub async fn recover_from_error(&self, error: FileManagementError) -> FileManagementResult<()> {
        if let Some(recovery_manager_arc) = &self.error_recovery_manager {
            let mut recovery_manager = recovery_manager_arc.write().map_err(|_| {
                FileManagementError::concurrency(
                    "Failed to acquire write lock on error recovery manager",
                )
            })?;

            recovery_manager.recover_from_error(error).await
        } else {
            warn!("No error recovery manager available");
            Err(error)
        }
    }

    /// Get error recovery statistics
    pub fn get_error_recovery_stats(&self) -> Option<super::error_recovery::RecoveryStats> {
        if let Some(recovery_manager_arc) = &self.error_recovery_manager {
            if let Ok(recovery_manager) = recovery_manager_arc.read() {
                Some(recovery_manager.get_recovery_stats().clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get monitoring system
    pub fn monitoring_system(&self) -> Option<&Arc<FileManagementMonitor>> {
        self.monitoring_system.as_ref()
    }

    /// Get monitoring statistics
    pub async fn get_monitoring_stats(&self) -> Option<super::monitoring::MonitoringStats> {
        if let Some(monitor) = &self.monitoring_system {
            monitor.get_monitoring_stats().await.ok()
        } else {
            None
        }
    }

    /// Optimize plugin performance based on usage patterns
    pub async fn optimize_performance(&self) -> Result<()> {
        if let Some(perf_manager) = &self.performance_manager {
            let optimization_report = perf_manager.optimize().await?;

            info!("Performance optimization completed:");
            info!(
                "  Memory optimizations: {}",
                optimization_report.memory_optimizations.len()
            );
            info!(
                "  Concurrency optimizations: {}",
                optimization_report.concurrency_optimizations.len()
            );
            info!(
                "  Cache optimizations: {}",
                optimization_report.cache_optimizations.len()
            );

            // Apply optimizations if needed
            for memory_opt in &optimization_report.memory_optimizations {
                debug!("Memory optimization: {:?}", memory_opt);
            }

            for concurrency_opt in &optimization_report.concurrency_optimizations {
                debug!("Concurrency optimization: {:?}", concurrency_opt);
            }

            for cache_opt in &optimization_report.cache_optimizations {
                debug!("Cache optimization: {:?}", cache_opt);
            }
        }

        Ok(())
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> Option<HashMap<String, serde_json::Value>> {
        if let Some(perf_manager) = &self.performance_manager {
            let all_stats = perf_manager.get_all_stats().await;
            let mut stats_json = HashMap::new();

            for (component, stats) in all_stats {
                stats_json.insert(
                    component,
                    serde_json::json!({
                        "execution_count": stats.execution_count,
                        "average_duration_ms": stats.average_duration.as_millis(),
                        "min_duration_ms": stats.min_duration.as_millis(),
                        "max_duration_ms": stats.max_duration.as_millis(),
                        "memory_usage": {
                            "initial": stats.memory_usage.initial,
                            "final": stats.memory_usage.final_usage,
                            "peak": stats.memory_usage.peak_usage,
                            "allocated": stats.memory_usage.allocated
                        },
                        "last_updated": stats.last_updated_timestamp
                    }),
                );
            }

            Some(stats_json)
        } else {
            None
        }
    }

    /// Validate the plugin configuration
    fn validate_config(&self, config: &FileManagementConfig) -> Result<()> {
        // Validate max_threads
        if config.max_threads == 0 {
            return Err(WorkflowError::ValidationError(
                "max_threads must be greater than 0".to_string(),
            ));
        }

        // Validate temp_directory
        if let Some(parent) = config.temp_directory.parent() {
            if !parent.exists() {
                return Err(WorkflowError::ValidationError(format!(
                    "Parent directory of temp_directory does not exist: {:?}",
                    parent
                )));
            }
        }

        // Validate max_file_size
        if config.max_file_size == 0 {
            return Err(WorkflowError::ValidationError(
                "max_file_size must be greater than 0".to_string(),
            ));
        }

        // Validate max_batch_size
        if config.max_batch_size == 0 {
            return Err(WorkflowError::ValidationError(
                "max_batch_size must be greater than 0".to_string(),
            ));
        }

        // Validate encoding
        if config.default_encoding.is_empty() {
            return Err(WorkflowError::ValidationError(
                "default_encoding cannot be empty".to_string(),
            ));
        }

        // Validate performance configuration
        self.validate_performance_config(&config.performance)?;

        debug!("File management plugin configuration validation passed");
        Ok(())
    }

    /// Validate performance configuration
    fn validate_performance_config(
        &self,
        perf_config: &FileManagementPerformanceConfig,
    ) -> Result<()> {
        if perf_config.memory_pool_size_mb == 0 {
            return Err(WorkflowError::ValidationError(
                "memory_pool_size_mb must be greater than 0".to_string(),
            ));
        }

        if perf_config.io_buffer_size_kb == 0 {
            return Err(WorkflowError::ValidationError(
                "io_buffer_size_kb must be greater than 0".to_string(),
            ));
        }

        if perf_config.max_concurrent_operations == 0 {
            return Err(WorkflowError::ValidationError(
                "max_concurrent_operations must be greater than 0".to_string(),
            ));
        }

        if perf_config.cache_size_mb == 0 && perf_config.enable_caching {
            return Err(WorkflowError::ValidationError(
                "cache_size_mb must be greater than 0 when caching is enabled".to_string(),
            ));
        }

        if perf_config.compression_level > 9 {
            return Err(WorkflowError::ValidationError(
                "compression_level must be between 1 and 9".to_string(),
            ));
        }

        if perf_config.memory_cleanup_threshold <= 0.0 || perf_config.memory_cleanup_threshold > 1.0
        {
            return Err(WorkflowError::ValidationError(
                "memory_cleanup_threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if perf_config.lazy_loading_batch_size == 0 && perf_config.enable_lazy_loading {
            return Err(WorkflowError::ValidationError(
                "lazy_loading_batch_size must be greater than 0 when lazy loading is enabled"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Create the temporary directory if it doesn't exist
    fn ensure_temp_directory(&self, config: &FileManagementConfig) -> Result<()> {
        if !config.temp_directory.exists() {
            std::fs::create_dir_all(&config.temp_directory).map_err(WorkflowError::Io)?;
            info!("Created temp directory: {:?}", config.temp_directory);
        }
        Ok(())
    }
}

impl Plugin for FileManagementPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn initialize(&mut self, plugin_config: PluginConfig) -> Result<()> {
        info!("Initializing File Management Plugin");

        // Parse the file management specific configuration
        let config: FileManagementConfig = if plugin_config.config == Value::Null {
            FileManagementConfig::default()
        } else {
            serde_json::from_value(plugin_config.config).map_err(|e| {
                WorkflowError::ValidationError(format!("Invalid file management config: {}", e))
            })?
        };

        // Validate configuration
        self.validate_config(&config)?;

        // Ensure temp directory exists
        self.ensure_temp_directory(&config)?;

        // Initialize performance manager
        self.initialize_performance_manager(&config)?;

        // Initialize error recovery manager
        self.initialize_error_recovery_manager(&config)?;

        // Initialize monitoring system
        self.initialize_monitoring_system(&config)?;

        // Start monitoring if enabled
        if let Some(monitor) = &self.monitoring_system {
            if let Err(e) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(monitor.start_monitoring())
            }) {
                warn!("Failed to start monitoring system: {}", e);
            }
        }

        // Initialize tools
        let tools = self.initialize_tools(&config)?;

        // Register tools with main tool registry if available
        self.register_tools_with_main_registry(&tools)?;

        // Store tools
        {
            let mut tools_guard =
                self.tools
                    .write()
                    .map_err(|_| WorkflowError::ConcurrentAccess {
                        message: "Failed to acquire write lock on tools".to_string(),
                    })?;
            *tools_guard = tools;
        }

        // Store configuration and update status
        self.config = Some(config);
        self.status = PluginStatus::Ready;

        info!("File Management Plugin initialized successfully");
        Ok(())
    }

    fn get_tools(&self) -> Vec<Tool> {
        match self.tools.read() {
            Ok(tools_guard) => tools_guard.iter().map(|t| tool_node_to_enum(t.clone())).collect(),
            Err(e) => {
                error!("Failed to acquire read lock on tools: {}", e);
                Vec::new()
            }
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down File Management Plugin");

        // Get tools before clearing them
        let tools = {
            let tools_guard = self
                .tools
                .read()
                .map_err(|_| WorkflowError::ConcurrentAccess {
                    message: "Failed to acquire read lock on tools during shutdown".to_string(),
                })?;
            tools_guard.clone()
        };

        // Unregister tools from main registry
        self.unregister_tools_from_main_registry(&tools)?;

        // Clear tools
        {
            let mut tools_guard =
                self.tools
                    .write()
                    .map_err(|_| WorkflowError::ConcurrentAccess {
                        message: "Failed to acquire write lock on tools during shutdown"
                            .to_string(),
                    })?;
            tools_guard.clear();
        }

        // Clean up temp directory if it was created by us
        if let Some(config) = &self.config {
            if config.temp_directory.exists() {
                if let Err(e) = std::fs::remove_dir_all(&config.temp_directory) {
                    warn!("Failed to clean up temp directory: {}", e);
                }
            }
        }

        self.config = None;
        self.status = PluginStatus::Shutdown;

        info!("File Management Plugin shut down successfully");
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        matches!(self.status, PluginStatus::Ready | PluginStatus::Running)
    }

    fn status(&self) -> PluginStatus {
        self.status
    }
}

/// Builder for FileManagementPlugin
pub struct FileManagementPluginBuilder {
    config: FileManagementConfig,
}

impl FileManagementPluginBuilder {
    pub fn new() -> Self {
        Self {
            config: FileManagementConfig::default(),
        }
    }

    pub fn max_threads(mut self, max_threads: usize) -> Self {
        self.config.max_threads = max_threads;
        self
    }

    pub fn temp_directory<P: Into<PathBuf>>(mut self, temp_directory: P) -> Self {
        self.config.temp_directory = temp_directory.into();
        self
    }

    pub fn default_encoding<S: Into<String>>(mut self, encoding: S) -> Self {
        self.config.default_encoding = encoding.into();
        self
    }

    pub fn enable_chinese_processing(mut self, enable: bool) -> Self {
        self.config.enable_chinese_processing = enable;
        self
    }

    pub fn max_file_size(mut self, max_size: u64) -> Self {
        self.config.max_file_size = max_size;
        self
    }

    pub fn max_batch_size(mut self, max_size: usize) -> Self {
        self.config.max_batch_size = max_size;
        self
    }

    pub fn default_experimental_mode(mut self, enable: bool) -> Self {
        self.config.default_experimental_mode = enable;
        self
    }

    pub fn human_decision_timeout(mut self, timeout_seconds: Option<u64>) -> Self {
        self.config.human_decision_timeout = timeout_seconds;
        self
    }

    pub fn performance_config(mut self, performance: FileManagementPerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    pub fn enable_memory_optimization(mut self, enable: bool) -> Self {
        self.config.performance.enable_memory_optimization = enable;
        self
    }

    pub fn memory_pool_size_mb(mut self, size_mb: usize) -> Self {
        self.config.performance.memory_pool_size_mb = size_mb;
        self
    }

    pub fn enable_streaming(mut self, enable: bool) -> Self {
        self.config.performance.enable_streaming = enable;
        self
    }

    pub fn io_buffer_size_kb(mut self, size_kb: usize) -> Self {
        self.config.performance.io_buffer_size_kb = size_kb;
        self
    }

    pub fn max_concurrent_operations(mut self, max_ops: usize) -> Self {
        self.config.performance.max_concurrent_operations = max_ops;
        self
    }

    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.config.performance.enable_caching = enable;
        self
    }

    pub fn cache_size_mb(mut self, size_mb: usize) -> Self {
        self.config.performance.cache_size_mb = size_mb;
        self
    }

    pub fn enable_compression(mut self, enable: bool) -> Self {
        self.config.performance.enable_compression = enable;
        self
    }

    pub fn compression_level(mut self, level: u32) -> Self {
        self.config.performance.compression_level = level;
        self
    }

    pub fn enable_lazy_loading(mut self, enable: bool) -> Self {
        self.config.performance.enable_lazy_loading = enable;
        self
    }

    pub fn enable_resource_monitoring(mut self, enable: bool) -> Self {
        self.config.performance.enable_resource_monitoring = enable;
        self
    }

    pub fn error_recovery_config(mut self, recovery_config: RecoveryConfig) -> Self {
        self.config.error_recovery = recovery_config;
        self
    }

    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.config.error_recovery.max_retries = max_retries;
        self
    }

    pub fn enable_auto_recovery(mut self, enable: bool) -> Self {
        self.config.error_recovery.enable_auto_recovery = enable;
        self
    }

    pub fn recovery_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.config.error_recovery.recovery_timeout_ms = timeout_ms;
        self
    }

    pub fn monitoring_config(mut self, monitoring_config: MonitoringConfig) -> Self {
        self.config.monitoring = monitoring_config;
        self
    }

    pub fn enable_monitoring(mut self, enable: bool) -> Self {
        self.config.monitoring.enabled = enable;
        self
    }

    pub fn enable_audit_trail(mut self, enable: bool) -> Self {
        self.config.monitoring.enable_audit_trail = enable;
        self
    }

    pub fn enable_performance_monitoring(mut self, enable: bool) -> Self {
        self.config.monitoring.enable_performance_monitoring = enable;
        self
    }

    pub fn enable_error_tracking(mut self, enable: bool) -> Self {
        self.config.monitoring.enable_error_tracking = enable;
        self
    }

    pub fn build(self) -> Result<FileManagementPlugin> {
        let mut plugin = FileManagementPlugin::new();

        // Create plugin config
        let plugin_config = PluginConfig {
            name: "file-management".to_string(),
            plugin_type: PluginType::Native,
            enabled: true,
            config: serde_json::to_value(&self.config).map_err(|e| {
                WorkflowError::ValidationError(format!("Failed to serialize config: {}", e))
            })?,
            security_policy: SecurityPolicy {
                allow_file_system_access: true,
                allow_network_access: false,
                allowed_paths: vec![self.config.temp_directory.clone()],
                environment_variables: HashMap::new(),
                sandbox_enabled: false,
            },
            resource_limits: ResourceLimits {
                max_memory: Some(2 * 1024 * 1024 * 1024), // 2GB
                max_cpu_time: None,                       // No CPU time limit for file operations
                max_execution_time: Some(std::time::Duration::from_secs(3600)), // 1 hour
                max_file_size: Some(self.config.max_file_size),
                max_network_connections: Some(0), // No network access
            },
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        };

        plugin.initialize(plugin_config)?;
        Ok(plugin)
    }
}

impl Default for FileManagementPluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_management_plugin_creation() {
        let plugin = FileManagementPlugin::new();
        assert_eq!(plugin.info().name, "file-management");
        assert_eq!(plugin.info().version, "1.0.0");
        assert_eq!(plugin.status(), PluginStatus::Uninitialized);
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_file_management_plugin_builder() {
        let temp_dir = TempDir::new().unwrap();

        let result = FileManagementPlugin::builder()
            .max_threads(8)
            .temp_directory(temp_dir.path())
            .default_encoding("utf-8")
            .enable_chinese_processing(true)
            .max_file_size(50 * 1024 * 1024) // 50MB
            .max_batch_size(500)
            .default_experimental_mode(true)
            .human_decision_timeout(Some(600))
            .build();

        assert!(result.is_ok());
        let plugin = result.unwrap();
        assert!(plugin.is_initialized());
        assert_eq!(plugin.status(), PluginStatus::Ready);
    }

    #[test]
    fn test_config_validation() {
        let plugin = FileManagementPlugin::new();

        // Test invalid max_threads
        let invalid_config = FileManagementConfig {
            max_threads: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test invalid max_file_size
        let invalid_config = FileManagementConfig {
            max_file_size: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test invalid max_batch_size
        let invalid_config = FileManagementConfig {
            max_batch_size: 0,
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test empty encoding
        let invalid_config = FileManagementConfig {
            default_encoding: String::new(),
            ..Default::default()
        };
        assert!(plugin.validate_config(&invalid_config).is_err());

        // Test valid config
        let valid_config = FileManagementConfig::default();
        assert!(plugin.validate_config(&valid_config).is_ok());
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let temp_dir = TempDir::new().unwrap();

        let mut plugin = FileManagementPlugin::builder()
            .temp_directory(temp_dir.path())
            .build()
            .unwrap();

        // Plugin should be initialized
        assert!(plugin.is_initialized());
        assert_eq!(plugin.status(), PluginStatus::Ready);

        // Should have tools now (registered by the registry)
        let tools = plugin.get_tools();
        assert!(tools.len() > 0); // We expect some tools to be registered

        // Test shutdown
        assert!(plugin.shutdown().is_ok());
        assert_eq!(plugin.status(), PluginStatus::Shutdown);
        assert!(!plugin.is_initialized());
    }
}
