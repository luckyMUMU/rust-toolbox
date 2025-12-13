# Design: Chinese Converter (text.convert_chinese)

## 1. 模块概述 (Module Overview)
`text.convert_chinese` 是一个用于在简体中文和繁体中文（包括台湾、香港变体）之间进行转换的工具。它底层封装了 `ferrous-opencc` 库。

## 2. 核心职责 (Core Responsibilities)
- 提供多种中文简繁转换模式。
- 确保转换的高性能和准确性（基于 OpenCC 字典）。
- 提供友好的多语言元数据（输入/输出 Schema、用户指南）。

## 3. 详细设计 (Detailed Design)

### 3.1 接口定义 (Tool Trait)
- **Name**: `text.convert_chinese`
- **Input Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "text": { "type": "string", "title": "文本" },
      "mode": { 
        "type": "string", 
        "enum": ["s2t", "t2s", ...],
        "x-enum-labels": { "s2t": "简体到繁体", ... } 
      }
    },
    "required": ["text", "mode"]
  }
  ```
- **Output Schema**:
  ```json
  {
    "type": "object",
    "properties": {
      "converted": { "type": "string", "title": "转换结果" }
    }
  }
  ```

### 3.2 逻辑流程
1. **Input Parsing**: 将输入 JSON 解析为 `ConvertChineseInput` 结构体。
2. **Config Mapping**: 将字符串模式（如 `s2t`）映射为 `ferrous_opencc::config::BuiltinConfig` 枚举。
3. **Initialization**: 使用指定配置初始化 `OpenCC` 实例。
4. **Conversion**: 调用 `converter.convert(&text)` 执行转换。
5. **Output**: 将结果封装为 `ConvertChineseOutput` 并序列化为 JSON。

### 3.3 错误处理
- **InvalidInput**: 如果 `mode` 不在支持列表中，返回错误。
- **ToolFailure**: 如果 OpenCC 初始化失败（通常是字典加载失败），返回错误。

## 4. 依赖 (Dependencies)
- `ferrous-opencc`: 纯 Rust 实现的 OpenCC 绑定，无需 C++ 依赖。
