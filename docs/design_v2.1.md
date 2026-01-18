这是一份基于 **LiteFlow 设计哲学** 并结合 **Rust 高性能异步特性** 的全景重构设计方案。

该方案将系统从单一的脚本执行器转型为 **工业级数字骨架 (Digital Backbone)**，核心逻辑是将“**做什么 (Tool)**”与“**怎么流转 (Orchestration)**”彻底分离。

---

# 工业级 Rust AI Agent 编排工具包 (R-Flow) 全景架构设计

## 1. 核心设计哲学 (Core Philosophy)

本架构深度借鉴 **LiteFlow** 的“组件化编排”思想 ，并针对 Rust 的所有权模型和异步运行时进行了原生适配。

1. **皆为组件 (Everything is a Component)**：无论是原子操作（如 HTTP 请求）还是复杂的业务流（如 RAG 检索），在引擎视角的接口是统一的。
2. 
**工作台模式 (Workbench Pattern)**：组件之间不直接传递参数，而是围绕一个共享的 **数据槽 (Data Slot)** 进行生产和消费，彻底解耦组件依赖 。


3. 
**声明式编排 (Declarative Orchestration)**：业务逻辑通过 **EL (Expression Language)** 或 YAML 规则定义，而非硬编码 。


4. 
**工具分级 (Tool Categorization)**：严格区分 **简单工具 (Simple Tools)** 与 **复杂工具 (Complex Tools)**，后者本质上是由编排引擎聚合的前者 。



---

## 2. 五层系统架构模型 (Five-Layer Architecture)

### 2.1 第一层：接入层 (User Interface Layer)

实现“逻辑一次编写，多端同步交付”，支持三种交互形态：

* **CLI (自动化)**：基于 `clap`，用于 CI/CD 管道和脚本调用。
* 
**TUI (实时监控)**：基于 `ratatui`，通过异步事件循环接收引擎快照，提供类似 Vim 的监控界面 。


* 
**MCP Server (AI 原生)**：遵循 **Model Context Protocol**，将编排好的“复杂工具”暴露给 Cursor/Claude 等 AI 助手调用 。



### 2.2 第二层：编排引擎层 (Orchestration Engine - The Core)

这是系统的“大脑”，负责解析规则并调度执行。核心借鉴 LiteFlow 的流程控制：

* 
**EL 解析器 (EL Parser)**：支持类似 LiteFlow 的语义 `THEN(a, WHEN(b, c), SWITCH(d))` 。


* **真正的并行调度 (True Parallelism)**：
* 利用 Rust 的 `tokio` 运行时，将 `WHEN` 语义下的组件映射为 `tokio::spawn` 任务 。


* 通过 `futures::join_all` 实现无锁等待，相比 Java 线程池模型，上下文切换开销降低至微秒级 。




* 
**隐式子流程 (Implicit Sub-flow)**：允许在一个组件内部通过代码动态调用另一个完整的流程链，实现递归式的逻辑复用 。



### 2.3 第三层：组件与工具层 (Components & Tools)

所有执行单元实现统一的 `NodeComponent` Trait。

#### **简单工具 (Simple Tools - Atomic)**

* **定义**：无状态、原子化、纯函数式操作。
* **实现**：Rust 原生代码，极低开销。
* **示例**：`FileReader`, `JsonParser`, `HttpRequest`, `VectorEmbed`。
* 
**特性**：遵循单一职责原则 (SRP) 。



#### **复杂工具 (Complex Tools - Composite)**

* **定义**：由多个简单工具通过 EL 规则编排而成的“虚拟组件”。
* **实现**：在引擎中注册为一个 `Chain`，但对外暴露为标准组件接口。
* **示例**：`DailyReportGenerator` (由 `FetchData` -> `Analyze` -> `Summarize` -> `Email` 组成)。
* 
**特性**：支持**有状态** (Stateful) 和 **环境隔离** (Docker/WASM) 。



### 2.4 第四层：扩展系统层 (Extensibility)

解决 Rust 静态编译与动态业务需求的矛盾，支持 **ABI 稳定** 的插件系统 。

* 
**Native 插件**：使用 `abi_stable` crate 加载 Rust 动态库 (.so/.dll)，性能损耗近乎为零 。


* 
**WASM 沙箱**：集成 `wasmtime`，允许用户使用 Python/JS/Go 编写逻辑，并在受限沙箱中运行，确保主进程安全 。


* 
**热重载机制 (Hot Reload)**：支持在不重启主进程的情况下，动态卸载并重新加载插件或规则文件 。



### 2.5 第五层：数据与状态层 (Data & Persistence)

* 
**上下文槽 (Context Slot)**：使用 `Arc<DashMap<String, Value>>` 实现线程安全的“工作台”，支持高并发读写 。


* 
**时间旅行 RAG (Persistence)**：集成 **LanceDB**，不仅存储向量数据，还利用其版本化特性记录每一次执行的上下文快照，实现“时间旅行”式的审计与回滚 。



---

## 3. 详细设计：组件与编排的二元性

### 3.1 定义工具 (Defining Tools)

所有的工具（无论是 Rust 原生还是 WASM 插件）都必须实现以下 Trait：

```rust
#[async_trait]
pub trait ToolNode: Send + Sync {
    // 核心执行逻辑
    async fn process(&self, ctx: &ExecutionContext) -> Result<ToolOutput>;
    
    // 准入判断 (对应 LiteFlow isAccess)
    async fn is_access(&self, ctx: &ExecutionContext) -> bool { true }
    
    // 异常回调 (对应 LiteFlow rollback)
    async fn rollback(&self, ctx: &ExecutionContext) -> Result<()> { Ok(()) }
}

```



### 3.2 编排规则 (Orchestration Rules)

采用声明式配置 (YAML/JSON) 描述业务流，引擎将其编译为执行图 DAG。

**场景示例：智能文档归档 (Smart Document Archiving)**

这是一个 **复杂工具**，由编排引擎调度内部的简单工具完成。

```yaml
id: "smart_archive_flow"
name: "智能文档归档流程"
description: "扫描目录，识别内容，自动分类并归档"

# 1. 定义简单工具引用
nodes:
  - id: scanner      # 简单工具：目录扫描
  - id: content_ocr  # 简单工具：OCR识别
  - id: llm_classify # 简单工具：LLM分类
  - id: archiver     # 简单工具：文件移动
  - id: error_log    # 简单工具：错误记录

# 2. 编排逻辑 (EL 风格)
# 逻辑：扫描 -> (并行处理: OCR & 提取特征) -> LLM分类 -> (成功? 归档 : 报错)
chain: 
  - THEN:
      - node: scanner
      - WHEN: # 并行执行
          - node: content_ocr
          - node: extract_metadata
      - node: llm_classify
      - SWITCH: # 选择分支
          on: "${context.classification_confidence > 0.8}"
          to:
            - CASE: 
                value: true
                node: archiver
            - CASE: 
                value: false
                node: human_review # 引入人工介入工具
      - FINALLY: # 兜底逻辑
          node: cleanup_temp_files

```



### 3.3 异常处理与回滚 (Resilience & Rollback)

系统引入 LiteFlow 的自动回滚机制。当流程在步骤 D 失败时，引擎会自动按 **反序** (C -> B -> A) 调用已执行节点的 `rollback()` 方法，实现事务性补偿 。

---

## 4. 核心优势总结

1. 
**解耦 (Decoupling)**：业务逻辑（YAML）与代码实现（Rust）完全分离。修改流程无需重新编译代码 。


2. 
**性能 (Performance)**：基于 Rust 零成本抽象，基准测试显示节点调度延迟在  -  级别 。


3. 
**可观测性 (Observability)**：结合 TUI 和 LanceDB，不仅能看到当前的执行状态，还能回溯任意历史时刻的完整上下文（时间旅行） 。


4. 
**AI 就绪 (AI-Ready)**：通过 MCP 协议，这套工具包不仅给人用，也能直接作为 Agent 的手和眼，赋予大模型操作本地文件和系统的能力 。

## 核心实现
这是一个基于 **Tokio** 异步运行时和 **LiteFlow** 设计理念的 `DagScheduler` 核心实现。

这段代码展示了如何通过递归的 `Flow` 结构（复合模式）来解析执行计划，并重点实现了 `WHEN` 语义下的 **真正并行（True Parallelism）**，利用 `tokio::spawn` 将无依赖的任务分发到不同的线程上执行，从而将调度延迟降低到微秒级。

### 核心实现代码 (`src/scheduler.rs`)

```rust
use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;
use tokio::task::JoinHandle;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::future::join_all;

/// ========================================================================
/// 1. 上下文与组件定义 (Data Slot & Component Layer)
/// ========================================================================

/// 执行上下文 (The "Workbench")
/// 线程安全的键值存储，所有组件通过它交换数据，实现“零拷贝”引用传递。
pub type ContextData = DashMap<String, Value>;

#[derive(Clone)]
pub struct ExecutionContext {
    pub data: Arc<ContextData>,
    pub trace_id: String,
}

impl ExecutionContext {
    pub fn new(trace_id: &str) -> Self {
        Self {
            data: Arc::new(DashMap::new()),
            trace_id: trace_id.to_string(),
        }
    }
}

/// 组件接口 (Standard Component Interface)
/// 所有的简单工具(Simple Tools)和复杂工具(Complex Tools)都必须实现此接口。
#[async_trait]
pub trait ToolNode: Send + Sync {
    /// 组件唯一标识
    fn id(&self) -> &str;

    /// 核心执行逻辑 (Stateless Execution)
    async fn process(&self, ctx: &ExecutionContext) -> Result<()>;

    /// 准入判断 (LiteFlow: isAccess)
    /// 返回 true 则执行，false 则跳过
    async fn is_access(&self, _ctx: &ExecutionContext) -> bool {
        true
    }

    /// 异常回滚 (LiteFlow: rollback)
    async fn rollback(&self, _ctx: &ExecutionContext) -> Result<()> {
        Ok(())
    }
}

/// ========================================================================
/// 2. 编排结构定义 (Orchestration Structure - The "EL")
/// ========================================================================

/// 流程节点类型，对应 LiteFlow 的 EL 语法结构
#[derive(Clone)]
pub enum FlowNode {
    /// 原子组件节点
    Node(Arc<dyn ToolNode>),
    
    /// 串行编排 (THEN)
    Chain(Vec<FlowNode>),
    
    /// 并行编排 (WHEN)
    Parallel(Vec<FlowNode>),
    
    /// 选择编排 (SWITCH)
    /// 包含一个决策闭包和分支映射
    Switch {
        condition: Arc<dyn Fn(&ExecutionContext) -> bool + Send + Sync>,
        positive: Box<FlowNode>,
        negative: Box<FlowNode>,
    },
}

/// ========================================================================
/// 3. 调度器实现 (The Core Engine)
/// ========================================================================

pub struct DagScheduler;

impl DagScheduler {
    /// 执行入口
    pub async fn execute(flow: &FlowNode, ctx: ExecutionContext) -> Result<()> {
        match flow {
            FlowNode::Node(tool) => Self::execute_node(tool.clone(), ctx).await,
            FlowNode::Chain(nodes) => Self::execute_chain(nodes, ctx).await,
            FlowNode::Parallel(nodes) => Self::execute_parallel(nodes, ctx).await,
            FlowNode::Switch { condition, positive, negative } => {
                Self::execute_switch(condition, positive, negative, ctx).await
            }
        }
    }

    /// 执行单个原子组件
    async fn execute_node(tool: Arc<dyn ToolNode>, ctx: ExecutionContext) -> Result<()> {
        // 1. 准入检查 (isAccess)
        if !tool.is_access(&ctx).await {
            println!("[Scheduler] Node {} skipped by access check.", tool.id());
            return Ok(());
        }

        println!("[Scheduler] Executing Node: {}", tool.id());
        
        // 2. 执行业务逻辑
        // 注意：此处发生错误时，外层捕捉后可触发 Rollback 逻辑
        tool.process(&ctx).await.map_err(|e| {
            anyhow!("Node {} failed: {}", tool.id(), e)
        })
    }

    /// 实现 THEN 语义：串行执行
    async fn execute_chain(nodes: &[FlowNode], ctx: ExecutionContext) -> Result<()> {
        for node in nodes {
            // 串行 await，前一个成功才执行下一个
            Self::execute(node, ctx.clone()).await?;
        }
        Ok(())
    }

    /// 实现 WHEN 语义：真正的并行执行 (True Parallelism)
    /// 利用 Tokio Spawn 将任务分发到线程池，而非简单的并发 Future
    async fn execute_parallel(nodes: &[FlowNode], ctx: ExecutionContext) -> Result<()> {
        let mut handles: Vec<JoinHandle<Result<()>>> = Vec::with_capacity(nodes.len());

        println!("[Scheduler] Spawning {} parallel tasks (WHEN)...", nodes.len());

        for node in nodes {
            let node_clone = node.clone();
            let ctx_clone = ctx.clone();

            // 关键优化：使用 tokio::spawn 生成独立的 Task。
            // 这允许 Tokio 运行时将这些任务调度到不同的 OS 线程上物理并行运行。
            let handle = tokio::spawn(async move {
                Self::execute(&node_clone, ctx_clone).await
            });
            handles.push(handle);
        }

        // 等待所有分支完成 (Barrier)
        // LiteFlow 默认行为是 "等待所有"，也可以配置 "any" (任意一个完成)
        let results = join_all(handles).await;

        // 结果聚合与错误检查
        for res in results {
            match res {
                Ok(inner_result) => inner_result?, // 传播内部业务错误
                Err(join_err) => return Err(anyhow!("Task panic: {}", join_err)), // 处理 Panic
            }
        }

        Ok(())
    }

    /// 实现 SWITCH 语义：条件分支
    async fn execute_switch(
        condition: &Arc<dyn Fn(&ExecutionContext) -> bool + Send + Sync>,
        positive: &FlowNode,
        negative: &FlowNode,
        ctx: ExecutionContext
    ) -> Result<()> {
        let result = (condition)(&ctx);
        if result {
            Self::execute(positive, ctx).await
        } else {
            Self::execute(negative, ctx).await
        }
    }
}

/// ========================================================================
/// 4. 使用示例 (Usage Example)
/// ========================================================================

// 模拟一个简单的打印工具
struct PrintTool { id: String, msg: String }
#[async_trait]
impl ToolNode for PrintTool {
    fn id(&self) -> &str { &self.id }
    async fn process(&self, _ctx: &ExecutionContext) -> Result<()> {
        // 模拟耗时操作
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        println!("[{}] Processing: {}", self.id, self.msg);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = ExecutionContext::new("trace-1001");

    // 定义工具
    let scanner = Arc::new(PrintTool { id: "Scanner".into(), msg: "Scanning dir...".into() });
    let ocr = Arc::new(PrintTool { id: "OCR".into(), msg: "Running OCR...".into() });
    let vectorizer = Arc::new(PrintTool { id: "Vec".into(), msg: "Vectorizing text...".into() });
    let archiver = Arc::new(PrintTool { id: "Archiver".into(), msg: "Uploading to S3...".into() });

    // 编排逻辑: THEN(Scanner, WHEN(OCR, Vectorizer), Archiver)
    // 逻辑：先扫描，然后 并行 进行OCR识别和向量化，最后归档。
    let flow = FlowNode::Chain(vec![
        FlowNode::Node(scanner),
        FlowNode::Parallel(vec![
            FlowNode::Node(ocr),
            FlowNode::Node(vectorizer),
        ]),
        FlowNode::Node(archiver),
    ]);

    println!("--- Workflow Start ---");
    let start = std::time::Instant::now();
    
    DagScheduler::execute(&flow, ctx).await?;
    
    println!("--- Workflow Finished in {:?} ---", start.elapsed());
    Ok(())
}

```

### 代码设计解析

1. **真正的并行 (True Parallelism)**：
* 在 `execute_parallel` 函数中，我们没有仅仅使用 `futures::join`（这在单线程运行时中只是并发），而是使用了 `tokio::spawn` 。


* 这使得每个 `WHEN` 分支都成为一个独立的 Tokio Task。在多核 CPU 上，Tokio 的 Work-stealing 调度器会将这些任务分配给不同的 OS 线程，实现物理并行，这是高性能 RAG 处理（如并发 OCR 和向量化）的关键 。




2. **上下文隔离与共享 (State Management)**：
* 
`ExecutionContext` 内部持有 `Arc<DashMap>`。克隆 `ExecutionContext` 是廉价的（只增加引用计数），这允许所有并行任务安全地访问和修改同一个数据槽，完美复刻了 LiteFlow 的“工作台模式” 。




3. **递归组合模式 (Composite Pattern)**：
* 
`FlowNode` 枚举是一个递归结构。`Chain` 和 `Parallel` 包含 `Vec<FlowNode>`，这意味着你可以构建任意深度的嵌套逻辑，例如“在并行分支中再串行执行几个步骤” (`WHEN(THEN(a, b), c)`) 。




4. **错误传播 (Error Propagation)**：
* 并行执行使用了 `join_all` 等待所有任务。如果其中一个任务返回 `Err`，调度器会捕获并向上传播，导致整个流程（默认策略）失败。这为后续实现 `rollback` 提供了切入点 。