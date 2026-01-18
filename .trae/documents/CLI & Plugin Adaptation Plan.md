# CLI & Plugin Adaptation Plan for R-Flow v2.1

This plan addresses the incompatibility between the legacy CLI/Plugin code and the newly refactored R-Flow v2.1 Core (DagScheduler, FlowNode, ToolNode).

## 1. Plugin System Upgrade (`src/plugins`)
**Goal**: Update plugin implementations to produce v2.1-compliant `ToolNode` instances.

*   **1.1 Update Python Plugin (`src/plugins/python.rs`)**
    *   Refactor `PythonToolNode` to implement the new `ToolNode` trait (async_trait).
    *   Map `execute` -> `process`, `name` -> `id`.
    *   Ensure `input_schema` returns the correct JSON schema from `ToolInfo`.
*   **1.2 Update Plugin Manager (`src/plugins/manager.rs`)**
    *   Fix `register_plugin_tools` to work with `Arc<dyn ToolNode>` (which is now the new trait).
    *   Ensure thread-safety with the new `DashMap`-based `ExecutionContext`.

## 2. Workflow Converter (`src/workflow/converter.rs`)
**Goal**: Bridge the gap between legacy YAML definitions and the new Recursive Flow structure.

*   **2.1 Implement `WorkflowConverter`**
    *   Input: `WorkflowDefinition` (Graph-based).
    *   Output: `FlowNode` (Recursive Tree).
    *   Logic:
        1.  Perform topological sort on `WorkflowDefinition`.
        2.  Identify parallelizable groups (optional, or just linearize for now).
        3.  Construct a `FlowNode::Chain` of `FlowNode::Node`s.
        4.  (Advanced) Map `Switch` nodes to `FlowNode::Switch`.

## 3. CLI Application Refactoring (`src/interfaces/cli`)
**Goal**: Make the CLI use the new `WorkflowEngine` and `DagScheduler`.

*   **3.1 Update `CliApp` Struct (`src/interfaces/cli/app.rs`)**
    *   Change `workflow_engine` field from `Option<Arc<dyn WorkflowEngine>>` to `Option<Arc<WorkflowEngine>>` (Concrete type).
    *   Remove legacy `state_manager` dependencies if incompatible.
*   **3.2 Refactor Workflow Execution**
    *   In `handle_workflow_command`:
        *   Load `WorkflowDefinition`.
        *   Convert to `FlowNode` using `WorkflowConverter`.
        *   Create `ExecutionContext`.
        *   Call `engine.execute(flow, ctx)`.
*   **3.3 Refactor Tool Execution**
    *   In `handle_tool_command`:
        *   Retrieve tool from `ToolRegistry`.
        *   Call `tool.process(&ctx)`.

## 4. Integration & Verification
*   **4.1 Fix Compilation Errors**: Resolve all mismatched types and trait bounds.
*   **4.2 Verification**: Run `cargo check` to ensure the adapter layer compiles correctly.

## Execution Order
1.  **Converter**: Create the bridge first.
2.  **Plugins**: Fix the data sources.
3.  **CLI**: Update the consumer.
4.  **Verify**: Compile.
