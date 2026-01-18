# CLI & Plugin Adaptation Implementation Plan

This plan addresses the missing components and refactoring required to bridge the legacy CLI/Plugin system with the new R-Flow v2.1 Core.

## 1. Core Definitions (`src/workflow` & `src/tools`)

We need to define the missing `FlowNode` structure and restore the missing `ToolNode` definition.

### 1.1 Create `src/workflow/flow_node.rs`
Define the recursive execution tree structure:
- **Enum `FlowNode`**:
    - `Chain(Vec<FlowNode>)`: Sequential execution.
    - `Parallel(Vec<FlowNode>)`: Concurrent execution.
    - `Switch(Vec<(Condition, FlowNode)>)`: Conditional branching.
    - `Loop(LoopConfig, Box<FlowNode>)`: Iteration.
    - `Tool(String, Value)`: Leaf node executing a tool (ID + Params).

### 1.2 Restore `src/tools/node.rs`
The `ToolNode` trait is referenced but missing. We will create it:
- **Trait `ToolNode`**: `async_trait` with methods `name()`, `execute()`, `validate_parameters()`, `get_info()`.
- **Struct `BasicTool`**: Default implementation.

## 2. Workflow Converter (`src/workflow/converter.rs`)

Implement the logic to convert the graph-based `WorkflowDefinition` into the tree-based `FlowNode`.

- **Struct `WorkflowConverter`**:
    - Uses `DagScheduler` (or topological sort) to linearize the graph.
    - Detects parallel branches (nodes with same dependencies).
    - Constructs the `FlowNode` tree.

## 3. Engine Update (`src/workflow/engine_v2.rs`)

Update `RefactoredWorkflowEngine` to support executing `FlowNode`.

- Add `execute_flow(&self, flow: FlowNode, context: &mut DataContext)` method.
- Implement recursive execution logic for `Chain`, `Parallel`, etc.
- Ensure `RefactoredWorkflowEngine` can be used by the CLI (implement `WorkflowEngine` trait or expose concrete type).

## 4. Plugin System Update (`src/plugins`)

Ensure plugins produce compliant `ToolNode` instances.

- **`src/plugins/python.rs`**: Verify `PythonToolNode` implements the restored `ToolNode` trait.
- **`src/plugins/manager.rs`**: Fix `register_plugin_tools` to work with `Arc<dyn ToolNode>`.

## 5. CLI Refactoring (`src/interfaces/cli/app.rs`)

Connect the pieces in the CLI application.

- Update `CliApp` to use `RefactoredWorkflowEngine` (Concrete or Trait).
- In `handle_workflow_command`:
    1. Load `WorkflowDefinition`.
    2. Convert to `FlowNode` using `WorkflowConverter`.
    3. Execute using `engine.execute_flow(...)`.

## Execution Steps

1.  **Create** `src/tools/node.rs` (fix missing dependency).
2.  **Create** `src/workflow/flow_node.rs`.
3.  **Create** `src/workflow/converter.rs`.
4.  **Update** `src/workflow/engine_v2.rs` to add `execute_flow`.
5.  **Refactor** `src/plugins/manager.rs` and `python.rs`.
6.  **Refactor** `src/interfaces/cli/app.rs`.
7.  **Verify** with `cargo check`.
