# R-Flow v2.1 重构报告

## 1. 概述

R-Flow v2.1 是一次重大架构升级，旨在将系统从单一的脚本执行器转型为 **工业级数字骨架 (Digital Backbone)**。本次重构引入了 LiteFlow 设计理念，实现了真正的并行调度、责任链模式执行器和细粒度的组件化架构。

## 2. 核心架构变更

### 2.1 工作流引擎 (RefactoredWorkflowEngine)

*   **LiteFlow 架构**: 废弃了旧版 `DefaultWorkflowEngine`，采用了基于 DAG 和组件的 `RefactoredWorkflowEngine`。
*   **Executor Chain**: 引入责任链模式，将横切关注点分离：
    *   `AuditExecutor`: 审计
    *   `CacheExecutor`: 缓存
    *   `RetryExecutor`: 重试
    *   `BasicExecutor`: 基础执行
*   **真正并行**: 利用 `tokio::spawn` 和 `futures::join_all` 实现无锁并行执行。

### 2.2 TUI 界面 (New TUI Architecture)

*   **Widget Trait**: 重构了 TUI 组件系统，所有界面元素（如 `ToolManager`, `SystemStatus`）都实现了统一的 `Widget` trait。
*   **EnhancedTuiApp**: 新的 TUI 应用入口，支持更灵活的布局和事件处理。
*   **系统监控**: 增强了系统状态监控，支持 `NetworkStatus`（包括 `Degraded` 状态）和更详细的健康评估。

### 2.3 核心数据结构 (Core Definitions)

*   **ToolInfo**: 统一了工具元数据定义，支持 `parameters_schema` (JSON Schema)。
*   **SystemStatus**: 优化了字段类型（`u32` -> `usize`/`f64`），提高了精度和兼容性。
*   **AuthConfig & RateLimitConfig**: 完善了配置结构，支持更细粒度的控制。

## 3. 插件系统

*   **多语言支持**: 完善了 Native (Rust), Python, Node.js, Docker 插件的支持。
*   **WASM 状态**: 由于依赖库兼容性问题，WASM 支持暂时禁用。
*   **统一接口**: 插件通过 `Plugin` 和 `ToolNode` trait 与核心系统交互。

## 4. 迁移指南

### 4.1 配置文件

旧版配置文件可能需要更新以匹配新的 `AuthConfig` 和 `RateLimitConfig` 结构。

### 4.2 工具开发

开发者在实现自定义工具时，应实现新的 `ToolNode` trait，并返回 `ToolInfo` 而非 `ToolDefinition`。

## 5. 总结

本次重构显著提升了系统的可维护性、扩展性和性能，为后续集成更多 AI 能力和复杂编排场景奠定了坚实基础。
