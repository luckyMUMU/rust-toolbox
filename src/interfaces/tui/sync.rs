//! Data Synchronization System for TUI
//!
//! This module provides comprehensive data synchronization capabilities,
//! including periodic updates, real-time notifications, connection management,
//! and offline caching.

use crate::core::{PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::interfaces::tui::state::{
    ConnectionStatus, ExecutionInfo, PerformanceMetrics, SharedAppState, StateChangeEvent,
    SystemStatus,
};
use crate::interfaces::tui::widgets::{
    log_viewer::LogEntry,
    workflow_list::{ExecutionStatus, WorkflowInfo, WorkflowStatus},
};
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep, timeout};
use tracing::{debug, error, info, trace, warn};

/// Configuration for data synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Interval for periodic data synchronization (seconds)
    pub sync_interval_seconds: u64,
    /// Timeout for sync operations (seconds)
    pub sync_timeout_seconds: u64,
    /// Maximum retry attempts for failed syncs
    pub max_retry_attempts: u32,
    /// Backoff multiplier for retry delays
    pub retry_backoff_multiplier: f64,
    /// Initial retry delay (milliseconds)
    pub initial_retry_delay_ms: u64,
    /// Enable real-time updates via WebSocket/SSE
    pub enable_realtime_updates: bool,
    /// Real-time connection URL
    pub realtime_url: Option<String>,
    /// Enable offline caching
    pub enable_offline_cache: bool,
    /// Cache expiry time (hours)
    pub cache_expiry_hours: u64,
    /// Maximum cache size (MB)
    pub max_cache_size_mb: u64,
    /// Connection health check interval (seconds)
    pub health_check_interval_seconds: u64,
    /// Connection timeout (seconds)
    pub connection_timeout_seconds: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 30,
            sync_timeout_seconds: 10,
            max_retry_attempts: 3,
            retry_backoff_multiplier: 2.0,
            initial_retry_delay_ms: 1000,
            enable_realtime_updates: true,
            realtime_url: None,
            enable_offline_cache: true,
            cache_expiry_hours: 24,
            max_cache_size_mb: 100,
            health_check_interval_seconds: 60,
            connection_timeout_seconds: 5,
        }
    }
}

/// Synchronization statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetrics {
    /// Total sync operations performed
    pub total_syncs: u64,
    /// Successful sync operations
    pub successful_syncs: u64,
    /// Failed sync operations
    pub failed_syncs: u64,
    /// Last sync timestamp
    pub last_sync_time: Option<DateTime<Utc>>,
    /// Last successful sync timestamp
    pub last_successful_sync: Option<DateTime<Utc>>,
    /// Average sync duration (milliseconds)
    pub average_sync_duration_ms: f64,
    /// Current sync status
    pub current_status: SyncStatus,
    /// Connection quality (0.0 to 1.0)
    pub connection_quality: f64,
    /// Data freshness score (0.0 to 1.0)
    pub data_freshness: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Real-time events received
    pub realtime_events_received: u64,
    /// Network latency (milliseconds)
    pub network_latency_ms: f64,
}

/// Current synchronization status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Not started
    Idle,
    /// Currently synchronizing
    Syncing,
    /// Sync completed successfully
    Success,
    /// Sync failed with error
    Failed(String),
    /// Connection lost, using cached data
    Offline,
    /// Retrying after failure
    Retrying {
        attempt: u32,
        next_retry_in: Duration,
    },
}

/// Data source for synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// Backend API
    Api,
    /// Real-time WebSocket/SSE
    Realtime,
    /// Local cache
    Cache,
    /// Mock data for testing
    Mock,
}

/// Synchronization event for monitoring
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Sync operation started
    SyncStarted { source: DataSource },
    /// Sync operation completed successfully
    SyncCompleted {
        source: DataSource,
        duration: Duration,
        items_updated: u32,
    },
    /// Sync operation failed
    SyncFailed {
        source: DataSource,
        error: String,
        retry_in: Option<Duration>,
    },
    /// Connection status changed
    ConnectionChanged {
        old_status: ConnectionStatus,
        new_status: ConnectionStatus,
    },
    /// Real-time event received
    RealtimeEvent { event: StateChangeEvent },
    /// Cache operation performed
    CacheOperation {
        operation: String,
        success: bool,
        size_bytes: Option<u64>,
    },
    /// Data quality changed
    DataQualityChanged { freshness: f64, completeness: f64 },
}

/// Trait for data synchronization backends
#[async_trait]
pub trait SyncBackend: Send + Sync {
    /// Fetch workflows from the backend
    async fn fetch_workflows(&self) -> Result<Vec<WorkflowInfo>>;

    /// Fetch executions from the backend
    async fn fetch_executions(&self) -> Result<Vec<ExecutionInfo>>;

    /// Fetch tools from the backend
    async fn fetch_tools(&self) -> Result<Vec<ToolInfo>>;

    /// Fetch plugins from the backend
    async fn fetch_plugins(&self) -> Result<Vec<PluginInfo>>;

    /// Fetch system status from the backend
    async fn fetch_system_status(&self) -> Result<SystemStatus>;

    /// Fetch recent logs from the backend
    async fn fetch_logs(&self, since: Option<DateTime<Utc>>) -> Result<Vec<LogEntry>>;

    /// Check backend health/connectivity
    async fn health_check(&self) -> Result<bool>;

    /// Get backend connection info
    fn connection_info(&self) -> String;
}

/// Trait for offline caching
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Store data in cache (using JSON serialization)
    async fn store_bytes(&self, key: &str, data: &[u8], ttl: Duration) -> Result<()>;

    /// Retrieve data from cache (returns raw bytes)
    async fn retrieve_bytes(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Check if cache entry exists and is valid
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Remove cache entry
    async fn remove(&self, key: &str) -> Result<()>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;

    /// Store data with metadata for offline access (using JSON serialization)
    async fn store_bytes_with_metadata(
        &self,
        key: &str,
        data: &[u8],
        ttl: Duration,
        metadata: CacheMetadata,
    ) -> Result<()>;

    /// Retrieve data with metadata (returns raw bytes and metadata)
    async fn retrieve_bytes_with_metadata(
        &self,
        key: &str,
    ) -> Result<Option<(Vec<u8>, CacheMetadata)>>;

    /// Get all cache keys matching a pattern
    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>>;

    /// Get cache entry metadata without retrieving data
    async fn get_metadata(&self, key: &str) -> Result<Option<CacheMetadata>>;

    /// Update cache entry TTL
    async fn extend_ttl(&self, key: &str, additional_ttl: Duration) -> Result<bool>;

    /// Compact cache by removing expired entries
    async fn compact(&self) -> Result<CompactionResult>;

    /// Export cache data for backup
    async fn export(&self) -> Result<Vec<u8>>;

    /// Import cache data from backup
    async fn import(&self, data: &[u8]) -> Result<()>;
}

/// Helper trait for type-safe cache operations
#[async_trait]
pub trait CacheOperations {
    /// Store typed data in cache
    async fn store<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
    ) -> Result<()>;

    /// Retrieve typed data from cache
    async fn retrieve<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>>;

    /// Store typed data with metadata
    async fn store_with_metadata<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
        metadata: CacheMetadata,
    ) -> Result<()>;

    /// Retrieve typed data with metadata
    async fn retrieve_with_metadata<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<(T, CacheMetadata)>>;
}

/// Blanket implementation for all CacheBackend implementations
#[async_trait]
impl<B: CacheBackend + ?Sized> CacheOperations for B {
    async fn store<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(data)
            .map_err(|e| WorkflowError::validation(format!("Serialization error: {}", e)))?;
        self.store_bytes(key, &bytes, ttl).await
    }

    async fn retrieve<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        if let Some(bytes) = self.retrieve_bytes(key).await? {
            let data = serde_json::from_slice(&bytes)
                .map_err(|e| WorkflowError::validation(format!("Deserialization error: {}", e)))?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    async fn store_with_metadata<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
        metadata: CacheMetadata,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(data)
            .map_err(|e| WorkflowError::validation(format!("Serialization error: {}", e)))?;
        self.store_bytes_with_metadata(key, &bytes, ttl, metadata)
            .await
    }

    async fn retrieve_with_metadata<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<(T, CacheMetadata)>> {
        if let Some((bytes, metadata)) = self.retrieve_bytes_with_metadata(key).await? {
            let data = serde_json::from_slice(&bytes)
                .map_err(|e| WorkflowError::validation(format!("Deserialization error: {}", e)))?;
            Ok(Some((data, metadata)))
        } else {
            Ok(None)
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub total_size_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub expired_entries: u64,
    pub cache_hit_rate: f64,
    pub average_entry_size: f64,
    pub oldest_entry_age: Option<Duration>,
    pub newest_entry_age: Option<Duration>,
}

/// Cache entry metadata for offline functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    /// When the entry was last accessed
    pub last_accessed: DateTime<Utc>,
    /// When the entry expires
    pub expires_at: DateTime<Utc>,
    /// Number of times accessed
    pub access_count: u64,
    /// Size of the cached data in bytes
    pub size_bytes: u64,
    /// Data source that created this cache entry
    pub source: DataSource,
    /// Cache priority (higher = more important to keep)
    pub priority: CachePriority,
    /// Tags for categorizing cache entries
    pub tags: Vec<String>,
    /// Whether this entry can be used offline
    pub offline_capable: bool,
    /// Checksum for data integrity verification
    pub checksum: Option<String>,
}

/// Cache entry priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CachePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Cache compaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    /// Number of entries removed
    pub entries_removed: u64,
    /// Bytes freed
    pub bytes_freed: u64,
    /// Time taken for compaction
    pub duration: Duration,
    /// Number of entries remaining
    pub entries_remaining: u64,
    /// Total size after compaction
    pub size_after_bytes: u64,
}

/// Offline cache manager for enhanced offline functionality
pub struct OfflineCacheManager {
    /// Cache backend
    cache: Arc<dyn CacheBackend>,
    /// Configuration
    config: SyncConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Offline mode flag
    offline_mode: Arc<RwLock<bool>>,
    /// Cache invalidation rules
    invalidation_rules: Arc<RwLock<HashMap<String, CacheInvalidationRule>>>,
}

/// Cache invalidation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInvalidationRule {
    /// Pattern to match cache keys
    pub key_pattern: String,
    /// Maximum age before invalidation
    pub max_age: Duration,
    /// Whether to invalidate on data source change
    pub invalidate_on_source_change: bool,
    /// Custom invalidation conditions
    pub conditions: Vec<InvalidationCondition>,
}

/// Cache invalidation condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationCondition {
    /// Invalidate when system status changes
    SystemStatusChanged,
    /// Invalidate when workflow status changes
    WorkflowStatusChanged { workflow_name: String },
    /// Invalidate when execution completes
    ExecutionCompleted { execution_id: String },
    /// Invalidate after specific time
    TimeElapsed { duration: Duration },
    /// Invalidate when cache size exceeds limit
    CacheSizeExceeded { limit_bytes: u64 },
}

impl OfflineCacheManager {
    /// Create a new offline cache manager
    pub fn new(cache: Arc<dyn CacheBackend>, config: SyncConfig) -> Self {
        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            offline_mode: Arc::new(RwLock::new(false)),
            invalidation_rules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enable offline mode
    pub async fn enable_offline_mode(&self) -> Result<()> {
        *self.offline_mode.write().await = true;
        info!("Offline mode enabled");
        Ok(())
    }

    /// Disable offline mode
    pub async fn disable_offline_mode(&self) -> Result<()> {
        *self.offline_mode.write().await = false;
        info!("Offline mode disabled");
        Ok(())
    }

    /// Check if in offline mode
    pub async fn is_offline_mode(&self) -> bool {
        *self.offline_mode.read().await
    }

    /// Store data with enhanced metadata for offline access
    pub async fn store_offline_data<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
        priority: CachePriority,
        tags: Vec<String>,
    ) -> Result<()> {
        let metadata = CacheMetadata {
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at: Utc::now()
                + ChronoDuration::from_std(ttl)
                    .map_err(|e| WorkflowError::validation(e.to_string()))?,
            access_count: 0,
            size_bytes: 0, // Will be calculated by the cache backend
            source: DataSource::Cache,
            priority,
            tags,
            offline_capable: true,
            checksum: None, // Will be calculated by the cache backend
        };

        self.cache
            .store_with_metadata(key, data, ttl, metadata)
            .await?;

        // Update statistics
        self.update_stats_after_store().await;

        debug!("Stored offline data for key: {}", key);
        Ok(())
    }

    /// Retrieve data with offline fallback
    pub async fn retrieve_offline_data<T: for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<T>> {
        match self.cache.retrieve_with_metadata::<T>(key).await? {
            Some((data, mut metadata)) => {
                // Update access statistics
                metadata.last_accessed = Utc::now();
                metadata.access_count += 1;

                // Update statistics
                self.update_stats_after_access(true).await;

                debug!("Cache hit for offline data: {}", key);
                Ok(Some(data))
            }
            None => {
                // Update statistics
                self.update_stats_after_access(false).await;

                debug!("Cache miss for offline data: {}", key);
                Ok(None)
            }
        }
    }

    /// Get cache data freshness score (0.0 = stale, 1.0 = fresh)
    pub async fn get_data_freshness(&self, key: &str) -> Result<f64> {
        if let Some(metadata) = self.cache.get_metadata(key).await? {
            let age = Utc::now() - metadata.created_at;
            let max_age = metadata.expires_at - metadata.created_at;

            if age >= max_age {
                Ok(0.0) // Expired
            } else {
                let freshness = 1.0 - (age.num_seconds() as f64 / max_age.num_seconds() as f64);
                Ok(freshness.max(0.0).min(1.0))
            }
        } else {
            Ok(0.0) // Not found
        }
    }

    /// Preload critical data for offline access
    pub async fn preload_critical_data(
        &self,
        state: &Arc<SharedAppState>,
        backend: &Arc<dyn SyncBackend>,
    ) -> Result<()> {
        info!("Preloading critical data for offline access");

        // Preload workflows with high priority
        if let Ok(workflows) = backend.fetch_workflows().await {
            self.store_offline_data(
                "critical_workflows",
                &workflows,
                Duration::from_secs(self.config.cache_expiry_hours * 3600),
                CachePriority::Critical,
                vec!["workflows".to_string(), "critical".to_string()],
            )
            .await?;
        }

        // Preload system status with high priority
        if let Ok(system_status) = backend.fetch_system_status().await {
            self.store_offline_data(
                "critical_system_status",
                &system_status,
                Duration::from_secs(3600), // 1 hour for system status
                CachePriority::High,
                vec!["system".to_string(), "status".to_string()],
            )
            .await?;
        }

        // Preload recent executions
        if let Ok(executions) = backend.fetch_executions().await {
            self.store_offline_data(
                "critical_executions",
                &executions,
                Duration::from_secs(self.config.cache_expiry_hours * 3600),
                CachePriority::High,
                vec!["executions".to_string(), "critical".to_string()],
            )
            .await?;
        }

        info!("Critical data preloading completed");
        Ok(())
    }

    /// Load cached data when offline
    pub async fn load_offline_data(&self, state: &Arc<SharedAppState>) -> Result<()> {
        info!("Loading cached data for offline mode");

        // Load workflows from cache
        if let Some(workflows) = self
            .retrieve_offline_data::<Vec<WorkflowInfo>>("critical_workflows")
            .await?
        {
            state.set_workflows(workflows).await?;
            debug!("Loaded workflows from cache");
        }

        // Load system status from cache
        if let Some(system_status) = self
            .retrieve_offline_data::<SystemStatus>("critical_system_status")
            .await?
        {
            state.set_system_status(system_status).await?;
            debug!("Loaded system status from cache");
        }

        // Load executions from cache
        if let Some(executions) = self
            .retrieve_offline_data::<Vec<crate::interfaces::tui::state::ExecutionInfo>>(
                "critical_executions",
            )
            .await?
        {
            state.set_executions(executions).await?;
            debug!("Loaded executions from cache");
        }

        // Load tools from cache if available
        if let Some(tools) = self.retrieve_offline_data::<Vec<ToolInfo>>("tools").await? {
            state.set_tools(tools).await?;
            debug!("Loaded tools from cache");
        }

        // Load plugins from cache if available
        if let Some(plugins) = self
            .retrieve_offline_data::<Vec<PluginInfo>>("plugins")
            .await?
        {
            state.set_plugins(plugins).await?;
            debug!("Loaded plugins from cache");
        }

        info!("Offline data loading completed");
        Ok(())
    }

    /// Perform intelligent cache cleanup
    pub async fn intelligent_cleanup(&self) -> Result<CompactionResult> {
        info!("Performing intelligent cache cleanup");

        let start_time = std::time::Instant::now();
        let mut entries_removed = 0u64;
        let mut bytes_freed = 0u64;

        // Get all cache keys
        let keys = self.cache.list_keys(None).await?;

        for key in keys {
            if let Some(metadata) = self.cache.get_metadata(&key).await? {
                let should_remove = self.should_remove_entry(&metadata).await;

                if should_remove {
                    bytes_freed += metadata.size_bytes;
                    entries_removed += 1;
                    self.cache.remove(&key).await?;
                    debug!(
                        "Removed cache entry: {} ({}bytes)",
                        key, metadata.size_bytes
                    );
                }
            }
        }

        // Perform cache compaction
        let compaction_result = self.cache.compact().await?;

        let total_result = CompactionResult {
            entries_removed: entries_removed + compaction_result.entries_removed,
            bytes_freed: bytes_freed + compaction_result.bytes_freed,
            duration: start_time.elapsed(),
            entries_remaining: compaction_result.entries_remaining,
            size_after_bytes: compaction_result.size_after_bytes,
        };

        info!(
            "Cache cleanup completed: removed {} entries, freed {} bytes in {:?}",
            total_result.entries_removed, total_result.bytes_freed, total_result.duration
        );

        Ok(total_result)
    }

    /// Check if a cache entry should be removed
    async fn should_remove_entry(&self, metadata: &CacheMetadata) -> bool {
        let now = Utc::now();

        // Never remove critical entries unless they're very old
        if metadata.priority == CachePriority::Critical {
            let max_critical_age = ChronoDuration::days(7);
            return now - metadata.created_at > max_critical_age;
        }

        // Remove expired entries
        if now > metadata.expires_at {
            return true;
        }

        // Remove entries that haven't been accessed recently
        let last_access_threshold = match metadata.priority {
            CachePriority::Low => ChronoDuration::hours(6),
            CachePriority::Normal => ChronoDuration::days(1),
            CachePriority::High => ChronoDuration::days(3),
            CachePriority::Critical => ChronoDuration::days(7),
        };

        if now - metadata.last_accessed > last_access_threshold {
            return true;
        }

        // Remove entries with very low access count
        if metadata.access_count == 0 && now - metadata.created_at > ChronoDuration::hours(1) {
            return true;
        }

        false
    }

    /// Update statistics after storing data
    async fn update_stats_after_store(&self) {
        if let Ok(cache_stats) = self.cache.stats().await {
            let mut stats = self.stats.write().await;
            *stats = cache_stats;
        }
    }

    /// Update statistics after accessing data
    async fn update_stats_after_access(&self, hit: bool) {
        let mut stats = self.stats.write().await;
        if hit {
            stats.hit_count += 1;
        } else {
            stats.miss_count += 1;
        }

        // Calculate hit rate
        let total_accesses = stats.hit_count + stats.miss_count;
        if total_accesses > 0 {
            stats.cache_hit_rate = stats.hit_count as f64 / total_accesses as f64;
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Add cache invalidation rule
    pub async fn add_invalidation_rule(&self, name: String, rule: CacheInvalidationRule) {
        self.invalidation_rules.write().await.insert(name, rule);
    }

    /// Remove cache invalidation rule
    pub async fn remove_invalidation_rule(&self, name: &str) {
        self.invalidation_rules.write().await.remove(name);
    }

    /// Apply invalidation rules
    pub async fn apply_invalidation_rules(&self, event: &StateChangeEvent) -> Result<()> {
        let rules = self.invalidation_rules.read().await;

        for (name, rule) in rules.iter() {
            if self.should_invalidate_for_event(rule, event).await {
                debug!("Applying invalidation rule: {}", name);

                // Get keys matching the pattern
                let keys = self.cache.list_keys(Some(&rule.key_pattern)).await?;

                for key in keys {
                    self.cache.remove(&key).await?;
                    debug!("Invalidated cache entry: {}", key);
                }
            }
        }

        Ok(())
    }

    /// Check if cache should be invalidated for an event
    async fn should_invalidate_for_event(
        &self,
        rule: &CacheInvalidationRule,
        event: &StateChangeEvent,
    ) -> bool {
        for condition in &rule.conditions {
            match (condition, event) {
                (
                    InvalidationCondition::SystemStatusChanged,
                    StateChangeEvent::SystemStatusUpdated,
                ) => return true,
                (
                    InvalidationCondition::WorkflowStatusChanged { workflow_name },
                    StateChangeEvent::WorkflowStatusChanged { name, .. },
                ) if name == workflow_name => return true,
                (
                    InvalidationCondition::ExecutionCompleted { execution_id },
                    StateChangeEvent::ExecutionCompleted { id },
                ) if id == execution_id => return true,
                _ => {}
            }
        }

        false
    }

    /// Export cache for backup
    pub async fn export_cache(&self) -> Result<Vec<u8>> {
        self.cache.export().await
    }

    /// Import cache from backup
    pub async fn import_cache(&self, data: &[u8]) -> Result<()> {
        self.cache.import(data).await
    }
}
pub struct DataSyncManager {
    /// Shared application state
    state: Arc<SharedAppState>,
    /// Synchronization configuration
    config: SyncConfig,
    /// Backend for data fetching
    backend: Arc<dyn SyncBackend>,
    /// Cache backend for offline support
    cache: Option<Arc<dyn CacheBackend>>,
    /// Offline cache manager
    offline_cache: Option<Arc<OfflineCacheManager>>,
    /// Synchronization metrics
    metrics: Arc<RwLock<SyncMetrics>>,
    /// Event broadcaster for sync events
    event_sender: broadcast::Sender<SyncEvent>,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Current sync operation handle
    sync_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Real-time connection handle
    realtime_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Connection health monitor handle
    health_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Cache cleanup handle
    cleanup_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl DataSyncManager {
    /// Create a new data synchronization manager
    pub fn new(
        state: Arc<SharedAppState>,
        config: SyncConfig,
        backend: Arc<dyn SyncBackend>,
        cache: Option<Arc<dyn CacheBackend>>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        // Create offline cache manager if caching is enabled
        let offline_cache = if config.enable_offline_cache && cache.is_some() {
            Some(Arc::new(OfflineCacheManager::new(
                cache.as_ref().unwrap().clone(),
                config.clone(),
            )))
        } else {
            None
        };

        Self {
            state,
            config,
            backend,
            cache,
            offline_cache,
            metrics: Arc::new(RwLock::new(SyncMetrics::default())),
            event_sender,
            shutdown_tx: None,
            sync_handle: Arc::new(Mutex::new(None)),
            realtime_handle: Arc::new(Mutex::new(None)),
            health_handle: Arc::new(Mutex::new(None)),
            cleanup_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the synchronization manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting data synchronization manager");

        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start periodic sync task
        let (sync_shutdown_tx, sync_shutdown_rx) = mpsc::channel(1);
        let sync_handle = self.start_periodic_sync(sync_shutdown_rx).await?;
        *self.sync_handle.lock().await = Some(sync_handle);

        // Start real-time updates if enabled
        if self.config.enable_realtime_updates {
            let (realtime_shutdown_tx, realtime_shutdown_rx) = mpsc::channel(1);
            let realtime_handle = self.start_realtime_updates(realtime_shutdown_rx).await?;
            *self.realtime_handle.lock().await = Some(realtime_handle);
        }

        // Start connection health monitoring
        let (health_shutdown_tx, health_shutdown_rx) = mpsc::channel(1);
        let health_handle = self.start_health_monitoring(health_shutdown_rx).await?;
        *self.health_handle.lock().await = Some(health_handle);

        // Start cache cleanup task if offline caching is enabled
        if self.config.enable_offline_cache && self.offline_cache.is_some() {
            let (cleanup_shutdown_tx, cleanup_shutdown_rx) = mpsc::channel(1);
            let cleanup_handle = self.start_cache_cleanup(cleanup_shutdown_rx).await?;
            *self.cleanup_handle.lock().await = Some(cleanup_handle);
        }

        // Perform initial sync or load from cache
        match self.sync_all_data(DataSource::Api).await {
            Ok(_) => {
                // Preload critical data for offline access
                if let Some(offline_cache) = &self.offline_cache {
                    if let Err(e) = offline_cache
                        .preload_critical_data(&self.state, &self.backend)
                        .await
                    {
                        warn!("Failed to preload critical data: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Initial sync failed: {}, trying to load from cache", e);
                if let Some(offline_cache) = &self.offline_cache {
                    if let Err(cache_err) = offline_cache.load_offline_data(&self.state).await {
                        error!("Failed to load from cache: {}", cache_err);
                        return Err(e); // Return original sync error
                    } else {
                        info!("Successfully loaded data from cache");
                        offline_cache.enable_offline_mode().await?;
                    }
                }
            }
        }

        info!("Data synchronization manager started successfully");
        Ok(())
    }

    /// Stop the synchronization manager
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping data synchronization manager");

        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        // Wait for tasks to complete
        if let Some(handle) = self.sync_handle.lock().await.take() {
            handle.abort();
        }

        if let Some(handle) = self.realtime_handle.lock().await.take() {
            handle.abort();
        }

        if let Some(handle) = self.health_handle.lock().await.take() {
            handle.abort();
        }

        if let Some(handle) = self.cleanup_handle.lock().await.take() {
            handle.abort();
        }

        info!("Data synchronization manager stopped");
        Ok(())
    }

    /// Subscribe to synchronization events
    pub fn subscribe_events(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_sender.subscribe()
    }

    /// Get current synchronization metrics
    pub async fn get_metrics(&self) -> SyncMetrics {
        let mut metrics = self.metrics.read().await.clone();

        // Update cache hit rate from offline cache if available
        if let Some(offline_cache) = &self.offline_cache {
            let cache_stats = offline_cache.get_stats().await;
            metrics.cache_hit_rate = cache_stats.cache_hit_rate;
        }

        metrics
    }

    /// Manually trigger a full data synchronization
    pub async fn sync_now(&self) -> Result<()> {
        info!("Manual sync triggered");
        self.sync_all_data(DataSource::Api).await
    }

    /// Force refresh from backend (bypass cache)
    pub async fn force_refresh(&self) -> Result<()> {
        info!("Force refresh triggered");

        // Clear cache if available
        if let Some(cache) = &self.cache {
            cache.clear().await?;
            self.broadcast_event(SyncEvent::CacheOperation {
                operation: "clear_all".to_string(),
                success: true,
                size_bytes: None,
            })
            .await;
        }

        // Disable offline mode temporarily
        if let Some(offline_cache) = &self.offline_cache {
            offline_cache.disable_offline_mode().await?;
        }

        // Sync from API
        let result = self.sync_all_data(DataSource::Api).await;

        // Re-enable offline mode if sync failed
        if result.is_err() {
            if let Some(offline_cache) = &self.offline_cache {
                offline_cache.enable_offline_mode().await?;
                offline_cache.load_offline_data(&self.state).await?;
            }
        }

        result
    }

    /// Enable offline mode
    pub async fn enable_offline_mode(&self) -> Result<()> {
        if let Some(offline_cache) = &self.offline_cache {
            offline_cache.enable_offline_mode().await?;
            offline_cache.load_offline_data(&self.state).await?;
            info!("Offline mode enabled and data loaded from cache");
        } else {
            warn!("Offline mode not available - cache not configured");
        }
        Ok(())
    }

    /// Disable offline mode
    pub async fn disable_offline_mode(&self) -> Result<()> {
        if let Some(offline_cache) = &self.offline_cache {
            offline_cache.disable_offline_mode().await?;
            info!("Offline mode disabled");
        }
        Ok(())
    }

    /// Check if in offline mode
    pub async fn is_offline_mode(&self) -> bool {
        if let Some(offline_cache) = &self.offline_cache {
            offline_cache.is_offline_mode().await
        } else {
            false
        }
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Option<CacheStats> {
        if let Some(offline_cache) = &self.offline_cache {
            Some(offline_cache.get_stats().await)
        } else {
            None
        }
    }

    /// Perform cache cleanup
    pub async fn cleanup_cache(&self) -> Result<Option<CompactionResult>> {
        if let Some(offline_cache) = &self.offline_cache {
            Ok(Some(offline_cache.intelligent_cleanup().await?))
        } else {
            Ok(None)
        }
    }

    /// Export cache for backup
    pub async fn export_cache(&self) -> Result<Option<Vec<u8>>> {
        if let Some(offline_cache) = &self.offline_cache {
            Ok(Some(offline_cache.export_cache().await?))
        } else {
            Ok(None)
        }
    }

    /// Import cache from backup
    pub async fn import_cache(&self, data: &[u8]) -> Result<()> {
        if let Some(offline_cache) = &self.offline_cache {
            offline_cache.import_cache(data).await?;
            offline_cache.load_offline_data(&self.state).await?;
            info!("Cache imported and data loaded");
        }
        Ok(())
    }

    /// Start periodic synchronization task
    async fn start_periodic_sync(
        &self,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let state = Arc::clone(&self.state);
        let backend = Arc::clone(&self.backend);
        let cache = self.cache.clone();
        let metrics = Arc::clone(&self.metrics);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.sync_interval_seconds));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = Self::perform_sync(
                            &state,
                            &backend,
                            &cache,
                            &metrics,
                            &event_sender,
                            &config,
                            DataSource::Api,
                        ).await {
                            error!("Periodic sync failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Periodic sync task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Start real-time updates task
    async fn start_realtime_updates(
        &self,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let state = Arc::clone(&self.state);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            // This would connect to WebSocket/SSE endpoint for real-time updates
            // For now, simulate with periodic checks for demonstration
            let mut interval = interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Simulate real-time event
                        let event = StateChangeEvent::DataRefreshed;
                        let sync_event = SyncEvent::RealtimeEvent { event: event.clone() };

                        if let Err(e) = event_sender.send(sync_event) {
                            warn!("Failed to broadcast real-time event: {}", e);
                        }

                        trace!("Real-time update check completed");
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Real-time updates task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Start cache cleanup task
    async fn start_cache_cleanup(
        &self,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let offline_cache = self.offline_cache.clone();
        let config = self.config.clone();
        let event_sender = self.event_sender.clone();

        let handle = tokio::spawn(async move {
            // Run cleanup every hour
            let mut interval = interval(Duration::from_secs(3600));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Some(cache) = &offline_cache {
                            match cache.intelligent_cleanup().await {
                                Ok(result) => {
                                    let _ = event_sender.send(SyncEvent::CacheOperation {
                                        operation: "cleanup".to_string(),
                                        success: true,
                                        size_bytes: Some(result.bytes_freed),
                                    });

                                    debug!(
                                        "Cache cleanup completed: {} entries removed, {} bytes freed",
                                        result.entries_removed,
                                        result.bytes_freed
                                    );
                                }
                                Err(e) => {
                                    error!("Cache cleanup failed: {}", e);
                                    let _ = event_sender.send(SyncEvent::CacheOperation {
                                        operation: "cleanup".to_string(),
                                        success: false,
                                        size_bytes: None,
                                    });
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Cache cleanup task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }
    async fn start_health_monitoring(
        &self,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let state = Arc::clone(&self.state);
        let backend = Arc::clone(&self.backend);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.health_check_interval_seconds));
            let mut last_status = ConnectionStatus::Disconnected;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let new_status = match timeout(
                            Duration::from_secs(config.connection_timeout_seconds),
                            backend.health_check()
                        ).await {
                            Ok(Ok(true)) => ConnectionStatus::Connected,
                            Ok(Ok(false)) => ConnectionStatus::Disconnected,
                            Ok(Err(e)) => ConnectionStatus::Error(e.to_string()),
                            Err(_) => ConnectionStatus::Error("Health check timeout".to_string()),
                        };

                        if new_status != last_status {
                            if let Err(e) = state.set_connection_status(new_status.clone()).await {
                                error!("Failed to update connection status: {}", e);
                            }

                            let sync_event = SyncEvent::ConnectionChanged {
                                old_status: last_status.clone(),
                                new_status: new_status.clone(),
                            };

                            if let Err(e) = event_sender.send(sync_event) {
                                warn!("Failed to broadcast connection change: {}", e);
                            }

                            last_status = new_status;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("Health monitoring task shutting down");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Perform comprehensive data synchronization
    async fn sync_all_data(&self, source: DataSource) -> Result<()> {
        Self::perform_sync(
            &self.state,
            &self.backend,
            &self.cache,
            &self.metrics,
            &self.event_sender,
            &self.config,
            source,
        )
        .await
    }

    /// Internal sync implementation with retry logic
    async fn perform_sync(
        state: &Arc<SharedAppState>,
        backend: &Arc<dyn SyncBackend>,
        cache: &Option<Arc<dyn CacheBackend>>,
        metrics: &Arc<RwLock<SyncMetrics>>,
        event_sender: &broadcast::Sender<SyncEvent>,
        config: &SyncConfig,
        source: DataSource,
    ) -> Result<()> {
        let start_time = Instant::now();
        let mut attempt = 0;
        let mut last_error: Option<String> = None;

        // Update metrics - sync started
        {
            let mut m = metrics.write().await;
            m.current_status = SyncStatus::Syncing;
            m.total_syncs += 1;
        }

        // Broadcast sync started event
        let _ = event_sender.send(SyncEvent::SyncStarted {
            source: source.clone(),
        });

        loop {
            attempt += 1;

            match Self::perform_single_sync(state, backend, cache, source.clone()).await {
                Ok(items_updated) => {
                    let duration = start_time.elapsed();

                    // Update metrics - sync succeeded
                    {
                        let mut m = metrics.write().await;
                        m.successful_syncs += 1;
                        m.current_status = SyncStatus::Success;
                        m.last_sync_time = Some(Utc::now());
                        m.last_successful_sync = Some(Utc::now());

                        // Update average duration
                        let total_duration =
                            m.average_sync_duration_ms * (m.successful_syncs - 1) as f64;
                        m.average_sync_duration_ms = (total_duration + duration.as_millis() as f64)
                            / m.successful_syncs as f64;

                        // Update data freshness
                        m.data_freshness = 1.0; // Fresh data

                        // Update connection quality based on success
                        m.connection_quality = (m.connection_quality * 0.9 + 0.1).min(1.0);
                    }

                    // Broadcast sync completed event
                    let _ = event_sender.send(SyncEvent::SyncCompleted {
                        source,
                        duration,
                        items_updated,
                    });

                    info!(
                        "Data sync completed successfully in {:?}, {} items updated",
                        duration, items_updated
                    );
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e.to_string());

                    if attempt >= config.max_retry_attempts {
                        // Update metrics - sync failed
                        {
                            let mut m = metrics.write().await;
                            m.failed_syncs += 1;
                            m.current_status = SyncStatus::Failed(e.to_string());
                            m.last_sync_time = Some(Utc::now());

                            // Decrease connection quality
                            m.connection_quality = (m.connection_quality * 0.8).max(0.0);

                            // Decrease data freshness over time
                            if let Some(last_success) = m.last_successful_sync {
                                let age = Utc::now() - last_success;
                                m.data_freshness =
                                    (1.0 - (age.num_minutes() as f64 / 60.0)).max(0.0);
                            }
                        }

                        // Broadcast sync failed event
                        let _ = event_sender.send(SyncEvent::SyncFailed {
                            source,
                            error: e.to_string(),
                            retry_in: None,
                        });

                        error!("Data sync failed after {} attempts: {}", attempt, e);

                        // Try to load from cache if available
                        if let Some(cache_backend) = cache {
                            if let Err(cache_err) =
                                Self::load_from_cache(state, cache_backend).await
                            {
                                warn!("Failed to load from cache: {}", cache_err);
                            } else {
                                info!("Loaded data from cache after sync failure");

                                // Update status to offline
                                let mut m = metrics.write().await;
                                m.current_status = SyncStatus::Offline;
                            }
                        }

                        return Err(e);
                    } else {
                        // Calculate retry delay with exponential backoff
                        let delay_ms = config.initial_retry_delay_ms as f64
                            * config.retry_backoff_multiplier.powi((attempt - 1) as i32);
                        let retry_delay = Duration::from_millis(delay_ms as u64);

                        // Update metrics - retrying
                        {
                            let mut m = metrics.write().await;
                            m.current_status = SyncStatus::Retrying {
                                attempt,
                                next_retry_in: retry_delay,
                            };
                        }

                        // Broadcast retry event
                        let _ = event_sender.send(SyncEvent::SyncFailed {
                            source: source.clone(),
                            error: e.to_string(),
                            retry_in: Some(retry_delay),
                        });

                        warn!(
                            "Sync attempt {} failed: {}, retrying in {:?}",
                            attempt, e, retry_delay
                        );
                        sleep(retry_delay).await;
                    }
                }
            }
        }
    }

    /// Perform a single synchronization attempt
    async fn perform_single_sync(
        state: &Arc<SharedAppState>,
        backend: &Arc<dyn SyncBackend>,
        cache: &Option<Arc<dyn CacheBackend>>,
        source: DataSource,
    ) -> Result<u32> {
        let mut items_updated = 0;

        match source {
            DataSource::Api => {
                // Fetch data from backend API
                let (workflows, executions, tools, plugins, system_status, logs) = tokio::try_join!(
                    backend.fetch_workflows(),
                    backend.fetch_executions(),
                    backend.fetch_tools(),
                    backend.fetch_plugins(),
                    backend.fetch_system_status(),
                    backend.fetch_logs(None)
                )?;

                // Update state
                state.set_workflows(workflows.clone()).await?;
                state.set_executions(executions.clone()).await?;
                state.set_tools(tools.clone()).await?;
                state.set_plugins(plugins.clone()).await?;
                state.set_system_status(system_status.clone()).await?;

                // Add logs
                for log in logs {
                    state.add_log_entry(log).await?;
                }

                items_updated = workflows.len() as u32
                    + executions.len() as u32
                    + tools.len() as u32
                    + plugins.len() as u32
                    + 1; // +1 for system status

                // Cache the data if caching is enabled
                if let Some(cache_backend) = cache {
                    let cache_ttl = Duration::from_secs(3600); // 1 hour

                    let _ = cache_backend
                        .store("workflows", &workflows, cache_ttl)
                        .await;
                    let _ = cache_backend
                        .store("executions", &executions, cache_ttl)
                        .await;
                    let _ = cache_backend.store("tools", &tools, cache_ttl).await;
                    let _ = cache_backend.store("plugins", &plugins, cache_ttl).await;
                    let _ = cache_backend
                        .store("system_status", &system_status, cache_ttl)
                        .await;
                }
            }
            DataSource::Cache => {
                if let Some(cache_backend) = cache {
                    Self::load_from_cache(state, cache_backend).await?;
                    items_updated = 1; // Approximate
                } else {
                    return Err(WorkflowError::ValidationError(
                        "Cache not available".to_string(),
                    ));
                }
            }
            DataSource::Mock => {
                // Load mock data for testing
                Self::load_mock_data(state).await?;
                items_updated = 10; // Mock count
            }
            DataSource::Realtime => {
                // Real-time updates would be handled separately
                items_updated = 0;
            }
        }

        Ok(items_updated)
    }

    /// Load data from cache
    async fn load_from_cache(
        state: &Arc<SharedAppState>,
        cache: &Arc<dyn CacheBackend>,
    ) -> Result<()> {
        let workflows: Option<Vec<WorkflowInfo>> = cache.retrieve("workflows").await?;
        let executions: Option<Vec<ExecutionInfo>> = cache.retrieve("executions").await?;
        let tools: Option<Vec<ToolInfo>> = cache.retrieve("tools").await?;
        let plugins: Option<Vec<PluginInfo>> = cache.retrieve("plugins").await?;
        let system_status: Option<SystemStatus> = cache.retrieve("system_status").await?;

        if let Some(workflows) = workflows {
            state.set_workflows(workflows).await?;
        }

        if let Some(executions) = executions {
            state.set_executions(executions).await?;
        }

        if let Some(tools) = tools {
            state.set_tools(tools).await?;
        }

        if let Some(plugins) = plugins {
            state.set_plugins(plugins).await?;
        }

        if let Some(system_status) = system_status {
            state.set_system_status(system_status).await?;
        }

        Ok(())
    }

    /// Load mock data for testing
    async fn load_mock_data(state: &Arc<SharedAppState>) -> Result<()> {
        use crate::interfaces::tui::state::{NetworkStatus, SystemHealth};
        use crate::interfaces::tui::widgets::workflow_list::{ExecutionStatus, WorkflowStatus};

        // Mock workflows
        let workflows = vec![
            WorkflowInfo {
                name: "数据处理流水线".to_string(),
                version: "1.0.0".to_string(),
                description: Some("处理和转换数据的工作流".to_string()),
                status: WorkflowStatus::Available,
                last_execution: Some(Utc::now() - ChronoDuration::hours(2)),
                execution_count: 15,
                tags: vec!["数据".to_string(), "ETL".to_string()],
                node_count: 5,
                estimated_duration: Some(Duration::from_secs(300)),
                success_rate: Some(0.95),
            },
            WorkflowInfo {
                name: "系统监控".to_string(),
                version: "2.1.0".to_string(),
                description: Some("监控系统健康状态".to_string()),
                status: WorkflowStatus::Running,
                last_execution: Some(Utc::now() - ChronoDuration::minutes(5)),
                execution_count: 142,
                tags: vec!["监控".to_string(), "系统".to_string()],
                node_count: 3,
                estimated_duration: Some(Duration::from_secs(60)),
                success_rate: Some(0.98),
            },
        ];

        // Mock executions
        let executions = vec![ExecutionInfo {
            id: "exec-001".to_string(),
            workflow_name: "系统监控".to_string(),
            status: ExecutionStatus::Running,
            started_at: Utc::now() - ChronoDuration::minutes(5),
            completed_at: None,
            progress: 0.65,
        }];

        // Mock system status
        let system_status = SystemStatus {
            cpu_usage: 45.2,
            memory_usage: 62.8,
            memory_total: 16 * 1024 * 1024 * 1024, // 16GB
            memory_used: 10 * 1024 * 1024 * 1024,  // 10GB
            disk_usage: 78.5,
            disk_total: 500 * 1024 * 1024 * 1024, // 500GB
            disk_used: 392 * 1024 * 1024 * 1024,  // 392GB
            active_workflows: 3,
            system_health: SystemHealth::Healthy,
            uptime: Duration::from_secs(86400 * 7), // 7 days
            network_status: NetworkStatus::Connected,
            load_average: [1.2, 1.5, 1.8],
            process_count: 245,
            thread_count: 1024,
        };

        // Update state
        state.set_workflows(workflows).await?;
        state.set_executions(executions).await?;
        state.set_tools(vec![]).await?; // Empty for mock
        state.set_plugins(vec![]).await?; // Empty for mock
        state.set_system_status(system_status).await?;

        Ok(())
    }

    /// Broadcast a synchronization event
    async fn broadcast_event(&self, event: SyncEvent) {
        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to broadcast sync event: {}", e);
        }
    }
}

impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            total_syncs: 0,
            successful_syncs: 0,
            failed_syncs: 0,
            last_sync_time: None,
            last_successful_sync: None,
            average_sync_duration_ms: 0.0,
            current_status: SyncStatus::Idle,
            connection_quality: 1.0,
            data_freshness: 1.0,
            cache_hit_rate: 0.0,
            realtime_events_received: 0,
            network_latency_ms: 0.0,
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            total_size_bytes: 0,
            hit_count: 0,
            miss_count: 0,
            eviction_count: 0,
            expired_entries: 0,
            cache_hit_rate: 0.0,
            average_entry_size: 0.0,
            oldest_entry_age: None,
            newest_entry_age: None,
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Idle => write!(f, "空闲"),
            SyncStatus::Syncing => write!(f, "同步中"),
            SyncStatus::Success => write!(f, "成功"),
            SyncStatus::Failed(err) => write!(f, "失败: {}", err),
            SyncStatus::Offline => write!(f, "离线"),
            SyncStatus::Retrying {
                attempt,
                next_retry_in,
            } => {
                write!(
                    f,
                    "重试中 ({}/3) - {}秒后重试",
                    attempt,
                    next_retry_in.as_secs()
                )
            }
        }
    }
}
