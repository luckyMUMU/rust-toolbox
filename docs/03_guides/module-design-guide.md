# Workflow Toolkit 模块设计指南

## 文档元数据

- **版本**: v1.0.0
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 模块设计原则

### 1.1 分层架构原则

Workflow Toolkit 采用严格的分层架构，每层有明确的职责边界：

```
┌─────────────────────────────────────────────────────────────┐
│                      接口层 (Interfaces)                      │
│  职责: 处理用户输入输出，提供多种界面                          │
│  依赖: 应用层、领域层                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      应用层 (Application)                     │
│  职责: 协调领域层完成用例，处理事务边界                        │
│  依赖: 领域层                                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       领域层 (Domain)                         │
│  职责: 核心业务逻辑，领域模型                                  │
│  依赖: 无（纯净）                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   基础设施层 (Infrastructure)                 │
│  职责: 技术实现，持久化，外部服务                              │
│  依赖: 领域层（实现接口）                                      │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 模块职责原则

1. **单一职责**: 每个模块只负责一个业务领域
2. **高内聚**: 模块内部的组件紧密相关
3. **低耦合**: 模块之间通过明确的接口通信
4. **依赖倒置**: 高层模块定义接口，低层模块实现

## 2. 领域层模块设计

### 2.1 目录结构

```
src/domain/
├── mod.rs              # 模块导出
├── model/              # 领域模型
│   ├── mod.rs
│   ├── execution.rs    # 执行模型
│   ├── workflow.rs     # 工作流模型
│   ├── plugin.rs       # 插件模型
│   ├── tool.rs         # 工具模型
│   └── value_object.rs # 值对象
├── port/               # 领域端口（接口）
│   ├── mod.rs
│   ├── plugin_manager.rs
│   ├── repository.rs
│   └── tool_registry.rs
├── service/            # 领域服务
│   ├── mod.rs
│   ├── execution_calculator.rs
│   └── workflow_validator.rs
└── event/              # 领域事件
    ├── mod.rs
    ├── events.rs
    └── bus.rs
```

### 2.2 设计规范

#### 聚合设计

```rust
// 聚合根
pub struct WorkflowDefinition {
    pub id: WorkflowId,
    pub name: String,
    pub nodes: Vec<WorkflowNode>,    // 实体
    pub edges: Vec<WorkflowEdge>,    // 实体
    pub config: WorkflowConfig,      // 值对象
}

// 实体
pub struct WorkflowNode {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub config: NodeConfig,
}

// 值对象
pub struct WorkflowConfig {
    pub concurrency: ConcurrencyConfig,
    pub retry: RetryPolicy,
    pub timeout: Option<Duration>,
}
```

#### 端口定义

```rust
// 领域端口（接口）
pub trait ToolRegistry: Send + Sync {
    fn register(&self, tool: Tool) -> Result<()>;
    fn get(&self, name: &str) -> Result<Tool>;
    fn execute(&self, tool_id: &ToolId, input: ToolInput) -> Result<ToolOutput>;
}
```

#### 领域服务

```rust
// 领域服务
pub struct ExecutionStateCalculator;

impl ExecutionStateCalculator {
    pub fn calculate_progress(&self, execution: &WorkflowExecution) -> f32 {
        // 业务逻辑
    }
}
```

### 2.3 约束规则

1. **禁止依赖外部框架**: 领域层不能依赖 Tokio、Serde 等外部框架
2. **禁止 I/O 操作**: 领域层不能进行文件、网络等 I/O 操作
3. **使用领域事件**: 状态变更通过领域事件发布

## 3. 应用层模块设计

### 3.1 目录结构

```
src/application/
├── mod.rs              # 模块导出
├── service/            # 应用服务
│   ├── mod.rs
│   └── workflow_service.rs
├── workflow/           # 工作流编排
│   ├── mod.rs
│   └── orchestrator.rs
├── port/               # 应用层端口
│   ├── mod.rs
│   └── unit_of_work.rs
└── usecase/            # 用例
    └── mod.rs
```

### 3.2 设计规范

#### 应用服务

```rust
pub struct WorkflowService {
    workflow_repository: Arc<dyn WorkflowRepository>,
    execution_repository: Arc<dyn ExecutionRepository>,
    engine: Arc<dyn WorkflowEngine>,
}

impl WorkflowService {
    pub async fn execute_workflow(
        &self,
        workflow_id: &WorkflowId,
        input: WorkflowInput,
    ) -> Result<ExecutionId> {
        // 1. 加载工作流定义
        let workflow = self.workflow_repository.load(workflow_id).await?;
        
        // 2. 验证工作流
        workflow.validate()?;
        
        // 3. 执行工作流
        let execution = self.engine.execute(&workflow, input).await?;
        
        // 4. 保存执行状态
        self.execution_repository.save(&execution).await?;
        
        Ok(execution.id)
    }
}
```

### 3.3 约束规则

1. **薄应用层**: 应用层只做协调，不包含业务逻辑
2. **事务边界**: 应用层负责定义事务边界
3. **依赖注入**: 通过构造函数注入依赖

## 4. 基础设施层模块设计

### 4.1 目录结构

```
src/infrastructure/
├── mod.rs              # 模块导出
├── persistence/        # 持久化
│   ├── mod.rs
│   ├── repository/     # 仓储实现
│   └── storage/        # 存储后端
├── plugin/             # 插件基础设施
│   └── mod.rs
├── config/             # 配置管理
│   └── mod.rs
└── external/           # 外部服务
    └── mod.rs
```

### 4.2 设计规范

#### 仓储实现

```rust
pub struct FileWorkflowRepository {
    data_dir: PathBuf,
}

#[async_trait]
impl WorkflowRepository for FileWorkflowRepository {
    async fn save(&self, workflow: &WorkflowDefinition) -> Result<()> {
        let path = self.data_dir.join(format!("{}.yaml", workflow.id));
        let content = serde_yaml::to_string(workflow)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
    
    async fn load(&self, id: &WorkflowId) -> Result<Option<WorkflowDefinition>> {
        let path = self.data_dir.join(format!("{}.yaml", id));
        if !path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(&path).await?;
        let workflow = serde_yaml::from_str(&content)?;
        Ok(Some(workflow))
    }
}
```

### 4.3 约束规则

1. **实现领域接口**: 基础设施层实现领域层定义的接口
2. **技术细节封装**: 技术实现细节不应暴露到上层
3. **可替换性**: 基础设施实现可以替换

## 5. 接口层模块设计

### 5.1 目录结构

```
src/interfaces/
├── mod.rs              # 模块导出
├── cli/                # 命令行界面
│   ├── mod.rs
│   ├── app.rs
│   ├── commands.rs
│   ├── output.rs
│   └── error.rs
├── tui/                # 终端用户界面
│   ├── mod.rs
│   ├── app.rs
│   ├── widget.rs
│   └── widgets/
├── mcp/                # MCP 协议
│   ├── mod.rs
│   ├── mcp.rs
│   └── mcp_server.rs
└── dto/                # 数据传输对象
    └── mod.rs
```

### 5.2 设计规范

#### CLI 命令

```rust
pub enum Command {
    Workflow(WorkflowCommand),
    Tool(ToolCommand),
    Plugin(PluginCommand),
}

pub enum WorkflowCommand {
    Execute { workflow: String, input: Option<String> },
    List { filter: Option<String> },
    Validate { workflow: String },
    Status { execution_id: String },
}

pub struct CliApp {
    workflow_service: Arc<WorkflowService>,
    tool_registry: Arc<ToolRegistry>,
    plugin_manager: Arc<PluginManager>,
}

impl CliApp {
    pub async fn execute(&self, command: Command) -> Result<()> {
        match command {
            Command::Workflow(cmd) => self.execute_workflow_command(cmd).await,
            Command::Tool(cmd) => self.execute_tool_command(cmd).await,
            Command::Plugin(cmd) => self.execute_plugin_command(cmd).await,
        }
    }
}
```

### 5.3 约束规则

1. **只依赖应用层**: 接口层只依赖应用层和领域层
2. **数据转换**: 接口层负责 DTO 和领域模型的转换
3. **错误处理**: 接口层负责将领域错误转换为用户友好的消息

## 6. 核心模块设计

### 6.1 工作流模块

```
src/workflow/
├── mod.rs              # 模块导出
├── engine.rs           # 工作流引擎
├── definition.rs       # 工作流定义
├── execution.rs        # 执行状态
├── component/          # 组件系统
│   ├── mod.rs
│   ├── registry.rs
│   ├── tool.rs
│   └── parallel.rs
├── executor/           # 执行器链
│   ├── mod.rs
│   ├── basic.rs
│   ├── retry.rs
│   ├── cache.rs
│   └── audit.rs
├── context/            # 上下文系统
│   ├── mod.rs
│   └── slot.rs
└── state/              # 状态管理
    ├── mod.rs
    ├── checkpoint.rs
    └── recovery.rs
```

### 6.2 插件模块

```
src/plugins/
├── mod.rs              # 模块导出
├── manager.rs          # 插件管理器
├── types.rs            # 插件类型定义
├── runtime.rs          # 运行时管理
├── process_pool.rs     # 进程池
├── native.rs           # Native 插件
├── python.rs           # Python 插件
├── nodejs.rs           # Node.js 插件
├── docker.rs           # Docker 插件
├── wasm.rs             # WASM 插件
├── wasm_sandbox.rs     # WASM 沙箱
└── file_management/    # 文件管理插件
    ├── mod.rs
    ├── plugin.rs
    └── ...
```

### 6.3 工具模块

```
src/tools/
├── mod.rs              # 模块导出
├── types.rs            # 工具类型定义
├── registry.rs         # 工具注册表
├── executor.rs         # 工具执行器
├── middleware.rs       # 中间件系统
├── composable.rs       # 可组合工具
├── composition.rs      # 组合执行
├── schema_validator.rs # Schema 验证
├── template.rs         # 模板引擎
├── version.rs          # 版本管理
└── external/           # 外部工具执行器
    ├── mod.rs
    ├── python.rs
    ├── nodejs.rs
    └── docker.rs
```

## 7. 模块间通信

### 7.1 同步通信

```rust
// 通过接口调用
let tool = tool_registry.get("text_processor")?;
let result = tool_registry.execute(&tool.id, input).await?;
```

### 7.2 异步通信（事件）

```rust
// 发布领域事件
event_bus.publish(WorkflowEvent::WorkflowStarted { 
    execution_id: execution.id.clone() 
}).await?;

// 订阅领域事件
event_bus.subscribe(|event: WorkflowEvent| {
    match event {
        WorkflowEvent::WorkflowStarted { execution_id } => {
            // 处理事件
        }
        _ => {}
    }
}).await?;
```

## 8. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
