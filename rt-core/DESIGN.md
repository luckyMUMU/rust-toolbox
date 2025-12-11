# rt-core Design Document

## 1. 模块概述 (Module Overview)
`rt-core` 是 Rust Toolbox 的核心库，定义了所有工具和工作流必须遵循的基础接口 (Traits) 以及通用数据结构。它不包含具体的业务逻辑工具（这些在 `rt-tools` 中），只提供“骨架”。

## 2. 核心职责 (Core Responsibilities)
- 定义 `Tool` Trait：所有具体工具必须实现的接口。
- 定义 `Workflow` 结构：管理工具执行顺序和上下文传递。
- 定义 `Context`：在工具间传递的数据载体。
- 定义 `CoreError`：统一错误处理类型。

## 3. 详细设计 (Detailed Design)

### 3.1 Tool Trait
```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称 (唯一标识)
    fn name(&self) -> &str;
    
    /// 工具描述 (用于 UI 展示)
    fn description(&self) -> &str;
    
    /// 执行逻辑
    /// @param input: 上一个工具的输出或初始输入
    /// @return: 工具执行结果 (JSON Value)
    async fn run(&self, input: Value) -> Result<Value, crate::CoreError>;
}
```

### 3.2 Workflow Engine
```rust
pub struct Workflow {
    pub name: String,
    pub steps: Vec<Box<dyn Tool>>,
}

impl Workflow {
    pub async fn execute(&self, initial_input: Value) -> Result<Value, CoreError>;
}
```

### 3.3 CoreError
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
- `async-trait`:用于支持 async trait 方法。
- `serde_json`: 用于通用的输入输出数据交换。
- `thiserror`: 错误定义。
- `anyhow`: 通用错误捕获。

## 5. 接口稳定性 (Stability)
本模块接口变更将影响所有下游 crate (`rt-tools`, `rt-cli`, `rt-gui`)，需谨慎修改。
