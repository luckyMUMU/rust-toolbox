use crate::{error::{CoreError, Result}, logger::{domain::LogRecord, port::LogFormatterPort}};

/// 文本日志格式化适配器，将日志记录格式化为文本字符串
pub struct TextFormatter {
    /// 日志格式模式
    pattern: String,
}

impl TextFormatter {
    /// 创建新的文本日志格式化适配器
    /// 
    /// # 参数
    /// * `pattern` - 日志格式模式，支持以下占位符：
    ///   - `%d` - 日志记录时间
    ///   - `%l` - 日志级别
    ///   - `%t` - 日志目标
    ///   - `%m` - 日志消息
    ///   - `%f` - 日志记录的文件名称
    ///   - `%L` - 日志记录的行号
    /// 
    /// # 返回值
    /// * `Ok(TextFormatter)` - 成功创建文本日志格式化适配器
    pub fn new(pattern: Option<String>) -> Result<Self> {
        Ok(Self {
            pattern: pattern.unwrap_or_else(|| "[%d] %l %t: %m".to_string()), // 默认格式：[时间] 级别 目标: 消息
        })
    }
}

impl LogFormatterPort for TextFormatter {
    fn format(&self, record: &LogRecord) -> Result<String> {
        let mut formatted = self.pattern.clone();
        
        // 替换时间占位符
        formatted = formatted.replace("%d", &record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string());
        
        // 替换级别占位符
        let level_str = match record.level {
            crate::logger::domain::LogLevel::Debug => "DEBUG",
            crate::logger::domain::LogLevel::Info => "INFO",
            crate::logger::domain::LogLevel::Warn => "WARN",
            crate::logger::domain::LogLevel::Error => "ERROR",
        };
        formatted = formatted.replace("%l", level_str);
        
        // 替换目标占位符
        formatted = formatted.replace("%t", &record.target);
        
        // 替换消息占位符
        formatted = formatted.replace("%m", &record.message);
        
        // 替换文件占位符
        if let Some(file) = &record.file {
            formatted = formatted.replace("%f", file);
        } else {
            formatted = formatted.replace("%f", "-");
        }
        
        // 替换行号占位符
        if let Some(line) = record.line {
            formatted = formatted.replace("%L", &line.to_string());
        } else {
            formatted = formatted.replace("%L", "-");
        }
        
        Ok(formatted)
    }
}

/// JSON日志格式化适配器，将日志记录格式化为JSON字符串
pub struct JsonFormatter {
    /// 是否启用美化输出
    pretty_print: bool,
}

impl JsonFormatter {
    /// 创建新的JSON日志格式化适配器
    /// 
    /// # 参数
    /// * `pretty_print` - 是否启用美化输出
    /// 
    /// # 返回值
    /// * `Ok(JsonFormatter)` - 成功创建JSON日志格式化适配器
    pub fn new(pretty_print: bool) -> Result<Self> {
        Ok(Self {
            pretty_print,
        })
    }
}

impl LogFormatterPort for JsonFormatter {
    fn format(&self, record: &LogRecord) -> Result<String> {
        // 创建一个包含日志记录所有字段的HashMap
        let mut log_map = std::collections::HashMap::new();
        
        // 添加基本字段
        log_map.insert("timestamp".to_string(), serde_json::Value::String(record.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()));
        log_map.insert("level".to_string(), serde_json::Value::String(format!("{:?}", record.level).to_lowercase()));
        log_map.insert("target".to_string(), serde_json::Value::String(record.target.clone()));
        log_map.insert("message".to_string(), serde_json::Value::String(record.message.clone()));
        
        // 添加可选字段
        if let Some(file) = &record.file {
            log_map.insert("file".to_string(), serde_json::Value::String(file.clone()));
        }
        
        if let Some(line) = record.line {
            log_map.insert("line".to_string(), serde_json::Value::Number(serde_json::Number::from(line)));
        }
        
        if let Some(module_path) = &record.module_path {
            log_map.insert("module_path".to_string(), serde_json::Value::String(module_path.clone()));
        }
        
        // 添加附加字段
        // 将 HashMap 转换为 serde_json::Map
        let mut map = serde_json::Map::new();
        for (key, value) in record.fields.clone() {
            map.insert(key, value);
        }
        log_map.insert("fields".to_string(), serde_json::Value::Object(map));
        
        // 格式化JSON字符串
        if self.pretty_print {
            serde_json::to_string_pretty(&log_map)
        } else {
            serde_json::to_string(&log_map)
        }
        .map_err(|e| CoreError::ConfigError(format!("Failed to format log as JSON: {}", e)))
    }
}