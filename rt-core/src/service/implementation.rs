use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use crate::{ServicePort, Result, CoreError, Tool, plugin::PluginManager, ConfigService, LogManager, LogWriter, LogManagerPort, LogWriterPort};
use serde_json::{Value, Map};
use crate::service::{ServiceResponse, ResponseStatus, ServiceContext};
use crate::config::domain::ConfigItem;
use crate::logger::domain::{LogLevel, LogRecord};

// ====================== 配置服务实现 ======================

/// 配置服务实现，包装配置管理器提供标准化的服务调用接口
pub struct ConfigServiceImpl {
    /// 配置服务实例
    config_service: Arc<ConfigService>,
}

impl ConfigServiceImpl {
    /// 创建新的配置服务实例
    pub fn new(config_service: Arc<ConfigService>) -> Self {
        Self {
            config_service,
        }
    }
}

#[async_trait]
impl ServicePort for ConfigServiceImpl {
    fn name(&self) -> &str {
        "config"
    }
    
    async fn call(&self, method: &str, params: &Value, context: &ServiceContext) -> Result<ServiceResponse> {
        match method {
            "get_config" => {
                let key = params.get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'key' parameter".to_string()))?;
                self.get_config(key, context).await
            },
            "set_config" => {
                let key = params.get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'key' parameter".to_string()))?;
                let value = params.get("value")
                    .ok_or_else(|| CoreError::InvalidInput("Missing 'value' parameter".to_string()))?;
                self.set_config(key, value, context).await
            },
            "reload_config" => {
                self.reload_config(context).await
            },
            _ => {
                Ok(ServiceResponse {
                    status: ResponseStatus::InvalidRequest,
                    data: None,
                    error: Some(format!("Invalid method: {}", method)),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
}

impl ConfigServiceImpl {
    /// 获取配置项
    async fn get_config(&self, key: &str, _context: &ServiceContext) -> Result<ServiceResponse> {
        match self.config_service.get_config::<Value>(key).await {
            Ok(Some(value)) => {
                // 构建 ConfigItem
                let config_item = ConfigItem {
                    key: key.to_string(),
                    value: serde_json::to_string(&value)?,
                    source: crate::config::domain::ConfigSource::Default,
                    priority: 0,
                    updated_at: chrono::Utc::now(),
                };
                
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: Some(serde_json::to_value(config_item)?),
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Ok(None) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: Some(Value::Null),
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
    
    /// 设置配置项
    async fn set_config(&self, key: &str, value: &Value, _context: &ServiceContext) -> Result<ServiceResponse> {
        match self.config_service.set_config(key, value, crate::config::domain::ConfigSource::Default).await {
            Ok(()) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: None,
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
    
    /// 重载配置
    async fn reload_config(&self, _context: &ServiceContext) -> Result<ServiceResponse> {
        match self.config_service.reload_config().await {
            Ok(()) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: None,
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
}

// ====================== 日志服务实现 ======================

/// 日志服务实现，包装日志管理器提供标准化的服务调用接口
pub struct LogServiceImpl {
    /// 日志管理器实例
    log_manager: Arc<LogManager>,
    
    /// 日志写入器实例
    log_writer: Arc<LogWriter>,
}

impl LogServiceImpl {
    /// 创建新的日志服务实例
    pub fn new(log_manager: Arc<LogManager>, log_writer: Arc<LogWriter>) -> Self {
        Self {
            log_manager,
            log_writer,
        }
    }
    
    /// 解析日志级别
    fn parse_log_level(level_str: &str) -> Option<LogLevel> {
        match level_str.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

#[async_trait]
impl ServicePort for LogServiceImpl {
    fn name(&self) -> &str {
        "logger"
    }
    
    async fn call(&self, method: &str, params: &Value, context: &ServiceContext) -> Result<ServiceResponse> {
        match method {
            "log" => {
                let level_str = params.get("level")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'level' parameter".to_string()))?;
                let level = Self::parse_log_level(level_str)
                    .ok_or_else(|| CoreError::InvalidInput(format!("Invalid log level: {}", level_str)))?;
                
                let message = params.get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'message' parameter".to_string()))?;
                
                let target = params.get("target")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                
                let fields_map = params.get("fields")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();
                
                let mut fields = HashMap::new();
                for (k, v) in fields_map {
                    fields.insert(k, v);
                }
                
                let record = LogRecord {
                    level,
                    message: message.to_string(),
                    target: target.to_string(),
                    timestamp: chrono::Utc::now(),
                    fields,
                    module_path: None,
                    file: None,
                    line: None,
                };
                
                self.log(&record, context).await
            },
            "set_log_level" => {
                let level_str = params.get("level")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'level' parameter".to_string()))?;
                let level = Self::parse_log_level(level_str)
                    .ok_or_else(|| CoreError::InvalidInput(format!("Invalid log level: {}", level_str)))?;
                
                self.set_log_level(level, context).await
            },
            "get_logs" => {
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                self.get_logs(limit, context).await
            },
            _ => {
                Ok(ServiceResponse {
                    status: ResponseStatus::InvalidRequest,
                    data: None,
                    error: Some(format!("Invalid method: {}", method)),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
}

impl LogServiceImpl {
    /// 写入日志
    async fn log(&self, record: &LogRecord, _context: &ServiceContext) -> Result<ServiceResponse> {
        // 使用日志写入器写入日志
        let result: Result<()> = match record.level {
            LogLevel::Debug => self.log_writer.debug(&record.target, &record.message, Some(record.fields.clone())),
            LogLevel::Info => self.log_writer.info(&record.target, &record.message, Some(record.fields.clone())),
            LogLevel::Warn => self.log_writer.warn(&record.target, &record.message, Some(record.fields.clone())),
            LogLevel::Error => self.log_writer.error(&record.target, &record.message, Some(record.fields.clone())),
        };
        
        match result {
            Ok(()) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: None,
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
    
    /// 设置日志级别
    async fn set_log_level(&self, level: LogLevel, _context: &ServiceContext) -> Result<ServiceResponse> {
        let result: Result<()> = self.log_manager.set_level(level).await;
        
        match result {
            Ok(()) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Success,
                    data: None,
                    error: None,
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            },
            Err(e) => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(e.to_string()),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
    
    /// 获取最近的日志记录
    async fn get_logs(&self, limit: Option<usize>, _context: &ServiceContext) -> Result<ServiceResponse> {
        let logs = self.log_manager.get_logs(limit).await?;
        
        Ok(ServiceResponse {
            status: ResponseStatus::Success,
            data: Some(serde_json::to_value(logs)?),
            error: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        })
    }
}

// ====================== 工具服务实现 ======================

/// 工具服务实现，包装插件管理器提供标准化的服务调用接口
pub struct ToolServiceImpl {
    /// 插件管理器实例
    plugin_manager: Arc<PluginManager>,
    
    /// 核心工具列表
    core_tools: Arc<tokio::sync::RwLock<Vec<Arc<dyn Tool>>>>,
}

impl ToolServiceImpl {
    /// 创建新的工具服务实例
    pub fn new(plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            plugin_manager,
            core_tools: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// 添加核心工具
    pub async fn add_core_tool(&self, tool: Arc<dyn Tool>) {
        let mut core_tools = self.core_tools.write().await;
        core_tools.push(tool);
    }
}

#[async_trait]
impl ServicePort for ToolServiceImpl {
    fn name(&self) -> &str {
        "tool"
    }
    
    async fn call(&self, method: &str, params: &Value, context: &ServiceContext) -> Result<ServiceResponse> {
        match method {
            "list_tools" => {
                self.list_tools(context).await
            },
            "execute_tool" => {
                let tool_name = params.get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| CoreError::InvalidInput("Missing or invalid 'tool_name' parameter".to_string()))?;
                
                // 获取工具参数，使用空对象作为默认值
                let default_params = Value::Object(Map::new());
                let tool_params = params.get("params").unwrap_or(&default_params);
                
                self.execute_tool(tool_name, tool_params, context).await
            },
            _ => {
                Ok(ServiceResponse {
                    status: ResponseStatus::InvalidRequest,
                    data: None,
                    error: Some(format!("Invalid method: {}", method)),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
}

impl ToolServiceImpl {
    /// 获取工具列表
    async fn list_tools(&self, _context: &ServiceContext) -> Result<ServiceResponse> {
        let mut tool_names = Vec::new();
        
        // 获取核心工具
        let core_tools = self.core_tools.read().await;
        for tool in core_tools.iter() {
            tool_names.push(tool.name().to_string());
        }
        
        // 获取插件工具
        let plugin_tools = self.plugin_manager.list_tools().await;
        for tool in plugin_tools {
            tool_names.push(tool.name().to_string());
        }
        
        Ok(ServiceResponse {
            status: ResponseStatus::Success,
            data: Some(serde_json::to_value(tool_names)?),
            error: None,
            trace_id: uuid::Uuid::new_v4().to_string(),
        })
    }
    
    /// 执行工具
    async fn execute_tool(
        &self,
        tool_name: &str,
        params: &Value,
        _context: &ServiceContext
    ) -> Result<ServiceResponse> {
        // 查找工具
        let core_tool = self.core_tools.read().await.iter()
            .find(|tool| tool.name() == tool_name)
            .cloned();
        
        let tool = match core_tool {
            Some(tool) => Some(tool),
            None => self.plugin_manager.get_tool(tool_name).await,
        };
        
        match tool {
            Some(tool) => {
                // 执行工具
                match tool.run(params.clone()).await {
                    Ok(result) => {
                        Ok(ServiceResponse {
                            status: ResponseStatus::Success,
                            data: Some(result),
                            error: None,
                            trace_id: uuid::Uuid::new_v4().to_string(),
                        })
                    },
                    Err(e) => {
                        Ok(ServiceResponse {
                            status: ResponseStatus::Failure,
                            data: None,
                            error: Some(e.to_string()),
                            trace_id: uuid::Uuid::new_v4().to_string(),
                        })
                    }
                }
            },
            None => {
                Ok(ServiceResponse {
                    status: ResponseStatus::Failure,
                    data: None,
                    error: Some(format!("Tool not found: {}", tool_name)),
                    trace_id: uuid::Uuid::new_v4().to_string(),
                })
            }
        }
    }
}
