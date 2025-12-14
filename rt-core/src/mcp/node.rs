use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::workflow::WorkflowNode;

/// MCP 工作流节点，扩展现有 WorkflowNode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNode {
    /// 基础工作流节点
    pub base: WorkflowNode,
    
    /// MCP 节点配置
    pub mcp_config: McpNodeConfig,
    
    /// 上下文映射规则
    pub context_mappings: Vec<ContextMapping>,
    
    /// 节点输出上下文更新规则
    pub output_context_updates: Vec<ContextUpdateRule>,
}

/// MCP 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNodeConfig {
    /// 是否支持 MCP
    pub mcp_supported: bool,
    
    /// MCP 节点类型
    pub node_type: McpNodeType,
    
    /// MCP 能力描述
    pub capabilities: Value,
    
    /// 是否需要完整上下文
    pub requires_full_context: bool,
    
    /// 上下文验证规则
    pub context_validation_rules: Value,
}

/// MCP 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum McpNodeType {
    /// 工具节点
    Tool,
    
    /// 插件节点
    Plugin,
    
    /// 决策节点
    Decision,
    
    /// 条件节点
    Condition,
    
    /// 循环节点
    Loop,
    
    /// 子工作流节点
    SubWorkflow,
    
    /// 系统节点
    System,
}

/// 上下文映射规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMapping {
    /// 源上下文路径
    pub source_path: String,
    
    /// 目标上下文路径
    pub target_path: String,
    
    /// 映射类型
    pub mapping_type: MappingType,
    
    /// 映射表达式（如模板、JSONPath 等）
    pub expression: Option<String>,
    
    /// 是否可选
    pub optional: bool,
    
    /// 默认值
    pub default_value: Option<Value>,
}

/// 映射类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MappingType {
    /// 直接映射
    Direct,
    
    /// 模板映射
    Template,
    
    /// JSONPath 映射
    JsonPath,
    
    /// 函数映射
    Function,
    
    /// 条件映射
    Condition,
}

/// 上下文更新规则
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

/// 更新类型
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

impl McpNode {
    /// 从基础工作流节点创建 MCP 节点
    pub fn from_base(base: WorkflowNode) -> Self {
        Self {
            base,
            mcp_config: McpNodeConfig {
                mcp_supported: false,
                node_type: McpNodeType::Tool,
                capabilities: Value::Object(serde_json::Map::new()),
                requires_full_context: false,
                context_validation_rules: Value::Object(serde_json::Map::new()),
            },
            context_mappings: Vec::new(),
            output_context_updates: Vec::new(),
        }
    }
    
    /// 设置为 MCP 支持的节点
    pub fn set_mcp_supported(mut self, supported: bool) -> Self {
        self.mcp_config.mcp_supported = supported;
        self
    }
    
    /// 设置节点类型
    pub fn set_node_type(mut self, node_type: McpNodeType) -> Self {
        self.mcp_config.node_type = node_type;
        self
    }
    
    /// 添加上下文映射
    pub fn add_context_mapping(mut self, mapping: ContextMapping) -> Self {
        self.context_mappings.push(mapping);
        self
    }
    
    /// 添加输出上下文更新规则
    pub fn add_output_context_update(mut self, update: ContextUpdateRule) -> Self {
        self.output_context_updates.push(update);
        self
    }
}
