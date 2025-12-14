use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use uuid::Uuid;
use crate::error::{Result, CoreError};
use crate::mcp::context::{McpContext, ExecutionRecord, ComponentType, ExecutionStatus};
use crate::mcp::node::McpNode;
use crate::mcp::workflow::{McpWorkflow, ContextPropagationStrategy};

/// 上下文管理器，负责上下文的创建、更新和传递
#[derive(Debug)]
pub struct ContextManager {
    /// 上下文存储，使用 Arc 和 RwLock 实现线程安全
    contexts: Arc<RwLock<std::collections::HashMap<String, McpContext>>>,
    
    /// 上下文传递策略
    default_propagation_strategy: ContextPropagationStrategy,
}

impl ContextManager {
    /// 创建一个新的上下文管理器
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_propagation_strategy: ContextPropagationStrategy::Full,
        }
    }
    
    /// 创建一个新的上下文
    pub async fn create_context(&self) -> McpContext {
        let context = McpContext::new();
        let context_id = context.id.clone();
        
        let mut contexts = self.contexts.write().await;
        contexts.insert(context_id, context.clone());
        
        context
    }
    
    /// 创建一个基于现有上下文的子上下文
    pub async fn create_child_context(&self, parent_context_id: &str) -> Result<McpContext> {
        let contexts = self.contexts.read().await;
        let parent_context = contexts.get(parent_context_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("Parent context not found: {}", parent_context_id)))?;
        
        let child_context = parent_context.create_child();
        drop(contexts);
        
        let mut contexts = self.contexts.write().await;
        contexts.insert(child_context.id.clone(), child_context.clone());
        
        Ok(child_context)
    }
    
    /// 获取上下文
    pub async fn get_context(&self, context_id: &str) -> Result<McpContext> {
        let contexts = self.contexts.read().await;
        contexts.get(context_id)
            .cloned()
            .ok_or_else(|| CoreError::InvalidInput(format!("Context not found: {}", context_id)))
    }
    
    /// 更新上下文
    pub async fn update_context(&self, context: &McpContext) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.id.clone(), context.clone());
        Ok(())
    }
    
    /// 删除上下文
    pub async fn delete_context(&self, context_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        contexts.remove(context_id);
        Ok(())
    }
    
    /// 传递上下文到下一个节点
    pub async fn propagate_context(
        &self, 
        source_context: &McpContext, 
        target_node: &McpNode,
        workflow: &McpWorkflow,
    ) -> Result<McpContext> {
        let strategy = workflow.mcp_config.context_propagation_strategy.clone();
        
        let propagated_context = match strategy {
            ContextPropagationStrategy::Full => {
                self.propagate_full_context(source_context, target_node).await
            },
            ContextPropagationStrategy::Incremental => {
                self.propagate_incremental_context(source_context, target_node).await
            },
            ContextPropagationStrategy::Selective => {
                self.propagate_selective_context(source_context, target_node).await
            },
            ContextPropagationStrategy::RuleBased => {
                self.propagate_rule_based_context(source_context, target_node, workflow).await
            },
        };
        
        Ok(propagated_context)
    }
    
    /// 完整传递上下文
    async fn propagate_full_context(&self, source_context: &McpContext, _target_node: &McpNode) -> McpContext {
        source_context.clone()
    }
    
    /// 增量传递上下文
    async fn propagate_incremental_context(&self, source_context: &McpContext, _target_node: &McpNode) -> McpContext {
        // 这里实现增量传递逻辑，只传递需要更新的部分
        // 目前简单实现为创建子上下文
        source_context.create_child()
    }
    
    /// 选择性传递上下文
    async fn propagate_selective_context(&self, source_context: &McpContext, target_node: &McpNode) -> McpContext {
        // 根据目标节点的上下文映射规则选择性传递上下文
        let mut context = source_context.create_child();
        
        // 应用上下文映射规则
        for mapping in &target_node.context_mappings {
            self.apply_context_mapping(&mut context, source_context, mapping).await;
        }
        
        context
    }
    
    /// 基于规则传递上下文
    async fn propagate_rule_based_context(
        &self, 
        source_context: &McpContext, 
        target_node: &McpNode,
        workflow: &McpWorkflow,
    ) -> McpContext {
        // 结合全局规则和节点规则传递上下文
        let mut context = self.propagate_selective_context(source_context, target_node).await;
        
        // 应用全局上下文更新规则
        for global_update in &workflow.global_context_updates {
            self.apply_global_context_update(&mut context, global_update).await;
        }
        
        context
    }
    
    /// 应用上下文映射规则
    async fn apply_context_mapping(&self, target: &mut McpContext, source: &McpContext, mapping: &crate::mcp::node::ContextMapping) {
        // 这里实现上下文映射逻辑
        // 目前简单实现，实际应该根据映射类型和表达式进行处理
        let source_value = self.get_value_from_path(&source.custom_data, &mapping.source_path);
        let value_to_set = match source_value {
            Some(v) => v,
            None if mapping.optional => return,
            None => mapping.default_value.clone().unwrap_or(Value::Null),
        };
        
        self.set_value_to_path(&mut target.custom_data, &mapping.target_path, value_to_set);
    }
    
    /// 应用全局上下文更新规则
    async fn apply_global_context_update(&self, context: &mut McpContext, update: &crate::mcp::workflow::GlobalContextUpdate) {
        // 这里实现全局上下文更新逻辑
        // 目前简单实现，实际应该根据触发条件和更新规则进行处理
        self.set_value_to_path(&mut context.custom_data, &update.update_rule.target_path, Value::Null);
    }
    
    /// 从路径获取值
    fn get_value_from_path(&self, data: &Value, path: &str) -> Option<Value> {
        let mut current = data;
        let parts: Vec<&str> = path.split('.').collect();
        
        for part in parts {
            match current {
                Value::Object(map) => {
                    if let Some(v) = map.get(part) {
                        current = v;
                    } else {
                        return None;
                    }
                },
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        if index < arr.len() {
                            current = &arr[index];
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                },
                _ => return None,
            }
        }
        
        Some(current.clone())
    }
    
    /// 设置值到路径
    fn set_value_to_path(&self, data: &mut Value, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;
        
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // 最后一个部分，直接设置值
                match current {
                    Value::Object(map) => {
                        map.insert(part.to_string(), value);
                    },
                    Value::Array(arr) => {
                        if let Ok(index) = part.parse::<usize>() {
                            if index < arr.len() {
                                arr[index] = value;
                            }
                        }
                    },
                    _ => {},
                }
                return;
            }
            
            // 创建路径中的缺失部分
            match current {
                Value::Object(map) => {
                    let part_str = part.to_string();
                    if !map.contains_key(&part_str) {
                        map.insert(part_str.clone(), Value::Object(serde_json::Map::new()));
                    }
                    current = map.get_mut(&part_str).unwrap();
                },
                _ => return,
            }
        }
    }
    
    /// 添加执行记录到上下文
    pub async fn add_execution_record(&self, context_id: &str, record: ExecutionRecord) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts.get_mut(context_id)
            .ok_or_else(|| CoreError::InvalidInput(format!("Context not found: {}", context_id)))?;
        
        context.add_execution_record(record);
        Ok(())
    }
    
    /// 创建执行记录
    pub fn create_execution_record(
        &self,
        component_type: ComponentType,
        component_name: &str,
        method: &str,
        input: Value,
        output: Option<Value>,
        status: ExecutionStatus,
        error: Option<String>,
        duration_ms: Option<u64>,
    ) -> ExecutionRecord {
        ExecutionRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            component_type,
            component_name: component_name.to_string(),
            method: method.to_string(),
            input,
            output,
            status,
            error,
            duration_ms,
        }
    }
    
    /// 验证上下文
    pub async fn validate_context(&self, _context: &McpContext, _validation_rules: &Value) -> Result<()> {
        // 这里实现上下文验证逻辑
        // 目前简单实现，实际应该根据验证规则进行处理
        Ok(())
    }
    
    /// 设置默认上下文传递策略
    pub fn set_default_propagation_strategy(&mut self, strategy: ContextPropagationStrategy) {
        self.default_propagation_strategy = strategy;
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}
