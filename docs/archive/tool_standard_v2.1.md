这是基于 **LiteFlow 组件化思想** 与 **Rust 工程实践** 的《工具 (Tool) 与插件 (Plugin) 开发规范指南》。

本规范的核心目标是将 **“业务逻辑的原子化实现”**（Tool）与 **“能力的动态分发载体”**（Plugin）解耦，建立一套标准化的扩展接口。

---

# Rust AI Agent 工具与插件开发规范 (v2.0)

## 1. 核心概念界定 (Definitions)

在开发之前，必须明确区分 **工具** 与 **插件** 的边界：

| 概念 | 定义 | 职责 | 类比 (LiteFlow) |
| --- | --- | --- | --- |
| **工具 (Tool)** | 执行特定任务的逻辑单元 | 实现 `process`、`rollback` 等核心逻辑；操作上下文数据。 | <br>**NodeComponent** (普通组件) 

 |
| **插件 (Plugin)** | 工具的容器与交付载体 | 负责工具的注册、版本管理、依赖加载；解决 ABI 兼容性与沙箱隔离。 | <br>**SPI / Jar包** 

 |

---

## 2. 工具开发规范 (Tool Standards)

所有工具（无论是原生还是脚本）本质上都是 **LiteFlow 组件** 的 Rust 映射。

### 2.1 标准 Trait 定义

所有工具必须实现 `ToolNode` Trait，该 Trait 深度借鉴了 LiteFlow 的生命周期设计 。

```rust
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

#[async_trait]
pub trait ToolNode: Send + Sync {
    /// [元数据] 工具唯一标识 (e.g., "file_reader")
    fn id(&self) -> &str;

    /// [元数据] 用于 MCP 协议的描述，帮助 LLM 理解用途
    fn description(&self) -> &str;

    /// [元数据] 输入参数的 JSON Schema 定义
    fn input_schema(&self) -> Value;

    /// [生命周期 1] 准入判断 (对应 LiteFlow isAccess)
    /// 返回 false 则跳过此节点，常用于前置参数校验或业务开关
    async fn is_access(&self, ctx: &ExecutionContext) -> bool {
        true 
    }

    /// [生命周期 2] 核心执行逻辑 (对应 LiteFlow process)
    /// 无状态设计：必须从 ctx 中获取数据，并将结果写入 ctx
    async fn process(&self, ctx: &ExecutionContext) -> Result<()>;

    /// [生命周期 3] 事务回滚 (对应 LiteFlow rollback)
    /// 当流程后续节点失败时触发，用于清理资源或补偿操作
    async fn rollback(&self, ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }
}

```

### 2.2 简单工具 (Simple Tools) 开发模式

**原则**：单一职责 (SRP)，无状态，原子操作 。

**示例：文件读取工具**

```rust
struct FileReader;

#[async_trait]
impl ToolNode for FileReader {
    fn id(&self) -> &str { "file_reader" }
    
    fn description(&self) -> &str { "读取本地文件内容" }

    async fn process(&self, ctx: &ExecutionContext) -> Result<()> {
        // 1. 从上下文获取参数 (LiteFlow Slot 模式)
        let path = ctx.get::<String>("file_path").await?;
        
        // 2. 执行逻辑
        let content = tokio::fs::read_to_string(&path).await?;
        
        // 3. 结果写回上下文
        ctx.set("file_content", content).await;
        Ok(())
    }
}

```

### 2.3 复杂工具 (Complex Tools) 开发模式

**原则**：组合即能力。复杂业务能力（Complex Capability）不一定非要封装为一个巨大的原子工具，而是可以通过 **“工具组 (Tool Set)”** 的形式体现，由多个职责单一的简单工具协同完成。

**示例：AC 自动机 (AC Automaton)**

对于 AC 自动机这一复杂能力，其在编排引擎中表现为三个独立的原子工具，通过共享上下文（Context）或外部存储协同工作：

1. **自动机管理器 (ac_manager)**: 负责自动机的初始化、构建与生命周期管理。
2. **模式串管理器 (ac_pattern)**: 负责向自动机添加、删除敏感词/模式串。
3. **模式串匹配器 (ac_matcher)**: 负责对输入文本进行多模式匹配。

这种设计使得业务编排更加灵活，例如可以在“模式串管理”和“匹配”之间插入“缓存更新”或“日志记录”等其他工具。

---

## 3. 插件开发规范 (Plugin Standards)

插件系统负责解决 Rust 的 **静态编译限制**，支持 **Native** (高性能) 和 **WASM** (强隔离) 两种模式 。

### 3.1 插件清单文件 (Manifest)

每个插件目录下必须包含 `plugin.toml`，用于描述插件元数据：

```toml
[plugin]
name = "pdf_processor"
version = "1.0.0"
description = "提供 PDF 解析与 OCR 功能"
authors = ["Team Rust"]

[tools]
# 导出的工具列表
exports = ["pdf_reader", "pdf_to_image", "ocr_extract"]

```

### 3.2 Native 插件 (基于 `abi_stable`)

适用于需要极致性能或调用系统底层 API 的场景 。

**接口定义 (FFI Safe):**
为了跨编译器版本兼容，必须使用 `abi_stable` 定义导出接口：

```rust
use abi_stable::std_types::{RString, RResult, RVec};
use abi_stable::StableAbi;

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginContext {
    // 跨边界传输的上下文
    pub data_slot: RVec<(RString, RString)>, 
}

#[repr(C)]
#[derive(StableAbi)]
pub struct NativeTool {
    pub process_fn: extern "C" fn(&PluginContext) -> RResult<(), RString>,
}

```

### 3.3 WASM 插件 (基于 `wasmtime`)

适用于运行不受信代码或多语言扩展（Python/JS 编译为 WASM）。

**WIT 接口定义 (wit-bindgen):**

```wit
// world.wit
package my:plugin;

interface tool-api {
    record context {
        trace-id: string,
    }
    // WASM 导出的处理函数
    process: func(ctx: context) -> result<string, string>;
}

world tool-plugin {
    export tool-api;
}

```

---

## 4. MCP 协议适配规范 (MCP Integration)

为了让 AI Agent (如 Claude/Cursor) 能直接调用这些工具，必须实现 **MCP 协议** 的自动映射 。

### 4.1 自动 Schema 生成

使用过程宏 `#[mcp_tool]` 自动生成 `input_schema`：

```rust
use mcp_macros::mcp_tool;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct ResizeParams {
    width: u32,
    height: u32,
}

#[mcp_tool(description = "调整图片大小")]
async fn resize_image(args: ResizeParams, ctx: &ExecutionContext) -> Result<()> {
    // 宏会自动生成 ToolNode 的实现代码
    // 并将 ResizeParams 转换为 JSON Schema
    // ... 业务逻辑
    Ok(())
}

```

### 4.2 资源 (Resources) 映射

LiteFlow 的上下文数据（Context）应映射为 MCP 的 `Resources`，允许 AI 读取执行过程中的中间结果（如日志、报告）。

* **URI 规范**: `flow://{trace_id}/data/{key}`
* **MIME Type**: 默认为 `application/json`

---

## 5. 开发最佳实践 (Best Practices)

1. **零拷贝上下文 (Zero-Copy Context)**：
* 在 Native 插件开发中，尽量传递 `Arc<DashMap>` 的引用而非克隆整个 Map，利用 Rust 的所有权机制保证安全 。




2. **错误传播 (Error Propagation)**：
* 工具内部 **禁止 panic**。所有错误必须通过 `Result::Err` 返回，以便引擎捕获并触发 `rollback` 。




3. **幂等性设计 (Idempotency)**：
* 考虑到 `Retry` (重试) 执行器的存在，工具的 `process` 方法应当设计为幂等的，即多次执行产生相同结果 。




4. **日志规范**:
* 作为 MCP Server 运行时，**严禁使用 `println!` 输出日志**（会破坏 JSON-RPC 协议帧）。必须使用 `tracing` 库将日志输出到 `stderr` 或文件 。





## 6. 总结图示 (Architecture Mapping)

这个规范体系通过 **Trait 约束行为**、**Manifest 管理元数据**、**插件隔离运行时**，完美复刻并增强了 LiteFlow 的设计，使其适应 Rust 的高性能异步生态。