# 应用层 (Application Layer)

## 1. 核心定义 (Stable)

### 1.1 层职责

应用层负责协调领域层完成具体的业务用例，处理事务边界、安全检查和跨领域逻辑。该层依赖领域层，通过领域端口与基础设施层交互。

### 1.2 模块结构

```
application/
├── port/               # 应用层端口（对外暴露的接口）
│   ├── mod.rs
│   ├── workflow_engine.rs   # 工作流引擎接口（从领域层移入）
│   ├── usecase_executor.rs  # 用例执行器接口
│   └── unit_of_work.rs      # 事务管理接口
├── service/            # 应用服务（协调领域对象）
│   ├── mod.rs
│   ├── workflow_service.rs  # 工作流应用服务
│   ├── plugin_service.rs    # 插件应用服务
│   └── system_service.rs    # 系统应用服务
├── usecase/            # 用例实现（业务流程编排）
│   ├── mod.rs
│   ├── execute_workflow.rs  # 执行工作流用例
│   ├── manage_workflows.rs  # 管理工作流用例
│   ├── manage_plugins.rs    # 管理插件用例
│   └── system_monitoring.rs # 系统监控用例
└── dto/                # 数据传输对象
    ├── mod.rs
    ├── workflow_dto.rs      # 工作流相关 DTO
    └── execution_dto.rs     # 执行相关 DTO
```

### 1.3 应用服务

应用服务负责协调领域对象完成用例，处理事务边界和跨领域逻辑。

#### WorkflowService

```rust
/// 工作流应用服务
/// 
/// 职责：
/// - 协调工作流相关的应用逻辑
/// - 管理事务边界
/// - 调用领域服务进行验证
/// - 发布领域事件
pub struct WorkflowService {
    workflow_repository: Arc<dyn WorkflowRepository>,
    execution_repository: Arc<dyn ExecutionRepository>,
    workflow_engine: Arc<dyn WorkflowEngine>,
    workflow_validator: Arc<dyn WorkflowValidator>,
    event_bus: Arc<dyn DomainEventBus>,
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl WorkflowService {
    /// 执行工作流
    /// 
    /// 事务边界：整个执行过程
    pub async fn execute_workflow(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowExecution> {
        // 1. 在事务中执行
        self.unit_of_work.execute(|| async {
            // 2. 验证工作流
            self.workflow_validator.validate(&definition)?;
            
            // 3. 保存工作流定义
            self.workflow_repository.save(&definition).await?;
            
            // 4. 启动执行
            let context = ExecutionContext::new(params);
            let execution = self.workflow_engine
                .start(definition, context)
                .await?;
            
            // 5. 保存执行记录
            self.execution_repository.save(&execution).await?;
            
            // 6. 发布事件
            self.event_bus.publish(ExecutionStarted {
                execution_id: execution.id.clone(),
                workflow_id: execution.workflow_id,
                started_at: Utc::now(),
                context: execution.context.clone(),
            }).await?;
            
            Ok(execution)
        }).await
    }
    
    /// 获取执行状态
    pub async fn get_execution_status(
        &self,
        execution_id: WorkflowId,
    ) -> Result<ExecutionStatus> {
        // 查询操作不需要事务
        let execution = self.execution_repository
            .find_by_id(&execution_id)
            .await?
            .ok_or_else(|| Error::ExecutionNotFound)?;
            
        Ok(execution.status)
    }
}
```

#### PluginService

```rust
/// 插件应用服务
/// 
/// 负责协调插件的生命周期管理
pub struct PluginService {
    plugin_manager: Arc<dyn PluginManager>,
    event_bus: Arc<dyn DomainEventBus>,
    unit_of_work: Arc<dyn UnitOfWork>,
}

impl PluginService {
    pub async fn load_plugin(&self, plugin: PluginInfo) -> Result<()> {
        self.unit_of_work.execute(|| async {
            self.plugin_manager.load(plugin.clone()).await?;
            
            self.event_bus.publish(PluginLoaded {
                plugin_name: plugin.name.clone(),
                version: plugin.version.clone(),
                loaded_at: Utc::now(),
            }).await?;
            
            Ok(())
        }).await
    }
}
```

### 1.4 用例实现

#### ExecuteWorkflowUseCase

```rust
/// 执行工作流用例
/// 
/// 负责完整的"执行工作流"业务流程
pub struct ExecuteWorkflowUseCase {
    workflow_service: Arc<WorkflowService>,
}

impl ExecuteWorkflowUseCase {
    pub async fn execute(
        &self,
        definition: WorkflowDefinition,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowExecution>;
}
```

#### ManageWorkflowsUseCase

```rust
/// 管理工作流用例
/// 
/// 负责工作流的生命周期管理
pub struct ManageWorkflowsUseCase;
```

#### ManagePluginsUseCase

```rust
/// 管理插件用例
/// 
/// 负责插件的生命周期管理
pub struct ManagePluginsUseCase;
```

#### SystemMonitoringUseCase

```rust
/// 系统监控用例
/// 
/// 负责系统健康检查和监控
pub struct SystemMonitoringUseCase;

impl SystemMonitoringUseCase {
    pub async fn health_check(&self) -> Result<HealthStatus>;
}
```

### 1.5 端口定义

#### 用例执行器端口

```rust
/// 用例执行器端口
/// 
/// 接口层通过此端口调用应用层用例
pub trait UseCaseExecutor: Send + Sync {
    fn execute(
        &self,
        use_case: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}
```

#### 工作流引擎端口

```rust
/// 工作流引擎端口（应用层核心）
/// 
/// 负责工作流执行的完整生命周期管理
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 启动工作流执行
    async fn start(
        &self,
        workflow: WorkflowDefinition,
        context: ExecutionContext,
    ) -> Result<WorkflowExecution>;
    
    /// 暂停执行
    async fn pause(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 恢复执行
    async fn resume(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 停止执行
    async fn stop(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 获取执行状态
    async fn get_status(&self, execution_id: WorkflowId) -> Result<ExecutionStatus>;
}
```

> **注意**：WorkflowEngine 已取代 WorkflowOrchestrator，职责由 WorkflowService 承担编排逻辑。

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-A001: 用例模式选择
- **决策**: 每个用例封装为独立结构体
- **理由**: 清晰的边界，便于测试和复用
- **风险**: 类数量增加

#### ADR-A002: 服务与用例分离
- **决策**: Service 负责协调，UseCase 负责业务流程
- **理由**: 单一职责，Service 可被多个 UseCase 复用
- **风险**: 层级增加，需要清晰的职责划分

### 2.2 任务清单

- [x] Task 0: 应用服务基础结构
- [x] Task 1: 核心用例定义
- [ ] Task 2: 事务管理实现
- [ ] Task 3: 权限检查集成
- [ ] Task 4: 用例编排优化

### 2.3 接口契约

#### 事务管理

```rust
/// 工作单元（事务边界）
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// 开始事务
    async fn begin(&self) -> Result<Transaction>;
    
    /// 提交事务
    async fn commit(&self, tx: Transaction) -> Result<()>;
    
    /// 回滚事务
    async fn rollback(&self, tx: Transaction) -> Result<()>;
    
    /// 在事务中执行操作
    async fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Future<Output = Result<T>> + Send;
}

/// 事务对象
pub struct Transaction {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
}
```

#### 工作流引擎端口（从领域层移动）

```rust
/// 工作流引擎接口
/// 
/// 协调工作流的完整生命周期，属于应用层职责
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 启动执行
    async fn start(
        &self,
        workflow: WorkflowDefinition,
        context: ExecutionContext,
    ) -> Result<WorkflowExecution>;
    
    /// 暂停执行
    async fn pause(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 恢复执行
    async fn resume(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 停止执行
    async fn stop(&self, execution_id: WorkflowId) -> Result<()>;
    
    /// 获取执行状态
    async fn get_status(&self, execution_id: WorkflowId) -> Result<ExecutionStatus>;
}
```

#### 用例执行契约

```rust
/// 用例执行结果
pub struct UseCaseResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub transaction_id: Option<Uuid>,
}

/// 用例上下文
pub struct UseCaseContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub transaction: Option<Transaction>,
}

/// 可复用的用例 trait
#[async_trait]
pub trait UseCase<Input, Output>: Send + Sync {
    async fn execute(&self, ctx: UseCaseContext, input: Input) -> Result<UseCaseResult<Output>>;
}
```

### 2.4 测试策略

- **集成测试**: 用例完整流程测试
- **Mock 测试**: 使用 Mock 领域服务
- **边界测试**: 事务回滚、异常处理

## 3. 状态记录

- `[进行中]` | 事务管理实现 | 2026-02-06
- `[已完成]` | 基础用例结构 | 2026-01-25
- `[已完成]` | 应用服务框架 | 2026-01-22

## 4. 依赖关系

```
application/
├── 依赖: domain (领域层)
│   ├── model::WorkflowDefinition
│   ├── model::WorkflowExecution
│   ├── event::DomainEvent
│   ├── event::DomainEventBus
│   ├── service::WorkflowValidator
│   ├── service::ExecutionStateCalculator
│   ├── port::WorkflowRepository
│   ├── port::ExecutionRepository
│   ├── port::ToolRegistry
│   └── port::PluginManager
├── 依赖: infrastructure (通过端口)
│   └── port::UnitOfWork (事务管理)
└── 被依赖: interfaces (接口层)
    └── 通过 UseCaseExecutor / WorkflowEngine 端口
```
