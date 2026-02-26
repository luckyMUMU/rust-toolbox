# 技术规范索引

> **工作流工具包技术文档导航**  
> *最后更新：2026-02-26*

---

## 概述

本索引提供工作流工具包技术规范的完整导航，涵盖 API 参考、接口定义和数据结构说明。

---

## 1. API 参考

### 1.1 CLI 参考

完整的命令行接口文档，包含所有命令、选项和使用示例。

**文档路径**: [api/CLI_REFERENCE.md](api/CLI_REFERENCE.md)

**主要内容**:
- 全局选项配置
- 工作流管理命令（`workflow`）
- 工具管理命令（`tool`）
- 插件管理命令（`plugin`）
- 批处理操作命令（`batch`）
- 其他命令（`tui`、`server`、`completion`）

### 1.2 Rust SDK 参考

Rust SDK 完整 API 参考，包含核心 traits、数据结构和用法示例。

**文档路径**: [api/RUST_SDK_REFERENCE.md](api/RUST_SDK_REFERENCE.md)

**主要内容**:
- WorkflowEngine Trait
- ToolRegistry Trait
- Plugin Trait
- StorageBackend Trait
- 快速开始示例
- 高级用法示例

---

## 2. 接口定义

### 2.1 接口契约规范

定义系统核心接口契约，包含前置条件、后置条件和约束说明。

**文档路径**: [interfaces.md](interfaces.md)

**主要内容**:

| 接口类型 | 核心接口 | 说明 |
|---------|---------|------|
| 工作流引擎 | `WorkflowEngine` | 工作流执行核心接口 |
| 工作流引擎 | `ExecutionManager` | 执行管理器接口 |
| 工具系统 | `ToolRegistry` | 工具注册表接口 |
| 工具系统 | `Tool` | 工具枚举统一抽象 |
| 插件系统 | `Plugin` | 插件接口 |
| 插件系统 | `PluginManager` | 插件管理器接口 |
| 存储系统 | `WorkflowRepository` | 工作流仓库接口 |
| 存储系统 | `StateManager` | 状态管理器接口 |

---

## 3. 数据结构说明

### 3.1 核心类型

| 类型 | 说明 | 定义位置 |
|------|------|---------|
| `WorkflowDefinition` | 工作流定义 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `WorkflowNode` | 工作流节点 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `WorkflowEdge` | 工作流边 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `ExecutionStatus` | 执行状态枚举 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `ToolInput` | 工具输入 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `ToolOutput` | 工具输出 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `PluginInfo` | 插件信息 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |
| `PluginType` | 插件类型枚举 | [interfaces.md#核心类型](interfaces.md#51-core-types核心类型) |

### 3.2 配置类型

| 类型 | 说明 | 定义位置 |
|------|------|---------|
| `ResourceRequirements` | 资源需求配置 | [interfaces.md#配置类型](interfaces.md#52-configuration-types配置类型) |
| `RetryPolicy` | 重试策略配置 | [interfaces.md#配置类型](interfaces.md#52-configuration-types配置类型) |
| `BackoffStrategy` | 退避策略枚举 | [interfaces.md#配置类型](interfaces.md#52-configuration-types配置类型) |
| `CachePolicy` | 缓存策略配置 | [interfaces.md#配置类型](interfaces.md#52-configuration-types配置类型) |
| `SecurityPolicy` | 安全策略配置 | [interfaces.md#配置类型](interfaces.md#52-configuration-types配置类型) |

### 3.3 并发配置

```rust
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,  // 默认: 10
    pub max_concurrent_tasks: usize,      // 默认: 50
    pub task_queue_size: usize,           // 默认: 100
}
```

---

## 4. 错误码参考

### 4.1 错误码分类

| 分类 | 错误码范围 | 详细说明 |
|------|-----------|---------|
| 系统错误 | `E0001` - `E0004` | [interfaces.md#系统错误码](interfaces.md#61-system-error-codes系统错误码) |
| 工作流错误 | `E1001` - `E1006` | [interfaces.md#工作流错误码](interfaces.md#62-workflow-error-codes工作流错误码) |
| 工具错误 | `E2001` - `E2005` | [interfaces.md#工具错误码](interfaces.md#63-tool-error-codes工具错误码) |
| 插件错误 | `E3001` - `E3005` | [interfaces.md#插件错误码](interfaces.md#64-plugin-error-codes插件错误码) |

---

## 5. 性能约束

### 5.1 响应时间目标

| 操作 | 目标 | 最大容忍 |
|------|------|---------|
| 工作流提交 | < 10ms | 100ms |
| 节点调度 | < 1ms | 10ms |
| 工具执行 | 取决于工具 | 可配置 timeout |
| 状态查询 | < 5ms | 50ms |

### 5.2 版本兼容性

- 公共 API 必须保持向后兼容
- 破坏性变更必须增加主版本号
- 废弃 API 必须标记 `#[deprecated]` 并保留至少一个版本

---

## 6. 文档导航

```
docs/03_technical_spec/
├── index.md                    # 本文档（技术规范索引）
├── interfaces.md               # 接口契约规范
└── api/
    ├── CLI_REFERENCE.md        # CLI 参考
    └── RUST_SDK_REFERENCE.md   # Rust SDK 参考
```
