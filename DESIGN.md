# Workflow Toolkit - 项目设计文档

## 项目概述

基于工作流的工具包是一个使用Rust开发的多接口工具系统，支持通过CLI、TUI和MCP服务器三种方式提供工作流执行功能。该系统旨在为用户提供灵活的工作流定义、执行和管理能力。

## 核心架构

### 模块结构

```
workflow-toolkit/
├── src/
│   ├── lib.rs              # 库入口，导出公共API
│   ├── main.rs             # CLI应用程序入口
│   ├── config.rs           # 配置管理
│   ├── core.rs             # 核心类型定义
│   ├── error.rs            # 错误处理
│   ├── storage/            # 存储层
│   │   ├── mod.rs
│   │   ├── backends.rs     # 存储后端trait
│   │   └── state_manager.rs # 状态管理器
│   ├── workflow/           # 工作流引擎
│   │   ├── mod.rs
│   │   ├── definition.rs   # 工作流定义
│   │   ├── engine.rs       # 执行引擎trait
│   │   └── execution.rs    # 执行状态
│   ├── tools/              # 工具系统
│   │   ├── mod.rs
│   │   ├── node.rs         # 工具节点trait
│   │   └── registry.rs     # 工具注册表
│   ├── plugins/            # 插件系统
│   │   ├── mod.rs
│   │   ├── types.rs        # 插件类型
│   │   └── manager.rs      # 插件管理器
│   └── interfaces/         # 用户接口
│       ├── mod.rs
│       ├── cli.rs          # CLI接口
│       ├── tui.rs          # TUI接口
│       └── mcp.rs          # MCP服务器
├── Cargo.toml              # 项目配置
└── DESIGN.md               # 设计文档
```

### 核心类型系统

#### 基础类型 (core.rs)

- `WorkflowId`: 工作流唯一标识符 (UUID)
- `ToolId`: 工具节点标识符 (String)
- `PluginId`: 插件标识符 (String)
- `ExecutionContext`: 执行上下文，包含全局变量和会话信息
- `ExecutionStatus`: 执行状态枚举 (Pending, Running, Paused, Completed, Failed, Cancelled, Timeout)
- `RetryPolicy`: 重试策略配置
- `WorkflowConfig`: 工作流配置参数

#### 错误处理 (error.rs)

采用 `thiserror` 库定义结构化错误类型：
- `WorkflowError`: 主要错误类型，包含各种子错误类别
- `Result<T>`: 类型别名，简化错误处理

#### 配置管理 (config.rs)

使用 `config` crate 实现分层配置：
- 默认配置 → 配置文件 → 环境变量 → 命令行参数
- 支持TOML格式配置文件
- 自动配置验证和目录创建

### 存储层设计

#### 存储后端抽象 (storage/backends.rs)

定义了两个核心trait：
- `StorageBackend`: 持久化存储接口
- `CacheBackend`: 缓存存储接口

支持的存储后端：
- LanceDB: 主要数据存储，支持向量数据库功能
- 本地内存缓存: 使用moka实现高性能缓存

#### 状态管理器 (storage/state_manager.rs)

- 统一的状态管理接口
- 缓存和持久化存储的协调
- 工作流执行历史管理
- 数据一致性保证

### 工作流系统

#### 工作流定义 (workflow/definition.rs)

- `WorkflowDefinition`: 工作流定义结构
- `WorkflowNode`: 工作流节点定义
- `WorkflowEdge`: 工作流边定义
- `NodeType`: 节点类型枚举 (Tool, Condition, Loop, Parallel, Checkpoint)

#### 工作流引擎 (workflow/engine.rs)

- `WorkflowEngine` trait: 定义工作流执行接口
- 支持异步执行、暂停、恢复、停止操作
- 基于petgraph的DAG调度

#### 执行状态 (workflow/execution.rs)

- `WorkflowExecution`: 工作流执行状态
- `NodeExecutionState`: 节点执行状态
- 实时状态跟踪和历史记录

### 工具系统

#### 工具节点 (tools/node.rs)

- `ToolNode` trait: 定义工具节点接口
- 参数验证和执行方法
- 工具元数据管理

#### 工具注册表 (tools/registry.rs)

- `ToolRegistry` trait: 工具注册和发现接口
- 工具执行和参数验证
- 支持动态工具加载

### 插件系统

#### 插件类型 (plugins/types.rs)

- `Plugin` trait: 插件基础接口
- `PluginType`: 支持多种插件类型
  - Native: Rust动态库
  - Python: Python包装器
  - NodeJs: Node.js包装器
  - Docker: Docker容器插件
  - Wasm: WebAssembly插件

#### 插件管理器 (plugins/manager.rs)

- 插件生命周期管理
- 插件配置和元数据
- 动态加载和卸载

### 接口层

#### CLI接口 (interfaces/cli.rs)

使用clap v4实现命令行接口：
- 工作流管理命令 (create, execute, status, pause, resume, stop)
- 工具管理命令 (list, execute)
- 插件管理命令 (install, list, reload)

#### TUI接口 (interfaces/tui.rs)

基于ratatui的终端用户界面：
- `TuiInterface` trait: TUI接口定义
- 响应式界面组件
- 实时状态监控

#### MCP服务器 (interfaces/mcp.rs)

Model Context Protocol服务器实现：
- `McpServerInterface` trait: MCP服务器接口
- JSON-RPC 2.0协议支持
- WebSocket实时通信
- 身份验证和授权

## 技术选型

### 核心依赖

- **异步运行时**: tokio 1.42 (完整功能)
- **序列化**: serde 1.0 (JSON/YAML支持)
- **CLI框架**: clap 4.5 (derive特性)
- **TUI框架**: ratatui 0.29 + crossterm 0.28
- **数据库**: lancedb 0.20 + arrow 54.0
- **缓存**: moka 0.12 (异步支持)
- **日志**: tracing 0.1 + tracing-subscriber 0.3
- **错误处理**: anyhow 1.0 + thiserror 2.0
- **并发**: dashmap 6.1 + parking_lot 0.12
- **插件系统**: libloading 0.8 + wasmtime 27.0 + extism 1.8
- **工作流调度**: petgraph 0.6
- **UUID**: uuid 1.11
- **时间处理**: chrono 0.4

### 开发依赖

- **属性测试**: proptest 1.6
- **异步测试**: tokio-test 0.4
- **临时文件**: tempfile 3.14
- **模拟**: mockall 0.13
- **容器测试**: testcontainers 0.23

## 设计原则

### 1. 模块化架构

- 清晰的模块边界和职责分离
- 基于trait的抽象接口设计
- 支持插件化扩展

### 2. 类型安全

- 强类型系统，编译时错误检查
- 使用newtype模式避免类型混淆
- 结构化错误处理

### 3. 异步优先

- 全面采用async/await模式
- 非阻塞I/O操作
- 高并发支持

### 4. 配置驱动

- 分层配置系统
- 环境变量和命令行参数支持
- 配置验证和默认值

### 5. 可观测性

- 结构化日志记录
- 性能指标收集
- 错误跟踪和调试支持

## 扩展点设计

### 1. 存储后端扩展

通过实现`StorageBackend`和`CacheBackend` trait，可以添加新的存储后端：
- 关系型数据库 (PostgreSQL, MySQL)
- NoSQL数据库 (MongoDB, Redis)
- 云存储服务 (AWS S3, Azure Blob)

### 2. 工具节点扩展

通过实现`ToolNode` trait，可以添加新的工具类型：
- HTTP API调用工具
- 数据处理工具
- 文件操作工具
- 系统命令工具

### 3. 插件系统扩展

支持多种插件类型：
- Native插件: Rust动态库
- 脚本插件: Python, Node.js, Go
- 容器插件: Docker容器
- WASM插件: WebAssembly模块

### 4. 接口扩展

- Web界面: 基于HTTP API的Web UI
- 移动应用: 通过MCP协议集成
- IDE插件: VS Code, IntelliJ等

## 性能考虑

### 1. 内存管理

- 使用Arc<T>进行共享所有权
- 避免不必要的数据克隆
- 合理的缓存策略

### 2. 并发控制

- 使用DashMap进行并发安全的哈希表操作
- parking_lot提供高性能锁机制
- 异步任务池管理

### 3. I/O优化

- 批量操作减少I/O次数
- 异步I/O避免阻塞
- 连接池管理

## 安全考虑

### 1. 插件沙箱

- WASM插件天然沙箱隔离
- Docker容器资源限制
- 脚本插件进程隔离

### 2. 身份验证

- JWT令牌认证
- 基于角色的访问控制
- API密钥管理

### 3. 数据保护

- 敏感数据加密存储
- 安全的配置管理
- 审计日志记录

## 测试策略

### 1. 单元测试

- 每个模块的核心功能测试
- 错误条件和边界情况测试
- 模拟依赖进行隔离测试

### 2. 属性测试

- 使用proptest进行随机化测试
- 验证系统不变量和属性
- 配置解析往返一致性测试

### 3. 集成测试

- 端到端工作流执行测试
- 多接口协同工作测试
- 插件系统集成测试

### 4. 性能测试

- 并发执行性能基准
- 内存使用情况监控
- 响应时间测量

## 部署和运维

### 1. 构建和打包

- Cargo构建系统
- 多平台交叉编译
- Docker容器化部署

### 2. 配置管理

- 环境特定配置文件
- 配置热重载支持
- 配置验证和迁移

### 3. 监控和日志

- Prometheus指标导出
- 结构化日志输出
- 健康检查接口

### 4. 备份和恢复

- 数据库备份策略
- 配置文件版本控制
- 灾难恢复计划

## 未来扩展

### 1. 分布式执行

- 多节点工作流执行
- 负载均衡和故障转移
- 分布式状态管理

### 2. 可视化界面

- 工作流图形化编辑器
- 实时执行监控面板
- 性能分析仪表板

### 3. 机器学习集成

- 智能工作流优化
- 异常检测和预警
- 自动化运维建议

### 4. 云原生支持

- Kubernetes集成
- 服务网格支持
- 云存储和计算服务集成

这个设计文档将随着项目的发展不断更新和完善，确保与实际实现保持一致。