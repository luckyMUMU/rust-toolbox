# Design: Chinese to Pinyin Plugin

## 1. 概述 (Overview)
本插件提供汉字转拼音的功能，支持声调控制和多音字处理。
作为 `Rust Toolbox` 的外部插件运行。

## 2. 核心职责 (Core Responsibilities)
- 将输入的中文文本转换为拼音列表。
- 支持带声调和不带声调的输出。

## 3. 接口定义 (Interface)

### 3.1 插件元数据 (Spec)
- **Name**: `text.pinyin`
- **Display Name**: 通过 `ToolI18n` 从 `locales` 目录加载，例如 `Chinese to Pinyin` (en) 或 `汉字转拼音` (zh)。
- **Description**: 通过 `ToolI18n` 从 `locales` 目录加载，例如 `Convert Chinese characters to Pinyin.` (en) 或 `将中文字符转换为拼音。` (zh)。

### 3.2 输入/输出 (Schema)

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "text": {
      "type": "string",
      "title": "Text",
      "description": "Chinese text to convert"
    },
    "tone": {
      "type": "boolean",
      "title": "With Tone",
      "description": "Include tone marks (e.g., hǎo)",
      "default": true
    }
  },
  "required": ["text"]
}
```

**Output Schema**:
```json
{
  "type": "object",
  "properties": {
    "pinyin": {
      "type": "string",
      "title": "Pinyin",
      "description": "Converted Pinyin text"
    }
  }
}
```

## 4. 逻辑流程 (Logic Flow)
1.  **Parse Args**: 解析命令行参数，识别 `spec` 或 `run` 命令。
2.  **Tool Trait 实现**:
    - `PinyinTool` 结构体实现了 `rt_core::tool::Tool` trait。
    - `name()` 方法：返回工具的唯一名称。
    - `display_name(locale)` 方法：根据语言环境返回工具的显示名称。
    - `description(locale)` 方法：根据语言环境返回工具的描述。
    - `user_guide(locale)` 方法：根据语言环境返回工具的用户指南。
    - `input_schema(locale)` 方法：返回工具输入参数的 JSON Schema。
    - `output_schema(locale)` 方法：返回工具输出结果的 JSON Schema。
    - `run(input)` 方法：异步执行工具的核心逻辑，将输入的 JSON 值转换为拼音，并返回 JSON 结果。

## 5. 错误处理 (Error Handling)
- JSON 解析错误 -> 输出 stderr 并退出非 0。
- 转换过程一般不会失败，非汉字直接保留。
