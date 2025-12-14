use async_trait::async_trait;
use crate::{error::Result, logger::domain::{LogLevel, LogRecord, LogEntry}}; // 删除未使用的 CoreError 导入

// ====================== 输入端口 ======================

/// 日志管理端口，定义外部系统管理日志记录的接口
#[async_trait]
pub trait LogManagerPort: Send + Sync {
    /// 设置日志级别
    /// 
    /// # 参数
    /// * `level` - 日志级别
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功设置日志级别
    /// * `Err(CoreError)` - 设置日志级别失败
    async fn set_level(&self, level: LogLevel) -> Result<()>;
    
    /// 添加日志输出目标
    /// 
    /// # 参数
    /// * `sink` - 日志输出目标实例
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功添加日志输出目标
    /// * `Err(CoreError)` - 添加日志输出目标失败
    async fn add_sink(&self, sink: Box<dyn LogSinkPort>) -> Result<()>;
    
    /// 移除日志输出目标
    /// 
    /// # 参数
    /// * `sink_name` - 日志输出目标名称
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功移除日志输出目标
    /// * `Err(CoreError)` - 移除日志输出目标失败
    async fn remove_sink(&self, sink_name: &str) -> Result<()>;
    
    /// 获取日志记录列表
    /// 
    /// # 参数
    /// * `limit` - 日志记录数量限制
    /// 
    /// # 返回值
    /// * `Ok(Vec<LogEntry>)` - 成功获取日志记录列表
    /// * `Err(CoreError)` - 获取日志记录列表失败
    async fn get_logs(&self, limit: Option<usize>) -> Result<Vec<LogEntry>>;
}

/// 日志写入端口，定义外部系统写入日志的接口
pub trait LogWriterPort: Send + Sync {
    /// 写入调试级别日志
    /// 
    /// # 参数
    /// * `target` - 日志目标（模块名称或组件名称）
    /// * `message` - 日志消息
    /// * `fields` - 日志附加字段，用于存储结构化日志数据
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    fn debug(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()>;
    
    /// 写入信息级别日志
    /// 
    /// # 参数
    /// * `target` - 日志目标（模块名称或组件名称）
    /// * `message` - 日志消息
    /// * `fields` - 日志附加字段，用于存储结构化日志数据
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    fn info(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()>;
    
    /// 写入警告级别日志
    /// 
    /// # 参数
    /// * `target` - 日志目标（模块名称或组件名称）
    /// * `message` - 日志消息
    /// * `fields` - 日志附加字段，用于存储结构化日志数据
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    fn warn(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()>;
    
    /// 写入错误级别日志
    /// 
    /// # 参数
    /// * `target` - 日志目标（模块名称或组件名称）
    /// * `message` - 日志消息
    /// * `fields` - 日志附加字段，用于存储结构化日志数据
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    fn error(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()>;
}

// ====================== 输出端口 ======================

/// 日志输出端口，定义日志记录模块向外部输出日志的接口
pub trait LogSinkPort: Send + Sync {
    /// 获取日志输出目标名称
    /// 
    /// # 返回值
    /// 日志输出目标名称
    fn name(&self) -> &str;
    
    /// 写入格式化后的日志
    /// 
    /// # 参数
    /// * `formatted_log` - 格式化后的日志字符串
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    fn write(&self, formatted_log: &str) -> Result<()>;
}

/// 日志格式化端口，定义日志格式化的接口
pub trait LogFormatterPort: Send + Sync {
    /// 格式化日志记录
    /// 
    /// # 参数
    /// * `record` - 日志记录
    /// 
    /// # 返回值
    /// * `Ok(String)` - 成功格式化日志，返回格式化后的日志字符串
    /// * `Err(CoreError)` - 格式化日志失败
    fn format(&self, record: &LogRecord) -> Result<String>;
}