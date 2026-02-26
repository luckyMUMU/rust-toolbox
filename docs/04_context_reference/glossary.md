# Workflow Toolkit 术语表

> **版本**: v1.0  
> **最后更新**: 2026-02-26  
> **目的**: 定义项目核心术语，确保文档和代码中术语使用一致

---

## 1. 核心概念

### 1.1 工作流相关

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **工作流** | Workflow | 一系列有序执行的节点组成的自动化流程 | `WorkflowDefinition` |
| **工作流定义** | Workflow Definition | 描述工作流结构、节点、边和配置的数据结构 | `src/workflow/definition.rs` |
| **工作流执行** | Workflow Execution | 工作流的一次具体运行实例，包含状态和结果 | `WorkflowExecution` |
| **节点** | Node | 工作流中的执行单元，代表一个操作步骤 | `WorkflowNode` |
| **边** | Edge | 连接两个节点的有向关系，定义执行顺序 | `WorkflowEdge` |
| **DAG** | Directed Acyclic Graph | 有向无环图，工作流的拓扑结构 | `DagScheduler` |

### 1.2 执行状态

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **执行状态** | Execution Status | 工作流或节点的当前状态 | `ExecutionStatus` |
| **待执行** | Pending | 等待开始执行 | `ExecutionStatus::Pending` |
| **执行中** | Running | 正在执行 | `ExecutionStatus::Running` |
| **已暂停** | Paused | 执行被暂停，可恢复 | `ExecutionStatus::Paused` |
| **已完成** | Completed | 执行成功完成 | `ExecutionStatus::Completed` |
| **已失败** | Failed | 执行失败 | `ExecutionStatus::Failed` |
| **已取消** | Cancelled | 执行被取消 | `ExecutionStatus::Cancelled` |
| **已超时** | Timeout | 执行超时 | `ExecutionStatus::Timeout` |

### 1.3 节点类型

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **工具节点** | Tool Node | 执行已注册工具的节点 | `NodeType::Tool` |
| **条件节点** | Condition Node | 根据条件决定分支的节点 | `NodeType::Condition` |
| **循环节点** | Loop Node | 重复执行的节点 | `NodeType::Loop` |
| **并行节点** | Parallel Node | 并发执行多个子节点的节点 | `NodeType::Parallel` |
| **检查点节点** | Checkpoint Node | 保存执行状态的节点 | `NodeType::Checkpoint` |

---

## 2. 组件系统

### 2.1 核心组件

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **组件** | Component | 工作流节点的执行逻辑抽象 | `Component` trait |
| **组件类型** | Component Type | 组件的分类标识 | `ComponentType` |
| **组件注册表** | Component Registry | 管理所有组件的注册中心 | `ComponentRegistry` |
| **组件输出** | Component Output | 组件执行的结果 | `ComponentOutput` |
| **组件状态** | Component Status | 组件执行的状态 | `ComponentStatus` |

### 2.2 执行器链

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **执行器** | Executor | 包装组件执行的横切关注点 | `Executor` trait |
| **基础执行器** | Basic Executor | 直接调用组件的执行器 | `BasicExecutor` |
| **重试执行器** | Retry Executor | 支持重试逻辑的执行器 | `RetryExecutor` |
| **缓存执行器** | Cache Executor | 缓存执行结果的执行器 | `CacheExecutor` |
| **审计执行器** | Audit Executor | 记录审计日志的执行器 | `AuditExecutor` |
| **执行器链** | Executor Chain | 多个执行器组成的责任链 | `BoxedExecutor` |

---

## 3. 数据传递

### 3.1 数据上下文

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **数据上下文** | Data Context | 工作流执行过程中的数据容器 | `DataContext` |
| **全局槽位** | Global Slots | 工作流级别的数据存储 | `global_slots` |
| **节点槽位** | Node Slots | 节点级别的数据存储 | `node_slots` |
| **执行上下文** | Execution Context | 包含执行元数据的上下文 | `ExecutionContext` |

### 3.2 参数解析

| 术语 | 英文 | 定义 | 示例 |
|------|------|------|------|
| **参数引用** | Parameter Reference | 引用其他节点输出的语法 | `${node_id.output}` |
| **全局变量** | Global Variable | 工作流级别的变量 | `${global.var_name}` |
| **表达式求值** | Expression Evaluation | 动态计算表达式值 | `${expr: a + b}` |

---

## 4. 状态管理

### 4.1 检查点机制

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **检查点** | Checkpoint | 工作流执行状态的快照 | `Checkpoint` |
| **检查点管理器** | Checkpoint Manager | 管理检查点的创建和恢复 | `CheckpointManager` |
| **检查点存储** | Checkpoint Storage | 检查点的持久化接口 | `CheckpointStorage` |

### 4.2 恢复策略

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **恢复策略** | Recovery Strategy | 工作流恢复执行的方式 | `RecoveryStrategy` |
| **从失败节点恢复** | From Failed Node | 从失败的节点重新执行 | `RecoveryStrategy::FromFailedNode` |
| **从检查点恢复** | From Last Checkpoint | 从最近的检查点恢复 | `RecoveryStrategy::FromLastCheckpoint` |
| **从指定节点恢复** | From Specific Node | 从指定的节点开始执行 | `RecoveryStrategy::FromSpecificNode` |
| **完全重启** | Restart | 从头重新执行整个工作流 | `RecoveryStrategy::Restart` |

---

## 5. 插件系统

### 5.1 插件类型

| 术语 | 英文 | 定义 | 状态 |
|------|------|------|------|
| **原生插件** | Native Plugin | Rust 动态库插件 | ✅ 已实现 |
| **Python 插件** | Python Plugin | Python 脚本插件 | ✅ 已实现 |
| **Node.js 插件** | Node.js Plugin | Node.js 模块插件 | ✅ 已实现 |
| **Docker 插件** | Docker Plugin | Docker 容器插件 | ✅ 已实现 |
| **WASM 插件** | WASM Plugin | WebAssembly 插件 | ⚠️ 暂时禁用 |

### 5.2 插件管理

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **插件管理器** | Plugin Manager | 管理插件生命周期的组件 | `PluginManager` |
| **运行时管理器** | Runtime Manager | 管理外部运行时的组件 | `RuntimeManager` |
| **进程池** | Process Pool | 复用外部进程的池化机制 | `ProcessPool` |
| **沙箱隔离** | Sandbox Isolation | 插件的安全隔离机制 | `SecurityLayer` |

---

## 6. 工具系统

### 6.1 工具定义

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **工具** | Tool | 可被工作流调用的操作单元 | `Tool` enum |
| **工具注册表** | Tool Registry | 管理所有工具的注册中心 | `ToolRegistry` |
| **工具元数据** | Tool Metadata | 工具的描述信息 | `ToolMetadata` |
| **工具输入** | Tool Input | 工具执行的输入参数 | `ToolInput` |
| **工具输出** | Tool Output | 工具执行的结果 | `ToolOutput` |

### 6.2 工具类型

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **原生工具** | Native Tool | Rust 实现的工具 | `Tool::Native` |
| **组合工具** | Composed Tool | 由多个工具组合而成的工具 | `Tool::Composed` |
| **外部工具** | External Tool | 外部进程执行的工具 | `Tool::Python/NodeJs/Docker` |

---

## 7. 存储系统

### 7.1 存储后端

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **存储后端** | Storage Backend | 数据持久化的抽象接口 | `StorageBackend` trait |
| **文件存储** | File Storage | 基于文件系统的存储 | `FileStorage` |
| **内存存储** | Memory Storage | 基于内存的存储 | `SimpleMemoryCache` |
| **Redis 存储** | Redis Storage | 基于 Redis 的存储 | `RedisStorage` |

### 7.2 备份恢复

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **全量备份** | Full Backup | 完整数据备份 | `BackupType::Full` |
| **增量备份** | Incremental Backup | 仅备份变更数据 | `BackupType::Incremental` |
| **差异备份** | Differential Backup | 相对于基准的备份 | `BackupType::Differential` |

---

## 8. 错误处理

### 8.1 错误类型

| 术语 | 英文 | 定义 | 错误码范围 |
|------|------|------|------------|
| **系统错误** | System Error | 系统级错误 | `E0001` - `E0007` |
| **工作流错误** | Workflow Error | 工作流相关错误 | `E1001` - `E1014` |
| **工具错误** | Tool Error | 工具执行错误 | `E2001` - `E2008` |
| **插件错误** | Plugin Error | 插件相关错误 | `E3001` - `E3010` |
| **存储错误** | Storage Error | 存储相关错误 | `E4001` - `E4005` |
| **认证错误** | Authentication Error | 认证授权错误 | `E5001` - `E5004` |

### 8.2 错误处理策略

| 术语 | 英文 | 定义 | 适用场景 |
|------|------|------|----------|
| **立即重试** | Immediate Retry | 立即重新执行 | 暂时性错误 |
| **指数退避** | Exponential Backoff | 逐步增加重试间隔 | 网络错误 |
| **熔断** | Circuit Breaker | 停止执行防止级联故障 | 连续失败 |
| **降级** | Fallback | 使用备用方案 | 非关键功能 |
| **快速失败** | Fail Fast | 立即返回错误 | 不可恢复错误 |

---

## 9. 并发控制

### 9.1 并发概念

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **并发控制器** | Concurrency Controller | 控制并行执行数量的组件 | `ConcurrencyController` |
| **信号量** | Semaphore | 限制并发数的同步原语 | `Arc<Semaphore>` |
| **执行许可** | Execution Permit | 允许执行的令牌 | `ExecutionPermit` |
| **背压** | Backpressure | 当负载过高时的控制机制 | `Backpressure` |

### 9.2 限流机制

| 术语 | 英文 | 定义 | 代码位置 |
|------|------|------|----------|
| **限流器** | Rate Limiter | 控制请求速率的组件 | `RateLimiter` |
| **令牌桶** | Token Bucket | 限流算法 | `tokens` |
| **熔断器** | Circuit Breaker | 防止级联故障的组件 | `CircuitBreaker` |

---

## 10. 架构术语

### 10.1 分层架构

| 术语 | 英文 | 定义 | 目录 |
|------|------|------|------|
| **接口层** | Interfaces Layer | 对外接口实现 | `src/interfaces/` |
| **应用层** | Application Layer | 业务编排逻辑 | `src/application/` |
| **领域层** | Domain Layer | 核心业务模型 | `src/domain/` |
| **基础设施层** | Infrastructure Layer | 技术实现 | `src/infrastructure/` |

### 10.2 设计模式

| 术语 | 英文 | 定义 | 应用场景 |
|------|------|------|----------|
| **责任链模式** | Chain of Responsibility | 处理器链式调用 | 执行器链 |
| **策略模式** | Strategy Pattern | 可替换的算法族 | 恢复策略 |
| **工厂模式** | Factory Pattern | 对象创建抽象 | 插件创建 |
| **适配器模式** | Adapter Pattern | 接口转换 | Tool-Component 适配 |

---

## 11. 术语使用规范

### 11.1 命名一致性

| 推荐使用 | 避免使用 | 说明 |
|----------|----------|------|
| `RecoveryStrategy` | `ResumeStrategy` | 恢复策略统一使用 Recovery |
| `ExecutionStatus` | `WorkflowStatus` | 执行状态统一使用 Execution |
| `Component` | `Node` | 代码层面使用 Component |
| `Node` | `Step` | 文档层面使用 Node |
| `Slot` | `Variable` | 数据槽位统一使用 Slot |

### 11.2 文档引用

在文档中引用术语时：
1. 首次出现使用完整术语：**工作流定义（Workflow Definition）**
2. 后续可使用简称：工作流定义
3. 代码引用使用反引号：`WorkflowDefinition`

---

## 12. 术语变更记录

| 日期 | 变更内容 | 原因 |
|------|----------|------|
| 2026-02-26 | 创建术语表 | P3-002: 统一术语表 |
| 2026-02-26 | 统一 `ResumeStrategy` → `RecoveryStrategy` | P1-002: 恢复策略命名不一致 |

---

**维护说明**：
- 新增术语时需更新本表
- 术语变更需记录变更记录
- 定期审查术语使用一致性
