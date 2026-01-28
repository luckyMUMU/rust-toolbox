# WORKFLOW TOOLKIT - Project Knowledge Base

**Generated**: 2026-01-24
**Commit**: current
**Branch**: master-v2
**Version**: 0.1.0

## 项目概述 - Project Overview
Multi-interface workflow execution system built with Rust: CLI, TUI, and MCP server with DAG-based orchestration, plugin architecture, and comprehensive file management capabilities. 163 Rust files, ~110,000+ lines of code.

**中文概述**: 基于 Rust 的多接口工作流执行系统，支持 CLI、TUI 和 MCP 服务器，具备 DAG 编排、插件架构和全面的文件管理能力。163 个 Rust 文件，约 110,000+ 行代码。

### 功能状态说明
- **CLI**: ✅ 完全实现
- **TUI**: ✅ 完全实现
- **MCP Server**: ⚠️ 存根实现 (Stub - 待完成)
- **WASM 插件**: ⚠️ 暂时禁用 (依赖问题)

## 全局规则 - Global Rules

### 语言规则 - Language Rules
**默认使用中文回复** - Default to Chinese responses

- **中文用户**: 使用简体中文回复
- **英文用户**: 使用英文回复
- **混合场景**: 根据用户输入语言自动切换

**示例**:
```
用户: "帮我分析代码"
回复: "好的，我来分析代码..."

用户: "Analyze the code"
回复: "I'll analyze the code..."
```

### 文档创建规则 - Documentation Creation Rules

**必须创建文档**:
1. **新模块**: 创建模块时必须有 AGENTS.md
2. **复杂功能**: 功能复杂度 > 5 分时
3. **公共API**: 所有公共接口必须有文档
4. **架构变更**: 任何架构调整都需要更新文档
5. **用户请求**: 用户明确要求创建文档

**文档类型**:
- **AGENTS.md**: 模块开发指南
- **README.md**: 用户文档
- **API文档**: Rust doc comments
- **教程**: 使用示例和最佳实践

**详见**: [GLOBAL_RULES.md](GLOBAL_RULES.md)

### 文档创建流程 - Documentation Creation Workflow

**步骤1: 分析需求**
- 确定文档范围（模块功能、用户群体、使用场景）
- 确定文档类型（API文档、用户指南、开发指南、参考手册）

**步骤2: 创建文档结构**
- 创建目录：`mkdir -p docs/ src/module_name/`
- 创建文档文件：`touch docs/ARCHITECTURE.md src/module_name/AGENTS.md`

**步骤3: 编写内容**
- 编写概述和核心组件
- 添加代码示例和最佳实践
- 添加注意事项和相关文档链接

**步骤4: 验证文档**
- 检查链接有效性
- 验证代码示例可运行
- 拼写和格式检查

**步骤5: 发布文档**
- 提交到版本控制
- 更新根文档
- 创建文档更新记录

### 文档质量标准 - Documentation Quality Standards

**完整性检查清单**:
- [ ] 模块概述清晰
- [ ] 所有公共项都有文档
- [ ] 代码示例可运行
- [ ] 链接有效
- [ ] 无拼写错误

**一致性检查清单**:
- [ ] 语言风格一致
- [ ] 格式统一
- [ ] 术语一致
- [ ] 链接格式统一

**可读性检查清单**:
- [ ] 结构清晰
- [ ] 层次分明
- [ ] 示例恰当
- [ ] 语言简洁

### 文档维护 - Documentation Maintenance

**定期检查**:
- **每周**: 检查文档完整性
- **每月**: 更新过时内容
- **每季度**: 全面审查文档

**版本控制**:
- **Git Hooks**: 提交前检查文档
- **CI/CD**: 自动验证文档
- **版本标记**: 文档随版本发布

**反馈机制**:
- **Issue追踪**: 文档问题使用 issue 跟踪
- **用户反馈**: 收集用户文档反馈
- **贡献指南**: 鼓励社区贡献文档

## STRUCTURE
```
workflow-toolkit/
├── src/                      # Core library (163 files, ~110,000+ lines)
│   ├── config.rs              # Hierarchical config (env > CLI > file > defaults)
│   ├── core.rs                # Shared types, ExecutionContext, RetryPolicy
│   ├── error.rs               # WorkflowError with 20+ variants, Result type
│   ├── workflow/              # DAG engine (15 files, 2,404 lines in engine.rs)
│   │   ├── executor/          # Execution strategies (5 files)
│   │   ├── component/         # Workflow components (4 files)
│   │   ├── context/           # Execution context (2 files)
│   │   └── state/             # State management (2 files)
│   ├── tools/                 # Tool system (6 files, 1,914 lines in ac_automaton.rs)
│   │   └── algo/              # Algorithm implementations
│   ├── plugins/               # Native, Python, Node.js, Docker, WASM
│   │   └── file_management/   # File ops plugin (20 files, 3,604 lines in utils.rs)
│   ├── interfaces/            # CLI, TUI, MCP server
│   │   ├── cli/               # Clap 4.5 (5 files, 1,583 lines in app.rs)
│   │   └── tui/               # Ratatui 0.29 (25 files, 3,275 lines in layout.rs)
│   │       └── widgets/       # 8 specialized widgets (3,553 lines in plugin_manager.rs)
│   ├── storage/               # FileStorage, memory cache, backup (5 files)
│   └── performance/           # Caching, concurrency, metrics (6 files)
├── examples/                  # 23 examples + workflow templates (19 files)
├── tests/                     # Integration + property-based tests (12 files)
├── docs/                      # Complete documentation
├── config/                    # default.toml configuration
└── scripts/                   # Legacy Python reference scripts
```

## AGENTS.MD FILES

### Existing Files (15)
1. `./AGENTS.md` (root) - 262 lines ✅
2. `./examples/AGENTS.md` - 109 lines ✅
3. `./examples/templates/AGENTS.md` - 114 lines ✅
4. `./src/AGENTS.md` - 64 lines ✅
5. `./src/interfaces/AGENTS.md` - 71 lines ✅
6. `./src/interfaces/cli/AGENTS.md` - 69 lines ✅ (updated)
7. `./src/interfaces/tui/AGENTS.md` - 85 lines ✅
8. `./src/interfaces/tui/widgets/AGENTS.md` - 134 lines ✅ (updated)
9. `./src/performance/AGENTS.md` - 81 lines ✅ (updated)
10. `./src/plugins/AGENTS.md` - 58 lines ✅ (updated)
11. `./src/plugins/file_management/AGENTS.md` - 50 lines ✅ (updated)
12. `./src/storage/AGENTS.md` - 62 lines ✅ (updated)
13. `./src/tools/AGENTS.md` - 60 lines ✅ (updated)
14. `./src/workflow/AGENTS.md` - 92 lines ✅ (updated)
15. `./tests/AGENTS.md` - 105 lines ✅ (updated)

### New Files Created
- None (all directories already had AGENTS.md files)

### Updated Files
- `./AGENTS.md` (root) - Updated with latest findings
- `./src/AGENTS.md` - Updated with module map
- `./src/interfaces/cli/AGENTS.md` - Comprehensive update (69 lines)
- `./src/interfaces/tui/widgets/AGENTS.md` - Comprehensive update (134 lines)
- `./src/plugins/file_management/AGENTS.md` - Comprehensive update (50 lines)
- `./src/performance/AGENTS.md` - Comprehensive update (81 lines)
- `./src/plugins/AGENTS.md` - Updated with file_management details
- `./src/storage/AGENTS.md` - Comprehensive update (62 lines)
- `./src/tools/AGENTS.md` - Comprehensive update (60 lines)
- `./src/workflow/AGENTS.md` - Updated with subdirectories
- `./tests/AGENTS.md` - Comprehensive update (105 lines)
- `./examples/AGENTS.md` - Updated with template details
- `./examples/templates/AGENTS.md` - Comprehensive update (114 lines)
- `./src/interfaces/AGENTS.md` - Updated with cli/widgets details
- `./src/interfaces/tui/AGENTS.md` - Updated with widgets details

### Total Files Modified
- 15 AGENTS.md files updated
- 0 new AGENTS.md files created
- 0 AGENTS.md files deleted

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| Quick start | README.md, docs/INDEX.md | Overview + quick start |
| Architecture | src/lib.rs, docs/PROJECT_OVERVIEW.md | Core design |
| Workflow engine | src/workflow/AGENTS.md | DAG execution |
| Plugin system | src/plugins/AGENTS.md | Multi-language plugins |
| CLI usage | src/interfaces/cli/AGENTS.md | Commands, output formatting |
| TUI usage | src/interfaces/tui/AGENTS.md | Event-driven widgets |
| File management | src/plugins/file_management/AGENTS.md | Classification, batch ops |
| Storage | src/storage/AGENTS.md | Persistence, caching, backup |
| Testing | tests/AGENTS.md | Integration + property tests |
| Templates | examples/templates/AGENTS.md | Production workflows |

## CODE MAP
| Symbol | Type | Location | Refs | Role |
|--------|------|----------|------|------|
| `ConfigManager` | struct | src/config.rs | High | Hierarchical config with hot reload |
| `WorkflowEngine` | trait | src/workflow/engine.rs | High | DAG-based execution |
| `ToolRegistry` | trait | src/tools/registry.rs | High | Concurrent tool management |
| `PluginManager` | struct | src/plugins/manager.rs | Medium | Multi-language plugins |
| `StateManager` | struct | src/storage/state.rs | Medium | Persistence + checkpointing |
| `CliApp` | struct | src/interfaces/cli/app.rs | Medium | CLI entry point |
| `TuiApp` | struct | src/interfaces/tui/app.rs | Medium | TUI event loop |
| `WorkflowError` | enum | src/error.rs | High | Centralized error type |
| `DefaultWorkflowEngine` | struct | src/workflow/engine.rs | High | Main engine implementation |
| `FileManagementPlugin` | struct | src/plugins/file_management/registry.rs | High | AI-powered file operations |
| `AsyncFunctionExecutor` | struct | src/tools/registry.rs | High | Async tool execution |

## CONVENTIONS

### Code Style
- **Import order**: std → external → internal
- **Error handling**: `thiserror` for custom errors, `anyhow` for conversion
- **Async patterns**: Full tokio async/await with `#[async_trait]`
- **Concurrency**: `DashMap` for concurrent maps, `Arc<Semaphore>` for limiting
- **No unwrap()**: All public APIs return `Result<T, WorkflowError>`

### Configuration Priority
1. CLI arguments (highest)
2. Environment variables (`WORKFLOW_TOOLKIT_*`)
3. Config file (`config/default.toml`)
4. Built-in defaults

### File Organization
- Each module has `AGENTS.md` for development guidance
- Integration tests in `tests/` directory
- Unit tests in-module `#[cfg(test)]`
- Examples in `examples/` with corresponding YAML templates

## ANTI-PATTERNS (THIS PROJECT)

### Forbidden
- **Type suppression**: Never use `as any`, `@ts-ignore`, `@ts-expect-error`
- **Unwrap in production**: All public code returns `Result`
- **Empty catch blocks**: Always handle or propagate errors
- **Hardcoded paths**: Use configuration or environment variables
- **Blocking I/O in async**: All operations must be async
- **Deleting tests to "pass"**: Fix the code, not the tests

### Disabled Features
- **WASM plugins**: Temporarily disabled (wasmtime commented)
- **MCP server**: Stub implementation only (dependencies commented out)
- **Plugin sandboxing**: In development (not fully implemented)

## UNIQUE STYLES

### Workflow Orchestration
- DAG-based execution with `petgraph`
- Parallel execution via semaphores (default: 4 concurrent)
- Checkpoint/recovery support (every 5 minutes)
- Comprehensive audit logging

### Plugin Architecture
- Multi-language: Native (Rust), Python, Node.js, Docker, WASM
- Tool registry with version management
- Template-based parameter expansion
- Human-in-the-loop decision support

### TUI Implementation
- Event-driven architecture with action system
- Reactive widgets with real-time updates
- System monitoring (CPU, memory, disk, network)
- Performance optimization with virtualization

### File Management
- AI-powered classification with confidence scoring
- Batch processing with progress tracking
- Human decision integration (accept/reject/modify)
- Experimental mode for safe testing

## COMMANDS
```bash
# Development
cargo build                    # Build debug
cargo build --release          # Build release
cargo test                     # Run all tests
cargo clippy                   # Lint code
cargo fmt                      # Format code

# Running
cargo run -- --help            # Show CLI help
cargo run -- workflow execute <file>  # Execute workflow
cargo run -- tui               # Start TUI
cargo run -- server            # Start MCP server (stub)

# Testing
cargo test -- --nocapture       # With output
cargo test -- --test-threads=1 # Single thread
cargo test property_tests       # Property-based only
cargo test --lib                # Library tests only
```

## NOTES

### Gotchas
- **MCP server**: Compiles but non-functional (stub implementation)
- **WASM support**: Commented out in Cargo.toml
- **Chinese docs**: `docs-zh/` for Chinese documentation
- **Legacy scripts**: `scripts/` contains Python reference code
- **Hot reload**: Config changes detected automatically in CLI mode
- **AGENTS.md files**: 15 files exist, all updated with latest findings

### Testing
- 298+ tests (unit, integration, property-based, performance)
- Property-based tests for critical paths (templates, versioning, TUI)
- Fixtures for isolation (`tempfile::TempDir`)
- All async tests use `#[tokio::test]`
- Test coverage: ~70% of codebase

### Performance
- Moka cache for high-performance LRU caching
- LanceDB optional (feature flag `lancedb`)
- Profiling built-in (disabled by default)
- Metrics collection (optional Prometheus export)
- Virtual scrolling for large datasets in TUI

### Development
- Rust 1.70+ (2021 Edition) required
- Use `init_logging()` for consistent tracing setup
- All public APIs return `Result<T, WorkflowError>`
- Follow import order strictly
- AGENTS.md files in each module for development guidance

### Documentation
- Each module has `AGENTS.md`
- Examples have corresponding `.yaml` templates
- `docs/INDEX.md` is documentation hub
- `examples/templates/` has production workflows
- Chinese documentation in `docs-zh/`

### AGENTS.MD Files
- **Total**: 15 files across the project
- **Root**: Project overview and quick reference
- **Modules**: Module-specific development guides
- **Subdirectories**: Detailed implementation guides
- **Tests**: Testing patterns and best practices
- **Examples**: Usage patterns and tutorials

## MODULE HIERARCHY

### Core Layer
```
src/
├── config.rs       # Hierarchical configuration (env > CLI > file > defaults)
├── core.rs         # Shared types, ExecutionContext, RetryPolicy
└── error.rs        # WorkflowError with 20+ variants, Result type
```

### Application Layer
```
src/
├── workflow/       # DAG execution engine (15 files, 2,404 lines in engine.rs)
│   ├── executor/  # Execution strategies (basic, parallel, retry)
│   ├── component/ # Workflow components (parallel, switch, loop)
│   ├── context/   # Execution context and data slots
│   └── state/     # State management and checkpoints
├── tools/         # Tool system (6 files, 1,914 lines in ac_automaton.rs)
│   └── algo/      # Algorithm implementations (Aho-Corasick)
└── plugins/       # Multi-language plugins
    └── file_management/ # File operations plugin (20 files, 3,604 lines)
```

### Infrastructure Layer
```
src/
├── storage/        # Persistence and caching (5 files)
└── performance/    # Optimization and metrics (6 files)
```

### Interface Layer
```
src/
└── interfaces/
    ├── cli/       # Command-line interface (5 files, 1,583 lines in app.rs)
    └── tui/       # Terminal UI (25 files, 3,275 lines in layout.rs)
        └── widgets/  # TUI components (8 widgets, 3,553 lines in plugin_manager.rs)
```
src/
├── config.rs       # Configuration management
├── core.rs         # Core types and traits
└── error.rs        # Error types
```

### Application Layer
```
src/
├── workflow/       # DAG execution engine
│   ├── executor/  # Execution strategies
│   ├── component/ # Workflow components
│   ├── context/   # Execution context
│   └── state/     # State management
├── tools/         # Tool registry and nodes
│   └── algo/      # Algorithm implementations
└── plugins/       # Multi-language plugins
    └── file_management/ # File operations plugin
```

### Infrastructure Layer
```
src/
├── storage/        # Persistence and caching
└── performance/    # Optimization and metrics
```

### Interface Layer
```
src/
└── interfaces/
    ├── cli/       # Command-line interface
    └── tui/       # Terminal UI
        └── widgets/  # TUI components (8 widgets)
```

## QUICK REFERENCE

### Key Re-exports
```rust
pub use crate::config::{Config, ConfigManager, CliConfigOverrides};
pub use crate::core::*;
pub use crate::error::{Result, WorkflowError};
pub use crate::workflow::{WorkflowDefinition, WorkflowEngine, ExecutionManager};
pub use crate::tools::{ToolNode, ToolRegistry};
pub use crate::performance::{PerformanceManager, PerformanceConfig};
```

### Common Patterns
```rust
// Initialize logging
use workflow_toolkit::init_logging;
init_logging()?;

// Load config with priority
use workflow_toolkit::Config;
let config = Config::load_with_priority()?;

// Create engine
use workflow_toolkit::workflow::DefaultWorkflowEngine;
let engine = DefaultWorkflowEngine::new(config, registry, state_manager)?;

// Execute workflow
let result = engine.execute_workflow(workflow).await?;
```

## CODEBASE STATISTICS

### File and Line Counts
- **Total Rust files**: 163
- **Total lines of code**: ~110,000+
- **Files >500 lines**: 85 (complexity hotspots)
- **Maximum directory depth**: 8 levels
- **Test files with `#[cfg(test)]`**: 72
- **Public traits**: 30
- **Public structs**: 646

### Complexity Hotspots (Top 10)
1. `src/plugins/file_management/utils.rs`: 3,604 lines
2. `src/interfaces/tui/widgets/plugin_manager.rs`: 3,553 lines
3. `src/interfaces/tui/layout.rs`: 3,275 lines
4. `src/interfaces/tui/widgets/system_status.rs`: 2,687 lines
5. `src/interfaces/tui_legacy.rs`: 2,506 lines
6. `src/workflow/engine.rs`: 2,404 lines
7. `src/tools/algo/ac_automaton.rs`: 1,914 lines
8. `src/plugins/file_management/ac_automaton.rs`: 1,914 lines
9. `src/plugins/file_management/registry.rs`: 1,773 lines
10. `src/interfaces/tui/sync.rs`: 1,715 lines

### Module Distribution
| Directory | Files | Lines (Top File) | Purpose |
|-----------|-------|------------------|---------|
| `src/interfaces/tui` | 25 | 3,275 (layout.rs) | Terminal UI |
| `src/plugins/file_management` | 20 | 3,604 (utils.rs) | File operations |
| `src/workflow` | 15 | 2,404 (engine.rs) | Workflow engine |
| `src/interfaces/cli` | 5 | 1,583 (app.rs) | CLI interface |
| `src/tools` | 6 | 1,914 (ac_automaton.rs) | Tool system |
| `src/performance` | 6 | Comprehensive | Optimization |
| `src/storage` | 5 | Comprehensive | Persistence |
| `src/plugins` | 10 | Comprehensive | Plugin system |
| `tests` | 12 | 1,697 (tui tests) | Test suite |
| `examples` | 19 | Comprehensive | Examples |

### Test Coverage
- **Unit tests**: In-module `#[cfg(test)]` (72 files)
- **Integration tests**: `tests/` directory (12 files)
- **Property-based tests**: Randomized input testing
- **E2E tests**: Full system workflows
- **Total test lines**: ~15,000+ lines

### Performance Characteristics
- **Build time**: ~2-3 minutes (release)
- **Test execution**: ~30 seconds (full suite)
- **Memory usage**: ~50-100MB typical
- **Binary size**: ~15-20MB (release)

## DEVELOPMENT WORKFLOW

### Code Changes
1. **Read existing AGENTS.md**: Understand module conventions
2. **Follow patterns**: Match existing code style
3. **Add tests**: Unit tests for new functionality
4. **Update docs**: Update AGENTS.md if needed
5. **Run verification**: `cargo test && cargo clippy && cargo fmt`

### Adding New Features
1. **Check existing patterns**: Look for similar features
2. **Follow conventions**: Import order, error handling, async patterns
3. **Add documentation**: Update AGENTS.md for new modules
4. **Write tests**: Comprehensive test coverage
5. **Update examples**: Add example if feature is user-facing

### Bug Fixes
1. **Minimal changes**: Fix only the bug, don't refactor
2. **Add regression test**: Prevent future occurrences
3. **Verify no side effects**: Run full test suite
4. **Update documentation**: Note the fix in AGENTS.md if needed

## SEE ALSO

- [README.md](README.md) - User documentation
- [GLOBAL_RULES.md](GLOBAL_RULES.md) - Global rules and guidelines
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Quick reference guide
- [docs/INDEX.md](docs/INDEX.md) - Complete documentation index
- [AGENTS.md files](#subdirectories-with-agents.md) - Module-specific guides
- [Examples](examples/AGENTS.md) - Usage patterns and tutorials

## SUBDIRECTORIES WITH AGENTS.md

See module-specific guidance:
- [src/](src/AGENTS.md) - Core library modules
- [src/workflow/](src/workflow/AGENTS.md) - DAG workflow engine (executor, component, context, state subdirs)
- [src/plugins/](src/plugins/AGENTS.md) - Multi-language plugin system
- [src/plugins/file_management/](src/plugins/file_management/AGENTS.md) - File operations plugin (20 files, 3.6k+ lines)
- [src/tools/](src/tools/AGENTS.md) - Tool system (algo subdirectory)
- [src/storage/](src/storage/AGENTS.md) - Storage layer
- [src/performance/](src/performance/AGENTS.md) - Performance optimization
- [src/interfaces/](src/interfaces/AGENTS.md) - User interfaces
- [src/interfaces/cli/](src/interfaces/cli/AGENTS.md) - Command-line interface (5 files, 1.5k+ lines)
- [src/interfaces/tui/](src/interfaces/tui/AGENTS.md) - Terminal user interface (25 files, 3.2k+ lines)
- [src/interfaces/tui/widgets/](src/interfaces/tui/widgets/AGENTS.md) - TUI widgets (8 widgets, 3.5k+ lines)
- [examples/](examples/AGENTS.md) - Comprehensive examples (23 examples)
- [examples/templates/](examples/templates/AGENTS.md) - Workflow templates (114 lines + docs)
- [tests/](tests/AGENTS.md) - Test suite (12 files, 1.7k+ lines in tui tests)

## ARCHITECTURE DECISIONS

### Why Multiple AGENTS.md Files?
1. **Modularity**: Each module has its own development guide
2. **Scalability**: Prevents single large document
3. **Maintainability**: Easier to update module-specific docs
4. **Discoverability**: Developers can find relevant docs quickly

### File Distribution Strategy
- **Root**: Project overview, quick reference
- **Module roots**: High-level architecture, key components
- **Subdirectories**: Detailed implementation guides
- **Tests**: Testing patterns and best practices
- **Examples**: Usage patterns and tutorials

### Update Workflow
1. **Code changes**: Update relevant AGENTS.md files
2. **New modules**: Create AGENTS.md with overview
3. **Major refactors**: Update all affected AGENTS.md files
4. **Review**: Verify cross-references remain valid
### Update Workflow
1. **Code changes**: Update relevant AGENTS.md files
2. **New modules**: Create AGENTS.md with overview
3. **Major refactors**: Update all affected AGENTS.md files
4. **Review**: Verify cross-references remain valid

### Update Workflow
1. **Code changes**: Update relevant AGENTS.md files
2. **New modules**: Create AGENTS.md with overview
3. **Major refactors**: Update all affected AGENTS.md files
4. **Review**: Verify cross-references remain valid
### Update Workflow
1. **Code changes**: Update relevant AGENTS.md files
2. **New modules**: Create AGENTS.md with overview
3. **Major refactors**: Update all affected AGENTS.md files
4. **Review**: Verify cross-references remain valid


