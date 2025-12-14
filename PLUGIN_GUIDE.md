# Rust Toolbox 插件开发指南

## 1. 概述

本指南旨在为 Rust Toolbox 的插件开发者提供一套标准的开发规范和文档格式。遵循这些规范，可以确保您的插件能够无缝集成到 Rust Toolbox 生态系统，并提供良好的用户体验，特别是多语言支持和 GUI 自动表单生成。

## 1.1 相关文档

- [设计文档](DESIGN.md): 项目的整体设计文档，包括技术选型和核心原则
- [架构设计文档](ARCHITECTURE_DESIGN.md): 详细描述项目的架构设计、核心组件和部署架构
- [用户指南](USER_GUIDE.md): 详细的用户使用指南，包括工具库和使用方式
- [AI工作规范](AI_WORK_PROTOCOL.md): AI辅助开发的工作规范
- [变更日志](CHANGELOG.md): 项目的变更历史

## 2. 核心原则

### 2.1 多语言支持 (Internationalization - i18n)

所有面向用户的文本，包括工具名称、描述、用户指南以及输入/输出字段的标题，都必须提供多语言版本。目前支持的语言包括英语 (`en`) 和简体中文 (`zh`)。

### 2.2 结构化 Schema 定义 (JSON Schema)

插件的输入和输出参数必须通过 JSON Schema 进行定义。这些 Schema 不仅用于验证数据，更是 GUI 自动生成表单和结果展示的关键依据。通过在 Schema 中注入本地化的 `title` 字段，可以实现字段级别的多语言显示。

### 2.3 国际化文件结构

工具的国际化现在通过 JSON 文件管理。每个工具目录下建议有一个 `locales` 文件夹，包含 `tool.en.json` 和 `tool.zh.json`。

JSON 文件结构示例：
```json
{
  "display_name": "Tool Name",
  "description": "Tool Description",
  "user_guide": "Markdown User Guide",
  "input_schema": {
    "field_name": { "title": "Field Title" }
  },
  "output_schema": {
    "field_name": { "title": "Field Title" }
  },
  "extra": {
    "key": "value"
  }
}
```

## 3. 插件结构

建议插件项目采用以下结构：

```
my-plugin/
├── Cargo.toml
├── src/
│   └── mod.rs       # 插件核心逻辑，实现 `plugin spec` 和 `plugin run` 命令
├── locales/         # 国际化资源文件夹
│   ├── tool.en.json # 英文翻译文件
│   └── tool.zh.json # 中文翻译文件
└── README.md
```

## 4. `plugin spec` 命令规范

插件必须实现 `plugin spec` 命令，该命令应向标准输出 (stdout) 返回一个 JSON 对象，其中包含工具的元数据。这个 JSON 对象必须遵循 `PluginMetadata` 结构。

### 4.1 `PluginMetadata` 结构

```json
{
  "name": "ext.my_tool",
  "display_name": { "en": "My Tool", "zh": "我的工具" },
  "description": { "en": "A tool that does something.", "zh": "一个做某事的工具。" },
  "user_guide": { "en": "# My Tool User Guide\n\nThis is how to use my tool...", "zh": "# 我的工具用户指南\n\n如何使用我的工具..." },
  "input_schema": { ... }, // JSON Schema for input parameters
  "output_schema": { ... }, // JSON Schema for output results
  "input_fields": { // Optional: Field-level localization for GUI form generation
    "param1": { "en": "Parameter 1", "zh": "参数1" },
    "param2": { "en": "Parameter 2", "zh": "参数2" }
  },
  "output_fields": { // Optional: Field-level localization for GUI result display
    "result1": { "en": "Result 1", "zh": "结果1" }
  }
}
```

### 4.2 字段说明

*   `name` (String): 工具的唯一标识符，格式为 `category.tool_name` (例如: `ext.my_tool`)。
*   `display_name` (Object): 工具的显示名称，包含多语言键值对。
    *   `en`: 英文显示名称。
    *   `zh`: 简体中文显示名称。
*   `description` (Object): 工具的简短描述，包含多语言键值对。
*   `user_guide` (Object): 工具的详细用户指南，支持 Markdown 格式，包含多语言键值对。
*   `input_schema` (Object): 工具输入参数的 JSON Schema 定义。GUI 将根据此 Schema 自动生成输入表单。
*   `output_schema` (Object): 工具输出结果的 JSON Schema 定义。GUI 将根据此 Schema 自动展示结果。
*   `input_fields` (Object, 可选): 针对 `input_schema` 中定义的每个字段，提供其在 GUI 中显示的多语言标题。例如，如果 `input_schema` 中有一个字段名为 `param1`，则可以在 `input_fields` 中定义 `"param1": { "en": "Parameter 1", "zh": "参数1" }`。
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
  "text": { "en": "Text", "zh": "文本" },
  "tone": { "en": "With Tone", "zh": "包含声调" }
}
```

在运行时，当语言环境为 `zh` 时，GUI 接收到的有效 Schema 将类似于：

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

### 6.1 `ToolI18n` 使用

`rt-tools` 提供了 `ToolI18n` 结构体，用于加载工具目录下的 `locales` 文件夹中的 JSON 文件，并在运行时提供本地化字符串。

**示例 `i18n.rs`:**

```rust
use rt_core::Locale;
use rt_tools::utils::ToolI18n; // 引入 ToolI18n

// 假设你的工具目录结构如下：
// my-tool/
// ├── src/
// │   └── i18n.rs
// └── locales/
//     ├── tool.en.json
//     └── tool.zh.json

/// 获取本地化字符串的辅助函数
///
/// 该函数加载指定语言环境的 ToolI18n 实例。
/// 如果加载失败，则返回一个默认的 ToolI18n 实例。
///
/// # 参数
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 返回一个包含本地化字符串的 `ToolI18n` 实例。
pub fn get_localized_strings(locale: Locale) -> ToolI18n {
    // ToolI18n::load 会自动查找当前工作目录下的 `locales` 文件夹
    // 并根据传入的 Locale 加载对应的 JSON 文件
    ToolI18n::load(locale).unwrap_or_else(|_| {
        // 如果加载失败，可以提供一个默认的 ToolI18n 实例或处理错误
        ToolI18n::default()
    })
}

/// 获取工具的显示名称。
///
/// # 参数
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 返回工具的本地化显示名称。
pub fn display_name(locale: Locale) -> String {
    get_localized_strings(locale).display_name
}

/// 获取工具的描述。
///
/// # 参数
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 返回工具的本地化描述。
pub fn description(locale: Locale) -> String {
    get_localized_strings(locale).description
}

/// 获取工具的用户指南。
///
/// # 参数
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 返回工具的本地化用户指南。
pub fn user_guide(locale: Locale) -> String {
    get_localized_strings(locale).user_guide
}

/// 获取输入字段的本地化标题。
///
/// # 参数
/// * `field` - 输入字段的名称。
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 如果找到，返回输入字段的本地化标题；否则返回 `None`。
pub fn input_field_title(field: &str, locale: Locale) -> Option<String> {
    get_localized_strings(locale).input_fields.get(field).cloned()
}

/// 获取输出字段的本地化标题。
///
/// # 参数
/// * `field` - 输出字段的名称。
/// * `locale` - 目标语言环境。
///
/// # 返回值
/// 如果找到，返回输出字段的本地化标题；否则返回 `None`。
pub fn output_field_title(field: &str, locale: Locale) -> Option<String> {
    get_localized_strings(locale).output_fields.get(field).cloned()
}
```

通过遵循这些规范，您可以创建功能强大、易于使用且支持多语言的 Rust Toolbox 插件。

## 7. 公共工具模块 (Utils Module)

`rt-tools` 提供了一个公共的 `utils` 模块，旨在简化插件开发并统一核心功能的引用。

### 7.1 目的

1.  **核心功能暴露**: 将 `rt-core` 中的常用类型 (如 `Tool`, `Locale`, `PersistenceManager`, `WorkflowEngine`, `WorkflowDefinition`, `WorkflowStatus`, `WorkflowInstance`, `CoreError`, `Result`) 重新导出，方便内部工具和插件统一引用，避免重复导入 `rt-core`。
2.  **通用辅助功能**: 提供如 `ToolI18n` 等辅助工具开发的功能。

### 7.2 使用示例

开发新工具或插件时，建议通过 `use rt_tools::utils::{...}` 引用所需的基础设施。

```rust
// 引入 rt-tools::utils 模块中的常用类型
use rt_tools::utils::{Locale, Tool, PersistenceManager, WorkflowEngine, Result, CoreError};

/// 定义一个名为 `MyTool` 的简单工具结构体。
pub struct MyTool;

impl Tool for MyTool {
    /// 返回工具的唯一名称。
    ///
    /// # 返回值
    /// 返回一个静态字符串切片，表示工具的名称。
    fn name(&self) -> &'static str {
        "my_category.my_tool"
    }

    /// 返回工具的本地化显示名称。
    ///
    /// # 参数
    /// * `locale` - 目标语言环境。
    ///
    /// # 返回值
    /// 返回工具的本地化显示名称字符串。
    fn display_name(&self, locale: Locale) -> String {
        // 使用 ToolI18n 获取本地化名称
        let i18n = rt_tools::utils::ToolI18n::load(locale).unwrap_or_default();
        i18n.display_name
    }

    // ... 其他 Tool trait 方法实现
}

/// 示例：使用 `PersistenceManager` 保存数据。
///
/// # 参数
/// * `manager` - `PersistenceManager` 的引用，用于数据持久化。
/// * `key` - 要保存数据的键。
/// * `value` - 要保存的数据值。
///
/// # 返回值
/// 如果操作成功，返回 `Ok(())`；否则返回 `Err(CoreError)`。
async fn save_data_example(manager: &PersistenceManager, key: &str, value: &str) -> Result<()> {
    manager.set_data(key, value).await?;
    Ok(())
}

/// 示例：使用 `PersistenceManager` 加载数据。
///
/// # 参数
/// * `manager` - `PersistenceManager` 的引用，用于数据持久化。
/// * `key` - 要加载数据的键。
///
/// # 返回值
/// 如果操作成功，返回 `Ok(Some(String))` 包含数据，如果键不存在则返回 `Ok(None)`；否则返回 `Err(CoreError)`。
async fn load_data_example(manager: &PersistenceManager, key: &str) -> Result<Option<String>> {
    manager.get_data(key).await
}
```

## 8. 编译与加载 (Compilation and Loading)

### 8.1 编译插件
开发完成后，使用 Cargo 编译插件的 Release 版本以获得最佳性能：

```bash
cargo build --release
```

编译产物通常位于 `target/release/` 目录下。

### 8.2 加载插件
Rust Toolbox 的宿主程序 (`rt-cli` 或 `rt-gui`) 会自动扫描工作目录下的 `plugins` 文件夹。

1.  确保宿主程序根目录下存在 `plugins` 文件夹。
2.  将编译好的插件可执行文件 (如 `rt-plugin-mypinyin.exe`) 复制到 `plugins` 文件夹中。
3.  文件名必须以 `rt-plugin-` 开头，否则将被忽略。

### 8.3 调试建议
在开发过程中，可以将 `plugins` 目录指向您的开发构建目录，或者使用符号链接将编译出的可执行文件链接到 `plugins` 目录，以便快速迭代。

## 9. WebAssembly (Wasm) 插件 (实验性)

除了原生可执行文件外，Rust Toolbox 还支持加载 WebAssembly (`.wasm`) 格式的插件。这提供了更强的安全性和跨平台能力。

### 9.1 开发要求
*   **Target**: `wasm32-wasi`
*   **Dependencies**: 避免使用不支持 WASI 的库 (如 `tokio` 的网络/多线程功能)。建议使用同步 IO 或 `wasi-common` 支持的异步功能。

### 9.2 编译
```bash
cargo build --target wasm32-wasi --release
```

### 9.3 部署
将生成的 `.wasm` 文件复制到 `plugins` 目录。
注意：文件名必须以 `rt-plugin-` 开头 (或者作为 `.wasm` 文件，扫描器会自动识别)。
*当前扫描策略*: 加载所有以 `rt-plugin-` 开头的 `.exe`/二进制文件，以及所有 `.wasm` 文件。

### 9.4 交互协议
Wasm 插件的交互协议与原生插件完全一致 (通过 stdin/stdout 传递 JSON)。
- `spec`: 传入参数 `spec`，输出 JSON Metadata。
- `run`: 传入参数 `run`，通过 stdin 读取 Input JSON，向 stdout 输出 Output JSON。

## 10. README.md 编写规范

每个插件项目都应该包含一个 `README.md` 文件，用于提供插件的概述、功能、安装、使用、开发和许可证信息。

### 9.1 推荐结构

*   **插件名称**: 插件的名称，通常与项目名称一致。
*   **概述**: 简要介绍插件的用途和核心功能。
*   **功能 (Features)**: 列出插件的主要功能点。
*   **安装 (Installation)**: 详细说明如何编译和部署插件。
    *   克隆仓库
    *   构建插件 (`cargo build --release --package <plugin-name>`)
    *   部署插件 (复制到 `plugins` 文件夹)
*   **使用 (Usage)**: 演示如何通过 `rt-cli` 或 `rt-gui` 使用插件。
    *   命令行界面 (CLI) 示例：`spec` 和 `run` 命令的输入输出示例。
    *   图形用户界面 (GUI) 描述。
*   **开发 (Development)**: 介绍插件的开发相关信息。
    *   国际化 (Internationalization)：说明多语言文件的位置和作用。
    *   结构 (Structure)：简要说明项目的主要文件和目录结构。
*   **许可证 (License)**: 插件的开源许可证信息。

### 9.2 示例

请参考 `rt-plugin-pinyin/README.md` 文件作为示例。
