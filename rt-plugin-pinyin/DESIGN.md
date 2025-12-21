# 模块名称：汉字转拼音插件

## 1. 目标 (Goal)
- **核心功能**：将中文文本转换为拼音，支持声调控制和多音字处理，作为 Rust Toolbox 的外部插件运行。

## 2. 核心定义 (Definitions)
- **PinyinTool**：插件的核心实现结构体，实现了 `rt_core::tool::Tool` trait。
- **PinyinInput**：输入参数结构体，包含文本和声调控制选项。
- **PinyinOutput**：输出结果结构体，包含转换后的拼音文本。
- **ToolI18n**：国际化资源管理器，用于加载不同语言的显示名称和描述。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心流程
1. **命令解析**：解析命令行参数，识别 `spec` 或 `run` 命令。
2. **工具初始化**：创建 `PinyinTool` 实例，加载国际化资源。
3. **输入处理**：
   - 解析 JSON 输入，验证必填字段。
   - 提取中文文本和声调控制选项。
4. **拼音转换**：
   - 遍历输入文本的每个字符。
   - 对每个汉字，查询拼音映射表获取对应的拼音。
   - 根据声调选项决定是否添加声调符号。
   - 非汉字字符直接保留。
5. **结果输出**：将转换后的拼音文本格式化为 JSON 输出。

### 算法细节
- **拼音映射**：使用预构建的汉字到拼音的映射表，支持多音字处理。
- **声调处理**：根据输入参数决定是否在拼音中包含声调符号。
- **非汉字处理**：直接保留非汉字字符，不进行转换。

### 伪代码
```
function convert_to_pinyin(text, with_tone):
    result = ""
    for char in text:
        if is_chinese(char):
            pinyin_list = get_pinyin(char)
            selected_pinyin = select_correct_pinyin(pinyin_list, context)
            if with_tone:
                result += add_tone_marks(selected_pinyin)
            else:
                result += remove_tone_marks(selected_pinyin)
        else:
            result += char
    return result
```

### 复杂度分析
- 时间复杂度：O(n)，其中 n 是输入文本的长度，每个字符的处理时间为常量。
- 空间复杂度：O(n)，需要存储转换后的拼音结果。

## 4. 接口契约 (Interface)

### 插件元数据
- **名称**: `text.pinyin`
- **显示名称**: 通过 `ToolI18n` 从 `locales` 目录加载，例如 `汉字转拼音` (zh) 或 `Chinese to Pinyin` (en)。
- **描述**: 通过 `ToolI18n` 从 `locales` 目录加载，例如 `将中文字符转换为拼音。` (zh) 或 `Convert Chinese characters to Pinyin.` (en)。

### 输入 Schema
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

### 输出 Schema
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

### Tool Trait 实现
- `name()`：返回工具的唯一名称 `text.pinyin`。
- `display_name(locale)`：根据语言环境返回工具的显示名称。
- `description(locale)`：根据语言环境返回工具的描述。
- `user_guide(locale)`：根据语言环境返回工具的用户指南。
- `input_schema(locale)`：返回工具输入参数的 JSON Schema。
- `output_schema(locale)`：返回工具输出结果的 JSON Schema。
- `run(input)`：异步执行工具的核心逻辑，将输入的 JSON 值转换为拼音，并返回 JSON 结果。

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，完善算法描述，统一命名规范 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20