//! 工作流模块
//!
//! 注册工作流引擎及相关服务

use crate::di::{AppModule, DiContainer};
use crate::storage::backends::SimpleMemoryCache;
use crate::storage::StateManager;
use crate::tools::registry::ToolRegistry;
use crate::workflow::engine::RefactoredWorkflowEngine;
use crate::workflow::AuditLogger;
use std::sync::Arc;

/// 内存存储后端（用于 DI 模块）
struct InMemoryStorage {
    data: std::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>,
}

impl InMemoryStorage {
    fn new() -> Self {
        Self {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::storage::StorageBackend for InMemoryStorage {
    async fn save(&self, key: &str, value: &[u8]) -> crate::error::Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn load(&self, key: &str) -> crate::error::Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }

    async fn delete(&self, key: &str) -> crate::error::Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> crate::error::Result<bool> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }

    async fn list_keys(&self, prefix: &str) -> crate::error::Result<Vec<String>> {
        let data = self.data.read().unwrap();
        Ok(data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> crate::error::Result<()> {
        let mut data = self.data.write().unwrap();
        for (key, value) in items {
            data.insert(key, value);
        }
        Ok(())
    }

    async fn batch_load(&self, keys: Vec<String>) -> crate::error::Result<Vec<Option<Vec<u8>>>> {
        let data = self.data.read().unwrap();
        Ok(keys.iter().map(|k| data.get(k).cloned()).collect())
    }
}

/// 工作流模块
///
/// 负责注册工作流引擎、审计日志器等服务
pub struct WorkflowModule {
    max_parallel_workflows: usize,
}

impl WorkflowModule {
    /// 创建新的工作流模块
    pub fn new() -> Self {
        Self {
            max_parallel_workflows: 4,
        }
    }

    /// 设置最大并行工作流数
    pub fn with_max_parallel_workflows(mut self, max: usize) -> Self {
        self.max_parallel_workflows = max;
        self
    }
}

impl Default for WorkflowModule {
    fn default() -> Self {
        Self::new()
    }
}

impl AppModule for WorkflowModule {
    fn name(&self) -> &str {
        "workflow"
    }

    fn configure(&self, container: &DiContainer) {
        let max_parallel = self.max_parallel_workflows;

        container.register_factory::<WorkflowEngineServiceImpl, _>(move || {
            let storage = Arc::new(InMemoryStorage::new());
            let cache = Arc::new(SimpleMemoryCache::new());
            let state_manager = Arc::new(StateManager::new(storage, cache));
            let tool_registry = Arc::new(ToolRegistry::new());

            let engine = RefactoredWorkflowEngine::new(state_manager, tool_registry, max_parallel);

            Arc::new(WorkflowEngineServiceImpl::new(engine))
        });

        container.register_factory::<AuditLogServiceImpl, _>(|| {
            let storage = Arc::new(InMemoryStorage::new());
            let cache = Arc::new(SimpleMemoryCache::new());
            let state_manager = Arc::new(StateManager::new(storage, cache));
            let logger = AuditLogger::new(state_manager, false, 30);
            Arc::new(AuditLogServiceImpl::new(logger))
        });
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["storage", "tools"]
    }
}

/// 工作流引擎服务 trait
pub trait WorkflowEngineService: Send + Sync {
    /// 获取最大并行工作流数
    fn max_parallel(&self) -> usize;
}

/// 工作流引擎服务实现
struct WorkflowEngineServiceImpl {
    engine: Arc<RefactoredWorkflowEngine>,
    max_parallel: usize,
}

impl WorkflowEngineServiceImpl {
    fn new(engine: RefactoredWorkflowEngine) -> Self {
        Self {
            max_parallel: 4,
            engine: Arc::new(engine),
        }
    }
}

impl WorkflowEngineService for WorkflowEngineServiceImpl {
    fn max_parallel(&self) -> usize {
        self.max_parallel
    }
}

/// 状态管理器服务 trait
pub trait StateManagerService: Send + Sync {}

/// 工具注册表服务 trait
pub trait ToolRegistryService: Send + Sync {}

/// 审计日志服务 trait
pub trait AuditLogService: Send + Sync {
    /// 获取是否启用
    fn is_enabled(&self) -> bool;
}

/// 审计日志服务实现
struct AuditLogServiceImpl {
    logger: AuditLogger,
}

impl AuditLogServiceImpl {
    fn new(logger: AuditLogger) -> Self {
        Self { logger }
    }
}

impl AuditLogService for AuditLogServiceImpl {
    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_module_creation() {
        let module = WorkflowModule::new();
        assert_eq!(module.name(), "workflow");
    }

    #[test]
    fn test_workflow_module_dependencies() {
        let module = WorkflowModule::new();
        let deps = module.dependencies();
        assert!(deps.contains(&"storage"));
        assert!(deps.contains(&"tools"));
    }

    #[test]
    fn test_workflow_module_with_config() {
        let module = WorkflowModule::new().with_max_parallel_workflows(8);
        assert_eq!(module.max_parallel_workflows, 8);
    }
}
