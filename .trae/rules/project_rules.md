# Workflow Toolkit 项目规范 (v2.3)

> **项目**: Workflow Toolkit  
> **版本**: v2.3  
> **更新**: 2026-02-05

---

## 引用说明

> 通用 AI 工作流规则参见 `user_rules.md`，包含：
> - 角色矩阵、任务分诊、标准作业程序 (SOP)
> - 文档规范、三错即停机制、交互规范

---

## 1. 领域驱动设计规范 (DDD)

### 1.1 分层架构

```
┌─────────────────────────────────────────────┐
│  接入层 (Interface Layer)                    │
│  - CLI: clap 派生宏实现                       │
│  - TUI: ratatui 异步事件循环                  │
│  - MCP: rmcp 协议实现                        │
├─────────────────────────────────────────────┤
│  应用层 (Application Layer)                  │
│  - UseCase: 业务流程编排                      │
│  - Service: 领域服务封装                      │
│  - DTO: 数据传输对象                          │
├─────────────────────────────────────────────┤
│  领域层 (Domain Layer)                       │
│  - Entity: 实体（有唯一标识）                 │
│  - Value Object: 值对象（不可变）             │
│  - Aggregate: 聚合根                          │
│  - Domain Event: 领域事件                     │
│  - Port: 仓库接口、服务接口                   │
├─────────────────────────────────────────────┤
│  基础设施层 (Infrastructure Layer)           │
│  - Repository: 仓库实现                       │
│  - Plugin: 插件系统实现                       │
│  - External: 外部服务客户端                   │
│  - Persistence: 存储实现                      │
├─────────────────────────────────────────────┤
│  适配器层 (Adapter Layer)                    │
│  - CLI Adapter: 命令行适配器                  │
│  - TUI Adapter: 终端界面适配器                │
│  - MCP Adapter: MCP 协议适配器                │
└─────────────────────────────────────────────┘
```

### 1.2 领域建模规范

#### 实体 (Entity)

```rust
/// 实体特征：具有唯一标识，状态可变
/// 命名规范：名词，CamelCase
/// 必须实现：PartialEq 基于 id 比较
pub struct Workflow {
    pub id: WorkflowId,           // 唯一标识
    pub name: String,             // 实体属性
    pub status: WorkflowStatus,   // 实体状态
    pub created_at: DateTime<Utc>,
}

impl Workflow {
    /// 业务方法：封装领域逻辑
    pub fn activate(&mut self) -> Result<()> {
        if self.status != WorkflowStatus::Draft {
            return Err(DomainError::InvalidStateTransition);
        }
        self.status = WorkflowStatus::Active;
        Ok(())
    }
}
```

**约束**:
- DDD-E001: 实体必须有唯一标识符 (id)
- DDD-E002: 实体相等性基于 id，而非属性
- DDD-E003: 实体方法封装业务逻辑，禁止贫血模型

#### 值对象 (Value Object)

```rust
/// 值对象特征：无标识，不可变，可替换
/// 命名规范：名词，CamelCase
/// 必须实现：Copy/Clone, PartialEq（基于所有属性）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

impl Money {
    /// 业务方法：返回新值对象，不修改自身
    pub fn add(&self, other: Money) -> Result<Money> {
        if self.currency != other.currency {
            return Err(DomainError::CurrencyMismatch);
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}
```

**约束**:
- DDD-V001: 值对象必须是不可变的
- DDD-V002: 值对象相等性基于所有属性
- DDD-V003: 值对象方法返回新实例，不修改自身

#### 聚合根 (Aggregate Root)

```rust
/// 聚合根特征：实体边界，事务一致性单元
/// 命名规范：名词，CamelCase，对应实体名称
/// 职责：维护聚合内所有实体和值对象的一致性
pub struct WorkflowAggregate {
    pub root: Workflow,                    // 聚合根实体
    pub nodes: Vec<WorkflowNode>,          // 聚合内实体
    pub edges: Vec<WorkflowEdge>,          // 聚合内实体
    pub version: AggregateVersion,         // 乐观锁版本
}

impl WorkflowAggregate {
    /// 聚合根方法：维护聚合一致性
    pub fn add_node(&mut self, node: WorkflowNode) -> Result<()> {
        // 验证节点 ID 唯一性
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(DomainError::DuplicateNodeId);
        }
        
        self.nodes.push(node);
        self.increment_version();
        Ok(())
    }
    
    /// 发布领域事件
    pub fn apply(&mut self, event: DomainEvent) {
        // 应用事件到聚合状态
        // 记录事件到待发布列表
    }
}
```

**约束**:
- DDD-A001: 聚合根是聚合的唯一入口点
- DDD-A002: 聚合内实体只能通过聚合根访问
- DDD-A003: 聚合根负责维护聚合内所有对象的一致性
- DDD-A004: 聚合根必须包含乐观锁版本号

#### 领域事件 (Domain Event)

```rust
/// 领域事件特征：记录领域状态变化，不可变
/// 命名规范：动词过去式 + Event，CamelCase
/// 必须包含：事件 ID、时间戳、聚合根 ID
#[derive(Clone, Debug)]
pub struct WorkflowCreatedEvent {
    pub event_id: EventId,
    pub aggregate_id: WorkflowId,
    pub occurred_at: DateTime<Utc>,
    pub name: String,
    pub created_by: UserId,
}

impl DomainEvent for WorkflowCreatedEvent {
    fn event_type(&self) -> &'static str {
        "workflow.created"
    }
    
    fn aggregate_id(&self) -> &str {
        &self.aggregate_id.0
    }
}
```

**约束**:
- DDD-DE001: 领域事件必须是不可变的
- DDD-DE002: 领域事件必须包含事件 ID、时间戳、聚合根 ID
- DDD-DE003: 领域事件命名使用动词过去式

#### 仓库接口 (Repository Port)

```rust
/// 仓库接口特征：定义在领域层，实现在基础设施层
/// 命名规范：实体名 + Repository，CamelCase
/// 职责：聚合根的持久化，不暴露存储细节
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    /// 通过 ID 查找聚合根
    async fn find_by_id(&self, id: &WorkflowId) -> Result<Option<WorkflowAggregate>>;
    
    /// 保存聚合根（包含乐观锁检查）
    async fn save(&self, aggregate: &mut WorkflowAggregate) -> Result<()>;
    
    /// 删除聚合根
    async fn delete(&self, id: &WorkflowId) -> Result<()>;
}
```

**约束**:
- DDD-R001: 仓库接口定义在领域层 (domain/port/)
- DDD-R002: 仓库实现定义在基础设施层 (infrastructure/)
- DDD-R003: 仓库只操作聚合根，不直接操作聚合内实体
- DDD-R004: 仓库方法必须包含乐观锁检查

### 1.3 模块依赖方向

```rust
// 允许: 上层依赖下层
src/interfaces/cli/ -> src/application/ -> src/domain/

// 禁止: 下层依赖上层
src/domain/ -> X -> src/application/

// 禁止: 同层跨模块直接依赖
src/plugins/ -> X -> src/workflow/ (必须通过 domain/port)

// 禁止: 领域层依赖基础设施
src/domain/ -> X -> src/infrastructure/
```

### 1.4 DDD 审查清单

#### 实体审查
- [ ] 是否有唯一标识符 (DDD-E001)
- [ ] 相等性是否基于 id (DDD-E002)
- [ ] 是否包含业务方法而非仅数据 (DDD-E003)

#### 值对象审查
- [ ] 是否是不可变的 (DDD-V001)
- [ ] 相等性是否基于所有属性 (DDD-V002)
- [ ] 方法是否返回新实例 (DDD-V003)

#### 聚合根审查
- [ ] 是否是聚合唯一入口 (DDD-A001)
- [ ] 是否维护聚合一致性 (DDD-A003)
- [ ] 是否包含乐观锁版本 (DDD-A004)

#### 领域事件审查
- [ ] 是否包含事件 ID、时间戳、聚合根 ID (DDD-DE002)
- [ ] 命名是否使用动词过去式 (DDD-DE003)

---

## 2. 软件约束规范 (SPEC)

### 2.1 架构约束 (Architecture Constraints)

**约束规则**:
- AC-001: 严禁跨层调用，必须通过公开接口
- AC-002: 领域层不得依赖其他层
- AC-003: 基础设施层通过 Port 接口与领域层交互

### 2.2 接口契约 (Interface Contracts)

#### 工作流引擎接口

```rust
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute(
        &self,
        definition: WorkflowDefinition,
        initial_params: HashMap<String, Value>,
    ) -> Result<WorkflowExecution>;
}

/// 约束:
/// - CE-001: execute 必须是幂等的
/// - CE-002: 并发执行数不得超过 max_concurrent_workflows
/// - CE-003: 执行超时后必须返回 Timeout 状态
```

#### 工具接口

```rust
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}

/// 约束:
/// - CT-001: execute 必须是线程安全的 (Send + Sync)
/// - CT-002: 执行时间不得超过 timeout_secs
/// - CT-003: 资源使用不得超过 ResourceRequirements 定义
```

### 2.3 数据模型约束 (Data Model Constraints)

```rust
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

/// 约束:
/// - CD-001: id 必须符合 ^[a-zA-Z][a-zA-Z0-9_-]*$ 模式
/// - CD-002: nodes 不能为空
/// - CD-003: edges 不能形成环 (DAG 约束)
/// - CD-004: 所有 edge.source 和 edge.target 必须对应存在的 node.id
```

### 2.4 行为约束 (Behavioral Constraints)

```rust
/// 约束:
/// - CP-001: 并行任务必须使用 tokio::spawn
/// - CP-002: 默认最大并发数: 4
/// - CP-003: 任一失败应取消其他任务 (可配置)
```

### 2.5 性能约束 (Performance Constraints)

| 操作 | 目标 | 最大容忍 |
|------|------|----------|
| 工作流提交 | < 10ms | 100ms |
| 节点调度 | < 1ms | 10ms |
| 工具执行 | 取决于工具 | 可配置 timeout |
| 状态查询 | < 5ms | 50ms |

### 2.6 错误处理约束 (Error Handling Constraints)

```rust
pub enum WorkflowError {
    Config(config::ConfigError),
    WorkflowValidation { message: String },
    WorkflowExecution { message: String },
    ExecutionCancelled,
    ExecutionTimeout,
    NotFound { resource: String },
    ResourceExhausted,
    ConcurrentAccess { message: String },
    Generic(anyhow::Error),
}

/// 约束:
/// - CEH-001: 严禁使用 unwrap/expect (除测试代码外)
/// - CEH-002: 所有错误必须包含上下文信息
/// - CEH-003: 用户-facing 错误消息必须是中文
```

### 2.7 安全约束 (Security Constraints)

```rust
/// 约束:
/// - CSI-001: 所有外部输入必须验证
/// - CSI-002: 字符串输入必须限制长度 (默认 10KB)
/// - CSI-003: 文件路径必须规范化 (防止目录遍历)
/// - CSP-001: 插件必须在沙箱中执行
/// - CSP-002: 插件资源使用必须受限 (CPU/内存)
/// - CSP-003: 插件网络访问默认关闭
```

### 2.8 可观测性约束 (Observability Constraints)

```rust
/// 约束:
/// - COL-001: 使用 tracing crate 进行结构化日志
/// - COL-002: 日志级别: ERROR > WARN > INFO > DEBUG > TRACE
/// - COL-003: 关键操作必须记录: workflow_id, node_id, duration
```

### 2.9 约束验证清单

#### 代码审查清单
- [ ] 是否违反分层架构 (AC-001, AC-002, AC-003)
- [ ] 是否违反 DDD 规范 (DDD-E001~DDD-A004)
- [ ] 是否存在 unwrap/expect (CEH-001)
- [ ] 错误消息是否为中文 (CEH-003)
- [ ] 公共 API 是否有文档
- [ ] 是否包含必要的 tracing 日志

#### 性能审查清单
- [ ] 是否使用 tokio::spawn 实现并行 (CP-001)
- [ ] 是否有背压机制 (CPC-001)

#### 安全审查清单
- [ ] 外部输入是否验证 (CSI-001)
- [ ] 文件路径是否规范化 (CSI-003)
- [ ] 插件是否沙箱化 (CSP-001)

---

## 3. 变更记录

### v2.3 (2026-02-05)
- 新增 DDD 领域驱动设计规范
- 分离通用规则到 user_rules.md
- 精简为项目专属内容

### v2.2 (2026-02-05)
- 整合 SPEC.md 软件约束规范
- 添加渐进式披露工作流

### v2.1 (2026-02-04)
- 初始版本
