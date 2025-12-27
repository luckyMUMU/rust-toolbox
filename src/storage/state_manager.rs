//! State management for workflows and executions

use crate::error::Result;
use crate::storage::{CacheBackend, StorageBackend, RetentionPolicy};
use crate::workflow::{WorkflowState, ExecutionRecord};
use crate::core::WorkflowId;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;

/// State manager for workflow data
pub struct StateManager {
    storage: Arc<dyn StorageBackend>,
    cache: Arc<dyn CacheBackend>,
}

impl StateManager {
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        cache: Arc<dyn CacheBackend>,
    ) -> Self {
        Self { storage, cache }
    }
    
    /// Create a StateManager with file storage and simple memory cache
    pub fn with_file_storage<P: Into<std::path::PathBuf>>(base_path: P) -> Result<Self> {
        let storage = Arc::new(crate::storage::FileStorage::new(base_path)?);
        let cache = Arc::new(crate::storage::SimpleMemoryCache::new());
        Ok(Self::new(storage, cache))
    }
    
    /// Create a StateManager with local memory cache
    pub fn with_local_cache(
        storage: Arc<dyn StorageBackend>,
        cache_capacity: u64,
        cache_ttl: Option<Duration>,
    ) -> Self {
        let cache_config = crate::storage::CacheConfig {
            max_capacity: cache_capacity,
            ttl: cache_ttl,
            enable_metrics: false,
        };
        let cache = Arc::new(crate::storage::LocalMemoryCache::new(cache_config));
        Self::new(storage, cache)
    }
}

impl StateManager {
    /// Save workflow state to storage and cache
    pub async fn save_workflow_state(&self, id: WorkflowId, state: WorkflowState) -> Result<()> {
        let key = format!("workflow:state:{}", id);
        let value = serde_json::to_vec(&state)?;
        
        // Save to storage backend first
        self.storage.save(&key, &value).await?;
        
        // Update cache with 1 hour TTL
        self.cache.set(&key, value, Some(Duration::from_secs(3600))).await?;
        
        Ok(())
    }
    
    /// Load workflow state from cache or storage
    pub async fn load_workflow_state(&self, id: WorkflowId) -> Result<Option<WorkflowState>> {
        let key = format!("workflow:state:{}", id);
        
        // Try cache first
        if let Some(cached_value) = self.cache.get(&key).await {
            if let Ok(state) = serde_json::from_slice(&cached_value) {
                return Ok(Some(state));
            }
        }
        
        // Cache miss, try storage
        if let Some(value) = self.storage.load(&key).await? {
            let state: WorkflowState = serde_json::from_slice(&value)?;
            
            // Update cache
            self.cache.set(&key, value, Some(Duration::from_secs(3600))).await?;
            
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
    
    /// Delete workflow state from storage and cache
    pub async fn delete_workflow_state(&self, id: WorkflowId) -> Result<()> {
        let key = format!("workflow:state:{}", id);
        
        // Remove from storage
        self.storage.delete(&key).await?;
        
        // Remove from cache
        self.cache.delete(&key).await?;
        
        Ok(())
    }
    
    /// List all workflow IDs with states
    pub async fn list_workflow_states(&self) -> Result<Vec<WorkflowId>> {
        let keys = self.storage.list_keys("workflow:state:").await?;
        let mut workflow_ids = Vec::new();
        
        for key in keys {
            if let Some(id_str) = key.strip_prefix("workflow:state:") {
                if let Ok(id) = id_str.parse() {
                    workflow_ids.push(id);
                }
            }
        }
        
        Ok(workflow_ids)
    }
    
    /// Check if workflow state exists
    pub async fn workflow_state_exists(&self, id: WorkflowId) -> Result<bool> {
        let key = format!("workflow:state:{}", id);
        
        // Check cache first
        if self.cache.get(&key).await.is_some() {
            return Ok(true);
        }
        
        // Check storage
        self.storage.exists(&key).await
    }
    
    /// Save multiple workflow states in batch
    pub async fn batch_save_workflow_states(&self, states: Vec<(WorkflowId, WorkflowState)>) -> Result<()> {
        let mut items = Vec::new();
        let mut cache_items = Vec::new();
        
        for (id, state) in states {
            let key = format!("workflow:state:{}", id);
            let value = serde_json::to_vec(&state)?;
            items.push((key.clone(), value.clone()));
            cache_items.push((key, value));
        }
        
        // Batch save to storage
        self.storage.batch_save(items).await?;
        
        // Update cache
        for (key, value) in cache_items {
            self.cache.set(&key, value, Some(Duration::from_secs(3600))).await?;
        }
        
        Ok(())
    }
    
    /// Load multiple workflow states in batch
    pub async fn batch_load_workflow_states(&self, ids: Vec<WorkflowId>) -> Result<Vec<Option<WorkflowState>>> {
        let keys: Vec<String> = ids.iter().map(|id| format!("workflow:state:{}", id)).collect();
        let values = self.storage.batch_load(keys.clone()).await?;
        
        let mut results = Vec::new();
        for (i, value_opt) in values.into_iter().enumerate() {
            if let Some(value) = value_opt {
                if let Ok(state) = serde_json::from_slice(&value) {
                    // Update cache
                    self.cache.set(&keys[i], value, Some(Duration::from_secs(3600))).await?;
                    results.push(Some(state));
                } else {
                    results.push(None);
                }
            } else {
                results.push(None);
            }
        }
        
        Ok(results)
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache.clear().await
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.size()
    }
}

// Execution History Management Implementation
impl StateManager {
    /// Save execution record to history
    pub async fn save_execution_record(&self, record: ExecutionRecord) -> Result<()> {
        let key = format!("execution:history:{}:{}", record.workflow_id, record.execution_id);
        let value = serde_json::to_vec(&record)?;
        
        // Save to storage
        self.storage.save(&key, &value).await?;
        
        // Cache with shorter TTL for history records
        self.cache.set(&key, value, Some(Duration::from_secs(1800))).await?; // 30 minutes
        
        Ok(())
    }
    
    /// Get execution history for a workflow
    pub async fn get_execution_history(&self, workflow_id: WorkflowId) -> Result<Vec<ExecutionRecord>> {
        let prefix = format!("execution:history:{}", workflow_id);
        let keys = self.storage.list_keys(&prefix).await?;
        let values = self.storage.batch_load(keys).await?;
        
        let mut history = Vec::new();
        for value_opt in values {
            if let Some(value) = value_opt {
                if let Ok(record) = serde_json::from_slice::<ExecutionRecord>(&value) {
                    history.push(record);
                }
            }
        }
        
        // Sort by timestamp (newest first)
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(history)
    }
    
    /// Get execution history with pagination
    pub async fn get_execution_history_paginated(
        &self, 
        workflow_id: WorkflowId, 
        limit: usize, 
        offset: usize
    ) -> Result<Vec<ExecutionRecord>> {
        let mut history = self.get_execution_history(workflow_id).await?;
        
        // Apply pagination
        let start = offset.min(history.len());
        let end = (offset + limit).min(history.len());
        
        if start < end {
            history.drain(..start);
            history.truncate(limit);
        } else {
            history.clear();
        }
        
        Ok(history)
    }
    
    /// Get execution record by ID
    pub async fn get_execution_record(&self, workflow_id: WorkflowId, execution_id: &str) -> Result<Option<ExecutionRecord>> {
        let key = format!("execution:history:{}:{}", workflow_id, execution_id);
        
        // Try cache first
        if let Some(cached_value) = self.cache.get(&key).await {
            if let Ok(record) = serde_json::from_slice(&cached_value) {
                return Ok(Some(record));
            }
        }
        
        // Try storage
        if let Some(value) = self.storage.load(&key).await? {
            let record: ExecutionRecord = serde_json::from_slice(&value)?;
            
            // Update cache
            self.cache.set(&key, value, Some(Duration::from_secs(1800))).await?;
            
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
    
    /// Delete execution record
    pub async fn delete_execution_record(&self, workflow_id: WorkflowId, execution_id: &str) -> Result<()> {
        let key = format!("execution:history:{}:{}", workflow_id, execution_id);
        
        // Remove from storage
        self.storage.delete(&key).await?;
        
        // Remove from cache
        self.cache.delete(&key).await?;
        
        Ok(())
    }
    
    /// Delete all execution history for a workflow
    pub async fn delete_workflow_history(&self, workflow_id: WorkflowId) -> Result<()> {
        let prefix = format!("execution:history:{}", workflow_id);
        let keys = self.storage.list_keys(&prefix).await?;
        
        // Delete from storage
        for key in &keys {
            self.storage.delete(key).await?;
        }
        
        // Delete from cache
        for key in &keys {
            self.cache.delete(key).await?;
        }
        
        Ok(())
    }
    
    /// Get execution statistics for a workflow
    pub async fn get_execution_statistics(&self, workflow_id: WorkflowId) -> Result<ExecutionStatistics> {
        let history = self.get_execution_history(workflow_id).await?;
        
        let mut stats = ExecutionStatistics::default();
        stats.total_executions = history.len();
        
        for record in &history {
            match record.status {
                crate::core::ExecutionStatus::Completed => stats.successful_executions += 1,
                crate::core::ExecutionStatus::Failed => stats.failed_executions += 1,
                crate::core::ExecutionStatus::Cancelled => stats.cancelled_executions += 1,
                _ => {}
            }
            
            if let Some(duration) = record.duration {
                stats.total_duration += duration;
                if stats.min_duration.is_none() || Some(duration) < stats.min_duration {
                    stats.min_duration = Some(duration);
                }
                if stats.max_duration.is_none() || Some(duration) > stats.max_duration {
                    stats.max_duration = Some(duration);
                }
            }
        }
        
        if stats.total_executions > 0 {
            stats.success_rate = stats.successful_executions as f64 / stats.total_executions as f64;
            if let Some(total_duration) = stats.total_duration.to_std().ok() {
                stats.average_duration = Some(chrono::Duration::from_std(
                    total_duration / stats.total_executions as u32
                ).unwrap_or_default());
            }
        }
        
        Ok(stats)
    }
    
    /// Clean up old execution records based on retention policy
    pub async fn cleanup_execution_history(&self, retention_policy: RetentionPolicy) -> Result<usize> {
        let cutoff_time = Utc::now() - retention_policy.max_age;
        let all_keys = self.storage.list_keys("execution:history:").await?;
        
        let mut deleted_count = 0;
        let mut records_to_check = Vec::new();
        
        // Load all execution records to check timestamps
        for key in &all_keys {
            if let Some(value) = self.storage.load(key).await? {
                if let Ok(record) = serde_json::from_slice::<ExecutionRecord>(&value) {
                    records_to_check.push((key.clone(), record));
                }
            }
        }
        
        // Sort by timestamp (oldest first)
        records_to_check.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
        
        // Delete old records
        for (key, record) in &records_to_check {
            let should_delete = record.timestamp < cutoff_time || 
                (retention_policy.max_count.is_some() && 
                 deleted_count < records_to_check.len().saturating_sub(retention_policy.max_count.unwrap()));
            
            if should_delete {
                self.storage.delete(key).await?;
                self.cache.delete(key).await?;
                deleted_count += 1;
            }
        }
        
        Ok(deleted_count)
    }
    
    /// Get recent execution records across all workflows
    pub async fn get_recent_executions(&self, limit: usize) -> Result<Vec<ExecutionRecord>> {
        let all_keys = self.storage.list_keys("execution:history:").await?;
        let values = self.storage.batch_load(all_keys).await?;
        
        let mut all_records = Vec::new();
        for value_opt in values {
            if let Some(value) = value_opt {
                if let Ok(record) = serde_json::from_slice::<ExecutionRecord>(&value) {
                    all_records.push(record);
                }
            }
        }
        
        // Sort by timestamp (newest first) and limit
        all_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_records.truncate(limit);
        
        Ok(all_records)
    }
}

/// Execution statistics for a workflow
#[derive(Debug, Clone, Default)]
pub struct ExecutionStatistics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub cancelled_executions: usize,
    pub success_rate: f64,
    pub total_duration: chrono::Duration,
    pub average_duration: Option<chrono::Duration>,
    pub min_duration: Option<chrono::Duration>,
    pub max_duration: Option<chrono::Duration>,
}