# Rust Toolbox Plugin Development Guide

## 1. Overview

This guide provides comprehensive standards and documentation formats for Rust Toolbox plugin developers. Following these standards ensures seamless integration into the Rust Toolbox ecosystem and delivers excellent user experience, particularly with multi-language support, GUI auto-form generation, and MCP integration.

### 1.1 Key Features

- **Multi-Tool Plugin Support**: Single plugins can provide multiple tools with array-based metadata format
- **Model Context Protocol (MCP)**: Standardized context management and tool calling protocol
- **JSON-based Internationalization**: Comprehensive i18n system for global accessibility
- **Plugin Validation and Testing**: Automated validation and testing procedures
- **WebAssembly Support**: Cross-platform plugin deployment with enhanced security

### 1.2 Related Documentation

- [Design Document](DESIGN.md): Overall project design, technology stack and core principles
- [Architecture Design](ARCHITECTURE_DESIGN.md): Detailed architecture design, core components and deployment architecture
- [User Guide](USER_GUIDE.md): Comprehensive user guide including tool library and usage patterns
- [AI Work Protocol](AI_WORK_PROTOCOL.md): AI-assisted development workflow standards
- [Changelog](CHANGELOG.md): Project change history

## 2. Core Principles

### 2.1 Multi-Tool Plugin Architecture

Plugins can now provide multiple tools through an array-based metadata format. This enables more efficient packaging and distribution of related functionality.

**Benefits:**
- Reduced plugin overhead for related tools
- Consistent versioning across tool families
- Simplified dependency management
- Better resource sharing between tools

### 2.2 Internationalization (i18n) Support

All user-facing text must provide multi-language versions, including tool names, descriptions, user guides, and input/output field titles. Currently supported languages include English (`en`) and Simplified Chinese (`zh-CN`).

**Requirements:**
- Display names and descriptions in all supported languages
- Localized user guides with proper formatting
- Field-level translations for GUI form generation
- Consistent terminology across all tools

### 2.3 JSON Schema Definition

Plugin input and output parameters must be defined through JSON Schema. These schemas serve multiple purposes:
- Data validation and type checking
- GUI auto-form generation
- API documentation generation
- MCP protocol integration

**Schema Features:**
- Comprehensive type definitions
- Validation constraints and rules
- Localized field titles through injection
- Enum value labeling for dropdowns

### 2.4 JSON-based Internationalization Structure

Tool internationalization is managed through JSON files. Each tool directory should contain a `locales` folder with `tool.en.json` and `tool.zh-CN.json` files.

**JSON File Structure:**
```json
{
  "display_name": "Tool Name",
  "description": "Tool Description", 
  "user_guide": "# Tool User Guide\n\nDetailed usage instructions...",
  "input_schema": {
    "field_name": { "title": "Field Title" },
    "enum_field": { 
      "title": "Enum Field",
      "enum_labels": {
        "value1": "Label 1",
        "value2": "Label 2"
      }
    }
  },
  "output_schema": {
    "result_field": { "title": "Result Field" }
  },
  "extra": {
    "validation_messages": {
      "required": "This field is required",
      "invalid_format": "Invalid format provided"
    }
  }
}
```

## 3. Plugin Project Structure

### 3.1 Single-Tool Plugin Structure

For plugins providing a single tool:

```
rt-plugin-example/
├── Cargo.toml
├── README.md
├── input.json              # Sample input for testing
├── src/
│   ├── main.rs            # Plugin entry point with spec/run commands
│   └── i18n.rs            # Internationalization helper functions
└── locales/               # Internationalization resources
    ├── tool.en.json       # English translations
    └── tool.zh-CN.json    # Chinese translations
```

### 3.2 Multi-Tool Plugin Structure

For plugins providing multiple tools (recommended for related functionality):

```
rt-plugin-czkawka/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs            # Plugin entry point
│   ├── i18n.rs            # Shared internationalization
│   └── tools/             # Tool implementations
│       ├── mod.rs         # Tool module exports
│       ├── duplicates.rs  # Duplicate files tool
│       ├── similar_images.rs
│       ├── empty_dirs.rs
│       ├── temp_files.rs
│       └── broken_symlinks.rs
└── locales/               # Shared localization resources
    ├── tool.en.json
    └── tool.zh-CN.json
```

### 3.3 Project Configuration

#### Cargo.toml Example
```toml
[package]
name = "rt-plugin-example"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rt-plugin-example"
path = "src/main.rs"

[dependencies]
rt-core = { path = "../rt-core" }
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = { version = "0.8", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"

# Tool-specific dependencies
# Add dependencies based on your tool's requirements
```

## 4. Plugin Command Specifications

### 4.1 `plugin spec` Command

Plugins must implement the `plugin spec` command, which outputs JSON metadata to stdout. The format now supports both single-tool and multi-tool plugins.

#### 4.1.1 Single-Tool Plugin Metadata

For plugins providing a single tool:

```json
{
  "name": "text.example_tool",
  "display_name": {
    "en": "Example Tool",
    "zh": "示例工具"
  },
  "description": {
    "en": "An example tool demonstrating plugin development",
    "zh": "演示插件开发的示例工具"
  },
  "user_guide": {
    "en": "# Example Tool User Guide\n\nThis tool demonstrates...",
    "zh": "# 示例工具用户指南\n\n此工具演示..."
  },
  "input_schema": {
    "type": "object",
    "properties": {
      "text": { "type": "string" },
      "mode": { 
        "type": "string",
        "enum": ["process", "validate"]
      }
    },
    "required": ["text"]
  },
  "output_schema": {
    "type": "object", 
    "properties": {
      "result": { "type": "string" },
      "success": { "type": "boolean" }
    }
  },
  "input_fields": {
    "text": { "en": "Input Text", "zh": "输入文本" },
    "mode": { "en": "Processing Mode", "zh": "处理模式" }
  },
  "output_fields": {
    "result": { "en": "Processing Result", "zh": "处理结果" },
    "success": { "en": "Success Status", "zh": "成功状态" }
  },
  "mcp_supported": false,
  "mcp_capabilities": {},
  "requires_full_context": false,
  "context_validation_rules": {},
  "author": "Plugin Developer",
  "version": "1.0.0"
}
```

#### 4.1.2 Multi-Tool Plugin Metadata (Array Format)

For plugins providing multiple tools, return a JSON array:

```json
[
  {
    "name": "file.duplicates",
    "display_name": {
      "en": "Duplicate Files",
      "zh": "重复文件"
    },
    "description": {
      "en": "Find duplicate files in specified directories",
      "zh": "在指定目录中查找重复文件"
    },
    "user_guide": {
      "en": "# Duplicate Files Tool\n\nFinds duplicate files...",
      "zh": "# 重复文件工具\n\n查找重复文件..."
    },
    "input_schema": {
      "type": "object",
      "properties": {
        "directories": {
          "type": "array",
          "items": { "type": "string" }
        },
        "min_size": { "type": "integer", "minimum": 0 }
      },
      "required": ["directories"]
    },
    "output_schema": {
      "type": "object",
      "properties": {
        "duplicate_groups": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "files": {
                "type": "array", 
                "items": { "type": "string" }
              },
              "size": { "type": "integer" }
            }
          }
        }
      }
    },
    "input_fields": {
      "directories": { "en": "Search Directories", "zh": "搜索目录" },
      "min_size": { "en": "Minimum File Size", "zh": "最小文件大小" }
    },
    "output_fields": {
      "duplicate_groups": { "en": "Duplicate File Groups", "zh": "重复文件组" }
    },
    "mcp_supported": false,
    "author": "Czkawka Team",
    "version": "1.0.0"
  },
  {
    "name": "file.similar_images",
    "display_name": {
      "en": "Similar Images",
      "zh": "相似图片"
    }
  }
]
```

### 4.2 Metadata Field Specifications

#### 4.2.1 Required Fields

- **`name`** (String): Unique tool identifier using format `category.tool_name` (e.g., `text.pinyin`, `file.duplicates`)
- **`display_name`** (Object): Localized display names
  - `en`: English display name
  - `zh`: Simplified Chinese display name (optional but recommended)
- **`description`** (Object): Localized tool descriptions
- **`user_guide`** (Object): Detailed user guides in Markdown format with localization
- **`input_schema`** (Object): JSON Schema for input parameters (used for GUI form generation)
- **`output_schema`** (Object): JSON Schema for output results (used for GUI result display)

#### 4.2.2 Optional Fields

- **`input_fields`** (Object): Field-level localization for input schema properties
  - Maps field names to localized titles for GUI form labels
  - Example: `"text": { "en": "Input Text", "zh": "输入文本" }`
- **`output_fields`** (Object): Field-level localization for output schema properties
- **`mcp_supported`** (Boolean): Whether the tool supports Model Context Protocol (default: false)
- **`mcp_capabilities`** (Object): MCP capability descriptions (required if mcp_supported is true)
- **`requires_full_context`** (Boolean): Whether tool requires complete MCP context (default: false)
- **`context_validation_rules`** (Object): MCP context validation rules
- **`author`** (String): Plugin author information
- **`version`** (String): Plugin version following semantic versioning

#### 4.2.3 MCP Integration Fields

For tools supporting Model Context Protocol:

```json
{
  "mcp_supported": true,
  "mcp_capabilities": {
    "context_aware": true,
    "stateful": false,
    "batch_processing": true
  },
  "requires_full_context": false,
  "context_validation_rules": {
    "required_fields": ["caller_id"],
    "optional_fields": ["execution_history", "environment_info"]
  }
}
```

### 4.3 Localized Title Injection in JSON Schema

The Rust Toolbox core library dynamically injects localized titles from `input_fields` and `output_fields` into the `title` properties of `input_schema` and `output_schema` at runtime based on the current locale.

#### 4.3.1 Basic Title Injection

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "text": { "type": "string" },
    "mode": { 
      "type": "string",
      "enum": ["s2t", "t2s", "s2tw", "tw2s"]
    }
  },
  "required": ["text"]
}
```

**Input Fields:**
```json
{
  "text": { "en": "Input Text", "zh": "输入文本" },
  "mode": { "en": "Conversion Mode", "zh": "转换模式" }
}
```

**Runtime Result (Chinese locale):**
```json
{
  "type": "object",
  "properties": {
    "text": { 
      "type": "string", 
      "title": "输入文本" 
    },
    "mode": { 
      "type": "string",
      "enum": ["s2t", "t2s", "s2tw", "tw2s"],
      "title": "转换模式"
    }
  },
  "required": ["text"]
}
```

#### 4.3.2 Enum Label Injection

For enum fields, you can provide localized labels for dropdown options:

**Schema with Enum:**
```json
{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": ["add", "remove", "list", "match"]
    }
  }
}
```

**Localization with Enum Labels:**
```json
{
  "action": { 
    "en": "Action Type", 
    "zh": "操作类型",
    "enum_labels": {
      "add": { "en": "Add Pattern", "zh": "添加模式" },
      "remove": { "en": "Remove Pattern", "zh": "删除模式" },
      "list": { "en": "List Patterns", "zh": "列出模式" },
      "match": { "en": "Match Text", "zh": "匹配文本" }
    }
  }
}
```

**Runtime Result:**
```json
{
  "type": "object",
  "properties": {
    "action": {
      "type": "string",
      "enum": ["add", "remove", "list", "match"],
      "title": "操作类型",
      "x-enum-labels": {
        "add": "添加模式",
        "remove": "删除模式", 
        "list": "列出模式",
        "match": "匹配文本"
      }
    }
  }
}
```

## 5. `plugin run` Command Specification

### 5.1 Basic Run Command

Plugins must implement the `plugin run` command for tool execution:

- **Command**: `path/to/plugin run`
- **Stdin**: JSON string conforming to `input_schema`
- **Stdout**: JSON string conforming to `output_schema`
- **Stderr**: Error logs for debugging (unstructured)
- **Exit Code**: `0` for success, non-zero for failure

### 5.2 Multi-Tool Plugin Execution

For multi-tool plugins, the input JSON must include a `tool_name` field to specify which tool to execute:

**Input Format:**
```json
{
  "tool_name": "file.duplicates",
  "directories": ["/home/user/documents"],
  "min_size": 1024
}
```

**Alternative Detection Method:**
If `tool_name` is not provided, plugins can implement tool detection based on input parameters:

```rust
// Example tool detection logic
let tool_name = if let Some(tool_name) = input_value.get("tool_name").and_then(|v| v.as_str()) {
    tool_name
} else {
    // Detect tool based on unique parameters
    if input_value.get("threshold").is_some() {
        "file.similar_images"
    } else if input_value.get("min_size").is_some() {
        "file.duplicates"
    } else {
        "file.empty_directories" // default
    }
};
```

### 5.3 Error Handling

Plugins should provide structured error information:

**Success Response:**
```json
{
  "success": true,
  "data": {
    "result": "Processing completed",
    "items_processed": 42
  }
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_INPUT",
    "message": "Directory path does not exist: /invalid/path",
    "details": {
      "field": "directories",
      "value": "/invalid/path"
    }
  }
}
```

### 5.4 Performance Considerations

- **Streaming Output**: For long-running operations, consider streaming progress updates
- **Memory Management**: Handle large datasets efficiently to avoid memory issues
- **Timeout Handling**: Implement reasonable timeouts for external operations
- **Resource Cleanup**: Ensure proper cleanup of temporary files and resources

## 6. JSON-based Internationalization System

### 6.1 Internationalization Architecture

The modern Rust Toolbox plugin system uses JSON-based internationalization for better maintainability and easier translation management.

#### 6.1.1 Localization File Structure

**English Localization (`locales/tool.en.json`):**
```json
{
  "display_name": "Chinese to Pinyin Converter",
  "description": "Convert Chinese text to Pinyin with optional tone marks",
  "user_guide": "# Chinese to Pinyin Converter\n\n## Overview\nThis tool converts Chinese characters to their Pinyin representations...\n\n## Usage\n1. Enter Chinese text in the input field\n2. Choose whether to include tone marks\n3. Click convert to get the Pinyin result",
  "input_schema": {
    "text": { "title": "Chinese Text" },
    "tone": { "title": "Include Tone Marks" }
  },
  "output_schema": {
    "pinyin": { "title": "Pinyin Result" }
  },
  "extra": {
    "validation_messages": {
      "empty_text": "Text cannot be empty",
      "invalid_characters": "Text contains invalid characters"
    },
    "help_text": {
      "tone_explanation": "Tone marks help with pronunciation accuracy"
    }
  }
}
```

**Chinese Localization (`locales/tool.zh-CN.json`):**
```json
{
  "display_name": "中文转拼音工具",
  "description": "将中文文本转换为拼音，支持可选声调标记",
  "user_guide": "# 中文转拼音工具\n\n## 概述\n此工具将中文字符转换为对应的拼音表示...\n\n## 使用方法\n1. 在输入框中输入中文文本\n2. 选择是否包含声调标记\n3. 点击转换获取拼音结果",
  "input_schema": {
    "text": { "title": "中文文本" },
    "tone": { "title": "包含声调标记" }
  },
  "output_schema": {
    "pinyin": { "title": "拼音结果" }
  },
  "extra": {
    "validation_messages": {
      "empty_text": "文本不能为空",
      "invalid_characters": "文本包含无效字符"
    },
    "help_text": {
      "tone_explanation": "声调标记有助于发音准确性"
    }
  }
}
```

### 6.2 Rust Implementation for Internationalization

#### 6.2.1 Modern i18n.rs Implementation

```rust
use rt_core::Locale;
use serde_json::{Value, Map};
use std::collections::HashMap;

/// Internationalization helper for plugins
pub struct I18nHelper {
    en_data: Value,
    zh_data: Option<Value>,
}

impl I18nHelper {
    /// Create new i18n helper with embedded JSON data
    pub fn new(en_json: &str, zh_json: &str) -> Result<Self, serde_json::Error> {
        let en_data: Value = serde_json::from_str(en_json)?;
        let zh_data: Value = serde_json::from_str(zh_json)?;
        
        Ok(Self {
            en_data,
            zh_data: Some(zh_data),
        })
    }
    
    /// Get localized display name
    pub fn display_name(&self, locale: Locale) -> String {
        self.get_string("display_name", locale)
            .unwrap_or_else(|| "Unknown Tool".to_string())
    }
    
    /// Get localized description
    pub fn description(&self, locale: Locale) -> String {
        self.get_string("description", locale)
            .unwrap_or_else(|| "No description available".to_string())
    }
    
    /// Get localized user guide
    pub fn user_guide(&self, locale: Locale) -> String {
        self.get_string("user_guide", locale)
            .unwrap_or_else(|| "# User Guide\n\nNo guide available.".to_string())
    }
    
    /// Get input field map for schema injection
    pub fn get_input_field_map(&self) -> HashMap<String, HashMap<String, String>> {
        let mut field_map = HashMap::new();
        
        if let Some(input_schema) = self.en_data.get("input_schema").and_then(|v| v.as_object()) {
            for (field_name, field_data) in input_schema {
                let mut translations = HashMap::new();
                
                // English translation
                if let Some(title) = field_data.get("title").and_then(|v| v.as_str()) {
                    translations.insert("en".to_string(), title.to_string());
                }
                
                // Chinese translation
                if let Some(zh_data) = &self.zh_data {
                    if let Some(zh_input) = zh_data.get("input_schema").and_then(|v| v.as_object()) {
                        if let Some(zh_field) = zh_input.get(field_name) {
                            if let Some(zh_title) = zh_field.get("title").and_then(|v| v.as_str()) {
                                translations.insert("zh".to_string(), zh_title.to_string());
                            }
                        }
                    }
                }
                
                if !translations.is_empty() {
                    field_map.insert(field_name.clone(), translations);
                }
            }
        }
        
        field_map
    }
    
    /// Get output field map for schema injection
    pub fn get_output_field_map(&self) -> HashMap<String, HashMap<String, String>> {
        let mut field_map = HashMap::new();
        
        if let Some(output_schema) = self.en_data.get("output_schema").and_then(|v| v.as_object()) {
            for (field_name, field_data) in output_schema {
                let mut translations = HashMap::new();
                
                // English translation
                if let Some(title) = field_data.get("title").and_then(|v| v.as_str()) {
                    translations.insert("en".to_string(), title.to_string());
                }
                
                // Chinese translation
                if let Some(zh_data) = &self.zh_data {
                    if let Some(zh_output) = zh_data.get("output_schema").and_then(|v| v.as_object()) {
                        if let Some(zh_field) = zh_output.get(field_name) {
                            if let Some(zh_title) = zh_field.get("title").and_then(|v| v.as_str()) {
                                translations.insert("zh".to_string(), zh_title.to_string());
                            }
                        }
                    }
                }
                
                if !translations.is_empty() {
                    field_map.insert(field_name.clone(), translations);
                }
            }
        }
        
        field_map
    }
    
    /// Get extra localized content
    pub fn get_extra(&self, key: &str, locale: Locale) -> Option<String> {
        let data = match locale {
            Locale::Zh => self.zh_data.as_ref().unwrap_or(&self.en_data),
            _ => &self.en_data,
        };
        
        data.get("extra")
            .and_then(|extra| extra.get(key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
    
    /// Helper to get localized string
    fn get_string(&self, key: &str, locale: Locale) -> Option<String> {
        let data = match locale {
            Locale::Zh => self.zh_data.as_ref().unwrap_or(&self.en_data),
            _ => &self.en_data,
        };
        
        data.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// Convenience functions for common use cases
pub fn display_name(locale: Locale) -> String {
    let i18n = I18nHelper::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh-CN.json"),
    ).expect("Failed to load i18n data");
    
    i18n.display_name(locale)
}

pub fn description(locale: Locale) -> String {
    let i18n = I18nHelper::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh-CN.json"),
    ).expect("Failed to load i18n data");
    
    i18n.description(locale)
}

pub fn user_guide(locale: Locale) -> String {
    let i18n = I18nHelper::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh-CN.json"),
    ).expect("Failed to load i18n data");
    
    i18n.user_guide(locale)
}

pub fn get_input_field_map() -> HashMap<String, HashMap<String, String>> {
    let i18n = I18nHelper::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh-CN.json"),
    ).expect("Failed to load i18n data");
    
    i18n.get_input_field_map()
}

pub fn get_output_field_map() -> HashMap<String, HashMap<String, String>> {
    let i18n = I18nHelper::new(
        include_str!("../locales/tool.en.json"),
        include_str!("../locales/tool.zh-CN.json"),
    ).expect("Failed to load i18n data");
    
    i18n.get_output_field_map()
}
```

## 7. Model Context Protocol (MCP) Integration

### 7.1 MCP Overview

The Model Context Protocol (MCP) provides standardized context management and tool calling capabilities. Plugins can optionally support MCP for enhanced integration with AI systems and workflow engines.

#### 7.1.1 MCP Benefits

- **Context Awareness**: Tools can access execution history and environment information
- **Stateful Operations**: Maintain state across multiple tool invocations
- **Enhanced Error Handling**: Structured error reporting with context preservation
- **Workflow Integration**: Seamless integration with complex workflow systems

### 7.2 MCP Implementation in Plugins

#### 7.2.1 Basic MCP Support

To add MCP support to a plugin, update the metadata:

```json
{
  "name": "text.advanced_processor",
  "mcp_supported": true,
  "mcp_capabilities": {
    "context_aware": true,
    "stateful": false,
    "batch_processing": true,
    "supports_streaming": false
  },
  "requires_full_context": false,
  "context_validation_rules": {
    "required_fields": ["caller_id"],
    "optional_fields": ["execution_history", "environment_info"],
    "max_history_length": 100
  }
}
```

#### 7.2.2 MCP Request Format

MCP-enabled tools receive enhanced request format:

```json
{
  "id": "req-12345",
  "component_type": "Tool",
  "component_name": "text.advanced_processor",
  "method": "run",
  "params": {
    "text": "Process this text",
    "options": ["normalize", "tokenize"]
  },
  "context": {
    "id": "ctx-12345",
    "parent_id": "ctx-parent",
    "model_state": {
      "current_language": "en",
      "processing_mode": "batch"
    },
    "execution_history": [
      {
        "id": "exec-prev",
        "timestamp": "2025-12-16T08:00:00Z",
        "component_name": "text.tokenizer",
        "input": {"text": "Previous text"},
        "output": {"tokens": ["Previous", "text"]},
        "status": "Success"
      }
    ],
    "environment_info": {
      "user_id": "user-123",
      "session_id": "session-456"
    },
    "metadata": {
      "workflow_id": "wf-789",
      "step_number": 2
    }
  },
  "service_context": {
    "caller_id": "workflow-engine",
    "caller_type": "System",
    "permission_level": "Standard",
    "extra": {}
  }
}
```

#### 7.2.3 MCP Response Format

MCP tools should return enhanced response format:

```json
{
  "id": "resp-67890",
  "request_id": "req-12345",
  "status": "Success",
  "data": {
    "processed_text": "Processed result",
    "tokens": ["Processed", "result"],
    "metadata": {
      "processing_time_ms": 150,
      "language_detected": "en"
    }
  },
  "error": null,
  "context": {
    "id": "ctx-12345",
    "parent_id": "ctx-parent",
    "model_state": {
      "current_language": "en",
      "processing_mode": "batch",
      "last_operation": "tokenize"
    },
    "execution_history": [
      {
        "id": "exec-current",
        "timestamp": "2025-12-16T08:01:00Z",
        "component_type": "Tool",
        "component_name": "text.advanced_processor",
        "method": "run",
        "input": {"text": "Process this text"},
        "output": {"processed_text": "Processed result"},
        "status": "Success",
        "duration_ms": 150
      }
    ],
    "environment_info": {
      "user_id": "user-123",
      "session_id": "session-456"
    },
    "metadata": {
      "workflow_id": "wf-789",
      "step_number": 2
    }
  },
  "duration_ms": 150
}
```

### 7.3 MCP Implementation Example

#### 7.3.1 Rust Plugin with MCP Support

```rust
use rt_core::{Tool, Locale, CoreError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use async_trait::async_trait;

#[derive(Deserialize)]
struct McpRequest {
    id: String,
    component_name: String,
    params: Value,
    context: Option<McpContext>,
    service_context: Option<ServiceContext>,
}

#[derive(Deserialize)]
struct McpContext {
    id: String,
    model_state: Option<Value>,
    execution_history: Option<Vec<ExecutionRecord>>,
    environment_info: Option<Value>,
}

#[derive(Serialize)]
struct McpResponse {
    id: String,
    request_id: String,
    status: String,
    data: Option<Value>,
    error: Option<String>,
    context: Option<McpContext>,
    duration_ms: u64,
}

pub struct AdvancedProcessor;

#[async_trait]
impl Tool for AdvancedProcessor {
    fn name(&self) -> &str {
        "text.advanced_processor"
    }
    
    fn mcp_supported(&self) -> bool {
        true
    }
    
    async fn run(&self, input: Value) -> Result<Value, CoreError> {
        // Standard tool execution
        self.process_text(input).await
    }
    
    async fn run_with_context(&self, request: McpRequest) -> Result<McpResponse, CoreError> {
        let start_time = std::time::Instant::now();
        
        // Extract context information
        let context = request.context.unwrap_or_default();
        let previous_results = self.extract_previous_results(&context);
        
        // Process with context awareness
        let result = self.process_with_context(request.params, previous_results).await?;
        
        // Update context with new execution record
        let updated_context = self.update_context(context, &request, &result);
        
        Ok(McpResponse {
            id: format!("resp-{}", uuid::Uuid::new_v4()),
            request_id: request.id,
            status: "Success".to_string(),
            data: Some(result),
            error: None,
            context: Some(updated_context),
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
}

impl AdvancedProcessor {
    async fn process_text(&self, input: Value) -> Result<Value, CoreError> {
        // Standard processing logic
        Ok(json!({"processed": "result"}))
    }
    
    async fn process_with_context(&self, input: Value, previous: Vec<Value>) -> Result<Value, CoreError> {
        // Context-aware processing logic
        // Can use previous results to inform current processing
        Ok(json!({"processed": "context_aware_result"}))
    }
    
    fn extract_previous_results(&self, context: &McpContext) -> Vec<Value> {
        // Extract relevant data from execution history
        vec![]
    }
    
    fn update_context(&self, mut context: McpContext, request: &McpRequest, result: &Value) -> McpContext {
        // Update context with current execution
        context
    }
}
```

### 7.4 MCP Best Practices

#### 7.4.1 Context Management

- **Selective History**: Only store relevant execution history to avoid memory bloat
- **State Validation**: Validate context state before processing
- **Error Propagation**: Preserve error context across tool invocations
- **Resource Cleanup**: Clean up temporary resources in context

#### 7.4.2 Performance Considerations

- **Lazy Loading**: Load context data only when needed
- **Compression**: Compress large context data for storage
- **Caching**: Cache frequently accessed context information
- **Timeout Handling**: Implement reasonable timeouts for context operations

## 8. Plugin Validation and Testing Procedures

### 8.1 Automated Validation

#### 8.1.1 Schema Validation

Plugins should validate their input/output schemas automatically:

```rust
use schemars::JsonSchema;
use serde_json::Value;

#[derive(JsonSchema)]
struct ToolInput {
    text: String,
    #[serde(default)]
    options: Vec<String>,
}

fn validate_input(input: &Value) -> Result<ToolInput, ValidationError> {
    // Validate against schema
    let schema = schemars::schema_for!(ToolInput);
    // Perform validation logic
    serde_json::from_value(input.clone())
        .map_err(|e| ValidationError::InvalidInput(e.to_string()))
}
```

#### 8.1.2 Metadata Validation

Validate plugin metadata completeness:

```rust
pub fn validate_plugin_metadata(metadata: &PluginMetadata) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Check required fields
    if metadata.name.is_empty() {
        errors.push(ValidationError::MissingField("name".to_string()));
    }
    
    // Validate naming convention
    if !metadata.name.contains('.') {
        errors.push(ValidationError::InvalidNaming("Tool name must follow 'category.tool_name' format".to_string()));
    }
    
    // Check localization completeness
    if metadata.display_name.en.is_empty() {
        errors.push(ValidationError::MissingLocalization("display_name.en".to_string()));
    }
    
    // Validate schema structure
    if let Err(e) = validate_json_schema(&metadata.input_schema) {
        errors.push(ValidationError::InvalidSchema(format!("input_schema: {}", e)));
    }
    
    errors
}
```

### 8.2 Testing Framework

#### 8.2.1 Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_tool_basic_functionality() {
        let tool = MyTool::new();
        
        let input = json!({
            "text": "test input",
            "mode": "process"
        });
        
        let result = tool.run(input).await.unwrap();
        
        assert!(result.get("success").unwrap().as_bool().unwrap());
        assert!(result.get("result").is_some());
    }
    
    #[tokio::test]
    async fn test_tool_error_handling() {
        let tool = MyTool::new();
        
        let invalid_input = json!({
            "invalid_field": "value"
        });
        
        let result = tool.run(invalid_input).await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_metadata_validation() {
        let metadata = get_plugin_metadata();
        let errors = validate_plugin_metadata(&metadata);
        assert!(errors.is_empty(), "Metadata validation failed: {:?}", errors);
    }
}
```

#### 8.2.2 Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;
    
    #[test]
    fn test_plugin_spec_command() {
        let output = Command::new("./target/debug/rt-plugin-example")
            .arg("spec")
            .arg("--locale")
            .arg("en")
            .output()
            .expect("Failed to execute plugin");
        
        assert!(output.status.success());
        
        let spec: Value = serde_json::from_slice(&output.stdout)
            .expect("Invalid JSON output");
        
        assert!(spec.get("name").is_some());
        assert!(spec.get("display_name").is_some());
    }
    
    #[test]
    fn test_plugin_run_command() {
        let input = json!({
            "text": "test input"
        });
        
        let mut child = Command::new("./target/debug/rt-plugin-example")
            .arg("run")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start plugin");
        
        let stdin = child.stdin.as_mut().unwrap();
        serde_json::to_writer(stdin, &input).unwrap();
        
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        
        let result: Value = serde_json::from_slice(&output.stdout)
            .expect("Invalid JSON output");
        
        assert!(result.get("success").is_some());
    }
}
```

### 8.3 Performance Testing

#### 8.3.1 Benchmark Testing

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn benchmark_tool_performance() {
        let tool = MyTool::new();
        let input = json!({"text": "benchmark input"});
        
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            let _ = tool.run(input.clone()).await.unwrap();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;
        
        println!("Average execution time: {:?}", avg_duration);
        assert!(avg_duration.as_millis() < 100, "Tool execution too slow");
    }
}
```

### 8.4 Continuous Integration

#### 8.4.1 CI Pipeline Configuration

```yaml
# .github/workflows/plugin-test.yml
name: Plugin Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Build Plugin
      run: cargo build --release
      
    - name: Run Unit Tests
      run: cargo test
      
    - name: Test Plugin Spec
      run: |
        ./target/release/rt-plugin-example spec --locale en > spec.json
        jq empty spec.json  # Validate JSON
        
    - name: Test Plugin Run
      run: |
        echo '{"text": "test"}' | ./target/release/rt-plugin-example run > output.json
        jq empty output.json  # Validate JSON
        
    - name: Validate Metadata
      run: |
        # Add custom validation scripts
        python scripts/validate_metadata.py spec.json
```

## 9. Compilation and Deployment

### 9.1 Building Plugins

#### 9.1.1 Release Build
For optimal performance, compile plugins in release mode:

```bash
# Build single plugin
cargo build --release --package rt-plugin-example

# Build all plugins in workspace
cargo build --release --workspace --exclude rt-core --exclude rt-tools --exclude rt-cli --exclude rt-gui
```

#### 9.1.2 Development Build
For faster iteration during development:

```bash
cargo build --package rt-plugin-example
```

### 9.2 Plugin Deployment

#### 9.2.1 Automatic Discovery
Rust Toolbox automatically scans the `plugins/` directory for:
- Files prefixed with `rt-plugin-` (executable format)
- Files with `.wasm` extension (WebAssembly format)

#### 9.2.2 Deployment Steps
1. Ensure `plugins/` directory exists in the application root
2. Copy compiled plugin executable to `plugins/` directory
3. Ensure proper file naming convention (`rt-plugin-*`)
4. Restart rt-cli or rt-gui to load new plugins

#### 9.2.3 Development Workflow
```bash
# Create symbolic link for faster development iteration
ln -s ../rt-plugin-example/target/release/rt-plugin-example plugins/rt-plugin-example

# Or use a development script
#!/bin/bash
cargo build --release --package rt-plugin-example
cp target/release/rt-plugin-example plugins/
```

## 10. WebAssembly (WASM) Plugin Support

### 10.1 WASM Plugin Benefits

- **Enhanced Security**: Sandboxed execution environment
- **Cross-Platform**: Run on any platform supporting WASI
- **Smaller Distribution**: Compact binary format
- **Isolation**: Memory and resource isolation between plugins

### 10.2 WASM Development Requirements

#### 10.2.1 Target Configuration
```toml
# Cargo.toml
[package]
name = "rt-plugin-wasm-example"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Avoid these dependencies in WASM:
# tokio = "1.0"  # Use wasi-compatible alternatives
# std::thread    # Use single-threaded approach
```

#### 10.2.2 WASI-Compatible Dependencies
```toml
[dependencies]
# Use WASI-compatible alternatives
wasi = "0.11"
getrandom = { version = "0.2", features = ["wasi"] }
chrono = { version = "0.4", features = ["wasi"] }

# Avoid these:
# tokio (networking/threading features)
# std::process
# std::net
```

### 10.3 WASM Compilation

#### 10.3.1 Install WASM Target
```bash
rustup target add wasm32-wasi
```

#### 10.3.2 Build WASM Plugin
```bash
cargo build --target wasm32-wasi --release
```

#### 10.3.3 Optimize WASM Binary
```bash
# Install wasm-opt
npm install -g wasm-opt

# Optimize the binary
wasm-opt -Oz target/wasm32-wasi/release/rt-plugin-example.wasm -o plugins/rt-plugin-example.wasm
```

### 10.4 WASM Plugin Implementation

#### 10.4.1 Entry Point Structure
```rust
use wasm_bindgen::prelude::*;
use serde_json::{Value, json};

#[wasm_bindgen]
pub fn plugin_spec(locale: &str) -> String {
    let metadata = create_plugin_metadata(locale);
    serde_json::to_string(&metadata).unwrap_or_default()
}

#[wasm_bindgen]
pub fn plugin_run(input: &str) -> String {
    match serde_json::from_str::<Value>(input) {
        Ok(input_value) => {
            match process_tool_input(input_value) {
                Ok(output) => serde_json::to_string(&output).unwrap_or_default(),
                Err(e) => json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => json!({"error": format!("Invalid input JSON: {}", e)}).to_string(),
    }
}

fn create_plugin_metadata(locale: &str) -> Value {
    json!({
        "name": "text.wasm_example",
        "display_name": {
            "en": "WASM Example Tool",
            "zh": "WASM示例工具"
        },
        "description": {
            "en": "Example WebAssembly plugin",
            "zh": "WebAssembly插件示例"
        },
        // ... rest of metadata
    })
}

fn process_tool_input(input: Value) -> Result<Value, Box<dyn std::error::Error>> {
    // Tool processing logic
    Ok(json!({"result": "processed"}))
}
```

### 10.5 WASM Limitations and Considerations

#### 10.5.1 Limitations
- **No Threading**: Single-threaded execution only
- **Limited I/O**: Restricted file system and network access
- **No Process Spawning**: Cannot execute external commands
- **Memory Constraints**: Limited memory allocation

#### 10.5.2 Best Practices
- **Stateless Design**: Avoid maintaining state between calls
- **Efficient Memory Usage**: Minimize memory allocations
- **Error Handling**: Robust error handling for constrained environment
- **Performance Optimization**: Use efficient algorithms for WASM constraints

### 10.6 WASM Testing

#### 10.6.1 Local Testing
```bash
# Install wasmtime for local testing
curl https://wasmtime.dev/install.sh -sSf | bash

# Test WASM plugin
echo '{"text": "test"}' | wasmtime run --invoke plugin_run target/wasm32-wasi/release/rt-plugin-example.wasm
```

#### 10.6.2 Integration Testing
```rust
#[cfg(test)]
mod wasm_tests {
    use wasmtime::*;
    
    #[test]
    fn test_wasm_plugin() {
        let engine = Engine::default();
        let module = Module::from_file(&engine, "target/wasm32-wasi/release/rt-plugin-example.wasm").unwrap();
        
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        
        let plugin_spec = instance.get_typed_func::<&str, String>(&mut store, "plugin_spec").unwrap();
        let result = plugin_spec.call(&mut store, "en").unwrap();
        
        let metadata: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(metadata.get("name").is_some());
    }
}
```

## 11. Plugin Documentation Standards

### 11.1 README.md Structure

Every plugin project should include a comprehensive `README.md` file following this structure:

#### 11.1.1 Required Sections

```markdown
# Plugin Name

Brief description of what the plugin does.

## Features

- Feature 1: Description
- Feature 2: Description
- Multi-tool support (if applicable)
- MCP integration (if supported)

## Installation

### Prerequisites
- Rust toolchain (latest stable)
- Additional dependencies (if any)

### Building from Source
```bash
# Clone the repository
git clone <repository-url>
cd rt-plugin-example

# Build the plugin
cargo build --release --package rt-plugin-example

# Deploy to plugins directory
cp target/release/rt-plugin-example /path/to/rt-box/plugins/
```

### Binary Installation
Download pre-built binaries from the releases page and place in the `plugins/` directory.

## Usage

### Command Line Interface (CLI)

#### Get Plugin Specification
```bash
./plugins/rt-plugin-example spec --locale en
```

#### Run Tool via rt-cli
```bash
cargo run --bin rt-cli -- run tool.name --input '{"param": "value"}'
```

### Graphical User Interface (GUI)
1. Launch rt-gui: `cargo run --bin rt-gui`
2. Select the tool from the sidebar
3. Fill in the input parameters
4. Click "Run" to execute

## Tools Provided

### Tool 1: tool.name
- **Description**: What the tool does
- **Input**: Description of input parameters
- **Output**: Description of output format
- **Example**: Usage example

## Development

### Project Structure
```
rt-plugin-example/
├── src/
│   ├── main.rs          # Plugin entry point
│   ├── i18n.rs          # Internationalization
│   └── tools/           # Tool implementations (multi-tool)
├── locales/             # Translation files
├── tests/               # Test files
└── README.md
```

### Internationalization
- English translations: `locales/tool.en.json`
- Chinese translations: `locales/tool.zh-CN.json`

### Testing
```bash
# Run unit tests
cargo test

# Test plugin commands
./target/release/rt-plugin-example spec
echo '{"test": "input"}' | ./target/release/rt-plugin-example run
```

## License

Specify the license (e.g., MIT, Apache 2.0, GPL-3.0)
```

#### 11.1.2 Multi-Tool Plugin Documentation

For plugins providing multiple tools:

```markdown
## Tools Provided

This plugin provides multiple file management tools:

### 1. Duplicate Files (`file.duplicates`)
Find duplicate files based on content comparison.

**Input Parameters:**
- `directories`: Array of directory paths to search
- `min_size`: Minimum file size in bytes (optional)

**Example:**
```json
{
  "directories": ["/home/user/documents"],
  "min_size": 1024
}
```

### 2. Similar Images (`file.similar_images`)
Find visually similar images using perceptual hashing.

**Input Parameters:**
- `directories`: Array of directory paths to search
- `threshold`: Similarity threshold (0-100, default: 10)

**Example:**
```json
{
  "directories": ["/home/user/photos"],
  "threshold": 15
}
```
```

### 11.2 API Documentation

#### 11.2.1 Schema Documentation

Generate API documentation from schemas:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input parameters for the text processing tool
#[derive(Deserialize, JsonSchema)]
pub struct TextProcessorInput {
    /// The text to process
    pub text: String,
    
    /// Processing mode
    #[serde(default = "default_mode")]
    pub mode: ProcessingMode,
    
    /// Additional options
    #[serde(default)]
    pub options: Vec<String>,
}

/// Processing modes available
#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingMode {
    /// Normalize text formatting
    Normalize,
    /// Tokenize into words
    Tokenize,
    /// Extract entities
    ExtractEntities,
}

fn default_mode() -> ProcessingMode {
    ProcessingMode::Normalize
}
```

#### 11.2.2 Error Documentation

Document error codes and handling:

```markdown
## Error Handling

### Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| `INVALID_INPUT` | Input parameters are invalid | Check input format and required fields |
| `FILE_NOT_FOUND` | Specified file does not exist | Verify file path and permissions |
| `PROCESSING_ERROR` | Error during tool execution | Check tool-specific requirements |

### Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "INVALID_INPUT",
    "message": "Required field 'text' is missing",
    "details": {
      "field": "text",
      "expected_type": "string"
    }
  }
}
```
```

### 11.3 Changelog and Versioning

#### 11.3.1 Semantic Versioning

Follow semantic versioning (semver) for plugin releases:
- **MAJOR**: Breaking changes to plugin interface
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

#### 11.3.2 Changelog Format

```markdown
# Changelog

All notable changes to this plugin will be documented in this file.

## [1.2.0] - 2025-12-16

### Added
- New similarity threshold parameter for image comparison
- Support for additional image formats (WebP, AVIF)
- MCP integration for context-aware processing

### Changed
- Improved performance for large directory scans
- Updated error messages for better clarity

### Fixed
- Fixed memory leak in image processing
- Corrected handling of symbolic links

## [1.1.0] - 2025-11-15

### Added
- Multi-language support (English, Chinese)
- Batch processing capabilities

### Fixed
- Fixed crash on empty directories
```

### 11.4 Contributing Guidelines

#### 11.4.1 Development Setup

```markdown
## Contributing

### Development Setup

1. Fork the repository
2. Clone your fork: `git clone <your-fork-url>`
3. Create a feature branch: `git checkout -b feature/new-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Commit changes: `git commit -m "Add new feature"`
7. Push to branch: `git push origin feature/new-feature`
8. Create a Pull Request

### Code Standards

- Follow Rust formatting: `cargo fmt`
- Pass all lints: `cargo clippy`
- Add tests for new functionality
- Update documentation for API changes
- Follow semantic versioning for releases

### Testing Requirements

- Unit tests for all new functions
- Integration tests for plugin commands
- Performance tests for resource-intensive operations
- Documentation tests for code examples
```
