//! 仓储实现 - 实现领域层定义的仓储端口

use super::storage::StorageBackend;
use crate::domain::port::repository::{ExecutionRepository, PluginRepository, WorkflowRepository};
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// 工作流仓储实现
pub struct WorkflowRepositoryImpl {
    storage: Arc<dyn StorageBackend>,
}

impl WorkflowRepositoryImpl {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl WorkflowRepository for WorkflowRepositoryImpl {
    async fn save(&self, id: &str, workflow: Value) -> Result<()> {
        let key = format!("workflow:def:{}", id);
        let value = serde_json::to_vec(&workflow)?;
        self.storage.save(&key, &value).await
    }

    async fn load(&self, id: &str) -> Result<Option<Value>> {
        let key = format!("workflow:def:{}", id);
        if let Some(value) = self.storage.load(&key).await? {
            let workflow: Value = serde_json::from_slice(&value)?;
            Ok(Some(workflow))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let key = format!("workflow:def:{}", id);
        self.storage.delete(&key).await
    }

    async fn list(&self) -> Result<Vec<String>> {
        self.storage.list_keys("workflow:def:").await
    }
}

/// 执行记录仓储实现
pub struct ExecutionRepositoryImpl {
    storage: Arc<dyn StorageBackend>,
}

impl ExecutionRepositoryImpl {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl ExecutionRepository for ExecutionRepositoryImpl {
    async fn save_execution(&self, execution_id: &str, state: Value) -> Result<()> {
        let key = format!("execution:state:{}", execution_id);
        let value = serde_json::to_vec(&state)?;
        self.storage.save(&key, &value).await
    }

    async fn load_execution(&self, execution_id: &str) -> Result<Option<Value>> {
        let key = format!("execution:state:{}", execution_id);
        if let Some(value) = self.storage.load(&key).await? {
            let state: Value = serde_json::from_slice(&value)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    async fn save_history(&self, workflow_id: &str, record: Value) -> Result<()> {
        let key = format!(
            "workflow:history:{}:{}",
            workflow_id,
            chrono::Utc::now().timestamp()
        );
        let value = serde_json::to_vec(&record)?;
        self.storage.save(&key, &value).await
    }

    async fn load_history(&self, workflow_id: &str) -> Result<Vec<Value>> {
        let prefix = format!("workflow:history:{}", workflow_id);
        let keys = self.storage.list_keys(&prefix).await?;
        let mut records = Vec::new();

        for key in keys {
            if let Some(value) = self.storage.load(&key).await? {
                if let Ok(record) = serde_json::from_slice(&value) {
                    records.push(record);
                }
            }
        }

        Ok(records)
    }
}

/// 插件元数据仓储实现
pub struct PluginRepositoryImpl {
    storage: Arc<dyn StorageBackend>,
}

impl PluginRepositoryImpl {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl PluginRepository for PluginRepositoryImpl {
    async fn save_metadata(&self, plugin_id: &str, metadata: Value) -> Result<()> {
        let key = format!("plugin:meta:{}", plugin_id);
        let value = serde_json::to_vec(&metadata)?;
        self.storage.save(&key, &value).await
    }

    async fn load_metadata(&self, plugin_id: &str) -> Result<Option<Value>> {
        let key = format!("plugin:meta:{}", plugin_id);
        if let Some(value) = self.storage.load(&key).await? {
            let metadata: Value = serde_json::from_slice(&value)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    async fn list_metadata(&self) -> Result<Vec<Value>> {
        let keys = self.storage.list_keys("plugin:meta:").await?;
        let mut metadata_list = Vec::new();

        for key in keys {
            if let Some(value) = self.storage.load(&key).await? {
                if let Ok(metadata) = serde_json::from_slice(&value) {
                    metadata_list.push(metadata);
                }
            }
        }

        Ok(metadata_list)
    }
}
