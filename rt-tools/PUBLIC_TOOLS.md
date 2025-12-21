# 模块名称：Rust Toolbox 公共工具能力说明

## 1. 目标 (Goal)
- **核心功能**：详细阐述 `rt-core` 模块为工具开发者提供的各项能力和支持，包括核心抽象、API 设计、工具注册、国际化支持等。`rt-core` 是 Rust Toolbox 的核心库，为工具开发提供了统一的基础框架和丰富的辅助功能。
- **非目标**：不包含具体工具的实现细节，不包含命令行或 GUI 实现。

## 2. 核心工具抽象

### 2.1 `Tool` 特性

`Tool` 是 `rt-core` 中所有工具的核心抽象，定义了工具的基本行为和接口。所有工具和插件都必须实现此特性。

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;
    
    /// 显示名称 (支持多语言)
    fn display_name(&self, _locale: Locale) -> String;
    
    /// 工具描述 (用于 UI 展示)
    fn description(&self, locale: Locale) -> String;
    
    /// 用户指南 (Markdown 格式)
    fn user_guide(&self, locale: Locale) -> String;
    
    /// 输入参数 Schema (JSON Schema)
    fn input_schema(&self, locale: Locale) -> Value;
    
    /// 输出结果 Schema (JSON Schema)
    fn output_schema(&self, _locale: Locale) -> Value;
    
    /// 执行逻辑
    async fn run(&self, input: Value) -> Result<Value>;
    
    /// 是否支持 MCP
    fn mcp_supported(&self) -> bool { false }
    
    /// 使用 MCP 上下文执行工具
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
```

### 2.2 `McpTool` 扩展特性

`McpTool` 是 `Tool` 的扩展，用于支持 Model Context Protocol (MCP)，为工具提供更高级的上下文交互能力。

```rust
#[async_trait]
pub trait McpTool: Tool {
    /// 获取 MCP 能力描述
    fn get_mcp_capabilities(&self) -> Value;
    
    /// 获取 MCP 上下文验证规则
    fn get_context_validation_rules(&self) -> Value;
    
    /// 是否需要完整上下文
    fn requires_full_context(&self) -> bool;
    
    /// 执行逻辑（带 MCP 上下文）
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
```

## 3. 工具注册与管理

### 3.1 工具注册宏

`rt-core` 提供了方便的宏来注册工具，简化工具的注册过程：

```rust
// 注册工具的宏
#[macro_export]
macro_rules! register_tool {
    ($tool:ty) => {
        lazy_static::lazy_static! {
            static ref _TOOL_REGISTRATION: () = {
                $crate::register_tool_impl(|| Box::new(<$tool>::new()));
            };
        }
    };
}
```

### 3.2 工具注册中心

`rt-core` 提供了工具注册中心，用于管理和获取所有注册的工具：

```rust
// 获取所有注册的工具
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    // 实现逻辑
}
```

## 4. 国际化支持

### 4.1 `ToolI18n` 结构体

`rt-core` 提供了 `ToolI18n` 结构体，用于加载和管理工具的国际化资源：

```rust
pub struct ToolI18n {
    pub display_name: String,
    pub description: String,
    pub user_guide: String,
    pub input_fields: HashMap<String, String>,
    pub output_fields: HashMap<String, String>,
    pub extra: HashMap<String, serde_json::Value>,
}

impl ToolI18n {
    /// 从指定目录加载工具的国际化资源
    pub fn load(locale: Locale) -> Result<Self> {
        // 实现逻辑
    }
}
```

### 4.2 国际化文件结构

`rt-core` 定义了标准化的国际化文件结构，每个工具目录下建议有一个 `locales` 文件夹，包含 `tool.en.json` 和 `tool.zh-CN.json` 文件：

```json
{
  "display_name": "Tool Name",
  "description": "Tool Description",
  "user_guide": "Markdown User Guide",
  "input_schema": {
    "field_name": { "title": "Field Title" }
  },
  "output_schema": {
    "field_name": { "title": "Field Title" }
  },
  "extra": {
    "key": "value"
  }
}
```

## 5. 持久化支持

### 5.1 `PersistenceManager`

`rt-core` 提供了 `PersistenceManager` 结构体，为工具提供统一的数据存储、缓存和配置管理服务：

```rust
pub struct PersistenceManager {
    // 内部实现
}

impl PersistenceManager {
    /// 获取 KV 数据 (优先查缓存，未命中查 DB)
    pub async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// 保存 KV 数据 (同时更新缓存和 DB)
    pub async fn set_data<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
    
    /// 加载配置 (应用级或工具级)
    pub fn load_config<T: Serialize + DeserializeOwned + Default>(&self, app_name: &str) -> Result<T>;
    
    /// 保存配置
    pub fn save_config<T: Serialize>(&self, app_name: &str, config: &T) -> Result<()>;
    
    /// 创建临时目录 (自动清理)
    pub async fn create_temp_dir(&self) -> Result<TempDir>;
}
```

### 5.2 缓存机制

`PersistenceManager` 集成了高效的缓存机制，使用 `moka` 库实现，支持 TTL/TTI 配置，减少对存储后端的访问次数，提高性能。

### 5.3 存储后端

`rt-core` 支持多种存储后端，默认实现了基于 `sled` 的嵌入式 KV 数据库，支持数据压缩 (`zstd`) 和高效序列化 (`bincode`)。

## 6. 工作流引擎支持

### 6.1 `WorkflowEngine` 特性

`rt-core` 提供了 `WorkflowEngine` 特性，为工具提供工作流编排能力：

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 验证工作流定义
    fn validate(&self, def: &WorkflowDefinition) -> Result<(), CoreError>;

    /// 启动工作流
    async fn start_workflow(&self, def: WorkflowDefinition) -> Result<String>;

    /// 获取工作流状态
    async fn get_status(&self, instance_id: &str) -> Result<WorkflowInstance>;

    /// 暂停/停止
    async fn pause_workflow(&self, instance_id: &str) -> Result<()>;
    async fn stop_workflow(&self, instance_id: &str) -> Result<()>;
    
    /// 获取执行日志
    async fn get_logs(&self, instance_id: &str) -> Result<Vec<LogEntry>>;
}
```

### 6.2 工作流数据结构

`rt-core` 定义了完整的工作流数据结构，包括工作流定义、节点、边和执行状态：

```rust
/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

/// 工作流节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub tool_name: String, // 引用已注册的工具
    pub label: Option<String>,
    pub input_mappings: HashMap<String, String>, // 输入字段 -> 表达式
    pub static_inputs: Value, // 静态配置值
}
```

## 7. 错误处理

### 7.1 `CoreError` 枚举

`rt-core` 定义了统一的错误类型 `CoreError`，用于处理工具执行过程中的各种错误：

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Tool error: {0}")]
    ToolError(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    // 更多错误类型...
}
```

### 7.2 `Result` 类型别名

`rt-core` 提供了方便的 `Result` 类型别名，简化错误处理：

```rust
pub type Result<T> = std::result::Result<T, CoreError>;
```

## 8. 多语言支持

### 8.1 `Locale` 枚举

`rt-core` 定义了 `Locale` 枚举，支持多语言环境：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "en")]
    English,
    
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
    
    // 更多语言...
}
```

### 8.2 国际化资源加载

`rt-core` 提供了国际化资源加载机制，支持从 JSON 文件加载多语言资源，并在运行时根据当前语言环境动态切换。

## 9. 日志系统

### 9.1 日志框架集成

`rt-core` 集成了 `tracing` 日志框架，为工具提供全面的日志记录能力：

- 支持不同级别的日志输出：DEBUG、INFO、WARN、ERROR
- 支持多种输出目标：控制台、文件
- 支持结构化日志记录
- 支持日志轮换和内存日志存储

### 9.2 日志宏

`rt-core` 提供了方便的日志宏，简化日志记录：

```rust
// 日志记录示例
debug!("Tool {} starting execution", tool.name());
info!("Tool {} completed successfully", tool.name());
warn!("Tool {} encountered a warning: {}", tool.name(), warning);
error!("Tool {} failed: {}", tool.name(), error);
```

## 10. 配置管理

### 10.1 `ConfigManager`

`rt-core` 提供了 `ConfigManager`，用于统一管理工具的配置：

- 支持多源配置加载：文件、环境变量
- 支持配置缓存和热重载
- 支持配置项优先级管理
- 支持类型安全的配置访问

### 10.2 配置文件格式

`rt-core` 支持多种配置文件格式，默认使用 TOML 格式，便于阅读和编辑。

## 11. 工具开发最佳实践

### 11.1 异步设计

所有工具都应采用异步设计，使用 Tokio 异步运行时，支持高并发执行。

### 11.2 原子性

工具应只做一件事，便于在工作流中组合使用。

### 11.3 结构化输出

工具输出必须是扁平或层级清晰的 JSON，便于后续节点通过 JSON Path 引用。

### 11.4 错误处理

工具应使用 `Result` 类型传播错误，避免使用 `unwrap()` 或 `expect()` 在生产逻辑中。

### 11.5 MCP 兼容性

对于支持 MCP 的工具，确保输出格式符合 MCP 规范，便于上下文传递。

## 12. 工具开发示例

### 12.1 基本工具实现

```rust
use rt_core::{Locale, Tool, Result, CoreError};
use rt_tools::utils::ToolI18n;

/// 定义一个名为 `MyTool` 的简单工具结构体。
pub struct MyTool;

impl MyTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for MyTool {
    /// 返回工具的唯一名称。
    fn name(&self) -> &'static str {
        "my_category.my_tool"
    }

    /// 返回工具的本地化显示名称。
    fn display_name(&self, locale: Locale) -> String {
        let i18n = ToolI18n::load(locale).unwrap_or_default();
        i18n.display_name
    }

    /// 返回工具的描述。
    fn description(&self, locale: Locale) -> String {
        let i18n = ToolI18n::load(locale).unwrap_or_default();
        i18n.description
    }

    /// 返回工具的用户指南。
    fn user_guide(&self, locale: Locale) -> String {
        let i18n = ToolI18n::load(locale).unwrap_or_default();
        i18n.user_guide
    }

    /// 返回工具的输入参数 Schema。
    fn input_schema(&self, locale: Locale) -> serde_json::Value {
        // 实现逻辑
        serde_json::json!({})
    }

    /// 返回工具的输出结果 Schema。
    fn output_schema(&self, _locale: Locale) -> serde_json::Value {
        // 实现逻辑
        serde_json::json!({})
    }

    /// 执行工具逻辑。
    async fn run(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // 实现工具逻辑
        Ok(serde_json::json!({ "result": "success" }))
    }
}
```

### 12.2 工具注册

```rust
// 在 lib.rs 中注册工具
register_tool!(MyTool);
```

## 13. 依赖关系

| 模块           | 依赖模块   | 说明                           |
|--------------|--------|------------------------------|
| rt-core      | tokio  | 异步运行时                       |
| rt-core      | serde  | JSON 序列化和反序列化              |
| rt-core      | async-trait | 异步特性支持                     |
| rt-core      | tracing | 日志系统                         |
| rt-core      | sled   | 嵌入式 KV 数据库                   |
| rt-core      | moka   | 高性能缓存                       |
| rt-core      | zstd   | 数据压缩                         |
| rt-core      | bincode | 二进制序列化                      |
| rt-core      | confy  | 配置管理                         |

## 14. 性能优化

### 14.1 异步设计

所有核心组件都采用异步设计，支持高并发执行。

### 14.2 缓存机制

关键数据和资源实现了缓存机制，减少重复计算和 IO 操作。

### 14.3 并行执行

工作流引擎支持并行执行无依赖的工具节点，提高工作流执行效率。

### 14.4 高效序列化

使用 `bincode` 进行高效的二进制序列化，减少数据传输和存储开销。

## 15. 监控和日志

所有工具执行过程都有详细的日志记录，包括执行时间、输入参数和输出结果，便于问题定位和性能分析。

## 16. 未来规划

1. **更多工具类型支持**：支持更多类型的工具，如网络工具、数据库工具等
2. **更丰富的国际化支持**：支持更多语言，如日语、韩语等
3. **更强大的工具组合能力**：支持工具之间的更复杂交互
4. **分布式执行**：支持跨节点的工具执行
5. **更完善的监控和统计**：提供更详细的工具执行统计和监控信息
6. **更多存储后端**：支持更多类型的存储后端，如 PostgreSQL、MongoDB 等
7. **Web 界面**：提供基于 Web 的界面，支持远程访问

## 17. 总结

`rt-core` 为工具开发者提供了全面的支持，包括核心抽象、工具注册、国际化支持、持久化、工作流引擎、错误处理等。通过使用 `rt-core` 提供的能力，开发者可以快速构建功能强大、易于使用且支持多语言的工具，无缝集成到 Rust Toolbox 生态系统中。

## 18. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一命名规范，添加变更记录 | 2025-12-21

### 历史记录
- `[已完成]`：初始文档创建 | 2025-12-20