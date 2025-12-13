# Rust Toolbox 插件开发指南

## 1. 概述

本指南旨在为 Rust Toolbox 的插件开发者提供一套标准的开发规范和文档格式。遵循这些规范，可以确保您的插件能够无缝集成到 Rust Toolbox 生态系统，并提供良好的用户体验，特别是多语言支持和 GUI 自动表单生成。

## 2. 核心原则

### 2.1 多语言支持 (Internationalization - i18n)

所有面向用户的文本，包括工具名称、描述、用户指南以及输入/输出字段的标题，都必须提供多语言版本。目前支持的语言包括英语 (`en`) 和简体中文 (`zh-CN`)。

### 2.2 结构化 Schema 定义 (JSON Schema)

插件的输入和输出参数必须通过 JSON Schema 进行定义。这些 Schema 不仅用于验证数据，更是 GUI 自动生成表单和结果展示的关键依据。通过在 Schema 中注入本地化的 `title` 字段，可以实现字段级别的多语言显示。

## 3. 插件结构

建议插件项目采用以下结构：

```
my-plugin/
├── Cargo.toml
├── src/
│   ├── main.rs       # 插件核心逻辑，实现 `plugin spec` 和 `plugin run` 命令
│   └── i18n.rs       # 多语言资源文件 (可选，如果插件是 Rust 编写)
└── README.md
```

## 4. `plugin spec` 命令规范

插件必须实现 `plugin spec` 命令，该命令应向标准输出 (stdout) 返回一个 JSON 对象，其中包含工具的元数据。这个 JSON 对象必须遵循 `PluginMetadata` 结构。

### 4.1 `PluginMetadata` 结构

```json
{
  "name": "ext.my_tool",
  "display_name": { "en": "My Tool", "zh-CN": "我的工具" },
  "description": { "en": "A tool that does something.", "zh-CN": "一个做某事的工具。" },
  "user_guide": { "en": "# My Tool User Guide\n\nThis is how to use my tool...", "zh-CN": "# 我的工具用户指南\n\n如何使用我的工具..." },
  "input_schema": { ... }, // JSON Schema for input parameters
  "output_schema": { ... }, // JSON Schema for output results
  "input_fields": { // Optional: Field-level localization for GUI form generation
    "param1": { "en": "Parameter 1", "zh-CN": "参数1" },
    "param2": { "en": "Parameter 2", "zh-CN": "参数2" }
  },
  "output_fields": { // Optional: Field-level localization for GUI result display
    "result1": { "en": "Result 1", "zh-CN": "结果1" }
  }
}
```

### 4.2 字段说明

*   `name` (String): 工具的唯一标识符，格式为 `category.tool_name` (例如: `ext.my_tool`)。
*   `display_name` (Object): 工具的显示名称，包含多语言键值对。
    *   `en`: 英文显示名称。
    *   `zh-CN`: 简体中文显示名称。
*   `description` (Object): 工具的简短描述，包含多语言键值对。
*   `user_guide` (Object): 工具的详细用户指南，支持 Markdown 格式，包含多语言键值对。
*   `input_schema` (Object): 工具输入参数的 JSON Schema 定义。GUI 将根据此 Schema 自动生成输入表单。
*   `output_schema` (Object): 工具输出结果的 JSON Schema 定义。GUI 将根据此 Schema 自动展示结果。
*   `input_fields` (Object, 可选): 针对 `input_schema` 中定义的每个字段，提供其在 GUI 中显示的多语言标题。例如，如果 `input_schema` 中有一个字段名为 `param1`，则可以在 `input_fields` 中定义 `"param1": { "en": "Parameter 1", "zh-CN": "参数1" }`。
*   `output_fields` (Object, 可选): 针对 `output_schema` 中定义的每个字段，提供其在 GUI 中显示的多语言标题。

### 4.3 JSON Schema 中的多语言标题注入

为了实现字段级别的多语言，Rust Toolbox 的核心库会在运行时根据当前语言环境，将 `input_fields` 和 `output_fields` 中定义的标题动态注入到 `input_schema` 和 `output_schema` 的 `properties` 字段的 `title` 属性中。因此，插件开发者无需在 `input_schema` 或 `output_schema` 中直接定义 `title`，只需在 `input_fields` 和 `output_fields` 中提供即可。

**示例:**

如果您的 `input_schema` 如下：

```json
{
  "type": "object",
  "properties": {
    "text": { "type": "string" },
    "tone": { "type": "boolean" }
  }
}
```

并且您的 `input_fields` 如下：

```json
{
  "text": { "en": "Text", "zh-CN": "文本" },
  "tone": { "en": "With Tone", "zh-CN": "包含声调" }
}
```

在运行时，当语言环境为 `zh-CN` 时，GUI 接收到的有效 Schema 将类似于：

```json
{
  "type": "object",
  "properties": {
    "text": { "type": "string", "title": "文本" },
    "tone": { "type": "boolean", "title": "包含声调" }
  }
}
```

## 5. `plugin run` 命令规范

插件必须实现 `plugin run` 命令，该命令应从标准输入 (stdin) 读取 JSON 格式的输入数据，执行工具逻辑，并将 JSON 格式的结果输出到标准输出 (stdout)。

*   **Command**: `path/to/plugin run`
*   **Stdin**: JSON 字符串 (符合 `input_schema` 定义的输入值)
*   **Stdout**: JSON 字符串 (符合 `output_schema` 定义的输出值)
*   **Stderr**: 错误日志 (用于调试，非结构化)
*   **Exit Code**: `0` 表示成功，非 `0` 表示失败。

## 6. Rust 插件的多语言实现 (i18n.rs)

对于使用 Rust 编写的插件，建议创建一个 `i18n.rs` 模块来集中管理多语言资源。该模块可以提供函数来根据 `Locale` 枚举返回对应的字符串。

**示例 `i18n.rs`:**

```rust
use rt_core::Locale;

pub fn display_name(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "My Tool",
        Locale::ZhCn => "我的工具",
    }
}

pub fn description(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "A tool that does something.",
        Locale::ZhCn => "一个做某事的工具。",
    }
}

pub fn user_guide(locale: Locale) -> &'static str {
    match locale {
        Locale::En => "# My Tool User Guide\n\nThis is how to use my tool...",
        Locale::ZhCn => "# 我的工具用户指南\n\n如何使用我的工具...",
    }
}

pub fn input_field_title(field: &str, locale: Locale) -> Option<&'static str> {
    match field {
        "param1" => match locale {
            Locale::En => Some("Parameter 1"),
            Locale::ZhCn => Some("参数1"),
        },
        "param2" => match locale {
            Locale::En => Some("Parameter 2"),
            Locale::ZhCn => Some("参数2"),
        },
        _ => None,
    }
}

pub fn output_field_title(field: &str, locale: Locale) -> Option<&'static str> {
    match field {
        "result1" => match locale {
            Locale::En => Some("Result 1"),
            Locale::ZhCn => Some("结果1"),
        },
        _ => None,
    }
}
```

通过遵循这些规范，您可以创建功能强大、易于使用且支持多语言的 Rust Toolbox 插件。