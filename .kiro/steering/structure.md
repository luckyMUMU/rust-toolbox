# Project Structure

## Root Directory Layout

```
workflow-toolkit/
├── src/                    # Source code (DDD layered architecture)
├── examples/               # Usage examples and demonstrations
├── docs/                   # Documentation (4-layer hierarchy)
├── config/                 # Configuration files
├── .kiro/                  # Kiro AI assistant configuration
├── .trae/                  # Project-specific rules and documents
├── Cargo.toml             # Rust package manifest
└── design.md              # Top-level design document
```

## Source Code Organization (src/)

The codebase follows Domain-Driven Design (DDD) with clear layer separation:

### DDD Layers

```
src/
├── domain/                 # Domain Layer - Business logic & models
│   ├── model/             # Domain entities and value objects
│   ├── port/              # Repository and service interfaces
│   └── design.md          # Domain layer design doc
├── application/            # Application Layer - Use cases & orchestration
│   ├── usecase/           # Application use cases
│   ├── service/           # Application services
│   ├── workflow/          # Workflow orchestration logic
│   └── design.md          # Application layer design doc
├── infrastructure/         # Infrastructure Layer - Technical implementations
│   ├── persistence/       # Database and storage implementations
│   ├── cache/             # Caching implementations
│   ├── plugin/            # Plugin infrastructure
│   ├── external/          # External service integrations
│   └── design.md          # Infrastructure layer design doc
├── interfaces/             # Interface Layer - User-facing APIs
│   ├── cli/               # Command-line interface
│   ├── tui/               # Terminal UI
│   ├── mcp.rs             # MCP server implementation
│   └── design.md          # Interface layer design doc
└── adapter/                # Adapters for interface implementations
    ├── cli/               # CLI adapters
    ├── tui/               # TUI adapters
    ├── mcp/               # MCP adapters
    └── dto/               # Data transfer objects
```

### Functional Modules

```
src/
├── workflow/               # Workflow engine core
│   ├── component/         # Workflow node components
│   ├── context/           # Execution context management
│   ├── executor/          # Execution chain (Audit → Cache → Retry → Basic)
│   ├── state/             # State management
│   ├── engine.rs          # Main workflow engine
│   ├── scheduler.rs       # DAG scheduler
│   ├── execution_manager.rs  # Execution lifecycle management
│   └── design.md          # Workflow module design doc
├── tools/                  # Tool system
│   ├── base/              # Base tool implementations
│   ├── fs/                # File system tools
│   ├── algo/              # Algorithm tools
│   ├── registry.rs        # Tool registry
│   ├── node.rs            # Tool node abstraction
│   └── design.md          # Tools module design doc
├── plugins/                # Plugin system
│   ├── file_management/   # File management plugin
│   ├── manager.rs         # Plugin manager
│   ├── native.rs          # Native Rust plugins
│   ├── python.rs          # Python plugin support
│   ├── nodejs.rs          # Node.js plugin support
│   ├── docker.rs          # Docker plugin support
│   ├── wasm.rs            # WASM plugin support
│   └── design.md          # Plugins module design doc
├── storage/                # Storage and persistence
│   ├── backends.rs        # Storage backend implementations
│   ├── state_manager.rs   # State management
│   ├── backup.rs          # Backup and recovery
│   └── design.md          # Storage module design doc
├── performance/            # Performance optimization
│   ├── cache.rs           # Result caching
│   ├── metrics.rs         # Performance metrics
│   ├── profiler.rs        # Profiling utilities
│   └── design.md          # Performance module design doc
├── di/                     # Dependency injection
│   ├── container.rs       # DI container
│   ├── module.rs          # Module definitions
│   └── provider.rs        # Service providers
├── core/                   # Core types and utilities
├── error/                  # Error types and handling
└── config.rs              # Configuration management
```

## Documentation Structure (docs/)

Four-layer documentation hierarchy:

```
docs/
├── 01_concept_overview.md              # L1: Core concepts and value propositions
├── 02_logical_workflow/                # L2: Logical flow (pseudocode)
│   ├── workflow_execution.pseudo
│   ├── tool_execution.pseudo
│   ├── plugin_loading.pseudo
│   └── error_handling.pseudo
├── 03_technical_spec/                  # L3: Technical specifications
│   ├── interfaces.md                   # Interface contracts
│   └── api/                            # API references
│       ├── CLI_REFERENCE.md
│       └── RUST_SDK_REFERENCE.md
├── 04_context_reference/               # L4: Decision records
│   └── architecture_decision.md        # ADR documentation
└── archive/                            # Archived/deprecated docs
```

## Examples Directory

```
examples/
├── workflow_example.rs                 # Basic workflow usage
├── python_plugin_example.rs            # Python plugin integration
├── docker_plugin_example.rs            # Docker plugin usage
├── nodejs_plugin_example.rs            # Node.js plugin usage
├── composable_tool_example.rs          # Tool composition
├── comprehensive_workflow_example.rs   # Complex workflow scenarios
├── python_tools/                       # Python tool examples
├── nodejs_tools/                       # Node.js tool examples
├── docker_tools/                       # Docker tool examples
└── templates/                          # Configuration templates
```

## Module Design Document Convention

Every major module MUST include a `design.md` file with:
- Module overview and responsibilities
- Key components and their relationships
- Implementation status
- Future plans and TODOs

Parent-level design documents should only contain:
- High-level summaries
- Links to child module design docs
- Cross-cutting concerns

## Naming Conventions

- **Modules**: Snake_case (e.g., `workflow_engine`, `tool_registry`)
- **Files**: Snake_case (e.g., `execution_manager.rs`, `state_manager.rs`)
- **Types**: PascalCase (e.g., `WorkflowEngine`, `ToolNode`)
- **Functions**: Snake_case (e.g., `execute_workflow`, `register_tool`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`, `DEFAULT_TIMEOUT`)

## Test Organization

- **Unit tests**: Inline with `#[cfg(test)]` or separate `*_tests.rs` files
- **Integration tests**: In module-specific test files (e.g., `execution_manager_simple_test.rs`)
- **Property-based tests**: Using `proptest` crate
- **Examples as tests**: Examples in `examples/` also serve as integration tests

## Configuration Files

- `config/default.toml` - Default configuration
- `.kiro/steering/*.md` - AI assistant steering rules
- `.trae/rules/project_rules.md` - Project-specific development rules
