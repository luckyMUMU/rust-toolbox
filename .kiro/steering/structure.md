# Project Structure

## Root Directory Layout

```
workflow-toolkit/
├── src/                    # Source code (DDD layered architecture)
├── examples/               # Runnable examples for different features
├── docs/                   # Documentation (4-layer structure)
├── config/                 # Runtime configuration files
├── workflow-toolkit-macros/ # Procedural macro crate
├── .kiro/                  # Kiro AI assistant settings
├── .trae/                  # Trae AI assistant artifacts
├── Cargo.toml              # Main project manifest
└── design.md               # Root architecture design document

```

## Source Code Structure (src/)

The codebase follows Domain-Driven Design (DDD) with clear layer separation:

```
src/
├── domain/                 # Domain Layer - Business logic and models
│   ├── model/              # Domain entities and value objects
│   └── port/               # Port interfaces (Repository, Service)
├── application/            # Application Layer - Use cases and orchestration
├── infrastructure/         # Infrastructure Layer - Technical implementations
├── interfaces/             # Interfaces Layer - User-facing interfaces
│   ├── cli/                # Command-line interface
│   ├── tui/                # Terminal user interface
│   └── mcp/                # Model Context Protocol server
├── workflow/               # Workflow engine core
│   ├── component/          # LiteFlow-style components
│   ├── executor/           # Executor chain (Audit → Cache → Retry → Basic)
│   ├── context/            # Data context and slot management
│   └── state/              # State management
├── tools/                  # Tool system
│   └── registry/           # Tool registration and discovery
├── plugins/                # Plugin system
│   ├── file_management/    # File management plugin
│   └── [other plugins]/    # Additional plugin implementations
├── storage/                # Storage backends (File, Memory, Redis)
├── performance/            # Performance monitoring and optimization
├── adapter/                # Adapters for external integrations
├── core/                   # Core types and utilities
├── di/                     # Dependency injection container
├── error/                  # Error types and handling
├── config.rs               # Configuration management
├── lib.rs                  # Library entry point and re-exports
└── main.rs                 # Binary entry point (CLI/TUI)
```

## Documentation Structure (docs/)

Four-layer documentation architecture:

```
docs/
├── INDEX.md                        # Documentation hub
├── 01_concept_overview.md          # L1: Core concepts
├── 01_requirements/                # L1: Requirements layer
│   └── workflow_toolkit_prd.md     # Product requirements document
├── 02_logical_workflow/            # L2: Logical flow layer
│   ├── error_handling.pseudo       # Error handling logic
│   ├── plugin_loading.pseudo       # Plugin loading logic
│   ├── tool_execution.pseudo       # Tool execution logic
│   └── workflow_execution.pseudo   # Workflow execution logic
├── 03_technical_spec/              # L3: Technical specification layer
│   ├── index.md                    # Technical spec overview
│   ├── interfaces.md               # Interface contracts
│   └── api/                        # API references
│       ├── CLI_REFERENCE.md        # CLI command reference
│       └── RUST_SDK_REFERENCE.md   # Rust SDK reference
├── 04_context_reference/           # L4: Decision context layer
│   ├── architecture_decision.md    # Architecture decisions overview
│   ├── design_review_report.md     # Design review findings
│   ├── glossary.md                 # Terminology definitions
│   └── adr/                        # Architecture Decision Records
│       ├── template.md             # ADR template
│       ├── adr_di_001_*.md         # DI container decision
│       ├── adr_workflow_002_*.md   # JoinSet parallel execution
│       ├── adr_plugin_003_*.md     # WASM sandbox isolation
│       └── adr_tools_004_*.md      # Schema validation strategy
├── archive/                        # Archived/deprecated documentation
└── 参考/                           # Reference materials (Chinese)
```

## Examples Directory

```
examples/
├── python_tools/           # Python plugin examples
├── nodejs_tools/           # Node.js plugin examples
├── docker_tools/           # Docker plugin examples
├── templates/              # Workflow templates
└── *.rs                    # Rust example files
```

## Design Documents

Each module has its own `design.md` file following a hierarchical structure:

- Root `design.md` - High-level architecture overview with links to submodules
- `src/domain/design.md` - Domain layer design
- `src/application/design.md` - Application layer design
- `src/workflow/design.md` - Workflow engine design
- And so on for each major module

## Key Conventions

### Module Organization
- Each layer is strictly separated with clear dependencies (Interfaces → Application → Domain ← Infrastructure)
- No circular dependencies between layers
- Domain layer has no dependencies on other layers
- Infrastructure implements domain ports

### File Naming
- Rust files: `snake_case.rs`
- Documentation: `snake_case.md` or numbered prefixes (e.g., `01_concept_overview.md`)
- ADR files: `adr_<domain>_<number>_<title>.md`
- Pseudocode files: `.pseudo` extension

### Design Document Rules
- Parent documents contain only summaries and links to child documents
- Use status markers: `[已完成]`, `[进行中]`, `[待开始]`
- Each module must have its own `design.md`
- Keep design docs synchronized with code changes

### Configuration
- Runtime config in `config/default.toml`
- Kiro AI settings in `.kiro/`
- Environment-specific overrides supported via `config` crate

### Testing
- Unit tests in module files (`#[cfg(test)]` blocks)
- Integration tests in `tests/` directory (if present)
- Property-based tests using `proptest`
- Example code serves as integration tests

## Navigation Tips

- Start with `docs/INDEX.md` for documentation overview
- Read `design.md` for architecture understanding
- Check `docs/01_concept_overview.md` for core concepts
- Review `docs/01_requirements/workflow_toolkit_prd.md` for product requirements
- Explore `src/lib.rs` for code structure and re-exports
- Look at `examples/` for usage patterns
