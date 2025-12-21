# rt-tools Design Document

## 1. Module Overview
`rt-tools` contains concrete tool implementations that provide the built-in functionality for the Rust Toolbox platform. All tools must implement the `rt-core::Tool` trait and follow the established patterns for internationalization, schema definition, and error handling.

The module is organized into functional categories (file operations, text processing) with each tool having its own subdirectory containing implementation, localization resources, and documentation.

## 2. Architecture

### 2.1 Tool Registration System
Tools are automatically registered using the `register_tool!` macro, which handles:
- Tool discovery and instantiation
- Integration with the core tool registry
- Automatic schema generation and validation

### 2.2 Internationalization Framework
All tools use the unified `ToolI18n` system that provides:
- JSON-based localization resources
- Runtime locale switching
- Schema field title injection for dynamic UI generation
- Consistent user guide formatting

### 2.3 Utils Module
The `utils` module provides shared infrastructure for tool development:
- **Core Type Re-exports**: Common types from `rt-core` (Tool, Locale, PersistenceManager, etc.)
- **I18n Helper**: `ToolI18n` struct for loading and managing localization resources
- **Consistent API**: Unified interface for tool development

## 3. Available Tools

### 3.1 File Operations (`file`)

#### Move Folder (`file.move_folder`)
- **Name**: `file.move_folder`
- **Description**: Move or rename folders with collision handling and validation
- **Features**:
  - Source and destination path validation
  - Overwrite protection with confirmation
  - Recursive directory moving
  - File count reporting
- **Input Schema**:
  ```json
  {
    "source": "path/to/source",
    "destination": "path/to/dest",
    "overwrite": false
  }
  ```
- **Output Schema**:
  ```json
  {
    "success": true,
    "moved_files": 10
  }
  ```
- **Error Conditions**:
  - Source path does not exist
  - Destination exists and overwrite is false
  - Permission denied
  - Invalid path format

### 3.2 Text Operations (`text`)

#### AC Automaton (`text.ac_automaton`)
- **Name**: `text.ac_automaton`
- **Description**: Aho-Corasick pattern matching with multi-pattern support
- **Features**:
  - Multi-pattern string matching using Aho-Corasick algorithm
  - Pattern management (add, remove, list operations)
  - Case-sensitive and case-insensitive matching
  - Parallel text processing support
  - Performance timing and statistics
- **Input Schema**:
  ```json
  {
    "action": "match",
    "patterns": ["pattern1", "pattern2"],
    "texts": ["text to search"],
    "ignore_case": false,
    "parallel": false,
    "confirm": false
  }
  ```
- **Output Schema**:
  ```json
  {
    "success": true,
    "message": "操作成功",
    "results": [
      {
        "pattern": "pattern1",
        "start": 0,
        "end": 8
      }
    ],
    "patterns": ["pattern1", "pattern2"],
    "elapsed_ms": 15
  }
  ```
- **Supported Actions**:
  - `add`: Add patterns to the automaton
  - `remove`: Remove patterns (requires confirmation)
  - `list`: List current patterns
  - `match`: Perform pattern matching on texts
  - `save`/`load`: Persistence operations (planned)

#### Chinese Converter (`text.convert_chinese`)
- **Name**: `text.convert_chinese`
- **Description**: Traditional/Simplified Chinese text conversion
- **Features**:
  - Multiple conversion modes (Simplified ↔ Traditional)
  - Regional variants support (Taiwan, Hong Kong)
  - Phrase-level conversion accuracy
- **Input Schema**:
  ```json
  {
    "text": "简体中文",
    "mode": "s2t"
  }
  ```
- **Output Schema**:
  ```json
  {
    "converted": "繁體中文"
  }
  ```
- **Conversion Modes**:
  - `s2t`: Simplified to Traditional
  - `t2s`: Traditional to Simplified
  - `s2tw`: Simplified to Taiwan Traditional
  - `tw2s`: Taiwan Traditional to Simplified
  - `s2hk`: Simplified to Hong Kong Traditional
  - `hk2s`: Hong Kong Traditional to Simplified
  - `s2twp`: Simplified to Taiwan Traditional with phrases
  - `tw2sp`: Taiwan Traditional to Simplified with phrases

## 4. Project Structure


#### AC Automaton (`text.ac_automaton`)
- **Name**: `text.ac_automaton`
- **Description**: Aho-Corasick 多模式匹配工具。
- **Input Schema**:
  ```json
  {
    "action": "match", // Enum: add, remove, list, match, save, load
    "patterns": ["pattern1", "pattern2"],
    "texts": ["text to match"],
    "confirm": false,
    "ignore_case": true,
    "parallel": false
  }
  ```
- **Output Schema**:
  ```json
  {
    "success": true,
    "message": "...",
    "results": [
      { "pattern": "...", "start": 0, "end": 5 }
    ],
    "patterns": ["..."],
    "elapsed_ms": 10
  }
  ```

## 3. 结构 (Structure)

```
rt-tools/
├── Cargo.toml                    # Dependencies and metadata
├── DESIGN.md                     # This architecture document
├── PUBLIC_TOOLS.md               # Public API documentation
└── src/
    ├── lib.rs                    # Tool registration and exports
    ├── utils/                    # Shared utilities module
    │   ├── mod.rs                # Core type re-exports
    │   └── i18n.rs               # Internationalization helper
    ├── file/                     # File operation tools
    │   ├── mod.rs                # File category module
    │   └── move_folder/          # Move folder tool
    │       ├── DESIGN.md         # Tool-specific design
    │       ├── mod.rs            # Implementation
    │       └── locales/          # Localization resources
    │           ├── tool.en.json  # English translations
    │           └── tool.zh-CN.json # Chinese translations
    └── text/                     # Text processing tools
        ├── mod.rs                # Text category module
        ├── ac_automaton/         # AC automaton tool
        │   ├── mod.rs            # Implementation
        │   ├── tests.rs          # Unit tests
        │   └── locales/          # Localization resources
        │       ├── tool.en.json  # English translations
        │       └── tool.zh-CN.json # Chinese translations
        └── convert_chinese/      # Chinese converter tool
            ├── DESIGN.md         # Tool-specific design
            ├── mod.rs            # Implementation
            └── locales/          # Localization resources
                ├── tool.en.json  # English translations
                └── tool.zh-CN.json # Chinese translations
```

## 5. Utils Module Architecture

The `utils` module provides essential infrastructure for tool development:

### 5.1 Core Type Re-exports
Provides unified access to commonly used types from `rt-core`:
- `Tool`: Core tool trait
- `Locale`: Language enumeration
- `Result`, `CoreError`: Error handling types
- `PersistenceManager`: Data storage interface
- `WorkflowEngine`: Workflow execution engine
- `WorkflowDefinition`, `WorkflowStatus`, `WorkflowInstance`: Workflow types

### 5.2 Internationalization Helper (`ToolI18n`)
Manages localization resources for tools:
- **JSON-based Configuration**: Loads translations from embedded JSON files
- **Runtime Locale Switching**: Supports dynamic language changes
- **Schema Integration**: Injects localized field titles into JSON schemas
- **Structured Data**: Handles display names, descriptions, user guides, and field titles

### 5.3 Usage Pattern
Tools should import utilities through the unified interface:
```rust
use crate::utils::{Tool, Locale, ToolI18n, Result, CoreError};
```

## 6. Internationalization System

### 6.1 JSON Structure
Each tool maintains localization files with the following structure:
```json
{
  "display_name": "Tool Display Name",
  "description": "Tool description for users",
  "user_guide": "Markdown-formatted user guide",
  "input_schema": {
    "field_name": { "title": "Localized Field Title" }
  },
  "output_schema": {
    "field_name": { "title": "Localized Output Title" }
  },
  "extra": {
    "custom_key": "Custom localized value"
  }
}
```

### 6.2 Schema Integration
The `ToolI18n` system automatically:
- Injects localized titles into JSON schemas
- Provides enum label translations for dropdown fields
- Supports custom field mappings through the `extra` section
- Maintains consistency across CLI and GUI interfaces

### 6.3 Supported Locales
- **English (`en`)**: Primary language for development and documentation
- **Chinese (`zh-CN`)**: Simplified Chinese for Chinese users

## 7. Tool Development Guidelines

### 7.1 Implementation Requirements
1. **Trait Implementation**: All tools must implement `rt_core::Tool`
2. **Registration**: Use `register_tool!` macro for automatic discovery
3. **Async Support**: All tool execution must be async-compatible
4. **Error Handling**: Use `rt_core::Result` and `CoreError` types
5. **Schema Validation**: Input/output must use `schemars::JsonSchema`

### 7.2 Testing Standards
- **Unit Tests**: Each tool should have comprehensive unit tests
- **Integration Tests**: Test tool registration and execution flow
- **Schema Validation**: Verify input/output schema correctness
- **Localization Tests**: Ensure all locales load correctly

### 7.3 Documentation Requirements
- **Tool-specific DESIGN.md**: For complex tools with multiple features
- **Inline Documentation**: Comprehensive rustdoc comments
- **User Guides**: Markdown-formatted guides in localization files
- **Example Usage**: Include practical examples in documentation

## 8. Performance Considerations

### 8.1 Lazy Loading
- Tools are instantiated only when needed
- Localization resources are loaded once per tool instance
- Schema generation is cached where possible

### 8.2 Async Execution
- All tools support async execution for non-blocking operations
- Parallel processing is available for applicable tools (e.g., AC automaton)
- Resource cleanup is handled automatically

### 8.3 Memory Management
- Tools use minimal memory footprint
- Large data structures are processed in streaming fashion where possible
- Temporary resources are properly cleaned up after execution
