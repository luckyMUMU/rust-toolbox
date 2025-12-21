# Technology Stack & Build System

## Core Technologies

### Language & Runtime
- **Rust 2021 Edition**: Primary development language
- **Tokio**: Async runtime for concurrent operations
- **async-trait**: Async trait support

### Serialization & Schema
- **serde**: JSON/YAML serialization with derive macros
- **schemars**: JSON Schema generation for dynamic UI
- **serde_json**: JSON processing

### Error Handling
- **thiserror**: Structured error types
- **anyhow**: Error context and chaining

### CLI & GUI Frameworks
- **clap v4**: Command-line argument parsing with derive macros
- **egui**: Cross-platform GUI framework (lightweight, immediate mode)

### Web & Network
- **warp**: Web framework for MCP server (REST API + WebSocket)
- **tokio-tungstenite**: WebSocket support
- **futures-util**: Future utilities

### Storage & Persistence
- **sled**: Embedded key-value database
- **moka**: In-memory caching with async support
- **bincode**: Binary serialization for storage
- **zstd + async-compression**: Data compression

### Plugin System
- **wasmtime**: WebAssembly runtime for WASM plugins
- **wasmtime-wasi**: WASI support for WASM plugins

### Utilities
- **chrono**: Date/time handling with serde support
- **uuid**: Unique identifier generation
- **regex**: Pattern matching
- **sha2 + hex**: Cryptographic hashing
- **tempfile**: Temporary file/directory management

## Build System

### Cargo Workspace Structure
```
[workspace]
resolver = "2"
members = [
    "rt-core",      # Core library and traits
    "rt-tools",     # Built-in tools
    "rt-cli",       # Command-line interface
    "rt-gui",       # Graphical interface
    "rt-plugin-*",  # External plugins
]
```

### Common Build Commands

#### Development
```bash
# Full build (all workspace members)
cargo build

# Build without plugins (faster for core development)
cargo build --workspace --exclude rt-plugin-pinyin --exclude rt-plugin-ytdlp --exclude rt-plugin-czkawka

# Build specific components
cargo build --package rt-core --package rt-tools --package rt-cli
```

#### Testing
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test --package rt-core
```

#### Release Builds
```bash
# Build optimized release version
cargo build --release

# Build GUI executable for distribution
cargo build --release --bin rt-gui
# Output: target/release/rt-gui.exe (Windows)
```

#### Running Applications
```bash
# List available tools
cargo run --bin rt-cli -- list

# Run specific tool
cargo run --bin rt-cli -- run <tool_name>

# Launch GUI
cargo run --bin rt-gui
```

#### Plugin Development
```bash
# Build plugin
cargo build --release --package rt-plugin-<name>

# Build WASM plugin
cargo build --target wasm32-wasi --release --package rt-plugin-<name>
```

#### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

## Development Dependencies

### Required for Plugin Development
- **Pure Rust libraries preferred** (avoid C/C++ dependencies for cross-platform compatibility)
- Example: Use `ferrous-opencc` instead of `opencc-rust` (which requires pkg-config)

### Build Requirements
- Rust toolchain (latest stable)
- For WASM plugins: `wasm32-wasi` target
- For GUI: Platform-specific dependencies (handled by egui)

## Architecture Patterns

### Hexagonal Architecture
- **Core**: Business logic in `rt-core`
- **Ports**: Trait definitions for external interfaces
- **Adapters**: Concrete implementations (CLI, GUI, plugins)

### Plugin Protocol
- **Process Plugins**: External executables communicating via stdin/stdout JSON
- **WASM Plugins**: WebAssembly modules with WASI support
- **Discovery**: Auto-scan `plugins/` directory for files prefixed with `rt-plugin-`

### Internationalization
- JSON-based i18n resources in `locales/` directories
- Runtime locale switching
- Schema title injection for dynamic UI generation