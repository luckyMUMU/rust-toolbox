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
- **Display Name**:
    - `en`: "Chinese to Pinyin"
    - `zh-CN`: "汉字转拼音"
- **Description**:
    - `en`: "Convert Chinese characters to Pinyin."
    - `zh-CN`: "将中文字符转换为拼音。"

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
2.  **Spec Mode**:
    - 输出上述 JSON 元数据到 stdout。
3.  **Run Mode**:
    - 从 stdin 读取 JSON 输入。
    - 解析 `text` 和 `tone` 字段。
    - 调用 `pinyin` 库进行转换：
        - 遍历每个字符。
        - 若是汉字，获取其拼音（处理多音字取第一个，或简单处理）。
        - 若是非汉字，保留原样。
        - 拼接结果，以空格分隔。
    - 将结果封装为 JSON `{ "pinyin": "..." }` 输出到 stdout。

## 5. 错误处理 (Error Handling)
- JSON 解析错误 -> 输出 stderr 并退出非 0。
- 转换过程一般不会失败，非汉字直接保留。
