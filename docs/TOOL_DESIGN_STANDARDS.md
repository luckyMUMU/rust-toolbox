# Tool Design Standards & Architecture Guide

## 1. Core Principle: Single Responsibility Principle (SRP)

Every tool in the `workflow-toolkit` ecosystem must adhere to the **Single Responsibility Principle**. A tool should do one thing, do it well, and do it completely.

### 1.1 What Defines a Tool?
*   **Atomic Operation**: A tool performs a single, logical unit of work (e.g., "Read File", "Parse JSON", "Send HTTP Request").
*   **No Business Logic**: Tools must **NOT** contain complex branching logic (`if/else`) that implements business rules. Business logic belongs in the **Workflow** orchestration layer.
*   **Statelessness**: Tools should be pure functions where `Output = f(Input)`. Avoid internal state that persists between executions unless explicitly designed as a Store (like `DataCacheTool`).

### 1.2 Naming Conventions
Tool names should clearly describe their action and target.

*   **Format**: `noun-verb` or `verb-noun` (kebab-case for IDs, PascalCase for Structs).
*   **Examples**:
    *   ✅ `file-reader` / `FileReader`
    *   ✅ `json-transformer` / `JsonTransformer`
    *   ✅ `directory-scanner` / `DirectoryScanner`
    *   ❌ `process-manager` (Too vague)
    *   ❌ `file-handler` (Does it read? write? delete?)

## 2. Tool Implementation Guidelines

### 2.1 The `ToolNode` Trait
All tools must implement the `ToolNode` trait.

```rust
#[async_trait]
impl ToolNode for MyTool {
    fn name(&self) -> String { "my-tool".to_string() }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // Strict schema validation required
    }
    
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> {
        // Atomic execution logic
    }
}
```

### 2.2 Error Handling
*   Return specific `WorkflowError` types.
*   Do not swallow errors; propagate them to the engine.
*   Let the Workflow Engine handle retries and error policies.

## 3. Workflow Orchestration

Complex behavior is achieved by **composing** simple tools, not by making tools complex.

### 3.1 Conditional Logic
*   **Anti-Pattern**: A tool taking a boolean `compress_files` parameter and branching internally.
*   **Best Practice**:
    *   Node A: `FileScanner`
    *   Node B: `Condition` (checks file size)
    *   Node C: `FileCompressor` (executed only if Node B is true)
    *   Node D: `FileMover`

### 3.2 Data Flow
*   Tools receive data via `parameters`.
*   Tools return data via `Result<Value>`.
*   Use `DataCacheTool` for sharing data across non-adjacent nodes.
*   Use `DataTransformTool` to map output of Tool A to input of Tool B.

## 4. System Tools

The following standard tools are available for flow control and data management:

*   **`data-cache`**: Store/Retrieve temporary data in the execution context.
*   **`data-transform`**: Apply JQ-like transformations or template rendering to JSON data.

## 5. Review Checklist
Before submitting a new tool, ask:
1.  Can I describe this tool's function in one sentence without using "and"?
2.  Does this tool make decisions about *what* to do next? (If yes, move that logic to the Workflow).
3.  Is the name specific enough?
