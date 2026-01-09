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
| `DagScheduler` | scheduler.rs | Topological sort + execution stats |
| `WorkflowValidator` | validator.rs | Validation logic |
| `ExecutionManager` | execution_manager.rs | Parallel execution control |
| `AuditLogger` | audit.rs | Compliance tracking |
| `ResultCache` | result_cache.rs | TTL-based caching |

## SCHEDULER
- Uses `petgraph` for DAG operations
- Topological sort for execution order
- Tracks node dependencies and parallelism
- Provides execution statistics

## ERROR RECOVERY
- **StopWorkflow**: Halt entire workflow
- **FailNode**: Mark node as failed, continue if possible
- **ContinueWithoutNode**: Skip failed node
- **PauseAndRetry**: Manual intervention required

## STATE MANAGEMENT
- **Checkpoint**: Every 5 minutes (configurable)
- **Persistence**: Via `StateManager` (FileStorage or LanceDB)
- **Recovery**: Resume from last checkpoint on restart

## AUDIT LOGGING
- All execution events logged
- Error details with stack traces
- Queryable audit reports
- Severity levels: Info, Warning, Error, Critical
