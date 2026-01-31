//! 工具注册表端口

use async_trait::async_trait;
use serde_json::Value;
use crate::core::{ExecutionContext, ToolInfo};
use crate::error::Result;

/// 工具注册表trait - 定义工具管理的核心接口
#[async_trait]
pub trait ToolRegistry: Send + Sync {
    /// 注册工具
    fn register_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;
    
    /// 获取工具
    fn get_tool(&self, name: &str) -> Option<&dyn Tool>;
    
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
    
    /// 获取工具数量
    fn tool_count(&self) -> usize;
}

/// 工具trait
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;
    
    /// 工具版本
    fn version(&self) -> &str;
    
    /// 执行工具
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value>;
    
    /// 验证参数
    fn validate_params(&self, params: &Value) -> Result<()>;
}
