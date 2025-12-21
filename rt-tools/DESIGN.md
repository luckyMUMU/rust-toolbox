# 模块名称：rt-tools 工具集

## 1. 目标 (Goal)
- **核心功能**：包含 Rust Toolbox 平台的内置功能具体工具实现。所有工具必须实现 `rt-core::Tool` trait，并遵循国际化、模式定义和错误处理的既定模式。
- **非目标**：不包含外部插件，不包含命令行入口，不包含 GUI 实现。

## 2. 核心定义 (Definitions)
- **ToolI18n**：国际化资源管理器，用于加载和管理本地化资源。
- **ToolRegistry**：工具注册中心，负责工具的发现、实例化和管理。
- **ToolConfig**：工具配置结构体，包含工具的元数据和配置信息。
- **ToolResult**：工具执行结果结构体，包含执行状态、输出数据和错误信息。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心流程
1. **工具注册**：
   - 使用 `register_tool!` 宏自动注册工具。
   - 处理工具发现和实例化。
   - 与核心工具注册表集成。
   - 自动生成和验证模式。

2. **工具执行**：
   - 接收 JSON 格式的输入参数。
   - 验证输入参数符合模式要求。
   - 执行工具的核心逻辑。
   - 返回 JSON 格式的执行结果。

3. **国际化处理**：
   - 加载工具的本地化资源。
   - 根据语言环境返回本地化的显示名称和描述。
   - 将本地化的字段标题注入到 JSON 模式中。

### 架构组件

#### 工具注册系统
- **自动注册**：使用 `register_tool!` 宏自动注册工具。
- **动态发现**：支持运行时动态发现工具。
- **统一管理**：所有工具通过核心工具注册表进行统一管理。

#### 国际化框架
- **JSON 本地化资源**：基于 JSON 的本地化资源文件。
- **运行时语言切换**：支持运行时动态切换语言环境。
- **模式字段标题注入**：为动态 UI 生成注入本地化的字段标题。
- **一致的用户指南格式**：统一的用户指南格式。

#### 工具分类
- **文件操作 (`file`)**：包含文件和目录相关的工具。
- **文本处理 (`text`)**：包含文本处理和转换相关的工具。

## 4. 接口与边界 (Interface & Boundary)

### 工具注册接口
所有工具必须实现 `rt-core::Tool` trait，该 trait 定义了以下方法：
- `name()`：返回工具的唯一名称。
- `display_name(locale)`：根据语言环境返回工具的显示名称。
- `description(locale)`：根据语言环境返回工具的描述。
- `user_guide(locale)`：根据语言环境返回工具的用户指南。
- `input_schema(locale)`：返回工具输入参数的 JSON Schema。
- `output_schema(locale)`：返回工具输出结果的 JSON Schema。
- `run(input)`：异步执行工具的核心逻辑，返回执行结果。

### 可用工具

#### 文件操作 (`file`)

##### 移动文件夹 (`file.move_folder`)
- **名称**: `file.move_folder`
- **描述**: 移动或重命名文件夹，带冲突处理和验证
- **输入 Schema**:
  ```json
  {
    "source": "path/to/source",
    "destination": "path/to/dest",
    "overwrite": false
  }
  ```
- **输出 Schema**:
  ```json
  {
    "success": true,
    "moved_files": 10
  }
  ```
- **错误条件**:
  - 源路径不存在
  - 目标存在且覆盖为 false
  - 权限被拒绝
  - 无效路径格式

#### 文本操作 (`text`)

##### AC 自动机 (`text.ac_automaton`)
- **名称**: `text.ac_automaton`
- **描述**: 使用 Aho-Corasick 算法的多模式匹配工具
- **输入 Schema**:
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
- **输出 Schema**:
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
- **支持的操作**:
  - `add`: 向自动机添加模式
  - `remove`: 移除模式（需要确认）
  - `list`: 列出当前模式
  - `match`: 对文本执行模式匹配
  - `save`/`load`: 持久化操作（计划中）

##### 中文转换 (`text.convert_chinese`)
- **名称**: `text.convert_chinese`
- **描述**: 简体/繁体中文文本转换
- **输入 Schema**:
  ```json
  {
    "text": "简体中文",
    "mode": "s2t"
  }
  ```
- **输出 Schema**:
  ```json
  {
    "converted": "繁體中文"
  }
  ```
- **转换模式**:
  - `s2t`: 简体到繁体
  - `t2s`: 繁体到简体
  - `s2tw`: 简体到台湾繁体
  - `tw2s`: 台湾繁体到简体
  - `s2hk`: 简体到香港繁体
  - `hk2s`: 香港繁体到简体
  - `s2twp`: 简体到台湾繁体（带短语）
  - `tw2sp`: 台湾繁体到简体（带短语）

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一语言为中文，添加变更记录，重命名为小写 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20

## 附加信息

### 项目结构
```
rt-tools/
├── Cargo.toml                    # 依赖和元数据
├── design.md                     # 此架构文档
├── PUBLIC_TOOLS.md               # 公共 API 文档
└── src/
    ├── lib.rs                    # 工具注册和导出
    ├── utils/                    # 共享工具模块
    │   ├── mod.rs                # 核心类型重新导出
    │   └── i18n.rs               # 国际化助手
    ├── file/                     # 文件操作工具
    │   ├── mod.rs                # 文件类别模块
    │   └── move_folder/          # 移动文件夹工具
    │       ├── design.md         # 工具特定设计
    │       ├── mod.rs            # 实现
    │       └── locales/          # 本地化资源
    │           ├── tool.en.json  # 英文翻译
    │           └── tool.zh-CN.json # 中文翻译
    └── text/                     # 文本处理工具
        ├── mod.rs                # 文本类别模块
        ├── ac_automaton/         # AC 自动机工具
        │   ├── mod.rs            # 实现
        │   ├── tests.rs          # 单元测试
        │   └── locales/          # 本地化资源
        │       ├── tool.en.json  # 英文翻译
        │       └── tool.zh-CN.json # 中文翻译
        └── convert_chinese/      # 中文转换工具
            ├── design.md         # 工具特定设计
            ├── mod.rs            # 实现
            └── locales/          # 本地化资源
                ├── tool.en.json  # 英文翻译
                └── tool.zh-CN.json # 中文翻译
```

### Utils 模块架构

#### 核心类型重新导出
提供对 `rt-core` 中常用类型的统一访问：
- `Tool`: 核心工具 trait
- `Locale`: 语言枚举
- `Result`, `CoreError`: 错误处理类型
- `PersistenceManager`: 数据存储接口
- `WorkflowEngine`: 工作流执行引擎
- `WorkflowDefinition`, `WorkflowStatus`, `WorkflowInstance`: 工作流类型

#### 国际化助手 (`ToolI18n`)
管理工具的本地化资源：
- **基于 JSON 的配置**：从嵌入式 JSON 文件加载翻译
- **运行时语言切换**：支持动态语言变更
- **模式集成**：将本地化的字段标题注入到 JSON 模式中
- **结构化数据**：处理显示名称、描述、用户指南和字段标题

### 国际化系统

#### JSON 结构
每个工具维护以下结构的本地化文件：
```json
{
  "display_name": "工具显示名称",
  "description": "用户工具描述",
  "user_guide": "Markdown 格式的用户指南",
  "input_schema": {
    "field_name": { "title": "本地化字段标题" }
  },
  "output_schema": {
    "field_name": { "title": "本地化输出标题" }
  },
  "extra": {
    "custom_key": "自定义本地化值"
  }
}
```

#### 支持的语言环境
- **英语 (`en`)**：开发和文档的主要语言
- **中文 (`zh-CN`)**：面向中文用户的简体中文

### 工具开发指南

#### 实现要求
1. **Trait 实现**：所有工具必须实现 `rt_core::Tool`
2. **注册**：使用 `register_tool!` 宏进行自动发现
3. **异步支持**：所有工具执行必须支持异步
4. **错误处理**：使用 `rt_core::Result` 和 `CoreError` 类型
5. **模式验证**：输入/输出必须使用 `schemars::JsonSchema`

#### 测试标准
- **单元测试**：每个工具应具有全面的单元测试
- **集成测试**：测试工具注册和执行流程
- **模式验证**：验证输入/输出模式的正确性
- **本地化测试**：确保所有语言环境正确加载

#### 文档要求
- **工具特定的 design.md**：适用于具有多个功能的复杂工具
- **内联文档**：全面的 rustdoc 注释
- **用户指南**：本地化文件中的 Markdown 格式指南
- **示例用法**：在文档中包含实际示例

### 性能考虑

#### 延迟加载
- 工具仅在需要时实例化
- 本地化资源每个工具实例只加载一次
- 模式生成在可能的情况下进行缓存

#### 异步执行
- 所有工具支持异步执行，用于非阻塞操作
- 适用的工具（如 AC 自动机）支持并行处理
- 资源清理自动处理

#### 内存管理
- 工具使用最小的内存占用
- 大型数据结构在可能的情况下以流方式处理
- 临时资源在执行后正确清理