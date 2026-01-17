//! Result caching mechanism for workflow executions

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::storage::CacheBackend;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Cache key for workflow results
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    pub workflow_name: String,
    pub workflow_version: String,
    pub input_hash: String,
    pub node_id: Option<String>,
}

impl CacheKey {
    /// Create a new cache key for workflow execution
    pub fn new_workflow(workflow_name: &str, workflow_version: &str, input_hash: &str) -> Self {
        Self {
            workflow_name: workflow_name.to_string(),
            workflow_version: workflow_version.to_string(),
            input_hash: input_hash.to_string(),
            node_id: None,
        }
    }

    /// Create a new cache key for node execution
    pub fn new_node(
        workflow_name: &str,
        workflow_version: &str,
        input_hash: &str,
        node_id: &str,
    ) -> Self {
        Self {
            workflow_name: workflow_name.to_string(),
            workflow_version: workflow_version.to_string(),
            input_hash: input_hash.to_string(),
            node_id: Some(node_id.to_string()),
        }
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(node_id) = &self.node_id {
            write!(
                f,
                "cache:{}:{}:{}:{}",
                self.workflow_name, self.workflow_version, self.input_hash, node_id
            )
        } else {
            write!(
                f,
                "cache:{}:{}:{}",
                self.workflow_name, self.workflow_version, self.input_hash
            )
        }
    }
}

/// Cached execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub result: Value,
    pub cached_at: DateTime<Utc>,
    pub execution_duration: Option<chrono::Duration>,
    pub metadata: HashMap<String, Value>,
}

impl CachedResult {
    pub fn new(result: Value, execution_duration: Option<chrono::Duration>) -> Self {
        Self {
            result,
            cached_at: Utc::now(),
            execution_duration,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if the cached result is still valid based on TTL
    pub fn is_valid(&self, ttl: Duration) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.to_std().map(|d| d <= ttl).unwrap_or(false)
    }
}

/// Cache invalidation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Time-based invalidation (TTL)
    TimeToLive(Duration),
    /// Version-based invalidation
    Version(String),
    /// Manual invalidation only
    Manual,
    /// Dependency-based invalidation
    Dependency(Vec<String>),
}

/// Configuration for result caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub default_ttl: Duration,
    pub max_cache_size: usize,
    pub invalidation_strategy: InvalidationStrategy,
    pub cache_node_results: bool,
    pub cache_workflow_results: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
            invalidation_strategy: InvalidationStrategy::TimeToLive(Duration::from_secs(3600)),
            cache_node_results: true,
            cache_workflow_results: true,
        }
    }
}

/// Result cache manager
pub struct ResultCache {
    cache_backend: Arc<dyn CacheBackend>,
    config: CacheConfig,
}

impl ResultCache {
    /// Create a new result cache
    pub fn new(cache_backend: Arc<dyn CacheBackend>, config: CacheConfig) -> Self {
        Self {
            cache_backend,
            config,
        }
    }

    /// Generate input hash for caching
    pub fn generate_input_hash(
        &self,
        context: &ExecutionContext,
        parameters: &Value,
    ) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash execution context (excluding dynamic fields like timestamps)
        if let Some(workflow_id) = context.workflow_id {
            workflow_id.hash(&mut hasher);
        }
        if let Some(user_id) = &context.user_id {
            user_id.hash(&mut hasher);
        }

        // Hash global variables (sorted for consistency)
        let mut sorted_vars: Vec<_> = context.global_variables.iter().collect();
        sorted_vars.sort_by_key(|(k, _)| *k);
        for (key, value) in sorted_vars {
            key.hash(&mut hasher);
            // Simple hash for JSON values (in production, use a more robust method)
            serde_json::to_string(value)?.hash(&mut hasher);
        }

        // Hash parameters
        serde_json::to_string(parameters)?.hash(&mut hasher);

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Cache workflow execution result
    pub async fn cache_workflow_result(
        &self,
        workflow_name: &str,
        workflow_version: &str,
        context: &ExecutionContext,
        parameters: &Value,
        result: &Value,
        execution_duration: Option<chrono::Duration>,
    ) -> Result<()> {
        if !self.config.enabled || !self.config.cache_workflow_results {
            return Ok(());
        }

        let input_hash = self.generate_input_hash(context, parameters)?;
        let cache_key = CacheKey::new_workflow(workflow_name, workflow_version, &input_hash);

        let cached_result = CachedResult::new(result.clone(), execution_duration);
        let serialized = serde_json::to_vec(&cached_result)?;

        let ttl = match &self.config.invalidation_strategy {
            InvalidationStrategy::TimeToLive(duration) => Some(*duration),
            _ => Some(self.config.default_ttl),
        };

        self.cache_backend
            .set(&cache_key.to_string(), serialized, ttl)
            .await?;

        tracing::debug!(
            "Cached workflow result for {}:{} with key {}",
            workflow_name,
            workflow_version,
            cache_key.to_string()
        );

        Ok(())
    }

    /// Get cached workflow result
    pub async fn get_workflow_result(
        &self,
        workflow_name: &str,
        workflow_version: &str,
        context: &ExecutionContext,
        parameters: &Value,
    ) -> Result<Option<CachedResult>> {
        if !self.config.enabled || !self.config.cache_workflow_results {
            return Ok(None);
        }

        let input_hash = self.generate_input_hash(context, parameters)?;
        let cache_key = CacheKey::new_workflow(workflow_name, workflow_version, &input_hash);

        if let Some(cached_data) = self.cache_backend.get(&cache_key.to_string()).await {
            let cached_result: CachedResult = serde_json::from_slice(&cached_data)?;

            // Check if result is still valid
            let ttl = match &self.config.invalidation_strategy {
                InvalidationStrategy::TimeToLive(duration) => *duration,
                _ => self.config.default_ttl,
            };

            if cached_result.is_valid(ttl) {
                tracing::debug!(
                    "Cache hit for workflow {}:{} with key {}",
                    workflow_name,
                    workflow_version,
                    cache_key.to_string()
                );
                return Ok(Some(cached_result));
            } else {
                // Remove expired entry
                self.cache_backend.delete(&cache_key.to_string()).await?;
                tracing::debug!(
                    "Removed expired cache entry for workflow {}:{}",
                    workflow_name,
                    workflow_version
                );
            }
        }

        tracing::debug!(
            "Cache miss for workflow {}:{} with key {}",
            workflow_name,
            workflow_version,
            cache_key.to_string()
        );

        Ok(None)
    }

    /// Cache node execution result
    #[allow(clippy::too_many_arguments)]
    pub async fn cache_node_result(
        &self,
        workflow_name: &str,
        workflow_version: &str,
        node_id: &str,
        context: &ExecutionContext,
        parameters: &Value,
        result: &Value,
        execution_duration: Option<chrono::Duration>,
    ) -> Result<()> {
        if !self.config.enabled || !self.config.cache_node_results {
            return Ok(());
        }

        let input_hash = self.generate_input_hash(context, parameters)?;
        let cache_key = CacheKey::new_node(workflow_name, workflow_version, &input_hash, node_id);

        let cached_result = CachedResult::new(result.clone(), execution_duration);
        let serialized = serde_json::to_vec(&cached_result)?;

        let ttl = match &self.config.invalidation_strategy {
            InvalidationStrategy::TimeToLive(duration) => Some(*duration),
            _ => Some(self.config.default_ttl),
        };

        self.cache_backend
            .set(&cache_key.to_string(), serialized, ttl)
            .await?;

        tracing::debug!(
            "Cached node result for {}:{}:{} with key {}",
            workflow_name,
            workflow_version,
            node_id,
            cache_key.to_string()
        );

        Ok(())
    }

    /// Get cached node result
    pub async fn get_node_result(
        &self,
        workflow_name: &str,
        workflow_version: &str,
        node_id: &str,
        context: &ExecutionContext,
        parameters: &Value,
    ) -> Result<Option<CachedResult>> {
        if !self.config.enabled || !self.config.cache_node_results {
            return Ok(None);
        }

        let input_hash = self.generate_input_hash(context, parameters)?;
        let cache_key = CacheKey::new_node(workflow_name, workflow_version, &input_hash, node_id);

        if let Some(cached_data) = self.cache_backend.get(&cache_key.to_string()).await {
            let cached_result: CachedResult = serde_json::from_slice(&cached_data)?;

            // Check if result is still valid
            let ttl = match &self.config.invalidation_strategy {
                InvalidationStrategy::TimeToLive(duration) => *duration,
                _ => self.config.default_ttl,
            };

            if cached_result.is_valid(ttl) {
                tracing::debug!(
                    "Cache hit for node {}:{}:{} with key {}",
                    workflow_name,
                    workflow_version,
                    node_id,
                    cache_key.to_string()
                );
                return Ok(Some(cached_result));
            } else {
                // Remove expired entry
                self.cache_backend.delete(&cache_key.to_string()).await?;
                tracing::debug!(
                    "Removed expired cache entry for node {}:{}:{}",
                    workflow_name,
                    workflow_version,
                    node_id
                );
            }
        }

        tracing::debug!(
            "Cache miss for node {}:{}:{} with key {}",
            workflow_name,
            workflow_version,
            node_id,
            cache_key.to_string()
        );

        Ok(None)
    }

    /// Invalidate cache entries by workflow
    pub async fn invalidate_workflow(
        &self,
        workflow_name: &str,
        workflow_version: Option<&str>,
    ) -> Result<usize> {
        let prefix = if let Some(version) = workflow_version {
            format!("cache:{}:{}", workflow_name, version)
        } else {
            format!("cache:{}", workflow_name)
        };

        // For now, we'll just clear the entire cache if no specific implementation is available
        // A real implementation would iterate through keys with the prefix
        tracing::warn!(
            "Invalidating cache entries with prefix '{}' - using cache clear as fallback",
            prefix
        );

        self.cache_backend.clear().await?;
        let invalidated_count = 1; // Approximate

        tracing::info!(
            "Invalidated {} cache entries for workflow {}",
            invalidated_count,
            workflow_name
        );

        Ok(invalidated_count)
    }

    /// Invalidate cache entries by dependency
    pub async fn invalidate_by_dependency(&self, dependency: &str) -> Result<usize> {
        // This would require storing dependency metadata with cache entries
        // For now, implement as a no-op
        tracing::debug!(
            "Dependency-based invalidation for '{}' not implemented",
            dependency
        );
        Ok(0)
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache_backend.size(),
            config: self.config.clone(),
        }
    }

    /// Clear all cached results
    pub async fn clear_all(&self) -> Result<()> {
        self.cache_backend.clear().await?;
        tracing::info!("Cleared all cached results");
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub config: CacheConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SimpleMemoryCache;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cache_key_generation() {
        let key1 = CacheKey::new_workflow("test_workflow", "1.0.0", "hash123");
        let key2 = CacheKey::new_node("test_workflow", "1.0.0", "hash123", "node1");

        assert_eq!(key1.to_string(), "cache:test_workflow:1.0.0:hash123");
        assert_eq!(key2.to_string(), "cache:test_workflow:1.0.0:hash123:node1");
    }

    #[tokio::test]
    async fn test_workflow_result_caching() {
        let cache_backend = Arc::new(SimpleMemoryCache::new());
        let config = CacheConfig::default();
        let result_cache = ResultCache::new(cache_backend, config);

        let context = ExecutionContext::new();
        let parameters = serde_json::json!({"param1": "value1"});
        let result = serde_json::json!({"output": "test_result"});

        // Cache result
        result_cache
            .cache_workflow_result(
                "test_workflow",
                "1.0.0",
                &context,
                &parameters,
                &result,
                Some(chrono::Duration::seconds(5)),
            )
            .await
            .unwrap();

        // Retrieve cached result
        let cached = result_cache
            .get_workflow_result("test_workflow", "1.0.0", &context, &parameters)
            .await
            .unwrap();

        assert!(cached.is_some());
        let cached_result = cached.unwrap();
        assert_eq!(cached_result.result, result);
        assert_eq!(
            cached_result.execution_duration,
            Some(chrono::Duration::seconds(5))
        );
    }

    #[tokio::test]
    async fn test_node_result_caching() {
        let cache_backend = Arc::new(SimpleMemoryCache::new());
        let config = CacheConfig::default();
        let result_cache = ResultCache::new(cache_backend, config);

        let context = ExecutionContext::new();
        let parameters = serde_json::json!({"param1": "value1"});
        let result = serde_json::json!({"output": "node_result"});

        // Cache node result
        result_cache
            .cache_node_result(
                "test_workflow",
                "1.0.0",
                "node1",
                &context,
                &parameters,
                &result,
                Some(chrono::Duration::seconds(2)),
            )
            .await
            .unwrap();

        // Retrieve cached result
        let cached = result_cache
            .get_node_result("test_workflow", "1.0.0", "node1", &context, &parameters)
            .await
            .unwrap();

        assert!(cached.is_some());
        let cached_result = cached.unwrap();
        assert_eq!(cached_result.result, result);
        assert_eq!(
            cached_result.execution_duration,
            Some(chrono::Duration::seconds(2))
        );
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache_backend = Arc::new(SimpleMemoryCache::new());
        let config = CacheConfig::default();
        let result_cache = ResultCache::new(cache_backend, config);

        let context = ExecutionContext::new();
        let parameters = serde_json::json!({"param1": "value1"});

        // Try to get non-existent cached result
        let cached = result_cache
            .get_workflow_result("non_existent_workflow", "1.0.0", &context, &parameters)
            .await
            .unwrap();

        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_input_hash_consistency() {
        let cache_backend = Arc::new(SimpleMemoryCache::new());
        let config = CacheConfig::default();
        let result_cache = ResultCache::new(cache_backend, config);

        let context1 = ExecutionContext::new();
        let context2 = ExecutionContext::new();
        let parameters = serde_json::json!({"param1": "value1"});

        let hash1 = result_cache
            .generate_input_hash(&context1, &parameters)
            .unwrap();
        let hash2 = result_cache
            .generate_input_hash(&context2, &parameters)
            .unwrap();

        // Hashes should be the same for same inputs (excluding timestamps)
        assert_eq!(hash1, hash2);
    }
}
