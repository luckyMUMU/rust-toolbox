# 上下文参考索引

> **项目**: Workflow Toolkit  
> **版本**: v0.1.0  
> **最后更新**: 2026-02-26

---

## 概述

本索引汇集项目开发过程中的关键决策、设计模式和技术债务记录，为团队成员和 AI 助手提供快速参考。

---

## 1. 架构决策记录（ADR）

架构决策记录（Architecture Decision Records）用于追踪项目中的重要技术决策及其背景。

### 核心决策索引

| 编号 | 标题 | 状态 | 文档链接 |
|------|------|------|----------|
| ADR-001 | 采用 Rust 作为核心开发语言 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-001) |
| ADR-002 | 采用 Enum 而非 Trait 实现工具系统 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-002) |
| ADR-003 | 采用 DAG 而非 State Machine 实现工作流 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-003) |
| ADR-004 | 采用 Layered Architecture 分层架构 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-004) |
| ADR-005 | 采用 Middleware Pattern 实现横切关注点 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-005) |
| ADR-006 | 支持 Multi-Language Plugins 多语言插件 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-006) |
| ADR-007 | 采用 Tokio 作为 Async Runtime | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-007) |
| ADR-008 | 采用 Layered Configuration System 分层配置 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-008) |
| ADR-009 | 采用 Pseudocode Specification 描述业务逻辑 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-009) |
| ADR-010 | 采用 Tiered Documentation Architecture 分级文档 | 已接受 | [architecture_decision.md](./architecture_decision.md#adr-010) |

### 详细决策文档

| 编号 | 模块 | 标题 | 文档链接 |
|------|------|------|----------|
| ADR-DI-001 | 依赖注入 | 自建 DI 容器选择 | [adr_di_001_自建DI容器选择.md](./adr/adr_di_001_自建DI容器选择.md) |
| ADR-Workflow-002 | 工作流 | 采用 JoinSet 并行执行 | [adr_workflow_002_采用JoinSet并行执行.md](./adr/adr_workflow_002_采用JoinSet并行执行.md) |
| ADR-Plugin-003 | 插件系统 | WASM 沙箱隔离策略 | [adr_plugin_003_WASM沙箱隔离策略.md](./adr/adr_plugin_003_WASM沙箱隔离策略.md) |
| ADR-Tools-004 | 工具系统 | Schema 验证策略 | [adr_tools_004_Schema验证策略.md](./adr/adr_tools_004_Schema验证策略.md) |

### 待决策事项

| 主题 | 状态 | 阻碍因素 |
|------|------|----------|
| WebAssembly 作为主要插件格式 | 考虑中 | 生态不成熟，调试困难 |
| 分布式执行支持 | 考虑中 | 增加复杂度，当前单机足够 |
| gRPC 作为主要通信协议 | 考虑中 | 增加复杂度，HTTP/REST 足够当前需求 |

---

## 2. 设计模式说明

### 2.1 核心架构模式

#### 分层架构（Layered Architecture）

```
Interface Layer (CLI/TUI/MCP)
    ↓
Application Layer (UseCase/Service)
    ↓
Domain Layer (Model/Port)
    ↓
Infrastructure Layer (Persistence/Plugin)
    ↓
Adapter Layer (DTO/适配器)
```

**职责划分**：
- **Interface Layer**: 用户交互入口，支持 CLI、TUI、MCP 等多种接口
- **Application Layer**: 用例编排，协调领域对象完成业务流程
- **Domain Layer**: 核心业务逻辑，领域模型和端口定义
- **Infrastructure Layer**: 技术实现，持久化、插件加载等
- **Adapter Layer**: 数据转换，DTO 与领域对象映射

#### 六边形架构（Hexagonal Architecture）

```
        ┌─────────────────────────────┐
        │      Domain Layer           │
        │   (Core Business Logic)     │
        └─────────────────────────────┘
              ↑                  ↑
        ┌─────┴─────┐      ┌─────┴─────┐
        │   Ports   │      │   Ports   │
        │  (Inbound)│      │ (Outbound)│
        └─────┬─────┘      └─────┬─────┘
              ↓                  ↓
        ┌──────────┐       ┌──────────┐
        │ Adapters │       │ Adapters │
        │  (CLI)   │       │ (DB/API) │
        └──────────┘       └──────────┘
```

### 2.2 行为模式

#### 中间件模式（Middleware Pattern）

用于实现横切关注点（缓存、重试、超时、日志等）：

```rust
pub trait Middleware {
    async fn execute(
        &self,
        ctx: &mut MiddlewareContext,
        next: Next<'_>,
    ) -> Result<ToolOutput>;
}
```

**特点**：
- 可组合：中间件可灵活组合
- 可复用：同一中间件可用于多个工具
- 可测试：中间件可独立测试

#### 责任链模式（Chain of Responsibility）

工具执行流程通过中间件链传递：

```
Request → [Logging] → [Caching] → [Retry] → [Timeout] → Tool → Response
```

### 2.3 结构模式

#### 枚举多态（Enum-based Polymorphism）

工具系统采用 Enum 而非 Trait Object：

```rust
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}
```

**优势**：
- 零成本抽象，无动态分发开销
- 编译时类型安全
- 易于序列化

### 2.4 数据模式

#### DAG 工作流模型

使用 `petgraph` 库实现有向无环图：

```
     ┌───┐
     │ A │
     └─┬─┘
       │
    ┌──┴──┐
    ↓     ↓
 ┌───┐ ┌───┐
 │ B │ │ C │  (并行执行)
 └─┬─┘ └─┬─┘
   │     │
   └──┬──┘
      ↓
   ┌───┐
   │ D │
   └───┘
```

**特点**：
- 自动识别可并行节点
- 支持复杂依赖关系
- 循环检测保证正确性

---

## 3. 技术债务跟踪

### 3.1 已知技术债务

| 编号 | 模块 | 描述 | 优先级 | 状态 | 创建日期 |
|------|------|------|--------|------|----------|
| TD-001 | - | 待补充 | - | - | - |

### 3.2 技术债务分类

#### 代码质量类

- 待补充

#### 架构类

- 待补充

#### 性能类

- 待补充

#### 文档类

- 待补充

### 3.3 偿还计划

| 编号 | 计划偿还版本 | 预计工作量 | 负责人 |
|------|-------------|-----------|--------|
| - | - | - | - |

---

## 4. 术语表

统一术语定义，确保文档和代码中术语使用一致。

**文档链接**: [glossary.md](./glossary.md)

### 核心术语速查

| 术语 | 定义 |
|------|------|
| **工作流定义** | 描述工作流结构、节点、边和配置的数据结构 |
| **节点** | 工作流中的执行单元 |
| **组件** | 工作流节点的执行逻辑抽象 |
| **执行器** | 包装组件执行的横切关注点 |
| **数据上下文** | 工作流执行过程中的数据容器 |
| **检查点** | 工作流执行状态的快照 |
| **恢复策略** | 工作流恢复执行的方式 |

---

## 5. 快速参考

### 5.1 关键技术栈

| 类别 | 技术选型 | 版本要求 |
|------|---------|---------|
| 语言 | Rust | ≥ 1.75 |
| 异步运行时 | Tokio | ≥ 1.0 |
| 序列化 | Serde | ≥ 1.0 |
| 图算法 | petgraph | ≥ 0.6 |
| CLI 框架 | clap | ≥ 4.0 |

### 5.2 项目结构速览

```
rust-tool-v2/
├── src/
│   ├── domain/        # 领域层
│   ├── application/   # 应用层
│   ├── infrastructure/# 基础设施层
│   └── interface/     # 接口层
├── docs/
│   ├── 01_concept_overview.md
│   ├── 02_logical_workflow/
│   ├── 03_technical_spec/
│   └── 04_context_reference/
└── tests/
```

### 5.3 编码规范速查

- **错误处理**: 严禁 `unwrap/expect`，使用 `Result` 和 `?` 操作符
- **命名**: 变量/函数用 `snake_case`，类型用 `PascalCase`，常量用 `UPPER_SNAKE_CASE`
- **注释**: 代码注释含中文说明，解释"为什么"而非"是什么"
- **测试**: 核心业务逻辑需有单元测试覆盖

---

## 更新日志

| 日期 | 更新内容 | 更新人 |
|------|---------|--------|
| 2026-02-26 | 创建上下文参考索引 | AI Assistant |

---

> **提示**: 本索引应随项目演进持续更新。新增架构决策、设计模式变更或技术债务时，请同步更新此文件。
