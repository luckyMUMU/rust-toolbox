//! 领域模型 - 工具相关

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub description: String,

    /// 参数schema
    #[serde(rename = "parameters")]
    pub parameters_schema: Value,
    pub return_schema: Value,

    /// 元数据
    pub category: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// 插件集成
    pub plugin_name: Option<String>,
    pub version_requirements: HashMap<String, String>,

    /// 时间戳
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}
