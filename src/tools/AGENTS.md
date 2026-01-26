# src/tools/ - Tool System

## OVERVIEW
Tool registry, node system, parameter templates, and version management. 6 files with 1,914+ lines in ac_automaton.rs.

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

**Template Syntax**:
```yaml
# Variable expansion
message: "{{ user.name }}"

# Function calls
timestamp: "{{ now() }}"

# Conditional
output: "{{ if condition then 'yes' else 'no' }}"

# Array operations
items: "{{ list.slice(0, 10) }}"
```

## VERSIONING
**DependencyResolver**: Handles tool dependencies
- **Version**: Semantic versioning
- **VersionRequirement**: Constraint matching
- **ToolDependency**: Dependency graph
- **ResolutionResult**: Dependency resolution output

**Version Requirements**:
- `Exact(Version)`: Exact match
- `GreaterThan(Version)`: Greater than
- `GreaterThanOrEqual(Version)`: Greater than or equal
- `LessThan(Version)`: Less than
- `LessThanOrEqual(Version)`: Less than or equal
- `Compatible(Version)`: Same major, minor >= required
- `Range { min, max }`: Version range
- `Any`: Any version

**Conflict Types**:
- `NoSatisfyingVersion`: No version satisfies all
- `IncompatibleVersions`: Multiple incompatible versions
- `CircularDependency`: Circular dependency detected
- `ToolConflict`: Tool conflicts with another

## ALGORITHMS
**File**: `algo/ac_automaton.rs` (1,914 lines)  
**Purpose**: Aho-Corasick algorithm for efficient multi-pattern matching

**Features**:
- **Pattern Matching**: O(n + m) time complexity
- **Multiple Patterns**: Match thousands simultaneously
- **Scored Patterns**: Weighted pattern matching
- **Category Support**: Group patterns by category
- **Statistics**: Detailed matching statistics

**Components**:
- `Pattern`: Pattern with weight and category
- `AutomatonNode`: Trie node with failure links
- `PatternMatch`: Match result with position and score
- `AhoCorasickMatcher`: Main matcher implementation
- `AutomatonConfig`: Configuration options
- `AutomatonStats`: Performance statistics
- `MatchStatistics`: Result statistics

**Usage**:
```rust
use workflow_toolkit::tools::algo::{AhoCorasickMatcher, Pattern, AutomatonConfig};

let mut matcher = AhoCorasickMatcher::new(AutomatonConfig::default());

matcher.add_pattern(Pattern {
    pattern: "doc".to_string(),
    category: "documents".to_string(),
    score: 1.0,
    id: 0,
});

let matches = matcher.find_matches("This is a document");
```

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

## SUBDIRECTORIES

### algo/
**Path**: `src/tools/algo/`  
**Purpose**: Algorithm implementations for tool operations

**Files**:
- `ac_automaton.rs`: Aho-Corasick pattern matching (1,914 lines)

**Algorithms**:
- **Aho-Corasick**: Efficient multi-pattern matching
  - Time complexity: O(n + m)
  - Space complexity: O(m) where m = total pattern length
  - Supports weighted patterns
  - Category-based grouping
  - Detailed statistics

### base/
**Path**: `src/tools/base/`  
**Purpose**: Base tool implementations

**Files**:
- `basic_tool.rs`: Standard tool implementation
- `async_function_executor.rs`: Async function wrapper

### fs/
**Path**: `src/tools/fs/`  
**Purpose**: File system tools

**Files**:
- `file_reader.rs`: File reading utilities
- `file_writer.rs`: File writing utilities
- `directory_scanner.rs`: Directory scanning

## TOOL COMPOSITION
Tools can be composed to create complex workflows:

```rust
// Create a tool that uses other tools
let composed_tool = BasicTool::builder()
    .name("composed_tool")
    .version("1.0.0")
    .executor(Arc::new(AsyncFunctionExecutor::new(|params, context| async move {
        // Step 1: Use tool A
        let result_a = tool_registry.execute_tool("tool_a", params.clone()).await?;
        
        // Step 2: Use tool B with result from A
        let result_b = tool_registry.execute_tool("tool_b", result_a).await?;
        
        // Step 3: Process and return
        Ok(result_b)
    })))
    .build()?;
```

## PARAMETER VALIDATION
**JsonSchema**: JSON Schema validation
- Type checking
- Required fields
- Constraints (min, max, pattern)
- Custom validators

**Validation Flow**:
1. Parse parameters as JSON
2. Validate against schema
3. Expand templates
4. Return validation result

## ERROR HANDLING
**Tool Errors**:
- `InvalidParameters`: Parameter validation failed
- `ExecutionFailed`: Tool execution error
- `DependencyConflict`: Version conflict
- `TemplateError`: Template expansion error

**Error Recovery**:
- Retry with different parameters
- Use default values
- Skip tool and continue
- Request user intervention

## PERFORMANCE
**Caching**: Tool results can be cached
- Cache key generation
- TTL-based expiration
- Invalidation strategies

**Concurrency**: Tools are Send + Sync
- Thread-safe execution
- Parallel tool execution
- Resource limiting

## TESTING
**Unit Tests**:
- Tool registration and lookup
- Parameter validation
- Template expansion
- Version resolution

**Integration Tests**:
- Tool execution in workflows
- Dependency resolution
- Error handling scenarios

## BEST PRACTICES

### Tool Design
1. **Single Responsibility**: One tool, one purpose
2. **Clear Parameters**: Well-defined input/output
3. **Validation**: Validate early, fail fast
4. **Documentation**: Clear help text and examples
5. **Error Messages**: Actionable error messages

### Versioning
1. **Semantic Versioning**: MAJOR.MINOR.PATCH
2. **Backward Compatibility**: Maintain compatibility when possible
3. **Dependency Management**: Specify minimum versions
4. **Conflict Resolution**: Handle version conflicts gracefully

### Performance
1. **Caching**: Cache expensive operations
2. **Async**: Use async for I/O operations
3. **Resource Limits**: Limit concurrent tool execution
4. **Memory Management**: Clean up resources promptly

## SEE ALSO

- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Plugins AGENTS.md](../plugins/AGENTS.md) - Plugin system
- [Workflow AGENTS.md](../workflow/AGENTS.md) - Workflow engine
- [File Management AGENTS.md](../plugins/file_management/AGENTS.md) - File tools

### 模块级补充细则
- 目标与范围
  - 本模块负责工具注册、执行、参数模板、版本管理等，确保工具在工作流中的可发现性、可重复性与可组合性。
- 设计与扩展
  - 新工具/模板应提供清晰的输入/输出、参数验证以及错误契约。
  - 版本冲突、依赖分离、向后兼容性评估应在设计评审阶段完成。
- 实现规范
  - 导入排序遵循通用规范：std -> external -> crate，分组后空一行。
  - 命名规则：snake_case、CamelCase、ALL_CAPS 与已有风格保持一致。
  - 文档化：对公开 API 提供 Rustdoc 注释，示例和使用案例。
  - 错误处理：集中化错误类型，返回 Result<T, WorkflowError>。
  - 异步/并发：工具执行应为异步，尽量无阻塞。
- 测试策略
  - 覆盖工具注册、参数验证、模板扩展、版本解析、集成执行等场景。
  - 集成测试涵盖工作流中工具的协作与依赖关系。
- 变更与审阅
  - 变更前提供设计动机、影响评估和回归测试计划。
- 文档与审阅
  - 模块级 AGENTS.md 变更需同步。
- Cursor/Copilot 规则
  - 将 Cursor/Copilot 规则合并到模块级 AGENTS.md 模板中。
