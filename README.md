# Rust Toolbox (rt-box)

Rust Toolbox is a modular tool integration platform built in Rust that provides unified management and orchestration of various tools through a workflow engine. The platform features a plugin-based architecture supporting dynamic loading and extension, with both CLI and GUI interfaces.

## Core Features

- **Unified Tool Management**: Centralized tool discovery, execution, and management
- **Plugin Architecture**: Dynamic loading of external tools as plugins (executable files and WebAssembly)
- **Workflow Engine**: DAG-based workflow orchestration for automated task processing
- **Multi-language Support**: Full internationalization (i18n) for English and Chinese
- **Dual Interface**: Both command-line (rt-cli) and graphical (rt-gui) interfaces
- **Model Context Protocol (MCP)**: Standardized context management and tool calling protocol
- **Persistence Layer**: Unified data storage, caching, and configuration management

## Related Documentation

- [Architecture Design](ARCHITECTURE_DESIGN.md): Detailed architecture design, core components and deployment architecture
- [Design Document](DESIGN.md): Overall project design document, including technology stack and core principles
- [User Guide](USER_GUIDE.md): Detailed user guide, including tool library and usage methods
- [Plugin Development Guide](PLUGIN_GUIDE.md): Plugin development specifications and guidelines
- [AI Work Protocol](AI_WORK_PROTOCOL.md): AI-assisted development work specifications
- [Changelog](CHANGELOG.md): Project change history

## Project Structure

This project uses Cargo Workspace structure, containing the following core crates:

- **`rt-core`**: Core library. Defines the `Tool` trait, common data structures, plugin system, and persistence management. Supports Model Context Protocol (MCP).
- **`rt-tools`**: Built-in tool collection. Contains specific business logic tools.
- **`rt-cli`**: Command-line interface. Provides command-line based tool listing and execution functionality.
- **`rt-gui`**: Graphical interface. Provides visual tool configuration and execution interface.
- **`rt-plugin-pinyin`**: Plugin. Provides Chinese to Pinyin conversion functionality.
- **`rt-plugin-ytdlp`**: Plugin. Provides video download functionality.
- **`rt-plugin-czkawka`**: Multi-tool plugin. Provides file system utilities (duplicate files, empty directories, similar images, temporary files, broken symlinks).

## Available Tools

### File Operations (`file`)
- **`file.move_folder`**: Move or rename folders (supports moving to existing directory interiors)
- **`file.duplicates`**: Find duplicate files in specified directories (via Czkawka plugin)
- **`file.similar_images`**: Find visually similar images (via Czkawka plugin)
- **`file.empty_directories`**: Find empty directories (via Czkawka plugin)
- **`file.temporary_files`**: Find temporary files (via Czkawka plugin)
- **`file.broken_symlinks`**: Find broken symbolic links (via Czkawka plugin)

### Text Operations (`text`)
- **`text.ac_automaton`**: Aho-Corasick pattern matching with multi-pattern support
- **`text.convert_chinese`**: Traditional/Simplified Chinese conversion
- **`text.pinyin`**: Convert Chinese text to Pinyin with or without tones (via plugin)

### Media Operations (`media`)
- **`media.ytdlp`**: Download videos and audio from YouTube and other supported sites (via plugin)

## Getting Started

### Building the Project

#### Full Build
```bash
cargo build
```

#### Build Without Plugins
If you only want to build core functionality without compiling plugins, you can use the `--workspace --exclude` parameter to exclude specific plugin packages:

```bash
# Build without any plugins
cargo build --workspace --exclude rt-plugin-pinyin --exclude rt-plugin-ytdlp --exclude rt-plugin-czkawka

# Build only core libraries and CLI
cargo build --package rt-core --package rt-tools --package rt-cli
```

Building without plugins can speed up the build process, especially when developing core functionality.

### Running Tests
```bash
cargo test
```

### Running CLI
```bash
# List available tools
cargo run --bin rt-cli -- list

# Run a specific tool
cargo run --bin rt-cli -- run <tool_name>
```

### Running GUI
```bash
cargo run --bin rt-gui
```

### Building Release Executable
To generate a standalone executable file for distribution, use release mode:

```bash
# Build release version
cargo build --release --bin rt-gui
```

After building, the executable is located at:
`target/release/rt-gui.exe` (Windows) or `target/release/rt-gui` (Linux/macOS)

> **Note**: Since font files are embedded in the program, the generated executable is completely standalone and can run on other machines without requiring the `assets` directory.

### MCP Server Support

The platform includes MCP (Model Context Protocol) server support for external system integration:

```bash
# Start MCP server (REST API + WebSocket)
cargo run --bin rt-cli -- server --port 8080

# Access REST API endpoints
curl http://localhost:8080/api/tools
curl http://localhost:8080/api/workflows

# WebSocket endpoint available at ws://localhost:8080/ws
```

## Development Standards
Please refer to [AI_WORK_PROTOCOL.md](AI_WORK_PROTOCOL.md).

## Target Users

- Developers needing automated tool workflows
- System administrators managing file operations
- Content creators working with media files
- Anyone requiring batch text processing or file management

## Technology Stack

- **Language**: Rust 2021 Edition
- **Async Runtime**: Tokio
- **Serialization**: Serde with JSON/YAML support
- **CLI Framework**: Clap v4 with derive macros
- **GUI Framework**: egui (cross-platform, immediate mode)
- **Web Framework**: Warp (for MCP server REST API + WebSocket)
- **Storage**: Sled embedded database with moka caching
- **Plugin System**: Process-based and WebAssembly (WASM) plugins

## License
GNU Affero General Public License v3
