use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::workflow::WorkflowDefinition;
use crate::mcp::node::McpNode;

/// MCP 工作流，扩展现有 WorkflowDefinition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflow {
    /// 基础工作流定义
    pub base: WorkflowDefinition,
    
    /// MCP 工作流配置
    pub mcp_config: McpWorkflowConfig,
    
    /// MCP 节点列表
    pub mcp_nodes: Vec<McpNode>,
    
    /// 上下文初始化配置
    pub context_initialization: ContextInitialization,
    
    /// 全局上下文更新规则
    pub global_context_updates: Vec<GlobalContextUpdate>,
}

/// MCP 工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflowConfig {
    /// 是否支持 MCP
    pub mcp_supported: bool,
    
    /// MCP 工作流版本
    pub mcp_version: String,
    
    /// 工作流能力描述
    pub capabilities: Value,
    
    /// 是否需要完整上下文
    pub requires_full_context: bool,
    
    /// 上下文验证规则
    pub context_validation_rules: Value,
    
    /// 上下文传递策略
    pub context_propagation_strategy: ContextPropagationStrategy,
}

/// 上下文初始化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInitialization {
    /// 初始上下文数据
    pub initial_context: Value,
    
    /// 上下文模板
    pub context_template: Option<String>,
    
    /// 上下文来源
    pub context_source: ContextSource,
    
    /// 验证规则
    pub validation_rules: Value,
}

/// 上下文来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextSource {
    /// 内置默认上下文
    BuiltIn,
    /// 外部提供的上下文
    External,
    /// 从文件加载
    File,
    /// 从环境变量加载
    Environment,
    /// 自定义生成
    Custom,
}

/// 上下文传递策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextPropagationStrategy {
    /// 完整传递（默认）
    Full,
    /// 增量传递
    Incremental,
    /// 选择性传递
    Selective,
    /// 基于规则的传递
    RuleBased,
}

/// 全局上下文更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalContextUpdate {
    /// 触发条件
    pub trigger_condition: String,
    /// 更新规则
    pub update_rule: ContextUpdateRule,
}

/// 上下文更新规则（与节点模块中的相同，这里重复定义是为了模块独立性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdateRule {
    /// 目标上下文路径
    pub target_path: String,
    /// 更新类型
    pub update_type: UpdateType,
    /// 更新表达式
    pub expression: Option<String>,
    /// 条件表达式
    pub condition: Option<String>,
}

/// 更新类型（与节点模块中的相同，这里重复定义是为了模块独立性）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateType {
    /// 替换
    Replace,
    /// 合并
    Merge,
    /// 追加（数组）
    Append,
    /// 移除
    Remove,
    /// 条件更新
    Conditional,
}

impl McpWorkflow {
    /// 从基础工作流定义创建 MCP 工作流
    pub fn from_base(base: WorkflowDefinition) -> Self {
        let mut mcp_nodes = Vec::new();
        for node in &base.nodes {
            mcp_nodes.push(crate::mcp::node::McpNode::from_base(node.clone()));
        }
        
        Self {
            base,
            mcp_config: McpWorkflowConfig {
                mcp_supported: false,
                mcp_version: "1.0.0".to_string(),
                capabilities: Value::Object(serde_json::Map::new()),
                requires_full_context: false,
                context_validation_rules: Value::Object(serde_json::Map::new()),
                context_propagation_strategy: ContextPropagationStrategy::Full,
            },
            mcp_nodes,
            context_initialization: ContextInitialization {
                initial_context: Value::Object(serde_json::Map::new()),
                context_template: None,
                context_source: ContextSource::BuiltIn,
                validation_rules: Value::Object(serde_json::Map::new()),
            },
            global_context_updates: Vec::new(),
        }
    }
    
    /// 设置为 MCP 支持的工作流
    pub fn set_mcp_supported(mut self, supported: bool) -> Self {
        self.mcp_config.mcp_supported = supported;
        for node in &mut self.mcp_nodes {
            node.mcp_config.mcp_supported = supported;
        }
        self
    }
    
    /// 设置上下文传递策略
    pub fn set_context_propagation_strategy(mut self, strategy: ContextPropagationStrategy) -> Self {
        self.mcp_config.context_propagation_strategy = strategy;
        self
    }
    
    /// 添加全局上下文更新规则
    pub fn add_global_context_update(mut self, update: GlobalContextUpdate) -> Self {
        self.global_context_updates.push(update);
        self
    }
    
    /// 更新 MCP 节点
    pub fn update_mcp_node(&mut self, node_id: &str, mcp_node: McpNode) {
        if let Some(index) = self.mcp_nodes.iter().position(|n| n.base.id == node_id) {
            self.mcp_nodes[index] = mcp_node;
        }
    }
    
    /// 获取 MCP 节点
    pub fn get_mcp_node(&self, node_id: &str) -> Option<&McpNode> {
        self.mcp_nodes.iter().find(|n| n.base.id == node_id)
    }
}
