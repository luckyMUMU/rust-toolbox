# Product Overview

## Rust Toolbox (rt-box)

Rust Toolbox is a modular tool integration platform built in Rust that provides unified management and orchestration of various tools through a workflow engine. The platform features a plugin-based architecture supporting dynamic loading and extension, with both CLI and GUI interfaces.

## Core Features

- **Unified Tool Management**: Centralized tool discovery, execution, and management
- **Plugin Architecture**: Dynamic loading of external tools as plugins (executable files and WebAssembly)
- **Workflow Engine**: DAG-based workflow orchestration for automated task processing
- **Multi-language Support**: Full internationalization (i18n) for English and Chinese
- **Dual Interface**: Both command-line (rt-cli) and graphical (rt-gui) interfaces
- **Model Context Protocol (MCP)**: Standardized context management and tool calling protocol
- **Persistence Layer**: Unified data storage, caching, and configuration management

## Built-in Tools

- **File Operations**: `file.move_folder` - Move/rename folders with collision handling
- **Text Processing**: 
  - `text.ac_automaton` - Aho-Corasick pattern matching with multi-pattern support
  - `text.convert_chinese` - Traditional/Simplified Chinese conversion
  - `text.pinyin` - Chinese to Pinyin conversion (via plugin)
- **Media Operations**: `media.ytdlp` - Video/audio downloading (via plugin)
- **System Utilities**: Czkawka integration for duplicate files, empty directories, etc. (via plugin)

## Target Users

- Developers needing automated tool workflows
- System administrators managing file operations
- Content creators working with media files
- Anyone requiring batch text processing or file management