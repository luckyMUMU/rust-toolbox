use async_trait::async_trait;
use serde_json::Value;
use crate::error::Result;
use crate::locale::Locale;
use crate::mcp::{McpRequest, McpResponse};

#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;

    /// 显示名称 (支持多语言)
    fn display_name(&self, _locale: Locale) -> String {
        self.name().to_string()
    }
    
    /// 工具描述 (用于 UI 展示)
    fn description(&self, locale: Locale) -> String;
    
    /// 用户指南 (Markdown 格式)
    fn user_guide(&self, locale: Locale) -> String;

    /// 输入参数 Schema (JSON Schema)
    fn input_schema(&self, locale: Locale) -> Value;

    /// 输出结果 Schema (JSON Schema)
    fn output_schema(&self, _locale: Locale) -> Value {
        serde_json::json!({ "type": "object" })
    }

    /// 执行逻辑
    async fn run(&self, input: Value) -> Result<Value>;
    
    /// 是否支持 MCP
    fn mcp_supported(&self) -> bool {
        false
    }
    
    /// 使用 MCP 上下文执行工具
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse> {
        // 默认实现：不使用上下文，直接调用 run 方法
        let params = request.params.clone();
        let request_copy = request.clone();
        let context = request.context;
        
        let result = self.run(params).await?;
        
        Ok(McpResponse::success_from_request(
            &request_copy,
            result,
            context,
            None,
        ))
    }
}

/// MCP 工具 trait，扩展 Tool trait，提供 MCP 特定功能
#[async_trait]
pub trait McpTool: Tool {
    /// 获取 MCP 能力描述
    fn get_mcp_capabilities(&self) -> Value {
        serde_json::json!({})
    }
    
    /// 获取 MCP 上下文验证规则
    fn get_context_validation_rules(&self) -> Value {
        serde_json::json!({})
    }
    
    /// 是否需要完整上下文
    fn requires_full_context(&self) -> bool {
        false
    }
    
    /// 执行逻辑（带 MCP 上下文）
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
