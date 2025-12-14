use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 配置源枚举，标识配置项的来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    /// 配置文件
    #[serde(rename = "file")]
    File,
    /// 环境变量
    #[serde(rename = "env")]
    Environment,
    /// 远程配置中心
    #[serde(rename = "remote")]
    Remote,
    /// 默认值
    #[serde(rename = "default")]
    Default,
}

/// 配置项结构体，存储单个配置项的完整信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigItem {
    /// 配置键
    pub key: String,
    /// 配置值（序列化后的字符串）
    pub value: String,
    /// 配置来源
    pub source: ConfigSource,
    /// 配置源优先级
    pub priority: u8,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}