//! 领域模型 - 插件相关

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Wasm,
    Docker,
    Go,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub plugin_type: PluginType,
    pub metadata: HashMap<String, Value>,
}
