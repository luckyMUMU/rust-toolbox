# R-Flow v2.1 重构报告 (Refactoring Report)

**日期**: 2026-01-18
**状态**: Phase 1-5 完成
**版本**: v2.1.0-alpha

## 1. 重构概述
本次重构旨在将 `workflow-toolkit` 升级为符合 v2.1 设计规范的 R-Flow 引擎。核心目标是实现真正的并行调度、零拷贝上下文管理和原子化工具体系。

## 2. 变更详情

### 2.1 核心架构 (Core Architecture)
*   **上下文 (`ExecutionContext`)**: 
    *   从 `HashMap` 升级为 `Arc<DashMap<String, Value>>`，实现了线程安全的高并发读写。
    *   新增 `state` 字段 (`Arc<DashMap<String, Arc<dyn Any>>>`)，支持存储运行时对象（如 AC 自动机实例、数据库连接）。
    *   遵循 **SRP**，剥离了配置和状态管理逻辑。
*   **工具接口 (`ToolNode`)**:
    *   采用 `async_trait` 定义标准接口，包含 `id`, `process`, `is_access`, `rollback`。
    *   移除了旧的 Builder 模式，遵循 **ISP**，仅暴露必要接口。
*   **流程定义 (`FlowNode`)**:
    *   引入递归枚举结构（Composite Pattern），支持 `Node`, `Chain`, `Parallel`, `Switch` 嵌套组合。

### 2.2 编排引擎 (Orchestration Engine)
*   **调度器 (`DagScheduler`)**:
    *   **完全重写**：弃用了 `petgraph`，改用递归遍历执行。
    *   **真并行 (`True Parallelism`)**: 在 `WHEN` (Parallel) 分支中使用 `tokio::spawn` 分发任务，实现了物理级并行。
    *   解决了递归异步函数的 `Send` 和 `Box` 问题。

### 2.3 工具体系 (Tool System)
*   **AC 自动机工具组 (New)**:
    *   实现了 `ac-manager` (初始化), `ac-pattern-pusher` (添加模式), `ac-matcher` (匹配)。
    *   利用 `ExecutionContext.state` 在不同工具间共享 `RwLock<AhoCorasickMatcher>`。
*   **基础工具**:
    *   重构了 `data-cache` 和 `data-transform` 以适配新接口。

### 2.4 目录结构 (Directory Structure)
调整后的 5 层架构：
```text
src/
├── core/           # Context, ToolNode, FlowNode, Config
├── workflow/       # DagScheduler, WorkflowEngine
├── tools/          # Atomic Tools
│   ├── base/       # Data Tools
│   ├── algo/       # AC Automaton
│   └── fs/         # File Tools (Placeholder)
├── plugins/        # (Temporarily Disabled)
└── interfaces/     # (Temporarily Disabled)
```

## 3. 验证结果
*   **集成测试 (`tests/integration_test.rs`)**: 
    *   场景: 初始化 AC 自动机 -> 动态添加模式串 -> 文本匹配。
    *   结果: **PASS**。验证了上下文共享、工具协同和调度器执行逻辑。

## 4. 后续计划 (Next Steps)
1.  **CLI 迁移**: 重写 `src/interfaces/cli` 以适配新的 `WorkflowEngine`。
2.  **插件适配**: 更新 `src/plugins` 下的 Python/Node.js 插件以实现新 `ToolNode` trait。
3.  **Engine 增强**: 在 `WorkflowEngine` 中恢复 Audit Log 和 Persistence 功能。
