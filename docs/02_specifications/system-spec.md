# Workflow Toolkit 系统规范 (System Specification)

## 文档元数据

- **版本**: v1.0.0
- **规范级别**: P1 (系统规范)
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 系统概述

### 1.1 系统定位

Workflow Toolkit 是一个基于 Rust 构建的多接口工作流执行系统，采用领域驱动设计（DDD）和分层架构，支持多种插件类型、工作流编排和多种用户界面。

### 1.2 核心价值主张

- **高性能**: 使用 Rust 的零成本抽象和并发原语，提供卓越的执行性能
- **可扩展**: 插件化架构支持多种插件类型和运行时
- **多界面**: 提供 CLI、TUI、MCP 三种用户界面
- **企业级**: 完整的审计、缓存、重试、熔断等企业级功能
- **可观测**: 完整的指标、追踪、日志系统

### 1.3 技术栈

- **语言**: Rust 1.70+
- **异步运行时**: Tokio
- **并发数据结构**: DashMap
- **缓存**: moka
- **序列化**: serde, serde_json
- **日志**: tracing
- **插件运行时**: Python, Node.js, Docker, WASM

## 2. 系统边界

### 2.1 系统范围

**包含**:
- 工作流定义和执行引擎
- 插件系统（Native, Python, Node.js, Docker, WASM）
- 工具系统（工具注册、执行、组合）
- 存储系统（状态管理、备份恢复）
- 用户界面（CLI, TUI, MCP）
- 性能监控和优化
- 审计和日志系统

**不包含**:
- 外部服务的具体实现（如云存储、消息队列等）
- 特定领域的业务逻辑（通过插件扩展）
- 用户认证和授权（由上层应用提供）

### 2.2 系统边界接口

#### 2.2.1 用户界面接口

**CLI 接口**:
- 命令行参数解析
- 工作流执行命令
- 工具管理命令
- 插件管理命令
- 批处理命令

**TUI 接口**:
- 交互式工作流编辑和执行
- 实时执行监控
- 工具和插件管理
- 系统状态监控
- 日志查看

**MCP 接口**:
- AI 助手集成
- 工作流执行 API
- 工具调用 API
- 状态查询 API

#### 2.2.2 插件接口

**Plugin trait**:
```rust
pub trait Plugin: Send + Sync {
    async fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn get_tools(&self) -> Vec<Tool>;
    fn info(&self) -> &PluginInfo;
    fn status(&self) -> PluginStatus;
}
```

#### 2.2.3 工具接口

**Tool enum**:
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

## 3. 模块职责

### 3.1 领域层 (src/domain/)

**职责**:
- 定义核心领域模型（执行、工作流、插件、工具）
- 定义领域端口（PluginManager, Repository, ToolRegistry）
- 提供领域服务（执行计算、工作流验证）
- 管理领域事件

**约束**:
- 不依赖任何基础设施细节
- 不依赖外部框架
- 纯粹的业务逻辑

**核心组件**:
- `model/`: 领域模型
- `port/`: 领域端口
- `service/`: 领域服务
- `event/`: 事件系统

### 3.2 应用层 (src/application/)

**职责**:
- 协调领域层完成具体用例
- 处理事务边界和安全检查
- 编排工作流执行流程

**约束**:
- 只依赖领域层
- 不包含业务逻辑
- 薄应用层原则

**核心组件**:
- `service/`: 应用服务
- `workflow/`: 工作流编排
- `port/`: 应用层端口
- `usecase/`: 用例定义

### 3.3 基础设施层 (src/infrastructure/)

**职责**:
- 提供技术实现
- 实现领域层定义的接口
- 持久化、外部服务集成、配置管理

**约束**:
- 实现领域层定义的接口
- 可插拔的存储后端
- 并发安全

**核心组件**:
- `persistence/`: 持久化
- `plugin/`: 插件基础设施
- `config/`: 配置管理
- `external/`: 外部服务

### 3.4 接口层 (src/interfaces/)

**职责**:
- 提供多种用户界面
- 处理用户输入和输出
- 转换接口层数据

**约束**:
- 只依赖应用层和领域层
- 不包含业务逻辑
- 清晰的职责分离

**核心组件**:
- `cli/`: 命令行界面
- `tui/`: 终端用户界面
- `mcp/`: MCP 协议
- `dto/`: 数据传输对象

### 3.5 工作流模块 (src/workflow/)

**职责**:
- 提供工作流引擎
- 组件系统（LiteFlow 风格）
- 执行器链
- 状态管理和检查点

**约束**:
- 遵循 LiteFlow 设计原则
- 支持真正的并行执行
- 统一状态管理

**核心组件**:
- `engine.rs`: 工作流引擎
- `definition.rs`: 工作流定义
- `component/`: 组件系统
- `executor/`: 执行器链
- `context/`: 上下文系统
- `state/`: 状态管理

### 3.6 插件模块 (src/plugins/)

**职责**:
- 提供可扩展的插件系统
- 支持多种插件类型
- 插件生命周期管理

**约束**:
- 并发安全
- 异步操作
- 自动工具注册

**核心组件**:
- `manager.rs`: 插件管理器
- `types.rs`: 插件类型定义
- `runtime.rs`: 运行时管理
- `process_pool.rs`: 进程池
- `native.rs`, `python.rs`, `nodejs.rs`, `docker.rs`, `wasm.rs`: 具体插件实现

### 3.7 存储模块 (src/storage/)

**职责**:
- 提供数据持久化
- 缓存和备份功能
- 状态管理

**约束**:
- 多层缓存
- 批量操作
- 备份恢复

**核心组件**:
- `state_manager.rs`: 状态管理器
- `backends.rs`: 存储后端
- `backup.rs`: 备份系统

### 3.8 工具模块 (src/tools/)

**职责**:
- 提供工具系统
- 工具注册和执行
- 工具组合和中间件

**约束**:
- 高性能注册表
- 多索引支持
- 灵活的组合

**核心组件**:
- `types.rs`: 工具类型定义
- `registry.rs`: 工具注册表
- `executor.rs`: 工具执行器
- `middleware.rs`: 中间件系统
- `composable.rs`: 可组合工具

### 3.9 性能模块 (src/performance/)

**职责**:
- 性能监控和优化
- 内存管理
- 并发管理
- 分布式追踪

**约束**:
- 统一的性能管理
- 全面的监控
- 自动优化

**核心组件**:
- `metrics/`: 指标收集
- `cache/`: 缓存管理
- `concurrency/`: 并发管理
- `memory/`: 内存管理
- `profiler/`: 性能分析
- `tracing/`: 分布式追踪

### 3.10 依赖注入模块 (src/di/)

**职责**:
- 提供依赖注入容器
- 模块系统
- 服务生命周期管理

**约束**:
- 模块化注册
- 灵活的提供者
- 类型安全

**核心组件**:
- `container.rs`: DI 容器
- `module.rs`: 模块系统
- `modules/`: 具体模块
- `provider.rs`: 提供者

## 4. 跨模块接口规范

### 4.1 工具注册表接口

**ToolRegistry trait** (定义在领域层，实现在工具模块):

```rust
pub trait ToolRegistry: Send + Sync {
    fn register(&self, tool: Tool) -> Result<()>;
    fn get(&self, name: &str) -> Result<Tool>;
    fn get_by_id(&self, id: &ToolId) -> Result<Tool>;
    fn get_by_category(&self, category: &str) -> Result<Vec<Tool>>;
    fn get_by_tag(&self, tag: &str) -> Result<Vec<Tool>>;
    fn get_by_kind(&self, kind: ToolKind) -> Result<Vec<Tool>>;
    async fn execute(&self, tool_id: &ToolId, input: ToolInput) -> Result<ToolOutput>;
    fn find_by_pattern(&self, pattern: &str) -> Result<Vec<Tool>>;
    fn check_version_conflicts(&self) -> Result<Vec<VersionConflict>>;
}
```

### 4.2 插件管理器接口

**PluginManager trait** (定义在领域层，实现在插件模块):

```rust
pub trait PluginManager: Send + Sync {
    async fn load_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()>;
    async fn unload_plugin(&mut self, plugin_id: &PluginId) -> Result<()>;
    async fn reload_plugin(&mut self, plugin_id: &PluginId) -> Result<()>;
    async fn get_all_tools(&self) -> Result<Vec<Tool>>;
    async fn shutdown_all(&mut self) -> Result<()>;
    fn get_plugin_status(&self, plugin_id: &PluginId) -> Result<PluginStatus>;
}
```

### 4.3 仓储接口

**ExecutionRepository trait** (定义在领域层，实现在基础设施层):

```rust
pub trait ExecutionRepository: Send + Sync {
    async fn save(&self, execution: &WorkflowExecution) -> Result<()>;
    async fn load(&self, id: &ExecutionId) -> Result<Option<WorkflowExecution>>;
    async fn list(&self, filter: ExecutionFilter) -> Result<Vec<WorkflowExecution>>;
    async fn delete(&self, id: &ExecutionId) -> Result<()>;
}
```

**PluginRepository trait**:

```rust
pub trait PluginRepository: Send + Sync {
    async fn save(&self, plugin: &PluginInfo) -> Result<()>;
    async fn load(&self, id: &PluginId) -> Result<Option<PluginInfo>>;
    async fn list(&self) -> Result<Vec<PluginInfo>>;
    async fn delete(&self, id: &PluginId) -> Result<()>;
}
```

**WorkflowRepository trait**:

```rust
pub trait WorkflowRepository: Send + Sync {
    async fn save(&self, workflow: &WorkflowDefinition) -> Result<()>;
    async fn load(&self, id: &WorkflowId) -> Result<Option<WorkflowDefinition>>;
    async fn list(&self) -> Result<Vec<WorkflowDefinition>>;
    async fn delete(&self, id: &WorkflowId) -> Result<()>;
}
```

### 4.4 存储后端接口

**StorageBackend trait** (定义在存储模块):

```rust
pub trait StorageBackend: Send + Sync {
    async fn save(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn batch_save(&self, items: Vec<(String, Vec<u8>)>) -> Result<()>;
    async fn batch_load(&self, keys: Vec<String>) -> Result<Vec<Option<Vec<u8>>>>;
}
```

**CacheBackend trait**:

```rust
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn size(&self) -> Result<usize>;
}
```

### 4.5 工作流引擎接口

**WorkflowEngine trait** (定义在工作流模块):

```rust
pub trait WorkflowEngine: Send + Sync {
    async fn execute(&self, workflow: &WorkflowDefinition, context: &DataContext) -> Result<WorkflowExecution>;
    async fn pause(&self, execution_id: &ExecutionId) -> Result<()>;
    async fn resume(&self, execution_id: &ExecutionId) -> Result<()>;
    async fn stop(&self, execution_id: &ExecutionId) -> Result<()>;
    fn get_execution_status(&self, execution_id: &ExecutionId) -> Result<ExecutionStatus>;
}
```

### 4.6 组件接口

**Component trait** (定义在工作流模块):

```rust
pub trait Component: Send + Sync {
    fn component_type(&self) -> ComponentType;
    async fn execute(&self, context: &mut DataContext) -> Result<ComponentOutput>;
    fn validate(&self) -> Result<()>;
    fn cacheable(&self) -> bool;
}
```

### 4.7 执行器接口

**Executor trait** (定义在工作流模块):

```rust
pub trait Executor: Send + Sync {
    async fn execute(&self, context: &mut ExecutionContext) -> Result<ExecutionResult>;
}
```

### 4.8 中间件接口

**Middleware trait** (定义在工具模块):

```rust
pub trait Middleware: Send + Sync {
    async fn before(&self, input: &ToolInput) -> Result<Option<ToolInput>>;
    async fn after(&self, output: &ToolOutput) -> Result<Option<ToolOutput>>;
    async fn on_error(&self, error: &ToolError) -> Result<Option<ToolOutput>>;
}
```

## 5. 数据流规范

### 5.1 工作流执行数据流

```
用户请求 (CLI/TUI/MCP)
    ↓
应用层 (WorkflowService)
    ↓
工作流引擎 (RefactoredWorkflowEngine)
    ↓
组件系统 (Component)
    ↓
工具执行 (ToolExecutor)
    ↓
插件运行时 (Plugin Runtime)
    ↓
外部工具 (Python/Node.js/Docker/WASM)
```

### 5.2 工具注册数据流

```
插件加载 (PluginManager)
    ↓
插件初始化 (Plugin::initialize)
    ↓
工具获取 (Plugin::get_tools)
    ↓
工具注册 (ToolRegistry::register)
    ↓
索引更新 (name_index, category_index, tag_index)
```

### 5.3 状态管理数据流

```
工作流执行 (WorkflowEngine)
    ↓
状态更新 (StateManager)
    ↓
缓存写入 (CacheBackend)
    ↓
持久化 (StorageBackend)
    ↓
备份 (BackupManager)
```

## 6. 错误处理规范

### 6.1 错误分类

- **领域错误**: 业务逻辑错误，如工作流验证失败
- **应用错误**: 用例执行错误，如事务失败
- **基础设施错误**: 技术实现错误，如存储失败
- **接口错误**: 用户输入错误，如命令格式错误

### 6.2 错误传播

- 领域错误 → 应用层 → 接口层 → 用户
- 基础设施错误 → 领域层 → 应用层 → 接口层 → 用户
- 接口错误 → 用户（不传播到下层）

### 6.3 错误恢复

- **重试**: 临时性错误（网络超时、存储不可用）
- **熔断**: 持续性错误（服务降级）
- **降级**: 非关键功能失败（性能监控）
- **回滚**: 事务失败（状态恢复）

## 7. 性能要求

### 7.1 响应时间

- CLI 命令响应: < 100ms
- TUI 界面响应: < 50ms
- 工具执行: < 1s (无外部工具)
- 工作流启动: < 200ms

### 7.2 吞吐量

- 工具执行吞吐: > 1000 ops/s
- 工作流执行吞吐: > 100 workflows/min
- 插件加载: < 1s per plugin

### 7.3 资源使用

- 内存使用: < 500MB (空闲)
- CPU 使用: < 10% (空闲)
- 并发支持: > 1000 并发任务

## 8. 安全要求

### 8.1 插件安全

- WASM 沙箱隔离（安全级别分级）
- 文件系统权限控制
- 网络权限控制
- 资源限制（内存、CPU、时间）

### 8.2 数据安全

- 敏感信息加密
- 审计日志记录
- 访问控制（由上层应用提供）

### 8.3 依赖安全

- 定期更新依赖
- 使用安全扫描工具
- 避免已知漏洞依赖

## 9. 可观测性要求

### 9.1 日志

- 结构化日志（tracing）
- 日志级别（ERROR, WARN, INFO, DEBUG, TRACE）
- 日志上下文（执行 ID、插件 ID、工具 ID）

### 9.2 指标

- Counter: 计数器（执行次数、错误次数）
- Gauge: 仪表（内存使用、并发数）
- Histogram: 直方图（执行时间、响应时间）

### 9.3 追踪

- 分布式追踪（tracing）
- 跨度（Span）: 工作流执行、工具执行
- 上下文传播: 执行 ID、用户 ID

## 10. 测试要求

### 10.1 单元测试

- 覆盖率 > 80%
- 核心模块覆盖率 > 90%
- 使用 property-based testing

### 10.2 集成测试

- 模块间集成测试
- 插件集成测试
- 存储后端集成测试

### 10.3 端到端测试

- 完整工作流执行测试
- 多界面测试
- 性能测试

## 11. 文档要求

### 11.1 代码文档

- 公共 API 必须有文档注释
- 复杂逻辑必须有行内注释
- 示例代码（可选）

### 11.2 设计文档

- 架构设计文档
- 模块设计文档
- ADR (Architecture Decision Record)

### 11.3 用户文档

- CLI 使用文档
- TUI 使用文档
- MCP API 文档
- 插件开发文档

## 12. 版本管理

### 12.1 语义化版本

- MAJOR: 不兼容的 API 变更
- MINOR: 向后兼容的功能新增
- PATCH: 向后兼容的问题修复

### 12.2 兼容性

- 工具接口: MINOR 版本向后兼容
- 插件接口: MAJOR 版本可能不兼容
- 数据格式: PATCH 版本向后兼容

## 13. 附录

### 13.1 术语表

- **工作流 (Workflow)**: 由节点和边组成的有向图，定义任务的执行顺序和条件
- **工具 (Tool)**: 可执行的最小单元，可以是 Native、Python、Node.js、Docker 或 WASM
- **插件 (Plugin)**: 工具的容器，提供工具的加载、卸载和生命周期管理
- **组件 (Component)**: 工作流节点的抽象，支持工具、条件、循环、并行等类型
- **执行器 (Executor)**: 负责执行组件的逻辑，支持重试、缓存、审计等横切关注点
- **中间件 (Middleware)**: 工具执行的拦截器，支持日志、指标、重试等横切关注点

### 13.2 参考文档

- [项目宪章](../01_constitution/project-charter.md)
- [架构原则](../01_constitution/architecture-principles.md)
- [质量红线](../01_constitution/quality-redlines.md)
- [安全基线](../01_constitution/security-baseline.md)
- [ADR 文档](../../.temp/ADR/)

### 13.3 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
