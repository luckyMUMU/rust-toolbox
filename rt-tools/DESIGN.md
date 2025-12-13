# rt-tools Design Document

## 1. 模块概述 (Module Overview)
`rt-tools` 包含具体的工具实现。所有工具必须实现 `rt-core::Tool` trait。

## 2. 工具列表 (Tool List)

### 2.1 File Operations (`file`)

#### Move Folder (`file.move_folder`)
- **Name**: `file.move_folder`
- **Description**: 移动或重命名文件夹。
- **Input Schema**:
  ```json
  {
    "source": "path/to/source",
    "destination": "path/to/dest",
    "overwrite": false // Optional, default false
  }
  ```
- **Output Schema**:
  ```json
  {
    "success": true,
    "moved_files": 10 // Count of moved files/items
  }
  ```
- **Error Conditions**:
  - Source path does not exist.
  - Destination exists and overwrite is false.
  - Permission denied.

### 2.2 Text Operations (`text`)

#### Chinese Converter (`text.convert_chinese`)
- **Name**: `text.convert_chinese`
- **Description**: 简繁体中文转换。
- **Input Schema**:
  ```json
  {
    "text": "简体中文",
    "mode": "s2t" // Enum: s2t, t2s, s2tw, tw2s, s2hk, hk2s, s2twp, tw2sp
  }
  ```
- **Output Schema**:
  ```json
  {
    "converted": "繁體中文"
  }
  ```

## 3. 结构 (Structure)
```
src/
├── lib.rs
├── i18n_utils.rs       # i18n helper module
├── file/
│   ├── mod.rs
│   └── move_folder/
│       ├── mod.rs
│       └── locales/    # Localization files
│           ├── tool.en.json
│           └── tool.zh-CN.json
└── text/
    ├── mod.rs
    └── convert_chinese/
        ├── mod.rs
        └── locales/    # Localization files
            ├── tool.en.json
            └── tool.zh-CN.json
```

## 4. 国际化 (Internationalization)
工具的国际化现在通过 JSON 文件管理。每个工具目录下有一个 `locales` 文件夹，包含 `tool.en.json` 和 `tool.zh-CN.json`。

`i18n_utils.rs` 提供了 `ToolI18n` 结构体，用于加载这些 JSON 文件并在运行时提供本地化字符串。

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

## 5. 插件系统架构 (Plugin System Architecture)

为了支持扩展性，rt-box 支持通过外部可执行文件添加工具。

### 5.1 协议定义 (Protocol Definition)

插件必须是一个独立的可执行文件（如 `.exe`, `.py` 脚本等），并支持以下命令行交互：

#### 5.1.1 获取元数据 (Metadata)
- **Command**: `path/to/plugin spec`
- **Stdin**: (无)
- **Stdout**: JSON 对象，包含工具描述。
  ```json
  {
    "name": "ext.my_tool",
    "display_name": { "en": "My Tool", "zh-CN": "我的工具" },
    "description": { "en": "...", "zh-CN": "..." },
    "user_guide": { "en": "...", "zh-CN": "..." },
    "input_schema": { ... }, // JSON Schema
    "output_schema": { ... }, // JSON Schema
    "input_fields": { // Optional: Field-level localization for GUI form generation
      "field_name_1": { "en": "Field 1", "zh-CN": "字段1" },
      "field_name_2": { "en": "Field 2", "zh-CN": "字段2" }
    },
    "output_fields": { // Optional: Field-level localization for GUI result display
      "result_field_1": { "en": "Result 1", "zh-CN": "结果1" }
    }
  }
  ```

#### 5.1.2 执行工具 (Execute)
- **Command**: `path/to/plugin run`
- **Stdin**: JSON 字符串 (Input Value)
