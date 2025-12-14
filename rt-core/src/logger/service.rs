use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::{error::Result, logger::{domain::{LogLevel, LogRecord, LogEntry}, port::{LogSinkPort, LogFormatterPort, LogManagerPort, LogWriterPort}}}; 

/// 日志服务，实现日志记录的核心业务逻辑
pub struct LogService {
    /// 日志级别，低于此级别的日志将被过滤
    min_level: Mutex<LogLevel>,
    
    /// 日志输出目标列表
    sinks: Mutex<Vec<Box<dyn LogSinkPort>>>,
    
    /// 日志格式化器
    formatter: Arc<dyn LogFormatterPort>,
    
    /// 日志记录列表，用于存储最近的日志记录
    log_entries: Mutex<std::collections::VecDeque<LogEntry>>,
    
    /// 最大日志记录数量，超过此数量的旧日志将被移除
    max_log_entries: usize,
}

impl LogService {
    /// 创建新的日志服务实例
    /// 
    /// # 参数
    /// * `formatter` - 日志格式化器实例
    /// 
    /// # 返回值
    /// * `Ok(LogService)` - 成功创建日志服务
    pub fn new(formatter: Arc<dyn LogFormatterPort>) -> Result<Self> {
        Ok(Self {
            min_level: Mutex::new(LogLevel::Info), // 默认日志级别为 Info
            sinks: Mutex::new(Vec::new()),
            formatter,
            log_entries: Mutex::new(std::collections::VecDeque::new()),
            max_log_entries: 1000, // 默认最多保存 1000 条日志
        })
    }
    
    /// 添加日志输出目标
    /// 
    /// # 参数
    /// * `sink` - 日志输出目标实例
    pub fn add_sink(&self, sink: Box<dyn LogSinkPort>) {
        let mut sinks = self.sinks.lock().unwrap();
        sinks.push(sink);
    }
    
    /// 设置日志级别
    /// 
    /// # 参数
    /// * `level` - 日志级别
    pub fn set_level(&self, level: LogLevel) {
        let mut min_level = self.min_level.lock().unwrap();
        *min_level = level;
    }
    
    /// 写入日志记录
    /// 
    /// # 参数
    /// * `record` - 日志记录
    /// 
    /// # 返回值
    /// * `Ok(())` - 成功写入日志
    /// * `Err(CoreError)` - 写入日志失败
    pub fn log(&self, record: LogRecord) -> Result<()> {
        // 检查日志级别，低于最小级别的日志将被过滤
        let min_level = *self.min_level.lock().unwrap();
        if record.level < min_level {
            return Ok(());
        }
        
        // 格式化日志
        let formatted = self.formatter.format(&record)?;
        
        // 写入所有日志输出目标
        let sinks = self.sinks.lock().unwrap();
        for sink in sinks.iter() {
            sink.write(&formatted)?;
        }
        
        // 保存日志记录到内存中
        let mut log_entries = self.log_entries.lock().unwrap();
        log_entries.push_back(LogEntry {
            record: record.clone(),
            formatted,
        });
        
        // 如果日志记录数量超过限制，移除最旧的日志
        if log_entries.len() > self.max_log_entries {
            log_entries.pop_front();
        }
        
        Ok(())
    }
    
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
    pub fn debug(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        let record = LogRecord {
            level: LogLevel::Debug,
            message: message.to_string(),
            target: target.to_string(),
            timestamp: Utc::now(),
            fields: fields.unwrap_or_default(),
            module_path: None,
            file: None,
            line: None,
        };
        
        self.log(record)
    }
    
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
    pub fn info(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        let record = LogRecord {
            level: LogLevel::Info,
            message: message.to_string(),
            target: target.to_string(),
            timestamp: Utc::now(),
            fields: fields.unwrap_or_default(),
            module_path: None,
            file: None,
            line: None,
        };
        
        self.log(record)
    }
    
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
    pub fn warn(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        let record = LogRecord {
            level: LogLevel::Warn,
            message: message.to_string(),
            target: target.to_string(),
            timestamp: Utc::now(),
            fields: fields.unwrap_or_default(),
            module_path: None,
            file: None,
            line: None,
        };
        
        self.log(record)
    }
    
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
    pub fn error(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        let record = LogRecord {
            level: LogLevel::Error,
            message: message.to_string(),
            target: target.to_string(),
            timestamp: Utc::now(),
            fields: fields.unwrap_or_default(),
            module_path: None,
            file: None,
            line: None,
        };
        
        self.log(record)
    }
    
    /// 获取最近的日志记录
    /// 
    /// # 参数
    /// * `limit` - 日志记录数量限制
    /// 
    /// # 返回值
    /// * `Ok(Vec<LogEntry>)` - 成功获取日志记录列表
    /// * `Err(CoreError)` - 获取日志记录列表失败
    pub fn get_logs(&self, limit: Option<usize>) -> Result<Vec<LogEntry>> {
        let log_entries = self.log_entries.lock().unwrap();
        
        match limit {
            Some(limit) => {
                // 获取最近的 limit 条日志
                let start = if log_entries.len() > limit {
                    log_entries.len() - limit
                } else {
                    0
                };
                
                Ok(log_entries.range(start..).cloned().collect())
            },
            None => {
                // 获取所有日志
                Ok(log_entries.iter().cloned().collect())
            },
        }
    }
}

/// 日志管理器，实现 LogManagerPort 接口
pub struct LogManager {
    /// 日志服务实例
    log_service: Arc<LogService>,
}

impl LogManager {
    /// 创建新的日志管理器实例
    /// 
    /// # 参数
    /// * `log_service` - 日志服务实例
    /// 
    /// # 返回值
    /// * `Ok(LogManager)` - 成功创建日志管理器
    pub fn new(log_service: Arc<LogService>) -> Result<Self> {
        Ok(Self {
            log_service,
        })
    }
}

#[async_trait::async_trait]
impl LogManagerPort for LogManager {
    async fn set_level(&self, level: LogLevel) -> Result<()> {
        self.log_service.set_level(level);
        Ok(())
    }
    
    async fn add_sink(&self, sink: Box<dyn LogSinkPort>) -> Result<()> {
        self.log_service.add_sink(sink);
        Ok(())
    }
    
    async fn remove_sink(&self, sink_name: &str) -> Result<()> {
        // 注意：当前 LogService 没有提供移除日志输出目标的方法，所以这里暂时不实现
        // 可以考虑在 LogService 中添加 remove_sink 方法来支持移除日志输出目标
        Ok(())
    }
    
    async fn get_logs(&self, limit: Option<usize>) -> Result<Vec<LogEntry>> {
        self.log_service.get_logs(limit)
    }
}

/// 日志写入器，实现 LogWriterPort 接口
pub struct LogWriter {
    /// 日志服务实例
    log_service: Arc<LogService>,
}

impl LogWriter {
    /// 创建新的日志写入器实例
    /// 
    /// # 参数
    /// * `log_service` - 日志服务实例
    /// 
    /// # 返回值
    /// * `Ok(LogWriter)` - 成功创建日志写入器
    pub fn new(log_service: Arc<LogService>) -> Result<Self> {
        Ok(Self {
            log_service,
        })
    }
}

impl LogWriterPort for LogWriter {
    fn debug(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        self.log_service.debug(target, message, fields)
    }
    
    fn info(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        self.log_service.info(target, message, fields)
    }
    
    fn warn(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        self.log_service.warn(target, message, fields)
    }
    
    fn error(&self, target: &str, message: &str, fields: Option<std::collections::HashMap<String, serde_json::Value>>) -> Result<()> {
        self.log_service.error(target, message, fields)
    }
}