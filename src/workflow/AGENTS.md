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
| `WorkflowDefinition` | definition.rs | DAG structure and node definitions |
| `ExecutionState` | execution.rs | Runtime state tracking |

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
| Manage parallel execution | `execution_manager.rs` |
| Track state | `execution.rs` |
| Log operations | `audit.rs` |
| Cache results | `result_cache.rs` |

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
