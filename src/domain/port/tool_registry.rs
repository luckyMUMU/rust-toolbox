//! 工具注册表端口

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use crate::domain::model::{ExecutionContext, ToolInfo};
use crate::error::Result;

/// 工具注册表trait - 定义工具管理的核心接口
/// 
/// 这是领域层端口，具体实现位于基础设施层
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// 注册工具
    fn register_tool(&mut self, tool: Arc<dyn ToolNode>) -> Result<()>;
    
    /// 获取工具
    fn get_tool(&self, name: &str) -> Option<Arc<dyn ToolNode>>;
    
    /// 列出所有工具
    fn list_tools(&self) -> Vec<ToolInfo>;
    
    /// 执行工具
    async fn execute_tool(
        &self,
        name: &str,
        params: Value,
        context: ExecutionContext,
    ) -> Result<Value>;
    
    /// 验证工具参数
    fn validate_tool_params(&self, name: &str, params: &Value) -> Result<()>;
    
    /// 检查工具是否存在
    fn has_tool(&self, name: &str) -> bool;
    
    /// 移除工具
    fn unregister_tool(&mut self, name: &str) -> Result<()>;
    
    /// 获取工具数量
    fn tool_count(&self) -> usize;
    
    /// 清空所有工具
    fn clear(&mut self);
}

/// 工具节点trait - 工作流中的可执行单元
/// 
/// 这是领域层核心trait，定义工具的基本行为
#[async_trait]
pub trait ToolNode: Send + Sync {
    /// 获取工具名称
    fn name(&self) -> &str;
    
    /// 获取工具版本
    fn version(&self) -> &str;
    
    /// 获取工具描述
    fn description(&self) -> String;
    
    /// 获取工具信息
    fn get_info(&self) -> ToolInfo;
    
    /// 验证参数
    fn validate_parameters(&self, params: &Value) -> Result<()>;
    
    /// 执行工具
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
}

/// 工具执行器trait - 解耦执行逻辑
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// 执行工具逻辑
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// 验证参数（可选）
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
}
