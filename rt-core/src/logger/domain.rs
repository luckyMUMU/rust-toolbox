use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 日志级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum LogLevel {
    /// 调试级别，用于开发调试
    #[serde(rename = "debug")]
    Debug,
    
    /// 信息级别，用于常规操作信息
    #[serde(rename = "info")]
    Info,
    
    /// 警告级别，用于警告信息
    #[serde(rename = "warn")]
    Warn,
    
    /// 错误级别，用于错误信息
    #[serde(rename = "error")]
    Error,
}

/// 日志记录结构体，包含完整的日志信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// 日志级别
    pub level: LogLevel,
    
    /// 日志消息
    pub message: String,
    
    /// 日志目标（模块名称或组件名称）
    pub target: String,
    
    /// 日志记录时间
    pub timestamp: DateTime<Utc>,
    
    /// 日志附加字段，用于存储结构化日志数据
    pub fields: std::collections::HashMap<String, serde_json::Value>,
    
    /// 日志记录的模块路径
    pub module_path: Option<String>,
    
    /// 日志记录的文件名称
    pub file: Option<String>,
    
    /// 日志记录的行号
    pub line: Option<u32>,
}

/// 日志条目结构体，用于存储日志记录的基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 日志记录
    pub record: LogRecord,
    
    /// 格式化后的日志字符串
    pub formatted: String,
}

/// 日志输出目标枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSinkType {
    /// 控制台输出
    Console,
    
    /// 文件输出
    File,
    
    /// 数据库输出
    Database,
    
    /// 远程日志服务输出
    Remote,
}

/// 日志旋转策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPolicy {
    /// 按时间旋转
    Daily,
    
    /// 按大小旋转
    Size(u64),
    
    /// 不旋转
    Never,
}