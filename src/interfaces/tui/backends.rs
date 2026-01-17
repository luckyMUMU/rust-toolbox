//! Backend implementations for TUI data synchronization
//!
//! This module provides concrete implementations of sync and cache backends
//! for the TUI data synchronization system.

use crate::core::{PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::interfaces::tui::action::LogLevel;
use crate::interfaces::tui::state::SystemStatus;
use crate::interfaces::tui::sync::{
    CacheBackend, CacheMetadata, CachePriority, CacheStats, CompactionResult, DataSource,
    SyncBackend,
};
use crate::interfaces::tui::widgets::log_viewer::LogEntry;
use crate::interfaces::tui::widgets::workflow_list::{
    ExecutionStatus, WorkflowInfo, WorkflowStatus,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Mock backend for testing and development
pub struct MockSyncBackend {
    /// Simulated network latency
    latency: Duration,
    /// Failure rate (0.0 to 1.0)
    failure_rate: f64,
    /// Connection status
    connected: Arc<RwLock<bool>>,
}

impl MockSyncBackend {
    /// Create a new mock backend
    pub fn new() -> Self {
        Self {
            latency: Duration::from_millis(100),
            failure_rate: 0.0,
            connected: Arc::new(RwLock::new(true)),
        }
    }

    /// Create a mock backend with custom settings
    pub fn with_settings(latency: Duration, failure_rate: f64) -> Self {
        Self {
            latency,
            failure_rate,
            connected: Arc::new(RwLock::new(true)),
        }
    }

    /// Set connection status
    pub async fn set_connected(&self, connected: bool) {
        *self.connected.write().await = connected;
    }

    /// Simulate network delay
    async fn simulate_delay(&self) {
        tokio::time::sleep(self.latency).await;
    }

    /// Check if operation should fail
    fn should_fail(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.failure_rate
    }
}

#[async_trait]
impl SyncBackend for MockSyncBackend {
    async fn fetch_workflows(&self) -> Result<Vec<WorkflowInfo>> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        Ok(vec![
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
            WorkflowInfo {
                name: "文件备份".to_string(),
                version: "1.2.1".to_string(),
                description: Some("定期备份重要文件".to_string()),
                status: WorkflowStatus::Completed,
                last_execution: Some(Utc::now() - ChronoDuration::hours(1)),
                execution_count: 87,
                tags: vec!["备份".to_string(), "文件".to_string()],
                node_count: 4,
                estimated_duration: Some(Duration::from_secs(180)),
                success_rate: Some(0.92),
            },
        ])
    }

    async fn fetch_executions(&self) -> Result<Vec<crate::interfaces::tui::state::ExecutionInfo>> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        Ok(vec![
            crate::interfaces::tui::state::ExecutionInfo {
                id: "exec-001".to_string(),
                workflow_name: "系统监控".to_string(),
                status: ExecutionStatus::Running,
                started_at: Utc::now() - ChronoDuration::minutes(5),
                completed_at: None,
                progress: 0.65,
            },
            crate::interfaces::tui::state::ExecutionInfo {
                id: "exec-002".to_string(),
                workflow_name: "文件备份".to_string(),
                status: ExecutionStatus::Completed,
                started_at: Utc::now() - ChronoDuration::hours(1),
                completed_at: Some(Utc::now() - ChronoDuration::minutes(45)),
                progress: 1.0,
            },
        ])
    }

    async fn fetch_tools(&self) -> Result<Vec<ToolInfo>> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        Ok(vec![
            ToolInfo {
                name: "file_processor".to_string(),
                version: "1.0.0".to_string(),
                description: "Process files with various operations".to_string(),
                category: Some("File Management".to_string()),
                tags: vec!["file".to_string(), "processing".to_string()],
                parameters_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                plugin_name: Some("file_management".to_string()),
                dependencies: vec![],
                version_requirements: HashMap::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            ToolInfo {
                name: "data_transformer".to_string(),
                version: "2.1.0".to_string(),
                description: "Transform data between formats".to_string(),
                category: Some("Data Processing".to_string()),
                tags: vec!["data".to_string(), "transform".to_string()],
                parameters_schema: serde_json::json!({}),
                return_schema: serde_json::json!({}),
                plugin_name: Some("python_tools".to_string()),
                dependencies: vec![],
                version_requirements: HashMap::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ])
    }

    async fn fetch_plugins(&self) -> Result<Vec<PluginInfo>> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        Ok(vec![
            PluginInfo {
                name: "file_management".to_string(),
                version: "1.0.0".to_string(),
                plugin_type: crate::core::PluginType::Native,
                description: Some("File management operations".to_string()),
                author: Some("Workflow Toolkit Team".to_string()),
                metadata: HashMap::new(),
            },
            PluginInfo {
                name: "python_tools".to_string(),
                version: "1.2.0".to_string(),
                plugin_type: crate::core::PluginType::Python,
                description: Some("Python-based tools and utilities".to_string()),
                author: Some("Community".to_string()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "dependencies".to_string(),
                        serde_json::json!(["python>=3.8"]),
                    );
                    meta
                },
            },
        ])
    }

    async fn fetch_system_status(&self) -> Result<SystemStatus> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        use crate::interfaces::tui::state::{NetworkStatus, SystemHealth};
        use rand::Rng;

        let mut rng = rand::thread_rng();

        Ok(SystemStatus {
            cpu_usage: rng.gen_range(20.0..80.0),
            memory_usage: rng.gen_range(40.0..90.0),
            memory_total: 16 * 1024 * 1024 * 1024, // 16GB
            memory_used: rng.gen_range(6..14) * 1024 * 1024 * 1024, // 6-14GB
            disk_usage: rng.gen_range(50.0..95.0),
            disk_total: 500 * 1024 * 1024 * 1024, // 500GB
            disk_used: rng.gen_range(250..475) * 1024 * 1024 * 1024, // 250-475GB
            active_workflows: rng.gen_range(1..5),
            system_health: if rng.gen_bool(0.8) {
                SystemHealth::Healthy
            } else {
                SystemHealth::Warning
            },
            uptime: Duration::from_secs(rng.gen_range(3600..604800)), // 1 hour to 1 week
            network_status: NetworkStatus::Connected,
            load_average: [
                rng.gen_range(0.5..3.0),
                rng.gen_range(0.5..3.0),
                rng.gen_range(0.5..3.0),
            ],
            process_count: rng.gen_range(200..400),
            thread_count: rng.gen_range(800..1500),
        })
    }

    async fn fetch_logs(&self, since: Option<DateTime<Utc>>) -> Result<Vec<LogEntry>> {
        self.simulate_delay().await;

        if !*self.connected.read().await {
            return Err(WorkflowError::validation("Not connected"));
        }

        if self.should_fail() {
            return Err(WorkflowError::validation("Simulated network failure"));
        }

        let cutoff = since.unwrap_or_else(|| Utc::now() - ChronoDuration::hours(1));
        let mut logs = Vec::new();

        // Generate some mock log entries
        for i in 0..10 {
            let timestamp = Utc::now() - ChronoDuration::minutes(i * 5);
            if timestamp > cutoff {
                logs.push(LogEntry {
                    id: format!("log_{}", i),
                    timestamp,
                    level: match i % 4 {
                        0 => LogLevel::Info,
                        1 => LogLevel::Debug,
                        2 => LogLevel::Warn,
                        3 => LogLevel::Error,
                        _ => LogLevel::Info,
                    },
                    source: Some(format!("component_{}", i % 3)),
                    message: format!("Mock log message {} - System operation completed", i),
                    execution_id: Some(format!("exec_{}", i % 2)),
                    workflow_id: Some(format!("workflow_{}", i % 3)),
                    node_id: Some(format!("node_{}", i % 4)),
                });
            }
        }

        Ok(logs)
    }

    async fn health_check(&self) -> Result<bool> {
        self.simulate_delay().await;
        Ok(*self.connected.read().await)
    }

    fn connection_info(&self) -> String {
        "Mock Backend (localhost:mock)".to_string()
    }
}

impl Default for MockSyncBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// In-memory cache backend for development and testing
pub struct MemoryCacheBackend {
    /// Cache storage
    storage: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Maximum cache size in bytes
    max_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: SystemTime,
    created_at: SystemTime,
    access_count: u64,
    metadata: Option<CacheMetadata>,
}

impl MemoryCacheBackend {
    /// Create a new memory cache backend
    pub fn new(max_size_mb: u64) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            max_size_bytes: max_size_mb * 1024 * 1024,
        }
    }

    /// Evict expired entries
    async fn evict_expired(&self) {
        let now = SystemTime::now();
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        let initial_count = storage.len();
        storage.retain(|_, entry| entry.expires_at > now);

        let evicted = initial_count - storage.len();
        if evicted > 0 {
            stats.eviction_count += evicted as u64;
            stats.total_entries = storage.len() as u64;

            // Recalculate total size
            stats.total_size_bytes = storage.values().map(|entry| entry.data.len() as u64).sum();

            debug!("Evicted {} expired cache entries", evicted);
        }
    }

    /// Evict entries to make space
    async fn evict_for_space(&self, needed_bytes: u64) {
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        if stats.total_size_bytes + needed_bytes <= self.max_size_bytes {
            return; // No eviction needed
        }

        // Collect entries to evict (LRU-like)
        let mut entries_to_evict: Vec<String> = storage
            .iter()
            .map(|(key, entry)| (key.clone(), entry.access_count))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(key, _)| key)
            .collect();

        // Sort by access count (least recently used first)
        entries_to_evict.sort_by_key(|key| storage.get(key).map(|e| e.access_count).unwrap_or(0));

        let mut freed_bytes = 0u64;
        let mut evicted_count = 0u64;

        for key in entries_to_evict {
            if stats.total_size_bytes - freed_bytes + needed_bytes <= self.max_size_bytes {
                break;
            }

            if let Some(entry) = storage.remove(&key) {
                freed_bytes += entry.data.len() as u64;
                evicted_count += 1;
            }
        }

        if evicted_count > 0 {
            stats.eviction_count += evicted_count;
            stats.total_entries = storage.len() as u64;
            stats.total_size_bytes -= freed_bytes;

            debug!(
                "Evicted {} cache entries to free {} bytes",
                evicted_count, freed_bytes
            );
        }
    }
}

#[async_trait]
impl CacheBackend for MemoryCacheBackend {
    async fn store_bytes(&self, key: &str, data: &[u8], ttl: Duration) -> Result<()> {
        let data_size = data.len() as u64;

        // Evict expired entries first
        self.evict_expired().await;

        // Evict entries if needed to make space
        self.evict_for_space(data_size).await;

        let now = SystemTime::now();
        let expires_at = now + ttl;

        let entry = CacheEntry {
            data: data.to_vec(),
            expires_at,
            created_at: now,
            access_count: 0,
            metadata: None,
        };

        // Store entry
        {
            let mut storage = self.storage.write().await;
            let mut stats = self.stats.write().await;

            let is_new = !storage.contains_key(key);
            storage.insert(key.to_string(), entry);

            if is_new {
                stats.total_entries += 1;
            }
            stats.total_size_bytes += data_size;
        }

        debug!("Cached {} bytes for key: {}", data_size, key);
        Ok(())
    }

    async fn retrieve_bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Evict expired entries first
        self.evict_expired().await;

        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = storage.get(key) {
            let now = SystemTime::now();

            // Check if entry is still valid
            if entry.expires_at > now {
                // Update access count and stats
                let data = entry.data.clone();
                if let Some(entry) = storage.get_mut(key) {
                    entry.access_count += 1;
                }
                stats.hit_count += 1;

                debug!("Cache hit for key: {}", key);
                Ok(Some(data))
            } else {
                // Entry expired, remove it
                let entry = storage.remove(key).unwrap();
                stats.total_entries -= 1;
                stats.total_size_bytes -= entry.data.len() as u64;
                stats.miss_count += 1;

                debug!("Cache miss (expired) for key: {}", key);
                Ok(None)
            }
        } else {
            stats.miss_count += 1;
            debug!("Cache miss for key: {}", key);
            Ok(None)
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.evict_expired().await;

        let storage = self.storage.read().await;
        let exists = storage.contains_key(key);

        Ok(exists)
    }

    async fn remove(&self, key: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = storage.remove(key) {
            stats.total_entries -= 1;
            stats.total_size_bytes -= entry.data.len() as u64;
            debug!("Removed cache entry for key: {}", key);
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        let cleared_count = storage.len();
        storage.clear();

        stats.total_entries = 0;
        stats.total_size_bytes = 0;
        stats.eviction_count += cleared_count as u64;

        info!("Cleared {} cache entries", cleared_count);
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        self.evict_expired().await;
        let storage = self.storage.read().await;
        let stats = self.stats.read().await.clone();

        // Calculate additional statistics
        let total_accesses = stats.hit_count + stats.miss_count;
        let cache_hit_rate = if total_accesses > 0 {
            stats.hit_count as f64 / total_accesses as f64
        } else {
            0.0
        };

        let average_entry_size = if stats.total_entries > 0 {
            stats.total_size_bytes as f64 / stats.total_entries as f64
        } else {
            0.0
        };

        // Find oldest and newest entries
        let now = SystemTime::now();
        let mut oldest_age = None;
        let mut newest_age = None;

        for entry in storage.values() {
            if let Ok(age) = now.duration_since(entry.created_at) {
                oldest_age = Some(oldest_age.map_or(age, |old: Duration| old.max(age)));
                newest_age = Some(newest_age.map_or(age, |new: Duration| new.min(age)));
            }
        }

        Ok(CacheStats {
            total_entries: stats.total_entries,
            total_size_bytes: stats.total_size_bytes,
            hit_count: stats.hit_count,
            miss_count: stats.miss_count,
            eviction_count: stats.eviction_count,
            expired_entries: 0, // Would need to track this separately
            cache_hit_rate,
            average_entry_size,
            oldest_entry_age: oldest_age,
            newest_entry_age: newest_age,
        })
    }

    async fn store_bytes_with_metadata(
        &self,
        key: &str,
        data: &[u8],
        ttl: Duration,
        metadata: CacheMetadata,
    ) -> Result<()> {
        let data_size = data.len() as u64;

        // Evict expired entries first
        self.evict_expired().await;

        // Evict entries if needed to make space
        self.evict_for_space(data_size).await;

        let now = SystemTime::now();
        let expires_at = now + ttl;

        let entry = CacheEntry {
            data: data.to_vec(),
            expires_at,
            created_at: now,
            access_count: 0,
            metadata: Some(metadata),
        };

        // Store entry
        {
            let mut storage = self.storage.write().await;
            let mut stats = self.stats.write().await;

            let is_new = !storage.contains_key(key);
            storage.insert(key.to_string(), entry);

            if is_new {
                stats.total_entries += 1;
            }
            stats.total_size_bytes += data_size;
        }

        debug!("Cached {} bytes with metadata for key: {}", data_size, key);
        Ok(())
    }

    async fn retrieve_bytes_with_metadata(
        &self,
        key: &str,
    ) -> Result<Option<(Vec<u8>, CacheMetadata)>> {
        // Evict expired entries first
        self.evict_expired().await;

        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = storage.get(key) {
            let now = SystemTime::now();

            // Check if entry is still valid
            if entry.expires_at > now {
                let data = entry.data.clone();
                let metadata = if let Some(metadata) = &entry.metadata {
                    metadata.clone()
                } else {
                    // Create default metadata if none exists
                    CacheMetadata {
                        created_at: Utc::now(),
                        last_accessed: Utc::now(),
                        expires_at: Utc::now() + ChronoDuration::hours(1),
                        access_count: entry.access_count,
                        size_bytes: entry.data.len() as u64,
                        source: DataSource::Cache,
                        priority: CachePriority::Normal,
                        tags: vec![],
                        offline_capable: true,
                        checksum: None,
                    }
                };

                // Update access count
                if let Some(entry) = storage.get_mut(key) {
                    entry.access_count += 1;
                }
                stats.hit_count += 1;

                debug!("Cache hit with metadata for key: {}", key);
                Ok(Some((data, metadata)))
            } else {
                // Entry expired, remove it
                let entry = storage.remove(key).unwrap();
                stats.total_entries -= 1;
                stats.total_size_bytes -= entry.data.len() as u64;
                stats.miss_count += 1;

                debug!("Cache miss (expired) for key: {}", key);
                Ok(None)
            }
        } else {
            stats.miss_count += 1;
            debug!("Cache miss for key: {}", key);
            Ok(None)
        }
    }

    async fn list_keys(&self, pattern: Option<&str>) -> Result<Vec<String>> {
        self.evict_expired().await;

        let storage = self.storage.read().await;
        let keys: Vec<String> = if let Some(pattern) = pattern {
            // Simple pattern matching (contains)
            storage
                .keys()
                .filter(|key| key.contains(pattern))
                .cloned()
                .collect()
        } else {
            storage.keys().cloned().collect()
        };

        Ok(keys)
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<CacheMetadata>> {
        self.evict_expired().await;

        let storage = self.storage.read().await;
        if let Some(entry) = storage.get(key) {
            Ok(entry.metadata.clone())
        } else {
            Ok(None)
        }
    }

    async fn extend_ttl(&self, key: &str, additional_ttl: Duration) -> Result<bool> {
        let mut storage = self.storage.write().await;
        if let Some(entry) = storage.get_mut(key) {
            entry.expires_at += additional_ttl;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn compact(&self) -> Result<CompactionResult> {
        let start_time = std::time::Instant::now();

        self.evict_expired().await;

        let _storage = self.storage.read().await;
        let stats = self.stats.read().await;

        Ok(CompactionResult {
            entries_removed: 0, // Expired entries already removed
            bytes_freed: 0,
            duration: start_time.elapsed(),
            entries_remaining: stats.total_entries,
            size_after_bytes: stats.total_size_bytes,
        })
    }

    async fn export(&self) -> Result<Vec<u8>> {
        self.evict_expired().await;

        let storage = self.storage.read().await;
        let export_data: HashMap<String, CacheEntry> = storage.clone();

        serde_json::to_vec(&export_data).map_err(|e| WorkflowError::validation(e.to_string()))
    }

    async fn import(&self, data: &[u8]) -> Result<()> {
        let import_data: HashMap<String, CacheEntry> =
            serde_json::from_slice(data).map_err(|e| WorkflowError::validation(e.to_string()))?;

        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;

        // Clear existing data
        storage.clear();

        // Import new data
        let mut total_size = 0u64;
        for (key, entry) in import_data {
            total_size += entry.data.len() as u64;
            storage.insert(key, entry);
        }

        // Update statistics
        stats.total_entries = storage.len() as u64;
        stats.total_size_bytes = total_size;

        info!(
            "Imported {} cache entries ({} bytes)",
            stats.total_entries, stats.total_size_bytes
        );
        Ok(())
    }
}

impl Default for MemoryCacheBackend {
    fn default() -> Self {
        Self::new(100) // 100MB default
    }
}

/// HTTP-based sync backend for production use
pub struct HttpSyncBackend {
    /// Base URL for the API
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Authentication token
    auth_token: Option<String>,
}

impl HttpSyncBackend {
    /// Create a new HTTP sync backend
    pub fn new(base_url: String, auth_token: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            auth_token,
        }
    }

    /// Build request with authentication
    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut request = self.client.request(method, &url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        request
    }
}

#[async_trait]
impl SyncBackend for HttpSyncBackend {
    async fn fetch_workflows(&self) -> Result<Vec<WorkflowInfo>> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/workflows")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let workflows: Vec<WorkflowInfo> = response
            .json()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(workflows)
    }

    async fn fetch_executions(&self) -> Result<Vec<crate::interfaces::tui::state::ExecutionInfo>> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/executions")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let executions: Vec<crate::interfaces::tui::state::ExecutionInfo> =
            response
                .json()
                .await
                .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(executions)
    }

    async fn fetch_tools(&self) -> Result<Vec<ToolInfo>> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/tools")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let tools: Vec<ToolInfo> = response
            .json()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(tools)
    }

    async fn fetch_plugins(&self) -> Result<Vec<PluginInfo>> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/plugins")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let plugins: Vec<PluginInfo> = response
            .json()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(plugins)
    }

    async fn fetch_system_status(&self) -> Result<SystemStatus> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/system/status")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let status: SystemStatus = response
            .json()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(status)
    }

    async fn fetch_logs(&self, since: Option<DateTime<Utc>>) -> Result<Vec<LogEntry>> {
        let mut request = self.build_request(reqwest::Method::GET, "/api/logs");

        if let Some(since_time) = since {
            request = request.query(&[("since", since_time.to_rfc3339())]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WorkflowError::validation(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let logs: Vec<LogEntry> = response
            .json()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(logs)
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self
            .build_request(reqwest::Method::GET, "/api/health")
            .send()
            .await
            .map_err(|e| WorkflowError::validation(e.to_string()))?;

        Ok(response.status().is_success())
    }

    fn connection_info(&self) -> String {
        format!("HTTP Backend ({})", self.base_url)
    }
}
