# rt-core Design Document

## 1. 模块概述 (Module Overview)
`rt-core` 是 Rust Toolbox 的核心库，定义了所有工具和工作流必须遵循的基础接口 (Traits) 以及通用数据结构。它不包含具体的业务逻辑工具（这些在 `rt-tools` 中），只提供“骨架”。

## 2. 核心职责 (Core Responsibilities)
- 定义 `Tool` Trait：所有具体工具必须实现的接口。
- 定义 `Locale` Enum：支持的多语言区域设置。
- 定义 `Workflow` 结构：管理工具执行顺序和上下文传递。
- 定义 `Context`：在工具间传递的数据载体。
- 定义 `CoreError`：统一错误处理类型。
- 定义 `Workflow Engine`：工作流编排与执行 (详见 `WORKFLOW_DESIGN.md`)。

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
```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::locale::Locale;

#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;
    
    /// 显示名称 (支持多语言)
    fn display_name(&self, locale: Locale) -> String;

    /// 工具描述 (用于 UI 展示)
    fn description(&self, locale: Locale) -> String;
    
    /// 用户指南 (Markdown, 支持多语言)
    fn user_guide(&self, locale: Locale) -> String;

    /// 输入 Schema
    fn input_schema(&self, locale: Locale) -> Value;

    /// 输出 Schema
    fn output_schema(&self, locale: Locale) -> Value;
    
    /// 执行逻辑
    async fn run(&self, input: Value) -> Result<Value, crate::CoreError>;
}
```

### 3.3 Plugin System (`plugin.rs`)
`rt-core` 提供了加载外部插件的能力。
- **`PluginTool`**: 一个实现了 `Tool` Trait 的结构体，负责包装外部可执行文件。
- **`load_plugins`**: 扫描指定目录，自动发现名为 `rt-plugin-*` 的可执行文件并加载。

### 3.4 Workflow Engine
```rust
pub struct Workflow {
    pub name: String,
    pub steps: Vec<Box<dyn Tool>>,
}

impl Workflow {
    pub async fn execute(&self, initial_input: Value) -> Result<Value, CoreError>;
}
```

### 3.5 CoreError
使用 `thiserror` 定义：
```rust
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Tool execution failed: {0}")]
    ToolFailure(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
```

## 4. 依赖 (Dependencies)
- `async-trait`: 用于支持 async trait 方法。
- `serde`: 序列化/反序列化。
- `serde_json`: 用于通用的输入输出数据交换。
- `thiserror`: 错误定义。
- `anyhow`: 通用错误捕获。
- `tokio`: 异步运行时 (Features: `process`, `io-util`, `fs`)。

## 5. 接口稳定性 (Stability)
本模块接口变更将影响所有下游 crate (`rt-tools`, `rt-cli`, `rt-gui`)，需谨慎修改。
