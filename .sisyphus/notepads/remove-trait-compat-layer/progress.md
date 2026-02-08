# Remove Trait Compat Layer - Work Progress

## Completed Tasks

### Wave 1: Core Trait and Domain Layer Cleanup ✅

1. **Updated `src/domain/port/tool_registry.rs`**
   - Removed all trait definitions (`ToolRegistry`, `ToolNode`, `ToolExecutor`)
   - Changed to re-export new system types

2. **Updated `src/domain/port/plugin_manager.rs`**
   - Changed `Plugin::get_tools()` return type from `Vec<Arc<dyn ToolNode>>` to `Vec<Tool>`

3. **Updated `src/tools/mod.rs`**
   - Removed all compat exports
   - Removed `BasicTool`, `BasicToolBuilder`, `ToolNode`, `ToolExecutor`, `ToolRegistry` trait exports
   - Updated to only export new enum-based system

4. **Deleted Files**
   - `src/tools/compat.rs` (345 lines) ✅
   - `src/tools/node.rs` (313 lines) ✅
   - `src/error.rs` (duplicate file) ✅

5. **Fixed Import Statements in Multiple Files**
   - `src/interfaces/tui/app.rs`
   - `src/plugins/docker.rs`
   - `src/plugins/python.rs`
   - `src/plugins/nodejs.rs`
   - `src/plugins/native.rs`
   - `src/plugins/manager.rs`
   - `src/plugins/types.rs`
   - `src/plugins/file_management/utils/registry.rs`
   - `src/plugins/file_management/classification/classification_flow.rs`
   - `src/plugins/file_management/classification/classification_tool.rs`

## Remaining Tasks

### Critical: Remove Trait Implementations

The following files still have `impl ToolNode for` blocks that need to be removed or converted:

**Plugin Files:**
- `src/plugins/docker.rs` - Line 905: `impl ToolNode for DockerToolNode`
- `src/plugins/nodejs.rs` - Line 674: `impl ToolNode for NodeJsToolNode`
- `src/plugins/python.rs` - Line 497: `impl ToolNode for PythonToolNode`

**File Management Classification Tools (11 implementations):**
- `src/plugins/file_management/classification/classification_flow.rs`
  - RuleLoaderTool (line 43)
  - RulePreprocessorTool (line 175)
  - AutomatonBuilderTool (line 251)
  - DirectoryScannerTool (line 295)
  - FolderNamePreprocessorTool (line 360)
  - ParallelMatcherTool (line 414)
  - ScoreCalculatorTool (line 501)
  - AmbiguityDetectorTool (line 574)
  - ResultMergerTool (line 658)
  - ExperimentalCheckTool (line 721)
  - ReportGeneratorTool (line 752)

### Fix ToolRegistry Usage

Files using `Arc<dyn ToolRegistry>` need to be changed to `Arc<ToolRegistry>`:
- `src/interfaces/mcp.rs`
- `src/interfaces/cli/app.rs`
- `src/workflow/component/tool.rs`
- `src/workflow/retry_tests.rs`
- `src/interfaces/tui/widgets/tool_manager.rs`
- `src/plugins/manager.rs`

### Other Files with Old Imports

- `src/plugins/file_management/plugin.rs` - Line 12: `use crate::tools::compat::tool_node_to_enum;`
- `src/plugins/file_management/ui/human_decision_tool.rs` - Line 655: `use crate::tools::compat::ToolNode;`

## Migration Strategy

### For Plugin Files (docker.rs, python.rs, nodejs.rs, native.rs)

**Current Pattern:**
```rust
impl ToolNode for XxxToolNode {
    fn name(&self) -> &str { ... }
    async fn execute(&self, params: Value, context: ExecutionContext) -> Result<Value> { ... }
}

fn get_tools(&self) -> Vec<Arc<dyn ToolNode>> {
    vec![Arc::new(XxxToolNode::new(...))]
}
```

**Target Pattern:**
```rust
// Remove impl ToolNode block entirely

fn get_tools(&self) -> Vec<Tool> {
    let native_tool = NativeToolBuilder::new()
        .name("tool_name")
        .version("1.0.0")
        .executor(|input: ToolInput, ctx| async move {
            // Tool execution logic here
            Ok(ToolOutput::success(result))
        })
        .build()
        .unwrap();
    vec![Tool::Native(Arc::new(native_tool))]
}
```

### For Classification Flow Tools

These tools implement `ToolNode` but may need to be converted to helper functions that return `Tool`:

```rust
// Old
impl ToolNode for RuleLoaderTool { ... }

// New
pub fn create_rule_loader_tool() -> Tool {
    let native_tool = NativeToolBuilder::new()
        .name("rule_loader")
        .executor(|input, ctx| async move { ... })
        .build()
        .unwrap();
    Tool::Native(Arc::new(native_tool))
}
```

## Current Status

- **Phase 1** (Infrastructure): ✅ Complete
- **Phase 2** (Import Fixes): 🔄 ~50% Complete
- **Phase 3** (Trait Implementations): ❌ Not Started
- **Phase 4** (ToolRegistry Usage): ❌ Not Started
- **Phase 5** (Verification): ❌ Not Started

## Estimated Remaining Work

- Remove ~14 trait implementations: 2-3 hours
- Fix ToolRegistry usage patterns: 1-2 hours
- Fix remaining import issues: 30 minutes
- Testing and verification: 1-2 hours

**Total: 4-8 hours of focused work remaining**

## Blockers

None identified - the remaining work is straightforward but time-consuming refactoring.

## Notes

- The `ToolNode` trait has been completely removed from the codebase
- All code referencing `dyn ToolNode` must be converted to use `Tool` enum directly
- The `ToolRegistry` is now a struct, not a trait - use `Arc<ToolRegistry>` instead of `Arc<dyn ToolRegistry>`
- Test files in `tests/` directory will also need updates once main code compiles
