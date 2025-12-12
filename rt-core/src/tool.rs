use async_trait::async_trait;
use serde_json::Value;
use crate::error::Result;
use crate::locale::Locale;

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
}
