# src/workflow/ - DAG Workflow Engine

## OVERVIEW
Core workflow execution with DAG-based scheduling, state management, and audit logging.

## EXECUTION FLOW
1. **Definition** → `WorkflowDefinition` with nodes and edges
2. **Validation** → `WorkflowValidator` checks for cycles, missing tools
3. **Scheduling** → `DagScheduler` determines execution order
4. **Execution** → `DefaultWorkflowEngine` runs with parallel semaphore
5. **State** → `WorkflowState` tracked in `StateManager`
6. **Audit** → `AuditLogger` records all operations
7. **Cache** → `ResultCache` stores intermediate results

## KEY COMPONENTS
| Component | File | Purpose |
|-----------|------|---------|
| `WorkflowEngine` | engine.rs | Trait for execution engines |
| `DefaultWorkflowEngine` | engine.rs | Main implementation with error recovery |
| `RefactoredWorkflowEngine` | engine_v2.rs | Refactored engine with parallel execution |
| `DagScheduler` | scheduler.rs | Topological sort + execution stats |
| `WorkflowValidator` | validator.rs | Validation logic |
| `ExecutionManager` | execution_manager.rs | Parallel execution control |
| `AuditLogger` | audit.rs | Compliance tracking |
| `ResultCache` | result_cache.rs | TTL-based caching |
| `WorkflowDefinition` | definition.rs | DAG structure and node definitions |
| `ExecutionState` | execution.rs | Runtime state tracking |
| **Subdirectories** | | |
| `executor/` | 5 files | Execution strategies (basic, parallel, retry) |
| `component/` | 4 files | Workflow components (parallel, switch, loop) |
| `context/` | 2 files | Execution context and data slots |
| `state/` | 2 files | State management and checkpoints |

## SCHEDULER
- Uses `petgraph` for DAG operations
- Topological sort for execution order
- Tracks node dependencies and parallelism
- Provides execution statistics
- Critical path analysis

## ERROR RECOVERY
- **StopWorkflow**: Halt entire workflow
- **FailNode**: Mark node as failed, continue if possible
- **ContinueWithoutNode**: Skip failed node
- **PauseAndRetry**: Manual intervention required
- **RetryPolicy**: Configurable retry with exponential backoff

## STATE MANAGEMENT
- **Checkpoint**: Every 5 minutes (configurable)
- **Persistence**: Via `StateManager` (FileStorage or LanceDB)
- **Recovery**: Resume from last checkpoint on restart
- **WorkflowState**: Tracks execution status, node states, results

## AUDIT LOGGING
- All execution events logged
- Error details with stack traces
- Queryable audit reports
- Severity levels: Info, Warning, Error, Critical
- Compliance tracking for regulated environments

## WHERE TO LOOK
| Task | Location |
|------|----------|
| Define workflow structure | `definition.rs` |
| Validate workflows | `validator.rs` |
| Schedule execution | `scheduler.rs` |
| Execute workflows | `engine.rs` |
| Refactored engine | `engine_v2.rs` |
| Manage parallel execution | `execution_manager.rs` |
| Track state | `execution.rs` |
| Log operations | `audit.rs` |
| Cache results | `result_cache.rs` |
| Execution strategies | `executor/` |
| Workflow components | `component/` |
| Execution context | `context/` |
| State management | `state/` |

## SUBDIRECTORIES

### executor/
**Path**: `src/workflow/executor/` (5 files)  
**Purpose**: Execution strategies for workflow nodes

**Files**:
- `basic.rs`: Basic sequential execution
- `parallel.rs`: Parallel execution with semaphore control
- `retry.rs`: Retry logic with exponential backoff
- `cache.rs`: Result caching for idempotent operations
- `audit.rs`: Audit logging for compliance

**Execution Strategies**:
- **Sequential**: Execute nodes one by one
- **Parallel**: Execute multiple nodes concurrently
- **Retry**: Automatic retry on failure
- **Cached**: Skip execution if result cached
- **Audited**: Log all execution events

### component/
**Path**: `src/workflow/component/` (4 files)  
**Purpose**: Workflow component implementations

**Files**:
- `mod.rs`: Component trait and types
- `parallel.rs`: Parallel execution component
- `switch.rs`: Conditional branching component
- `loop.rs`: Loop execution component

**Component Types**:
- `Tool`: Execute a tool
- `Condition`: Conditional branching
- `Loop`: Iterative execution
- `Parallel`: Concurrent execution
- `Switch`: Multi-way branching
- `Checkpoint`: State saving

**Component Status**:
- `Success`: Completed successfully
- `Failure(String)`: Failed with error
- `Skip`: Skipped (e.g., condition false)
- `Break`: Break out of loop
- `Continue`: Continue to next iteration

### context/
**Path**: `src/workflow/context/` (2 files)  
**Purpose**: Execution context and data slots

**Files**:
- `mod.rs`: DataContext implementation
- `slot.rs`: Type-safe data slots

**DataContext**:
- Global slots (shared across workflow)
- Node-local slots (private to component)
- Parent chain (for nested scopes)
- Current node tracking

**SlotValue**:
- Type-safe value wrapper
- JSON serialization support
- Type checking on extraction
- Default value support

**Slot Operations**:
- `set`: Store value
- `get`: Retrieve value with type
- `get_or_default`: Retrieve or use default
- `remove`: Remove value
- `contains`: Check existence

### state/
**Path**: `src/workflow/state/` (2 files)  
**Purpose**: State management and checkpoints

**Files**:
- `mod.rs`: ExecutionTracker and state types
- `checkpoint.rs`: Checkpoint management

**ExecutionTracker**:
- Tracks workflow execution state
- Node execution states
- Control signals (pause, stop)
- Execution statistics

**Checkpoint**:
- Periodic state snapshots
- Recovery from checkpoints
- Checkpoint metadata
- Sequence numbering

**Control Signals**:
- `should_pause`: Pause execution
- `should_stop`: Stop execution
- `pause_requested_at`: Timestamp
- `stop_requested_at`: Timestamp

## EXECUTION FLOW

### Refactored Engine (engine_v2.rs)
**Purpose**: True parallel execution with single responsibility

**Components**:
- **StateManager**: Persistence layer
- **ToolRegistry**: Tool management
- **Executor**: Component execution chain
- **AuditLogger**: Compliance tracking
- **ResultCache**: Optional caching
- **WorkflowSemaphore**: Concurrent workflow control

**Execution Chain**:
```
WorkflowDefinition
    ↓
WorkflowValidator
    ↓
DagScheduler
    ↓
ExecutionManager
    ↓
ComponentExecutor
    ↓
ToolExecutor
    ↓
ResultCache
    ↓
AuditLogger
    ↓
StateManager
```

### Parallel Execution
**Controlled by Semaphore**:
- Default: 4 concurrent workflows
- Configurable via `max_concurrent_workflows`
- Prevents resource exhaustion
- Backpressure handling

**Component Parallelism**:
- Parallel components execute nodes concurrently
- Wait strategies: WaitAll, WaitAny, WaitN
- Resource limits via semaphores
- Error isolation

## ERROR RECOVERY

### Recovery Strategies
1. **StopWorkflow**: Halt entire workflow
2. **FailNode**: Mark node as failed, continue if possible
3. **ContinueWithoutNode**: Skip failed node
4. **PauseAndRetry**: Manual intervention required
5. **RetryPolicy**: Configurable retry with exponential backoff

### Retry Configuration
```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Option<Duration>,
    pub backoff_multiplier: f64,
    pub strategy: RetryStrategy,
}

pub enum RetryStrategy {
    None,
    FixedInterval,
    ExponentialBackoff,
    LinearBackoff,
    Custom(Value),
}
```

## STATE MANAGEMENT

### Checkpoint System
**Interval**: Every 5 minutes (configurable)  
**Persistence**: Via StateManager (FileStorage or LanceDB)  
**Recovery**: Resume from last checkpoint on restart

**Checkpoint Data**:
- Workflow execution ID
- Node execution states
- Global context slots
- Timestamp and sequence number

### Workflow State
```rust
pub struct WorkflowState {
    pub execution: WorkflowExecution,
    pub checkpoints: Vec<Checkpoint>,
    pub metadata: HashMap<String, Value>,
}
```

## AUDIT LOGGING

### Audit Events
- **Workflow lifecycle**: Created, started, paused, completed, failed
- **Node execution**: Started, completed, failed, retried, skipped
- **System events**: Checkpoint created, state recovered
- **Security events**: Access granted/denied, authentication
- **Configuration events**: Changes, plugin loads

### Audit Severity
- `Debug`: Detailed debugging information
- `Info`: General information
- `Warning`: Potential issues
- `Error`: Errors that occurred
- `Critical`: Critical failures

### Audit Reports
- Queryable audit trails
- Compliance tracking
- Error analysis
- Performance metrics

## CACHING

### ResultCache
**Purpose**: Cache workflow execution results  
**TTL**: Configurable time-to-live  
**Invalidation Strategies**:
- Time-based (TTL)
- Version-based
- Manual invalidation
- Dependency-based

### Cache Key Generation
```rust
pub fn generate_cache_key(
    workflow_id: &str,
    parameters: &Value,
    context: &ExecutionContext,
) -> String {
    // Hash of workflow_id + parameters + context
}
```

## CONVENTIONS

### Node IDs
- Must be unique within workflow
- Should be descriptive
- Use kebab-case: `fetch-data`, `process-files`

### Edge Definitions
- Source → Target dependencies
- Defines execution order
- Enables parallel execution

### Parameter Passing
- Template syntax: `{{ node_id.result_field }}`
- Global variables: `{{ global.variable }}`
- Context variables: `{{ context.variable }}`

## PERFORMANCE

### Parallel Execution
- Default: 4 concurrent workflows
- Configurable via `max_concurrent_workflows`
- Component-level parallelism
- Semaphore-based resource limiting

### Memory Management
- DashMap for concurrent access
- Arc for shared ownership
- RwLock for mutable state
- Checkpointing for recovery

### Caching
- Moka cache for high performance
- TTL-based expiration
- LRU eviction
- Metrics tracking

## TESTING

### Unit Tests
- Component execution
- State management
- Checkpoint creation/restoration
- Error recovery scenarios

### Integration Tests
- Full workflow execution
- Parallel execution
- Error handling
- State persistence

### Property-Based Tests
- Template expansion
- Version resolution
- DAG validation

## SEE ALSO

- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Tools AGENTS.md](../tools/AGENTS.md) - Tool system
- [Storage AGENTS.md](../storage/AGENTS.md) - State persistence
- [Performance AGENTS.md](../performance/AGENTS.md) - Caching and metrics

## CONVENTIONS
- All nodes must have unique IDs
- Edges define dependencies (source → target)
- Parallel execution limited by semaphore (default: 4)
- Checkpoints save full execution state
- Audit events include workflow_id, node_id, timestamp

## IMPORTANT NOTES
- **Circular dependencies**: Detected and rejected during validation
- **Missing tools**: Validation fails if tool not in registry
- **Timeout**: Configurable per workflow or globally
- **Cancellation**: Graceful shutdown with cleanup
- **Performance**: Uses DashMap for concurrent state access

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **component/**: Contains 4 files (e.g., mod.rs, parallel.rs, registry.rs).
- **context/**: Contains 2 files (e.g., mod.rs, slot.rs).
- **executor/**: Contains 5 files (e.g., audit.rs, basic.rs, cache.rs).
- **state/**: Contains 2 files (e.g., checkpoint.rs, mod.rs).

<!-- AUTO-GENERATED-AGENT-MAP:END -->
 
### 模块级补充细则
- 目标与范围
  - 本模块作为 DAG 流程核心，覆盖工作流定义、验证、调度、执行、状态、审计等核心职责，确保对外接口稳定且易于测试。对外入口应给出使用约定与示例。
- 设计与扩展
  - 新执行策略/组件需给出设计评审点、向后兼容性分析、以及对现有任务流的影响评估。
- 实现规范
  - 导入排序：std -> external -> crate，分组后空一行。
  - 命名规范：函数/变量 snake_case；类型/枚举 CamelCase；常量 ALL_CAPS。
  - 公共接口需 Rustdoc 注释，包含参数、返回、错误信息、示例。
  - 错误处理以 thiserror 的错误类型为主，所有公共 API 返回 Result<T, WorkflowError>。
  - 异步模式统一使用 Tokio，避免阻塞，必要时使用异步工具/信号量。
- 测试策略
  - 覆盖核心路径、并发场景、错误分支、边界条件、以及恢复点测试。
  - 引入集成测试覆盖 DAG 的端到端执行、状态持久化与审计。
- 变更与审阅
  - 变更前提供设计动机、对现有 API 的影响、回归测试计划。
- 文档与审阅
  - 模块级 AGENTS.md 需随变更更新，新增子模块需补充模板。
- 跨模块协作
  - 与 plugins/tools 的对接契约应明确化，避免隐式耦合，变更需提前沟通。
- Cursor/Copilot 规则
  - 将 Cursor/Copilot 规则合并到模块级 AGENTS.md 模板中，便于执行。
