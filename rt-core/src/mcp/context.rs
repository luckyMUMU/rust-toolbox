use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// MCP 上下文，包含模型状态、执行历史和环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    /// 上下文 ID
    pub id: String,
    
    /// 父上下文 ID，用于跟踪上下文继承关系
    pub parent_id: Option<String>,
    
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// 模型状态信息
    pub model_state: ModelState,
    
    /// 执行历史记录
    pub execution_history: Vec<ExecutionRecord>,
    
    /// 环境变量和配置
    pub environment: EnvironmentInfo,
    
    /// 自定义上下文数据
    pub custom_data: Value,
}

/// 模型状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    /// 模型名称
    pub model_name: Option<String>,
    
    /// 模型版本
    pub model_version: Option<String>,
    
    /// 模型参数
    pub model_params: Value,
    
    /// 当前模型状态
    pub state: Value,
}

/// 执行历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// 记录 ID
    pub id: String,
    
    /// 执行时间
    pub timestamp: DateTime<Utc>,
    
    /// 执行组件类型
    pub component_type: ComponentType,
    
    /// 执行组件名称
    pub component_name: String,
    
    /// 执行方法
    pub method: String,
    
    /// 输入参数
    pub input: Value,
    
    /// 输出结果
    pub output: Option<Value>,
    
    /// 执行状态
    pub status: ExecutionStatus,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 执行耗时（毫秒）
    pub duration_ms: Option<u64>,
}

/// 组件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentType {
    /// 工具组件
    Tool,
    
    /// 插件组件
    Plugin,
    
    /// 工作流组件
    Workflow,
    
    /// 服务组件
    Service,
    
    /// 系统组件
    System,
}

/// 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// 执行成功
    Success,
    
    /// 执行失败
    Failed,
    
    /// 执行中
    Running,
    
    /// 已取消
    Cancelled,
}

/// 环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// 系统信息
    pub system: SystemInfo,
    
    /// 运行时信息
    pub runtime: RuntimeInfo,
    
    /// 配置信息
    pub config: Value,
    
    /// 环境变量
    pub env_vars: std::collections::HashMap<String, String>,
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 操作系统类型
    pub os_type: String,
    
    /// 操作系统版本
    pub os_version: String,
    
    /// 架构信息
    pub architecture: String,
    
    /// CPU 核心数
    pub cpu_cores: u32,
    
    /// 可用内存（MB）
    pub available_memory_mb: u64,
}

/// 运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// 运行时名称
    pub name: String,
    
    /// 运行时版本
    pub version: String,
    
    /// 语言版本
    pub language_version: String,
    
    /// 进程 ID
    pub process_id: u32,
    
    /// 启动时间
    pub start_time: DateTime<Utc>,
}

impl Default for McpContext {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            model_state: ModelState {
                model_name: None,
                model_version: None,
                model_params: Value::Object(serde_json::Map::new()),
                state: Value::Object(serde_json::Map::new()),
            },
            execution_history: Vec::new(),
            environment: EnvironmentInfo {
                system: SystemInfo {
                    os_type: std::env::consts::OS.to_string(),
                    os_version: "unknown".to_string(),
                    architecture: std::env::consts::ARCH.to_string(),
                    cpu_cores: num_cpus::get() as u32,
                    available_memory_mb: 0,
                },
                runtime: RuntimeInfo {
                    name: "rust".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    language_version: rustc_version::version().unwrap().to_string(),
                    process_id: std::process::id(),
                    start_time: Utc::now(),
                },
                config: Value::Object(serde_json::Map::new()),
                env_vars: std::env::vars().collect(),
            },
            custom_data: Value::Object(serde_json::Map::new()),
        }
    }
}

impl McpContext {
    /// 创建一个新的 MCP 上下文
    pub fn new() -> Self {
        Default::default()
    }
    
    /// 从现有上下文创建一个子上下文
    pub fn create_child(&self) -> Self {
        let mut child = Self::new();
        child.parent_id = Some(self.id.clone());
        child.model_state = self.model_state.clone();
        child.environment = self.environment.clone();
        child
    }
    
    /// 添加执行记录
    pub fn add_execution_record(&mut self, record: ExecutionRecord) {
        self.execution_history.push(record);
        self.updated_at = Utc::now();
    }
    
    /// 更新模型状态
    pub fn update_model_state(&mut self, model_state: ModelState) {
        self.model_state = model_state;
        self.updated_at = Utc::now();
    }
    
    /// 更新自定义数据
    pub fn update_custom_data(&mut self, key: &str, value: Value) {
        if let Value::Object(map) = &mut self.custom_data {
            map.insert(key.to_string(), value);
            self.updated_at = Utc::now();
        }
    }
    
    /// 获取自定义数据
    pub fn get_custom_data(&self, key: &str) -> Option<&Value> {
        if let Value::Object(map) = &self.custom_data {
            map.get(key)
        } else {
            None
        }
    }
}
