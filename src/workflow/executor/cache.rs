//! Cache executor implementation.
//!
//! Wraps component execution with caching to avoid redundant executions
//! for deterministic components.

use crate::core::ExecutionContext;
use crate::error::Result;
use crate::workflow::component::{Component, ComponentOutput};
use crate::workflow::context::DataContext;
use crate::workflow::executor::{BoxedExecutor, Executor};
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;

/// Cache executor that caches component outputs.
///
/// Only caches results from components that return `cacheable() == true`.
pub struct CacheExecutor {
    inner: BoxedExecutor,
    cache: Arc<Cache<String, ComponentOutput>>,
}

impl CacheExecutor {
    /// Create a new cache executor.
    pub fn new(inner: BoxedExecutor, cache: Arc<Cache<String, ComponentOutput>>) -> Self {
        Self { inner, cache }
    }

    /// Create a cache key for a component execution.
    ///
    /// The key includes:
    /// - Workflow ID
    /// - Component ID
    /// - Hash of input parameters
    fn create_cache_key(
        &self,
        component: &dyn Component,
        _context: &DataContext,
        execution_ctx: &ExecutionContext,
    ) -> String {
        // Create a deterministic key based on component and inputs
        let params_hash = {
            let params = &execution_ctx.global_variables;
            let json = serde_json::to_string(params).unwrap_or_default();
            // Simple hash for now - could use a proper hash function
            format!("{:x}", md5_hash(&json))
        };

        let workflow_id = execution_ctx.workflow_id.unwrap_or_else(uuid::Uuid::nil);
        format!("{}:{}:{}", workflow_id, component.id(), params_hash)
    }
}

/// Simple hash function for cache keys
fn md5_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

#[async_trait]
impl Executor for CacheExecutor {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        // Only cache if the component is cacheable
        if !component.cacheable() {
            return self.inner.execute(component, context, execution_ctx).await;
        }

        let cache_key = self.create_cache_key(component, context, execution_ctx);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!(
                component_id = component.id(),
                cache_key = %cache_key,
                "Cache hit for component"
            );
            return Ok(cached);
        }

        tracing::debug!(
            component_id = component.id(),
            cache_key = %cache_key,
            "Cache miss for component"
        );

        // Execute the component
        let output = self
            .inner
            .execute(component, context, execution_ctx)
            .await?;

        // Cache successful results
        if output.status.is_success() {
            self.cache.insert(cache_key, output.clone()).await;
        }

        Ok(output)
    }

    fn name(&self) -> &str {
        "CacheExecutor"
    }
}

/// Builder for creating a cache with common configurations.
pub struct CacheBuilder {
    max_capacity: u64,
    time_to_live: Option<std::time::Duration>,
    time_to_idle: Option<std::time::Duration>,
}

impl CacheBuilder {
    /// Create a new cache builder with default settings.
    pub fn new() -> Self {
        Self {
            max_capacity: 1000,
            time_to_live: None,
            time_to_idle: None,
        }
    }

    /// Set the maximum number of entries in the cache.
    pub fn max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Set the time-to-live for cache entries.
    pub fn time_to_live(mut self, ttl: std::time::Duration) -> Self {
        self.time_to_live = Some(ttl);
        self
    }

    /// Set the time-to-idle for cache entries.
    pub fn time_to_idle(mut self, tti: std::time::Duration) -> Self {
        self.time_to_idle = Some(tti);
        self
    }

    /// Build the cache.
    pub fn build(self) -> Arc<Cache<String, ComponentOutput>> {
        let mut builder = Cache::builder().max_capacity(self.max_capacity);

        if let Some(ttl) = self.time_to_live {
            builder = builder.time_to_live(ttl);
        }

        if let Some(tti) = self.time_to_idle {
            builder = builder.time_to_idle(tti);
        }

        Arc::new(builder.build())
    }
}

impl Default for CacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::component::ComponentType;
    use crate::workflow::executor::BasicExecutor;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    struct CacheableComponent {
        id: String,
        call_count: Arc<AtomicU32>,
    }

    #[async_trait]
    impl Component for CacheableComponent {
        fn id(&self) -> &str {
            &self.id
        }

        fn component_type(&self) -> ComponentType {
            ComponentType::Tool
        }

        async fn execute(
            &self,
            _context: &mut DataContext,
            _execution_ctx: &ExecutionContext,
        ) -> Result<ComponentOutput> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(ComponentOutput::success_with_result(serde_json::json!({
                "result": "computed"
            })))
        }

        fn cacheable(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_cache_executor_caches_results() {
        let cache = CacheBuilder::new()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(60))
            .build();

        let basic = Arc::new(BasicExecutor) as BoxedExecutor;
        let cached = CacheExecutor::new(basic, cache);

        let call_count = Arc::new(AtomicU32::new(0));
        let component = CacheableComponent {
            id: "test".to_string(),
            call_count: call_count.clone(),
        };

        let mut context = DataContext::new();
        let mut exec_ctx = ExecutionContext::new().with_workflow_id(uuid::Uuid::new_v4());
        exec_ctx
            .global_variables
            .insert("input".to_string(), serde_json::json!("value"));

        // First call - should execute
        let result1 = cached.execute(&component, &mut context, &exec_ctx).await;
        assert!(result1.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call with same params - should use cache
        let result2 = cached.execute(&component, &mut context, &exec_ctx).await;
        assert!(result2.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Still 1, cached!

        // Call with different params - should execute
        let mut exec_ctx2 = exec_ctx.clone();
        exec_ctx2
            .global_variables
            .insert("input".to_string(), serde_json::json!("different"));
        let result3 = cached.execute(&component, &mut context, &exec_ctx2).await;
        assert!(result3.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 2); // Now 2
    }
}
