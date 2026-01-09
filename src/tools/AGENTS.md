# src/tools/ - Tool System

## OVERVIEW
Tool registry, node system, parameter templates, and version management.

## TOOL NODES
**Trait**: `ToolNode` - Send + Sync async tools
- `name()`, `version()`, `validate_parameters()`
- `execute()` with ExecutionContext
- `get_info()` for metadata
- `expand_parameters()` for templates

**BasicTool**: Standard implementation
- Builder pattern for construction
- Schema validation via jsonschema
- Optional plugin association

## REGISTRY
**Trait**: `ToolRegistry` - Concurrent tool management
- `register_tool()`, `get_tool()`, `list_tools()`
- `execute_tool()` with parameters
- `validate_tool_params()` without execution
- Dependency resolution

**BasicToolRegistry**: DashMap-based implementation
- Thread-safe concurrent access
- Version conflict detection
- Template expansion support

## TEMPLATES
**ParameterTemplate**: Reusable parameter patterns
- **TemplateEngine**: Expands templates with context
- **TemplateContext**: Variables for expansion
- **TemplateFn**: Custom expansion functions

## VERSIONING
**DependencyResolver**: Handles tool dependencies
- **Version**: Semantic versioning
- **VersionRequirement**: Constraint matching
- **ToolDependency**: Dependency graph
- **ResolutionResult**: Dependency resolution output

## USAGE
```rust
use workflow_toolkit::tools::{BasicTool, AsyncFunctionExecutor, BasicToolRegistry};

let executor = Arc::new(AsyncFunctionExecutor::new(|params, context| async move {
    // Tool logic
    Ok(result)
}));

let tool = BasicTool::builder()
    .name("my_tool")
    .version("1.0.0")
    .executor(executor)
    .build()?;

registry.register_tool(Arc::new(tool))?;
```
