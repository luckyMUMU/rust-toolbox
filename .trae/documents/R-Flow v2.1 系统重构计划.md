# R-Flow v2.1 系统重构计划 (Refined with SOLID Principles)

本计划旨在将 `workflow-toolkit` 升级为符合 v2.1 规范的 R-Flow 引擎，并严格遵循 SOLID、DRY、KISS 等软件工程原则。

## 1. 设计原则映射 (Principles Application)

*   **SRP (单一职责)**:
    *   **Context**: 仅负责数据存储与传递，剥离任何业务逻辑。
    *   **Tools**: 严格拆分“决策”与“执行”。例如，`FolderClassifier` 仅输出分类结果（Plan），不执行移动操作；移动操作由 `FsExecutor` 负责。
    *   **Scheduler**: 仅负责调度 DAG 执行，不关心具体节点业务。
*   **OCP (开闭原则)**:
    *   通过 `ToolNode` Trait 实现扩展。新增业务功能只需增加新的 Tool struct，无需修改调度器代码。
    *   复杂业务通过 `YAML/JSON` 编排配置扩展，无需修改 Rust 源码。
*   **DIP (依赖倒置)**:
    *   调度器依赖 `Box<dyn ToolNode>` 抽象接口，而非具体工具实现。
    *   工具实现依赖 `ExecutionContext` 抽象数据结构，不依赖具体运行环境（CLI/Web）。
*   **KISS & YAGNI**:
    *   弃用复杂的图算法库 (`petgraph`)，改用更直观的递归组合模式 (`Composite Pattern`) 实现调度。
    *   仅实现文档要求的核心功能，暂不引入过度设计的动态加载机制，除非文档明确要求。

## 2. 核心架构重构 (Phase 1: Core Abstractions)
**目标**: 建立高内聚、低耦合的底层抽象。

*   **2.1 上下文重构 (`src/core/context.rs`)**
    *   实现 `ExecutionContext`，使用 `Arc<DashMap>` 替代 `HashMap`，支持并发读写（**Thread-Safe**）。
    *   **DRY**: 提供统一的泛型 `get_param<T>` 方法，避免在每个工具中重复类型转换代码。
*   **2.2 统一接口定义 (`src/core/tool.rs`)**
    *   定义 `ToolNode` trait (v2.1 标准)。
    *   **ISP**: 为 `rollback` 和 `is_access` 提供默认实现 (`Ok(())`, `true`)，避免简单工具实现不必要的方法。

## 3. 编排引擎升级 (Phase 2: The Scheduler)
**目标**: 实现轻量级、高性能的递归调度器。

*   **3.1 递归流程定义 (`src/core/flow.rs`)**
    *   定义 `FlowNode` 枚举（Composite 模式），支持嵌套结构。
*   **3.2 异步调度器 (`src/workflow/scheduler.rs`)**
    *   **SRP**: 调度器只负责遍历 `FlowNode` 并生成 `tokio::spawn` 任务。
    *   实现 `WHEN` 语义的真并行（True Parallelism）。

## 4. 组件化重构 (Phase 3: Components Refactoring)
**目标**: 将现有“大工具”拆解为“原子工具”。

*   **4.1 文件管理工具组**
    *   `fs-scanner`: 仅返回文件列表（纯读取）。
    *   `logic-classifier`: 纯函数，输入文件元数据，输出目标路径（无副作用）。
    *   `fs-executor`: 接收操作指令，执行物理 IO（有副作用）。
*   **4.2 AC 自动机工具组 (New)**
    *   实现 `ac-manager`, `ac-pattern-pusher`, `ac-matcher`。
    *   确保每个工具只做一件事。

## 5. 目录结构规范化 (Phase 4: Directory Structure)
调整为 5 层架构，清晰分离关注点：
```text
src/
├── core/           # 抽象层 (Traits, Context)
├── workflow/       # 调度层 (Scheduler)
├── tools/          # 实现层 (Atomic Tools)
│   ├── base/       # 基础工具
│   └── algo/       # 算法工具
├── extension/      # 扩展层 (Plugins, Adapters)
└── storage/        # 数据层 (Persistence)
```

## 6. 验证 (Phase 5: Verification)
*   编写单元测试，模拟高并发场景下的 `Context` 读写安全性。
*   验证 `DagScheduler` 的错误传播与回滚机制。

## 执行步骤
1.  **Core**: 创建 `src/core/`，迁移 Context 与 ToolNode 定义。
2.  **Scheduler**: 重写调度器。
3.  **Tools**: 拆分现有工具并适配新接口。
4.  **Integration**: 更新入口文件。
