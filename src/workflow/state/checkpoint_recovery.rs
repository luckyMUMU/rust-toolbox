//! 检查点恢复模块
//!
//! 提供工作流执行状态的保存、恢复和验证功能

use crate::core::ExecutionStatus;
use crate::error::{Result, WorkflowError};
use crate::storage::StateManager;
use crate::workflow::context::DataContext;
use crate::workflow::execution::NodeExecutionState;
use crate::workflow::state::ExecutionTracker;
use crate::workflow::WorkflowDefinition;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// 检查点验证结果
#[derive(Debug, Clone)]
pub enum CheckpointValidation {
    /// 验证通过
    Valid,
    /// 验证通过但有警告
    ValidWithWarnings(Vec<String>),
    /// 验证失败
    Invalid(Vec<String>),
}

/// 检查点恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// 从失败节点重新执行
    FromFailedNode,
    /// 从上一个成功检查点重新执行
    FromLastCheckpoint,
    /// 从指定节点重新执行
    FromSpecificNode(String),
    /// 完全重新开始
    Restart,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self::FromFailedNode
    }
}

/// 检查点恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// 恢复策略
    pub strategy: RecoveryStrategy,
    /// 最大恢复尝试次数
    pub max_recovery_attempts: u32,
    /// 恢复超时时间
    pub recovery_timeout: Duration,
    /// 是否验证工作流定义一致性
    pub validate_definition: bool,
    /// 是否清理无效检查点
    pub cleanup_invalid: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            strategy: RecoveryStrategy::default(),
            max_recovery_attempts: 3,
            recovery_timeout: Duration::from_secs(300),
            validate_definition: true,
            cleanup_invalid: true,
        }
    }
}

/// 增强的执行检查点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedCheckpoint {
    /// 检查点 ID
    pub id: String,
    /// 工作流执行 ID
    pub workflow_id: Uuid,
    /// 工作流名称
    pub workflow_name: String,
    /// 工作流定义哈希（用于验证一致性）
    pub definition_hash: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 节点执行状态
    pub node_states: HashMap<String, NodeExecutionState>,
    /// 全局上下文槽
    pub global_slots: HashMap<String, serde_json::Value>,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 序列号
    pub sequence: u64,
    /// 检查点类型
    pub checkpoint_type: CheckpointType,
    /// 检查点状态
    pub status: CheckpointStatus,
}

/// 检查点类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CheckpointType {
    /// 定时检查点
    Scheduled,
    /// 节点完成后检查点
    AfterNode,
    /// 错误时检查点
    OnError,
    /// 手动检查点
    Manual,
    /// 暂停时检查点
    OnPause,
}

/// 检查点状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointStatus {
    /// 有效
    Valid,
    /// 已过期
    Expired,
    /// 已损坏
    Corrupted,
    /// 已恢复
    Restored,
}

/// 检查点恢复器
pub struct CheckpointRecovery {
    state_manager: Arc<StateManager>,
    config: RecoveryConfig,
    checkpoints: tokio::sync::RwLock<HashMap<String, EnhancedCheckpoint>>,
}

impl CheckpointRecovery {
    /// 创建新的检查点恢复器
    pub fn new(state_manager: Arc<StateManager>, config: RecoveryConfig) -> Self {
        Self {
            state_manager,
            config,
            checkpoints: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// 使用默认配置创建恢复器
    pub fn default_recovery(state_manager: Arc<StateManager>) -> Self {
        Self::new(state_manager, RecoveryConfig::default())
    }

    /// 验证检查点
    pub fn validate_checkpoint(
        &self,
        checkpoint: &EnhancedCheckpoint,
        current_definition: Option<&WorkflowDefinition>,
    ) -> CheckpointValidation {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        if checkpoint.status == CheckpointStatus::Corrupted {
            errors.push("检查点已损坏".to_string());
        }

        if checkpoint.status == CheckpointStatus::Expired {
            warnings.push("检查点已过期".to_string());
        }

        if let Some(definition) = current_definition {
            if self.config.validate_definition {
                let current_hash = Self::compute_definition_hash(definition);
                if current_hash != checkpoint.definition_hash {
                    errors.push("工作流定义与检查点不匹配".to_string());
                }
            }
        }

        let failed_nodes: Vec<_> = checkpoint
            .node_states
            .iter()
            .filter(|(_, state)| state.status == ExecutionStatus::Failed)
            .map(|(id, _)| id.clone())
            .collect();

        if !failed_nodes.is_empty() {
            warnings.push(format!("检查点包含失败节点: {:?}", failed_nodes));
        }

        if !errors.is_empty() {
            CheckpointValidation::Invalid(errors)
        } else if !warnings.is_empty() {
            CheckpointValidation::ValidWithWarnings(warnings)
        } else {
            CheckpointValidation::Valid
        }
    }

    /// 计算工作流定义哈希
    fn compute_definition_hash(definition: &WorkflowDefinition) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        definition.name.hash(&mut hasher);
        definition.version.hash(&mut hasher);
        for node in &definition.nodes {
            node.id.hash(&mut hasher);
        }
        for edge in &definition.edges {
            edge.from.hash(&mut hasher);
            edge.to.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// 从检查点恢复执行状态
    pub async fn restore_from_checkpoint(
        &self,
        checkpoint: &EnhancedCheckpoint,
        tracker: &Arc<ExecutionTracker>,
    ) -> Result<RecoveryContext> {
        info!(
            workflow_id = %checkpoint.workflow_id,
            checkpoint_id = %checkpoint.id,
            "从检查点恢复执行状态"
        );

        let context = DataContext::new();
        context.import_global_slots(checkpoint.global_slots.clone())?;

        for (node_id, state) in &checkpoint.node_states {
            tracker.set_node_state(node_id, state.clone());
        }

        let recovery_context = RecoveryContext {
            checkpoint_id: checkpoint.id.clone(),
            workflow_id: checkpoint.workflow_id,
            node_states: checkpoint.node_states.clone(),
            input_params: checkpoint.input_params.clone(),
            recovery_strategy: self.config.strategy.clone(),
            recovered_at: Utc::now(),
        };

        Ok(recovery_context)
    }

    /// 确定恢复起始点
    pub fn determine_recovery_start(
        &self,
        checkpoint: &EnhancedCheckpoint,
    ) -> Result<Vec<String>> {
        match &self.config.strategy {
            RecoveryStrategy::FromFailedNode => {
                let failed_nodes: Vec<_> = checkpoint
                    .node_states
                    .iter()
                    .filter(|(_, state)| state.status == ExecutionStatus::Failed)
                    .map(|(id, _)| id.clone())
                    .collect();

                if failed_nodes.is_empty() {
                    let pending_nodes: Vec<_> = checkpoint
                        .node_states
                        .iter()
                        .filter(|(_, state)| state.status == ExecutionStatus::Pending)
                        .map(|(id, _)| id.clone())
                        .collect();

                    if pending_nodes.is_empty() {
                        Err(WorkflowError::validation("检查点没有需要恢复的节点"))
                    } else {
                        Ok(pending_nodes)
                    }
                } else {
                    Ok(failed_nodes)
                }
            }
            RecoveryStrategy::FromLastCheckpoint => {
                let pending_nodes: Vec<_> = checkpoint
                    .node_states
                    .iter()
                    .filter(|(_, state)| {
                        state.status == ExecutionStatus::Pending
                            || state.status == ExecutionStatus::Failed
                    })
                    .map(|(id, _)| id.clone())
                    .collect();

                Ok(pending_nodes)
            }
            RecoveryStrategy::FromSpecificNode(node_id) => {
                if checkpoint.node_states.contains_key(node_id) {
                    Ok(vec![node_id.clone()])
                } else {
                    Err(WorkflowError::validation(format!(
                        "检查点不包含节点: {}",
                        node_id
                    )))
                }
            }
            RecoveryStrategy::Restart => {
                Ok(checkpoint.node_states.keys().cloned().collect())
            }
        }
    }

    /// 保存检查点
    pub async fn save_checkpoint(
        &self,
        checkpoint: EnhancedCheckpoint,
    ) -> Result<()> {
        let checkpoint_id = checkpoint.id.clone();
        debug!(
            checkpoint_id = %checkpoint_id,
            workflow_id = %checkpoint.workflow_id,
            "保存检查点"
        );

        let key = format!(
            "checkpoint:{}:{}:{}",
            checkpoint.workflow_id,
            checkpoint.sequence,
            checkpoint_id
        );

        let value = serde_json::to_string(&checkpoint).map_err(|e| {
            WorkflowError::serialization(format!("检查点序列化失败: {}", e))
        })?;

        self.state_manager
            .get_cache_backend()
            .set(&key, value.as_bytes().to_vec(), Some(Duration::from_secs(86400)))
            .await
            .map_err(|e| {
                WorkflowError::storage(format!("检查点保存失败: {}", e))
            })?;

        self.checkpoints
            .write()
            .await
            .insert(checkpoint_id, checkpoint);

        Ok(())
    }

    /// 加载检查点
    pub async fn load_checkpoint(
        &self,
        workflow_id: Uuid,
        checkpoint_id: &str,
    ) -> Result<Option<EnhancedCheckpoint>> {
        // 首先检查内存缓存
        let checkpoints = self.checkpoints.read().await;
        if let Some(checkpoint) = checkpoints.get(checkpoint_id) {
            if checkpoint.workflow_id == workflow_id {
                return Ok(Some(checkpoint.clone()));
            }
        }
        drop(checkpoints);

        // 如果内存中没有，尝试从存储中加载
        let key = format!("checkpoint:{}:{}", workflow_id, checkpoint_id);

        let value = self
            .state_manager
            .get_cache_backend()
            .get(&key)
            .await;

        match value {
            Some(data) => {
                let checkpoint: EnhancedCheckpoint = serde_json::from_slice(&data).map_err(|e| {
                    WorkflowError::serialization(format!("检查点反序列化失败: {}", e))
                })?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    /// 获取工作流的最新检查点
    pub async fn get_latest_checkpoint(
        &self,
        workflow_id: Uuid,
    ) -> Result<Option<EnhancedCheckpoint>> {
        let checkpoints = self.checkpoints.read().await;

        let latest = checkpoints
            .values()
            .filter(|c| c.workflow_id == workflow_id && c.status == CheckpointStatus::Valid)
            .max_by_key(|c| c.sequence);

        Ok(latest.cloned())
    }

    /// 列出工作流的所有检查点
    pub async fn list_checkpoints(
        &self,
        workflow_id: Uuid,
    ) -> Vec<EnhancedCheckpoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints
            .values()
            .filter(|c| c.workflow_id == workflow_id)
            .cloned()
            .collect()
    }

    /// 清理过期检查点
    pub async fn cleanup_expired_checkpoints(
        &self,
        workflow_id: Uuid,
        keep_count: usize,
    ) -> Result<usize> {
        let mut checkpoints = self.checkpoints.write().await;
        let workflow_checkpoints: Vec<_> = checkpoints
            .values()
            .filter(|c| c.workflow_id == workflow_id)
            .cloned()
            .collect();

        if workflow_checkpoints.len() <= keep_count {
            return Ok(0);
        }

        let mut sorted: Vec<_> = workflow_checkpoints.into_iter().collect();
        sorted.sort_by_key(|c| std::cmp::Reverse(c.sequence));

        let to_remove: Vec<_> = sorted
            .into_iter()
            .skip(keep_count)
            .map(|c| c.id)
            .collect();

        let removed_count = to_remove.len();
        for id in to_remove {
            checkpoints.remove(&id);
        }

        info!(
            workflow_id = %workflow_id,
            removed_count = removed_count,
            "清理过期检查点"
        );

        Ok(removed_count)
    }

    /// 标记检查点为已恢复
    pub async fn mark_checkpoint_restored(
        &self,
        checkpoint_id: &str,
    ) -> Result<()> {
        let mut checkpoints = self.checkpoints.write().await;
        if let Some(checkpoint) = checkpoints.get_mut(checkpoint_id) {
            checkpoint.status = CheckpointStatus::Restored;
        }
        Ok(())
    }

    /// 获取恢复配置
    pub fn config(&self) -> &RecoveryConfig {
        &self.config
    }
}

/// 恢复上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryContext {
    /// 检查点 ID
    pub checkpoint_id: String,
    /// 工作流 ID
    pub workflow_id: Uuid,
    /// 节点状态
    pub node_states: HashMap<String, NodeExecutionState>,
    /// 输入参数
    pub input_params: HashMap<String, serde_json::Value>,
    /// 恢复策略
    pub recovery_strategy: RecoveryStrategy,
    /// 恢复时间
    pub recovered_at: DateTime<Utc>,
}

/// 检查点构建器
pub struct CheckpointBuilder {
    workflow_id: Uuid,
    workflow_name: String,
    definition_hash: String,
    node_states: HashMap<String, NodeExecutionState>,
    global_slots: HashMap<String, serde_json::Value>,
    input_params: HashMap<String, serde_json::Value>,
    checkpoint_type: CheckpointType,
}

impl CheckpointBuilder {
    /// 创建新的检查点构建器
    pub fn new(workflow_id: Uuid, workflow_name: String, definition_hash: String) -> Self {
        Self {
            workflow_id,
            workflow_name,
            definition_hash,
            node_states: HashMap::new(),
            global_slots: HashMap::new(),
            input_params: HashMap::new(),
            checkpoint_type: CheckpointType::Scheduled,
        }
    }

    /// 添加节点状态
    pub fn with_node_states(mut self, states: HashMap<String, NodeExecutionState>) -> Self {
        self.node_states = states;
        self
    }

    /// 添加全局槽
    pub fn with_global_slots(mut self, slots: HashMap<String, serde_json::Value>) -> Self {
        self.global_slots = slots;
        self
    }

    /// 添加输入参数
    pub fn with_input_params(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.input_params = params;
        self
    }

    /// 设置检查点类型
    pub fn with_type(mut self, checkpoint_type: CheckpointType) -> Self {
        self.checkpoint_type = checkpoint_type;
        self
    }

    /// 构建检查点
    pub fn build(self) -> EnhancedCheckpoint {
        static SEQUENCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

        EnhancedCheckpoint {
            id: Uuid::new_v4().to_string(),
            workflow_id: self.workflow_id,
            workflow_name: self.workflow_name,
            definition_hash: self.definition_hash,
            created_at: Utc::now(),
            node_states: self.node_states,
            global_slots: self.global_slots,
            input_params: self.input_params,
            sequence: SEQUENCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            checkpoint_type: self.checkpoint_type,
            status: CheckpointStatus::Valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert!(matches!(config.strategy, RecoveryStrategy::FromFailedNode));
        assert_eq!(config.max_recovery_attempts, 3);
    }

    #[test]
    fn test_checkpoint_builder() {
        let checkpoint = CheckpointBuilder::new(
            Uuid::new_v4(),
            "test-workflow".to_string(),
            "hash123".to_string(),
        )
        .with_type(CheckpointType::Manual)
        .build();

        assert_eq!(checkpoint.workflow_name, "test-workflow");
        assert_eq!(checkpoint.checkpoint_type, CheckpointType::Manual);
        assert_eq!(checkpoint.status, CheckpointStatus::Valid);
    }

    #[test]
    fn test_checkpoint_validation() {
        let checkpoint = CheckpointBuilder::new(
            Uuid::new_v4(),
            "test".to_string(),
            "hash".to_string(),
        )
        .build();

        let recovery = CheckpointRecovery::default_recovery(Arc::new(StateManager::default()));
        let validation = recovery.validate_checkpoint(&checkpoint, None);

        assert!(matches!(validation, CheckpointValidation::Valid));
    }

    #[test]
    fn test_recovery_strategy_determine_start() {
        let mut node_states = HashMap::new();
        node_states.insert(
            "node1".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Completed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: None,
                error: None,
                retry_count: 0,
            },
        );
        node_states.insert(
            "node2".to_string(),
            NodeExecutionState {
                status: ExecutionStatus::Failed,
                started_at: Some(Utc::now()),
                completed_at: Some(Utc::now()),
                result: None,
                error: Some("error".to_string()),
                retry_count: 1,
            },
        );

        let checkpoint = CheckpointBuilder::new(
            Uuid::new_v4(),
            "test".to_string(),
            "hash".to_string(),
        )
        .with_node_states(node_states)
        .build();

        let recovery = CheckpointRecovery::default_recovery(Arc::new(StateManager::default()));
        let start_nodes = recovery.determine_recovery_start(&checkpoint).unwrap();

        assert_eq!(start_nodes, vec!["node2"]);
    }
}
