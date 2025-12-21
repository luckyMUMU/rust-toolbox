# Project Structure & Organization

## Workspace Layout

```
rt-box/                          # Root workspace
├── Cargo.toml                   # Workspace definition
├── .gitignore
├── LICENSE                      # GNU AGPL v3
├── README.md                    # Project overview
├── USER_GUIDE.md               # User documentation
├── PLUGIN_GUIDE.md             # Plugin development guide
├── AI_WORK_PROTOCOL.md         # AI development standards
├── ARCHITECTURE_DESIGN.md      # Detailed architecture docs
├── DESIGN.md                   # Overall design principles
├── CHANGELOG.md                # Version history
├── input.json                  # Sample input file
├── assets/                     # Static resources
│   ├── ArialBlack.ttf         # Embedded fonts
│   └── SimHei.ttf
├── plugins/                    # Plugin binaries directory
│   ├── rt-plugin-czkawka.exe
│   ├── rt-plugin-pinyin.exe
│   └── rt-plugin-ytdlp.exe
├── logs/                       # Runtime logs
├── static/                     # Static web assets
├── tmp/                        # Temporary files
├── dist/                       # Distribution builds
└── target/                     # Cargo build artifacts
```

## Core Modules

### rt-core/ - Foundation Library
```
rt-core/
├── Cargo.toml
├── DESIGN.md                   # Core architecture design
├── PERSISTENCE_DESIGN.md       # Data layer design
├── WORKFLOW_DESIGN.md          # Workflow engine design
└── src/
    ├── lib.rs                  # Public API exports
    ├── error.rs                # Unified error types
    ├── locale.rs               # Language enumeration
    ├── tool.rs                 # Core Tool trait
    ├── workflow.rs             # Workflow engine
    ├── config/                 # Configuration management
    │   ├── mod.rs
    │   ├── domain.rs           # Config domain models
    │   ├── port.rs             # Config interfaces
    │   ├── service.rs          # Config business logic
    │   └── adapter/            # Config implementations
    │       ├── mod.rs
    │       ├── cache.rs        # Config caching
    │       ├── env.rs          # Environment variables
    │       └── file.rs         # File-based config
    ├── logger/                 # Logging system
    │   ├── mod.rs
    │   ├── domain.rs           # Log domain models
    │   ├── port.rs             # Log interfaces
    │   ├── service.rs          # Log business logic
    │   └── adapter/            # Log implementations
    │       ├── mod.rs
    │       ├── formatter.rs    # Log formatting
    │       └── sink.rs         # Log output targets
    ├── mcp/                    # Model Context Protocol
    │   ├── mod.rs
    │   ├── context.rs          # MCP context management
    │   ├── manager.rs          # Context lifecycle
    │   ├── node.rs             # MCP node abstraction
    │   ├── request.rs          # MCP request types
    │   ├── response.rs         # MCP response types
    │   ├── workflow.rs         # MCP workflow integration
    │   └── tests.rs            # MCP unit tests
    ├── persistence/            # Data persistence
    │   ├── mod.rs
    │   ├── cache.rs            # In-memory caching
    │   ├── file_ops.rs         # File operations
    │   ├── manager.rs          # Persistence coordinator
    │   └── sled_backend.rs     # Sled database backend
    ├── plugin/                 # Plugin system
    │   ├── mod.rs
    │   ├── manifest.rs         # Plugin metadata
    │   ├── process.rs          # Process-based plugins
    │   └── wasm.rs             # WebAssembly plugins
    ├── server/                 # MCP server implementation
    │   ├── mod.rs
    │   ├── api.rs              # REST API endpoints
    │   ├── mcp.rs              # MCP protocol handlers
    │   └── websocket.rs        # WebSocket support
    └── service/                # Service layer
        ├── mod.rs
        ├── implementation.rs   # Service implementations
        ├── manager.rs          # Service coordinator
        └── port.rs             # Service interfaces
```

### rt-tools/ - Built-in Tools
```
rt-tools/
├── Cargo.toml
├── DESIGN.md                   # Tools architecture
├── PUBLIC_TOOLS.md             # Available tools documentation
└── src/
    ├── lib.rs                  # Tool registration
    ├── file/                   # File operation tools
    │   ├── mod.rs
    │   └── move_folder/        # Folder move/rename tool
    │       ├── DESIGN.md       # Tool-specific design
    │       ├── mod.rs          # Implementation
    │       └── locales/        # Internationalization
    │           ├── tool.en.json
    │           └── tool.zh-CN.json
    ├── text/                   # Text processing tools
    │   ├── mod.rs
    │   ├── ac_automaton/       # Aho-Corasick pattern matching
    │   │   ├── mod.rs
    │   │   ├── tests.rs
    │   │   └── locales/
    │   │       ├── tool.en.json
    │   │       └── tool.zh-CN.json
    │   └── convert_chinese/    # Chinese text conversion
    │       ├── DESIGN.md
    │       ├── mod.rs
    │       └── locales/
    │           ├── tool.en.json
    │           └── tool.zh-CN.json
    └── utils/                  # Shared utilities
        ├── mod.rs
        └── i18n.rs             # Internationalization helpers
```

### rt-cli/ - Command Line Interface
```
rt-cli/
├── Cargo.toml
├── DESIGN.md                   # CLI architecture
└── src/
    └── main.rs                 # CLI entry point and commands
```

### rt-gui/ - Graphical Interface
```
rt-gui/
├── Cargo.toml
├── DESIGN.md                   # GUI architecture
└── src/
    ├── main.rs                 # GUI entry point
    ├── layout_manager.rs       # UI layout management
    ├── settings.rs             # Settings management
    └── components/             # UI components
        ├── mod.rs
        ├── base/               # Basic UI elements
        │   ├── mod.rs
        │   ├── button.rs
        │   ├── icon.rs
        │   ├── label.rs
        │   └── progress.rs
        ├── data_display/       # Data presentation
        │   ├── mod.rs
        │   ├── card.rs
        │   ├── list.rs
        │   └── table.rs
        ├── form/               # Input forms
        │   ├── mod.rs
        │   ├── checkbox.rs
        │   ├── dropdown.rs
        │   ├── file_input.rs
        │   ├── number_input.rs
        │   ├── slider.rs
        │   ├── text_input.rs
        │   └── toggle_switch.rs
        ├── layout/             # Layout components
        └── navigation/         # Navigation elements
            ├── mod.rs
            ├── sidebar.rs
            └── tab_nav.rs
```

## Plugin Projects

### Plugin Structure Template
```
rt-plugin-<name>/
├── Cargo.toml                  # Plugin dependencies
├── README.md                   # Plugin documentation
├── input.json                  # Sample input (optional)
└── src/
    ├── main.rs                 # Plugin entry point
    ├── i18n.rs                 # Internationalization
    └── tools/                  # Tool implementations (if multi-tool)
        ├── mod.rs
        └── <tool_name>.rs
└── locales/                    # Internationalization resources
    ├── tool.en.json
    └── tool.zh.json
```

## Development Artifacts

### Documentation Hierarchy
- **Root Level**: Project overview, user guides, development protocols
- **Module Level**: Architecture and design documents (`DESIGN.md`)
- **Tool Level**: Individual tool documentation and localization

### Configuration & Metadata
- **Workspace**: `Cargo.toml` defines member crates and shared dependencies
- **Individual Crates**: Each has its own `Cargo.toml` with specific dependencies
- **Internationalization**: JSON files in `locales/` directories
- **Plugin Metadata**: Exposed via `plugin spec` command

### Build Artifacts
- **Development**: `target/debug/` for fast iteration
- **Release**: `target/release/` for optimized binaries
- **Plugins**: Copied to `plugins/` directory for runtime discovery
- **Distribution**: `dist/` for packaged releases

## Naming Conventions

### Crates & Modules
- **Core Library**: `rt-core`
- **Built-in Tools**: `rt-tools`
- **Applications**: `rt-cli`, `rt-gui`
- **Plugins**: `rt-plugin-<name>`

### Tool Identifiers
- **Format**: `category.tool_name`
- **Examples**: `file.move_folder`, `text.ac_automaton`, `media.ytdlp`

### File Naming
- **Design Documents**: `DESIGN.md` (uppercase)
- **Internationalization**: `tool.{locale}.json`
- **Plugin Binaries**: `rt-plugin-<name>.exe` (Windows)

## Development Workflow

### Adding New Tools
1. Create tool directory in `rt-tools/src/{category}/{tool_name}/`
2. Write `DESIGN.md` with tool specification
3. Implement `mod.rs` with `Tool` trait
4. Create `locales/` with i18n resources
5. Register in `rt-tools/src/lib.rs`

### Plugin Development
1. Create new crate `rt-plugin-<name>`
2. Implement `plugin spec` and `plugin run` commands
3. Build and copy to `plugins/` directory
4. Test with `rt-cli list` and `rt-gui`

### Documentation Updates
- Keep `DESIGN.md` files synchronized with code changes
- Update user guides when adding new features
- Maintain changelog for version tracking