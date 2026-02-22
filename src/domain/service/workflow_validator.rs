//! 工作流验证领域服务
//!
//! 提供工作流定义的业务规则验证

use crate::workflow::definition::WorkflowDefinition;
use crate::error::Result;
use std::collections::{HashMap, HashSet};

/// 验证规则
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationRule {
    /// 节点 ID 唯一性
    UniqueNodeIds,
    /// 无循环依赖
    NoCyclicDependencies,
    /// 有效起始节点
    HasStartNode,
    /// 有效终止节点
    HasEndNode,
    /// 工具节点必须有工具名
    ToolNodeHasToolName,
    /// 条件节点必须有条件
    ConditionNodeHasCondition,
    /// 循环节点必须有配置
    LoopNodeHasConfig,
    /// 边引用有效节点
    EdgesReferenceValidNodes,
    /// 最大节点数限制
    MaxNodeCount(usize),
    /// 最大边数限制
    MaxEdgeCount(usize),
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct DomainValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误列表
    pub errors: Vec<ValidationError>,
    /// 警告列表
    pub warnings: Vec<ValidationWarning>,
    /// 应用的规则
    pub applied_rules: Vec<ValidationRule>,
}

impl DomainValidationResult {
    /// 创建新的验证结果
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            applied_rules: Vec::new(),
        }
    }

    /// 添加错误
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// 添加警告
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// 记录应用的规则
    pub fn record_rule(&mut self, rule: ValidationRule) {
        self.applied_rules.push(rule);
    }
}

impl Default for DomainValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 相关节点 ID
    pub node_id: Option<String>,
    /// 相关字段
    pub field: Option<String>,
}

impl ValidationError {
    /// 创建新的验证错误
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            node_id: None,
            field: None,
        }
    }

    /// 设置节点 ID
    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    /// 设置字段
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }
}

/// 验证警告
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// 警告代码
    pub code: String,
    /// 警告消息
    pub message: String,
    /// 相关节点 ID
    pub node_id: Option<String>,
}

impl ValidationWarning {
    /// 创建新的验证警告
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            node_id: None,
        }
    }
}

/// 领域工作流验证器
pub struct DomainWorkflowValidator {
    /// 启用的规则
    enabled_rules: HashSet<ValidationRule>,
    /// 可用工具列表
    available_tools: HashSet<String>,
}

impl DomainWorkflowValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        let mut enabled_rules = HashSet::new();
        enabled_rules.insert(ValidationRule::UniqueNodeIds);
        enabled_rules.insert(ValidationRule::NoCyclicDependencies);
        enabled_rules.insert(ValidationRule::HasStartNode);
        enabled_rules.insert(ValidationRule::HasEndNode);
        enabled_rules.insert(ValidationRule::ToolNodeHasToolName);
        enabled_rules.insert(ValidationRule::ConditionNodeHasCondition);
        enabled_rules.insert(ValidationRule::LoopNodeHasConfig);
        enabled_rules.insert(ValidationRule::EdgesReferenceValidNodes);
        enabled_rules.insert(ValidationRule::MaxNodeCount(10000));
        enabled_rules.insert(ValidationRule::MaxEdgeCount(50000));

        Self {
            enabled_rules,
            available_tools: HashSet::new(),
        }
    }

    /// 创建带有工具列表的验证器
    pub fn with_tools(tools: HashSet<String>) -> Self {
        let mut validator = Self::new();
        validator.available_tools = tools;
        validator
    }

    /// 启用规则
    pub fn enable_rule(&mut self, rule: ValidationRule) {
        self.enabled_rules.insert(rule);
    }

    /// 禁用规则
    pub fn disable_rule(&mut self, rule: &ValidationRule) {
        self.enabled_rules.remove(rule);
    }

    /// 检查规则是否启用
    pub fn is_rule_enabled(&self, rule: &ValidationRule) -> bool {
        self.enabled_rules.contains(rule)
    }

    /// 验证工作流
    pub fn validate(&self, workflow: &WorkflowDefinition) -> Result<DomainValidationResult> {
        let mut result = DomainValidationResult::new();

        self.validate_basic_structure(workflow, &mut result);
        self.validate_nodes(workflow, &mut result)?;
        self.validate_edges(workflow, &mut result)?;
        self.validate_dag(workflow, &mut result)?;

        Ok(result)
    }

    /// 验证基本结构
    fn validate_basic_structure(&self, workflow: &WorkflowDefinition, result: &mut DomainValidationResult) {
        if workflow.name.is_empty() {
            result.add_error(ValidationError::new("EMPTY_NAME", "工作流名称不能为空"));
        }

        if workflow.version.is_empty() {
            result.add_error(ValidationError::new("EMPTY_VERSION", "工作流版本不能为空"));
        }

        if let Some(ValidationRule::MaxNodeCount(max)) = self.enabled_rules.iter().find_map(|r| {
            if let ValidationRule::MaxNodeCount(m) = r {
                Some(ValidationRule::MaxNodeCount(*m))
            } else {
                None
            }
        }) {
            result.record_rule(ValidationRule::MaxNodeCount(max));
            if workflow.nodes.len() > max {
                result.add_error(ValidationError::new(
                    "MAX_NODES_EXCEEDED",
                    format!("节点数量超过限制: {} > {}", workflow.nodes.len(), max),
                ));
            }
        }
    }

    /// 验证节点
    fn validate_nodes(&self, workflow: &WorkflowDefinition, result: &mut DomainValidationResult) -> Result<()> {
        let mut node_ids = HashSet::new();

        for node in &workflow.nodes {
            if self.is_rule_enabled(&ValidationRule::UniqueNodeIds) {
                if !node_ids.insert(node.id.clone()) {
                    result.add_error(
                        ValidationError::new("DUPLICATE_NODE_ID", "节点 ID 重复")
                            .with_node(&node.id),
                    );
                }
            }

            self.validate_node_type(node, result)?;
        }

        if self.is_rule_enabled(&ValidationRule::UniqueNodeIds) {
            result.record_rule(ValidationRule::UniqueNodeIds);
        }

        Ok(())
    }

    /// 验证节点类型
    fn validate_node_type(&self, node: &crate::workflow::definition::WorkflowNode, result: &mut DomainValidationResult) -> Result<()> {
        match node.node_type {
            crate::workflow::NodeType::Tool => {
                if self.is_rule_enabled(&ValidationRule::ToolNodeHasToolName) {
                    if node.tool_name.is_none() || node.tool_name.as_ref().map_or(true, |s| s.is_empty()) {
                        result.add_error(
                            ValidationError::new("MISSING_TOOL_NAME", "工具节点缺少工具名称")
                                .with_node(&node.id)
                                .with_field("tool_name"),
                        );
                    } else if !self.available_tools.is_empty() {
                        if let Some(tool_name) = &node.tool_name {
                            if !self.available_tools.contains(tool_name) {
                                result.add_error(
                                    ValidationError::new(
                                        "UNKNOWN_TOOL",
                                        format!("未知工具: {}", tool_name),
                                    )
                                    .with_node(&node.id)
                                    .with_field("tool_name"),
                                );
                            }
                        }
                    }
                    result.record_rule(ValidationRule::ToolNodeHasToolName);
                }
            }
            crate::workflow::NodeType::Condition => {
                if self.is_rule_enabled(&ValidationRule::ConditionNodeHasCondition) {
                    if node.parameters.is_null() {
                        result.add_error(
                            ValidationError::new("MISSING_CONDITION", "条件节点缺少条件配置")
                                .with_node(&node.id)
                                .with_field("parameters"),
                        );
                    }
                    result.record_rule(ValidationRule::ConditionNodeHasCondition);
                }
            }
            crate::workflow::NodeType::Loop => {
                if self.is_rule_enabled(&ValidationRule::LoopNodeHasConfig) {
                    if node.parameters.is_null() {
                        result.add_error(
                            ValidationError::new("MISSING_LOOP_CONFIG", "循环节点缺少循环配置")
                                .with_node(&node.id)
                                .with_field("parameters"),
                        );
                    }
                    result.record_rule(ValidationRule::LoopNodeHasConfig);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 验证边
    fn validate_edges(&self, workflow: &WorkflowDefinition, result: &mut DomainValidationResult) -> Result<()> {
        let node_ids: HashSet<_> = workflow.nodes.iter().map(|n| n.id.clone()).collect();

        if self.is_rule_enabled(&ValidationRule::EdgesReferenceValidNodes) {
            for edge in &workflow.edges {
                if !node_ids.contains(&edge.from) {
                    result.add_error(
                        ValidationError::new(
                            "INVALID_EDGE_SOURCE",
                            format!("边引用不存在的源节点: {}", edge.from),
                        )
                        .with_field("from"),
                    );
                }
                if !node_ids.contains(&edge.to) {
                    result.add_error(
                        ValidationError::new(
                            "INVALID_EDGE_TARGET",
                            format!("边引用不存在的目标节点: {}", edge.to),
                        )
                        .with_field("to"),
                    );
                }
            }
            result.record_rule(ValidationRule::EdgesReferenceValidNodes);
        }

        Ok(())
    }

    /// 验证 DAG 结构
    fn validate_dag(&self, workflow: &WorkflowDefinition, result: &mut DomainValidationResult) -> Result<()> {
        if self.is_rule_enabled(&ValidationRule::NoCyclicDependencies) {
            if self.has_cycle(workflow) {
                result.add_error(ValidationError::new(
                    "CYCLIC_DEPENDENCY",
                    "工作流存在循环依赖",
                ));
            }
            result.record_rule(ValidationRule::NoCyclicDependencies);
        }

        if self.is_rule_enabled(&ValidationRule::HasStartNode) {
            if !self.has_start_node(workflow) {
                result.add_warning(ValidationWarning::new(
                    "NO_START_NODE",
                    "工作流没有明确的起始节点",
                ));
            }
            result.record_rule(ValidationRule::HasStartNode);
        }

        if self.is_rule_enabled(&ValidationRule::HasEndNode) {
            if !self.has_end_node(workflow) {
                result.add_warning(ValidationWarning::new(
                    "NO_END_NODE",
                    "工作流没有明确的终止节点",
                ));
            }
            result.record_rule(ValidationRule::HasEndNode);
        }

        Ok(())
    }

    /// 检查是否有循环
    fn has_cycle(&self, workflow: &WorkflowDefinition) -> bool {
        let node_ids: Vec<_> = workflow.nodes.iter().map(|n| n.id.clone()).collect();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for node_id in &node_ids {
            adj.insert(node_id.clone(), Vec::new());
        }
        for edge in &workflow.edges {
            if let Some(neighbors) = adj.get_mut(&edge.from) {
                neighbors.push(edge.to.clone());
            }
        }

        for node_id in &node_ids {
            if self.dfs_has_cycle(node_id, &adj, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    /// DFS 检查循环
    fn dfs_has_cycle(
        &self,
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if self.dfs_has_cycle(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// 检查是否有起始节点
    fn has_start_node(&self, workflow: &WorkflowDefinition) -> bool {
        let target_nodes: HashSet<_> = workflow.edges.iter().map(|e| e.to.clone()).collect();
        workflow
            .nodes
            .iter()
            .any(|n| !target_nodes.contains(&n.id))
    }

    /// 检查是否有终止节点
    fn has_end_node(&self, workflow: &WorkflowDefinition) -> bool {
        let source_nodes: HashSet<_> = workflow.edges.iter().map(|e| e.from.clone()).collect();
        workflow
            .nodes
            .iter()
            .any(|n| !source_nodes.contains(&n.id))
    }
}

impl Default for DomainWorkflowValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = DomainWorkflowValidator::new();
        assert!(validator.is_rule_enabled(&ValidationRule::UniqueNodeIds));
        assert!(validator.is_rule_enabled(&ValidationRule::NoCyclicDependencies));
    }

    #[test]
    fn test_validation_result() {
        let mut result = DomainValidationResult::new();
        assert!(result.is_valid);

        result.add_error(ValidationError::new("TEST_ERROR", "测试错误"));
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_error_with_node() {
        let error = ValidationError::new("TEST", "message")
            .with_node("node1")
            .with_field("name");

        assert_eq!(error.node_id, Some("node1".to_string()));
        assert_eq!(error.field, Some("name".to_string()));
    }
}
