# Workflow Toolkit - 系统架构与设计总览

## 版本记录

| 版本 | 日期 | 作者 | 变更内容 |
|------|------|------|----------|
| v1.0 | 2024-xx-xx | Team | 初始版本 |
| v2.0 | 2026-01-16 | Trae AI | 更新LiteFlow架构，添加LLM全局设计规范 |

## 1. 系统概览

**Workflow Toolkit** 是一个高性能、模块化的 Rust 工作流执行系统，专为构建复杂的 AI Agent、数据处理管道和自动化任务而设计。它结合了 DAG（有向无环图）调度、插件化架构和强大的状态管理能力，能够支持从简单的脚本自动化到复杂的 LLM 多轮对话编排。

### 核心价值主张
*   **高性能**: 基于 Rust 和 Tokio 构建，支持真正的并行执行和微秒级工具调用。
*   **模块化**: 核心引擎与业务逻辑分离，通过插件系统无限扩展。
*   **可靠性**: 内置检查点、自动重试、事务性状态管理和全面的审计日志。
*   **LLM 就绪**: 专为 AI Agent 设计的上下文管理、Token 计数和流式响应支持。

## 2. 系统架构设计

### 2.1 架构总览图

```mermaid
graph TB
    subgraph "User Interface Layer"
        CLI[Command Line]
        TUI[Terminal UI]
        MCP[MCP Server]
    end

    subgraph "Application Layer"
        EM[Execution Manager]
        Config[Config Manager]
    end

    subgraph "Core Engine (LiteFlow)"
        WE[Refactored Workflow Engine]
        Scheduler[DAG Scheduler]
        Tracker[Execution Tracker]
        
        WE <--> Scheduler
        WE <--> Tracker
    end

    subgraph "Executor Layer"
        Chain[Executor Chain]
        Audit[Audit] --> Cache[Cache] --> Retry[Retry] --> Basic[Basic]
    end

    subgraph "Component & Plugin Layer"
        TR[Tool Registry]
        PM[Plugin Manager]
        
        Native[Native Plugin]
        Python[Python Plugin]
        Node[Node.js Plugin]
        Docker[Docker Plugin]
    end

    subgraph "Infrastructure Layer"
        Store[Storage (LanceDB)]
        Mem[Memory Cache (Moka)]
        Monitor[System Monitor]
    end

    CLI & TUI & MCP --> EM
    EM --> WE
    WE --> Chain
    Basic --> TR
    TR --> PM
    PM --> Native & Python & Node & Docker
    
    WE --> Store
    WE --> Monitor
```

### 2.2 模块功能说明

1.  **用户接口层 (User Interface Layer)**
    *   **CLI**: 适合 CI/CD 和脚本调用的命令行工具。
    *   **TUI**: 基于 Ratatui 的交互式终端界面，提供实时的任务监控和管理。
    *   **MCP Server**: 遵循 Model Context Protocol，使 IDE 和 LLM 能够直接调用本系统的能力。

2.  **核心引擎层 (Core Engine)**
    *   **Execution Manager**: 统一的入口点，处理并发限制和任务队列。
    *   **Workflow Engine**: 驱动主循环，管理生命周期。详见 [WORKFLOW_DESIGN.md](WORKFLOW_DESIGN.md)。
    *   **DAG Scheduler**: 负责拓扑排序和动态依赖解析。

3.  **插件系统层 (Plugin System)**
    *   支持多语言插件（Rust, Python, Node.js, Docker）。
    *   **隔离性**: 不同插件运行在独立的运行时或进程中，互不干扰。

4.  **基础设施层 (Infrastructure)**
    *   **Storage**: 集成 LanceDB 向量数据库，支持语义搜索和长期记忆。
    *   **Cache**: 多级缓存策略（内存 + 磁盘），加速重复执行。

## 3. API 接口规范

系统主要通过以下几种方式暴露能力：

### 3.1 Rust SDK
供其他 Rust 项目集成使用。
```rust
use workflow_toolkit::prelude::*;

let engine = ExecutionManager::new(config);
let result = engine.execute_workflow(definition).await?;
```

### 3.2 MCP Protocol
提供标准的 JSON-RPC 接口：
*   `tools/list`: 列出可用工具。
*   `tools/call`: 执行特定工具。
*   `resources/list`: 列出工作流模版。

### 3.3 CLI 命令
```bash
workflow-toolkit workflow execute <file> [flags]
workflow-toolkit plugin install <path>
```

## 4. 性能指标与扩展性

### 4.1 性能设计
*   **异步 I/O**: 全链路异步设计，最大化利用系统资源。
*   **零拷贝**: 在核心路径上尽量减少数据复制，使用 `Arc` 共享状态。
*   **细粒度锁**: 使用 `DashMap` 和 `RwLock` 替代全局互斥锁，减少竞争。

### 4.2 扩展性设计
*   **水平扩展**: 无状态的 Worker 设计，支持未来扩展到分布式执行。
*   **插件热加载**: 支持在不重启主进程的情况下加载/卸载插件（部分支持）。

### 4.3 基准指标
*   **调度延迟**: < 1ms
*   **吞吐量**: 单机支持 100+ 并发工作流（取决于任务负载）
*   **内存占用**: 空闲 < 50MB

## 5. 错误处理机制

系统采用分层错误处理策略，确保健壮性。

1.  **组件级**: 
    *   自动重试（Exponential Backoff）。
    *   超时控制（Hard Timeout）。
2.  **工作流级**:
    *   **错误隔离**: 单个节点的失败不会导致整个引擎崩溃。
    *   **级联阻塞**: 依赖失败节点的后续任务会被标记为 `Blocked`，而非失败。
    *   **补偿机制**: 支持定义 `on_failure` 钩子执行清理操作（规划中）。
3.  **系统级**:
    *   Panic 捕获与恢复。
    *   优雅停机（Graceful Shutdown），保存当前状态到 Checkpoint。

## 6. 安全合规要求

1.  **沙箱执行**: 
    *   Docker 插件提供完全的文件系统和网络隔离。
    *   WASM 插件提供内存级隔离。
2.  **权限控制**:
    *   工具执行需显式授权（配置文件白名单）。
    *   敏感参数（如 API Key）自动脱敏。
3.  **审计日志**:
    *   记录所有关键操作（执行、数据访问、配置变更）。
    *   日志不可篡改（通过追加写模式）。

## 7. 文档索引

*   **详细设计**:
    *   [工作流引擎设计](WORKFLOW_DESIGN.md)
    *   [插件系统设计](../../src/plugins/DESIGN.md)
*   **指南**:
    *   [开发指南](DEVELOPMENT_GUIDE.md)
    *   [用户指南](../guides/USER_GUIDE.md)
