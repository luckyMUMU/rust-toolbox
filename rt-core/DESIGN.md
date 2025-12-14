# rt-core Design Document

## 1. 模块概述 (Module Overview)
`rt-core` 是 Rust Toolbox 的核心库，定义了所有工具和工作流必须遵循的基础接口 (Traits) 以及通用数据结构。它不包含具体的业务逻辑工具（这些在 `rt-tools` 中），只提供“骨架”和核心功能支持。

## 2. 核心职责 (Core Responsibilities)
- 定义 `Tool` Trait：所有具体工具必须实现的接口，支持同步和 MCP 上下文执行。
- 定义 `Locale` Enum：支持的多语言区域设置。
- 定义工作流相关结构：管理工具执行顺序和上下文传递。
- 实现 Model Context Protocol (MCP) 支持：提供上下文管理、请求/响应处理。
- 定义 `CoreError`：统一错误处理类型。
- 实现插件系统：支持外部插件的加载和管理。
- 实现工作流引擎：工作流编排与执行 (详见 `WORKFLOW_DESIGN.md`)。
- 实现 MCP 服务器：提供 REST API 和 WebSocket 端点，用于外部工具调用。

## 3. 详细设计 (Detailed Design)

### 3.1 Locale (`locale.rs`)
定义了支持的语言区域：
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-CN")]
    Zh,
}
```

### 3.2 Tool Trait (`tool.rs`)
所有工具必须实现的核心接口，支持 MCP 上下文执行：
```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::locale::Locale;
use crate::mcp::{McpRequest, McpResponse};

#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;
    
    /// 显示名称 (支持多语言)
    fn display_name(&self, _locale: Locale) -> String {
        self.name().to_string()
    }
    
    /// 工具描述 (用于 UI 展示)
    fn description(&self, locale: Locale) -> String;
    
    /// 用户指南 (Markdown, 支持多语言)
    fn user_guide(&self, locale: Locale) -> String;

    /// 输入 Schema (JSON Schema)
    fn input_schema(&self, locale: Locale) -> Value;

    /// 输出结果 Schema (JSON Schema)
    fn output_schema(&self, _locale: Locale) -> Value {
        serde_json::json!({ "type": "object" })
    }
    
    /// 执行逻辑
    async fn run(&self, input: Value) -> Result<Value>;
    
    /// 是否支持 MCP
    fn mcp_supported(&self) -> bool {
        false
    }
    
    /// 使用 MCP 上下文执行工具
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse>;
}
```

### 3.3 McpTool Trait (`tool.rs`)
扩展 `Tool` Trait，提供 MCP 特定功能：
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

### 3.4 Model Context Protocol (MCP) 支持 (`mcp/`)

#### 3.4.1 McpContext (`mcp/context.rs`)
MCP 上下文，包含执行状态和历史记录：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub id: String,                 // 上下文唯一标识
    pub parent_id: Option<String>,  // 父上下文 ID
    pub model_state: ModelState,    // 模型状态
    pub execution_history: Vec<ExecutionRecord>, // 执行历史
    pub environment_info: EnvironmentInfo, // 环境信息
    pub metadata: Value,            // 元数据
}
```

#### 3.4.2 McpRequest 和 McpResponse (`mcp/request.rs`, `mcp/response.rs`)
标准化的 MCP 请求和响应格式：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,                  // 请求唯一标识
    pub component_type: ComponentType, // 组件类型 (Tool/Plugin/Workflow)
    pub component_name: String,      // 组件名称
    pub method: String,              // 调用方法
    pub params: Value,               // 参数
    pub context: McpContext,         // 请求上下文
    pub service_context: McpServiceContext, // 服务上下文
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,                  // 响应唯一标识
    pub request_id: String,          // 关联的请求 ID
    pub status: ResponseStatus,      // 响应状态
    pub data: Option<Value>,         // 响应数据
    pub error: Option<McpError>,     // 错误信息
    pub context: McpContext,         // 响应上下文
    pub duration_ms: Option<u64>,    // 执行时长
}
```

#### 3.4.3 ContextManager (`mcp/manager.rs`)
管理 MCP 上下文的创建、更新和传播：
```rust
pub struct ContextManager {
    contexts: RwLock<HashMap<String, McpContext>>, // 上下文存储
}

impl ContextManager {
    pub fn new() -> Self;
    pub async fn create_context(&self) -> McpContext;
    pub async fn create_child_context(&self, parent_id: &str) -> Result<McpContext>;
    pub async fn get_context(&self, context_id: &str) -> Result<McpContext>;
    pub async fn update_context(&self, context: McpContext) -> Result<()>;
    pub async fn delete_context(&self, context_id: &str) -> Result<()>;
}
```

### 3.5 插件系统 (`plugin/`)

#### 3.5.1 PluginMetadata (`plugin/manifest.rs`)
插件元数据，包含 MCP 支持信息：
```rust
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PluginMetadata {
    pub name: String,                      // 插件名称
    pub display_name: LocalizedString,     // 显示名称
    pub description: LocalizedString,      // 描述
    pub user_guide: LocalizedString,       // 用户指南
    pub input_schema: Value,               // 输入 Schema
    pub output_schema: Option<Value>,      // 输出 Schema
    pub input_fields: Option<HashMap<String, LocalizedString>>, // 输入字段
    pub output_fields: Option<HashMap<String, LocalizedString>>, // 输出字段
    pub version: Option<String>,           // 版本
    pub author: Option<String>,            // 作者
    pub mcp_supported: bool,               // 是否支持 MCP
    pub mcp_capabilities: Value,           // MCP 能力
    pub requires_full_context: bool,       // 是否需要完整上下文
    pub context_validation_rules: Value,   // 上下文验证规则
}
```

#### 3.5.2 PluginManager (`plugin/mod.rs`)
管理插件的加载和访问：
```rust
pub struct PluginManager {
    plugin_dir: PathBuf,                  // 插件目录
    plugins: RwLock<HashMap<String, Arc<dyn Tool>>>, // 加载的插件
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self;
    pub async fn load_all(&self) -> Result<()>;
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
    pub async fn list_tools(&self) -> Vec<Arc<dyn Tool>>;
    pub async fn list_mcp_tools(&self) -> Vec<Arc<dyn Tool>>;
    pub async fn get_mcp_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
}
```

### 3.6 工作流相关结构 (`workflow.rs`)

#### 3.6.1 WorkflowNode
工作流节点，代表一个工具执行步骤：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,                         // 节点 ID
    pub tool_name: String,                  // 工具名称
    pub label: Option<String>,              // 节点标签
    pub input_mappings: HashMap<String, String>, // 输入映射
    pub static_inputs: Value,               // 静态输入
}
```

#### 3.6.2 WorkflowDefinition
工作流定义，包含节点和边的配置：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,                         // 工作流 ID
    pub name: String,                       // 工作流名称
    pub description: String,                // 工作流描述
    pub nodes: Vec<WorkflowNode>,           // 节点列表
    pub edges: Vec<WorkflowEdge>,           // 边列表
}
```

### 3.7 MCP 工作流扩展 (`mcp/node.rs`, `mcp/workflow.rs`)

#### 3.7.1 McpNode
扩展 `WorkflowNode`，添加 MCP 配置：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNode {
    pub base: WorkflowNode,                 // 基础节点
    pub mcp_config: McpNodeConfig,          // MCP 配置
    pub context_mappings: HashMap<String, String>, // 上下文映射
    pub output_context_updates: HashMap<String, String>, // 输出上下文更新
}
```

#### 3.7.2 McpWorkflow
扩展 `WorkflowDefinition`，支持 MCP 上下文：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWorkflow {
    pub base: WorkflowDefinition,           // 基础工作流
    pub mcp_config: McpWorkflowConfig,     // MCP 配置
    pub mcp_nodes: Vec<McpNode>,           // MCP 节点列表
}
```

### 3.8 MCP 服务器 (`server/mcp.rs`)
基于 Warp 框架实现的 MCP 服务器，提供 REST API 和 WebSocket 端点：

#### 3.8.1 REST API 端点
- `GET /health`：健康检查
- `GET /tools`：获取工具列表
- `GET /tools/mcp`：获取 MCP 支持的工具列表
- `POST /tools/{name}/call`：调用工具
- `POST /mcp/call`：MCP 调用端点

#### 3.8.2 WebSocket 支持
- `ws://{address}/ws/mcp`：WebSocket 端点，用于实时通信

### 3.9 CoreError (`error.rs`)
统一错误处理类型：
```rust
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Tool execution failed: {0}")]
    ToolFailure(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Plugin error: {0}")]
    PluginError(String),
    #[error("Workflow error: {0}")]
    WorkflowError(String),
    #[error("MCP error: {0}")]
    McpError(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}
```

## 4. 依赖 (Dependencies)
- `async-trait`: 用于支持 async trait 方法。
- `serde`: 序列化/反序列化。
- `serde_json`: 用于通用的输入输出数据交换。
- `serde_yaml`: 用于 YAML 配置文件解析。
- `thiserror`: 错误定义。
- `anyhow`: 通用错误捕获。
- `tokio`: 异步运行时 (Features: `process`, `io-util`, `fs`, `sync`, `time`, `net`).
- `tracing`: 日志记录。
- `chrono`: 时间处理。
- `uuid`: 生成唯一标识。
- `warp`: Web 服务器框架。
- `futures-util`: 异步流处理。
- `moka`: 缓存支持。
- `sled`: 嵌入式数据库，用于持久化。

## 5. 接口稳定性 (Stability)
本模块接口变更将影响所有下游 crate (`rt-tools`, `rt-cli`, `rt-gui`)，需谨慎修改。所有公共接口遵循语义化版本控制原则。

## 6. 公共工具能力说明

### 6.1 Tool 接口
`rt-core` 提供的 `Tool` 接口是所有工具和插件必须实现的核心接口。通过这个接口，工具可以：
- 定义自己的元数据（名称、描述、用户指南）
- 提供输入输出 Schema
- 实现执行逻辑
- 支持 MCP 上下文执行

### 6.2 插件加载与管理
`PluginManager` 提供了以下能力：
- 自动扫描指定目录加载插件
- 管理核心工具和插件工具
- 支持按名称获取工具
- 支持获取 MCP 支持的工具列表

### 6.3 MCP 上下文管理
`ContextManager` 提供了：
- 上下文的创建、获取、更新和删除
- 父子上下文关系管理
- 上下文传播支持

### 6.4 工作流引擎
工作流引擎支持：
- 工作流定义的解析和执行
- 节点间的上下文传递
- 支持 MCP 上下文的工作流执行
- 工作流状态管理

### 6.5 MCP 服务器
MCP 服务器提供了标准化的外部接口：
- REST API 用于工具调用和状态查询
- WebSocket 用于实时通信
- 支持 MCP 标准请求和响应格式

## 7. 使用示例

### 7.1 创建和使用 Tool
```rust
use rt_core::{Tool, Locale, Result, CoreError};
use async_trait::async_trait;
use serde_json::Value;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my-tool" }
    
    fn description(&self, _locale: Locale) -> String { "My test tool" }
    
    fn user_guide(&self, _locale: Locale) -> String { "# My Tool Guide" }
    
    fn input_schema(&self, _locale: Locale) -> Value { 
        serde_json::json!({ "type": "object", "properties": { "input": { "type": "string" } } })
    }
    
    async fn run(&self, input: Value) -> Result<Value> {
        let input_str = input["input"].as_str().ok_or(CoreError::InvalidInput("Missing input"))?;
        Ok(Value::String(format!("Hello, {}!", input_str)))
    }
}
```

### 7.2 使用 PluginManager
```rust
use rt_core::PluginManager;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let plugin_dir = PathBuf::from("./plugins");
    let manager = PluginManager::new(plugin_dir);
    
    // 加载所有插件
    manager.load_all().await.unwrap();
    
    // 获取工具列表
    let tools = manager.list_tools().await;
    println!("Loaded {} tools", tools.len());
    
    // 获取 MCP 工具列表
    let mcp_tools = manager.list_mcp_tools().await;
    println!("Loaded {} MCP tools", mcp_tools.len());
}
```

## 8. 扩展指南

### 8.1 添加新的 Tool
1. 实现 `Tool` Trait
2. 可选：实现 `McpTool` Trait 以支持 MCP
3. 在工具库中注册该工具

### 8.2 创建插件
1. 实现 `Tool` Trait
2. 使用 `PluginMetadata` 定义插件元数据
3. 构建可执行文件，命名遵循 `rt-plugin-*` 模式
4. 将插件放在指定目录中

### 8.3 扩展 MCP 功能
1. 修改 `McpContext` 添加新的上下文字段
2. 更新 `McpRequest` 和 `McpResponse` 支持新的字段
3. 扩展 `ContextManager` 支持新的上下文操作

## 9. 性能和安全性考虑

### 9.1 性能
- 使用异步编程模型，提高并发处理能力
- 上下文使用 Arc 进行共享，减少复制
- 插件加载使用异步 IO，提高启动速度
- 工作流执行支持并行处理

### 9.2 安全性
- 插件执行使用沙箱机制（Wasm 插件）
- 外部命令执行使用安全参数传递
- 输入输出 Schema 验证，防止恶意输入
- MCP 上下文隔离，防止上下文泄露

## 10. 测试策略

### 10.1 单元测试
- 对核心组件进行单元测试
- 测试 Tool 接口的各种实现
- 测试 MCP 上下文管理
- 测试插件加载和管理

### 10.2 集成测试
- 测试工作流引擎的完整执行
- 测试 MCP 服务器的 API 端点
- 测试插件与核心系统的集成

### 10.3 性能测试
- 测试大量工具和插件的加载性能
- 测试工作流执行的性能
- 测试 MCP 服务器的并发处理能力

## 11. 版本控制和发布

- 遵循语义化版本控制 (SemVer)
- 每个版本包含详细的 CHANGELOG
- 发布前运行完整的测试套件
- 向下兼容原则，除非是重大版本变更

## 12. 贡献指南

### 12.1 代码风格
- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量

### 12.2 文档要求
- 所有公共接口必须有文档注释
- 添加适当的示例代码
- 更新相关设计文档

### 12.3 测试要求
- 新增功能必须添加单元测试
- 修复 bug 必须添加回归测试
- 测试覆盖率目标：80% 以上

## 13. 未来规划

- 支持更多 MCP 标准功能
- 增强工作流可视化支持
- 提供更多插件类型支持
- 增强安全性和沙箱机制
- 支持分布式工作流执行
- 添加更多监控和日志功能

## 14. 联系方式

- 项目地址：[GitHub Repository]
- 问题跟踪：[GitHub Issues]
- 讨论区：[GitHub Discussions]
- 贡献指南：CONTRIBUTING.md

## 15. 许可证

本项目采用 [MIT License] 许可证。
