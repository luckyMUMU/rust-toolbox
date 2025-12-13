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
├── file/
│   ├── mod.rs
│   └── move_folder/
│       ├── mod.rs
│       └── i18n.rs
└── text/
    ├── mod.rs
    └── convert_chinese/
        ├── mod.rs
        └── i18n.rs
```

## 4. 插件系统架构 (Plugin System Architecture)

为了支持扩展性，rt-box 支持通过外部可执行文件添加工具。

### 4.1 协议定义 (Protocol Definition)

插件必须是一个独立的可执行文件（如 `.exe`, `.py` 脚本等），并支持以下命令行交互：

#### 4.1.1 获取元数据 (Metadata)
- **Command**: `path/to/plugin spec`
- **Stdin**: (无)
- **Stdout**: JSON 对象，包含工具描述。
  ```json
  {
    "name": "ext.my_tool",
    "display_name": { "en": "My Tool", "zh": "我的工具" },
    "description": { "en": "...", "zh": "..." },
    "user_guide": { "en": "...", "zh": "..." },
    "input_schema": { ... }, // JSON Schema
    "output_schema": { ... } // JSON Schema
  }
  ```

#### 4.1.2 执行工具 (Execute)
- **Command**: `path/to/plugin run`
- **Stdin**: JSON 字符串 (Input Value)
- **Stdout**: JSON 字符串 (Output Value)
- **Stderr**: 错误日志 (用于调试，非结构化)
- **Exit Code**: 0 表示成功，非 0 表示失败。

### 4.2 发现机制 (Discovery)
- **路径**: 默认扫描应用根目录下的 `plugins/` 文件夹。
- **命名**: 建议以 `rt-plugin-` 前缀命名，以便识别。

### 4.3 核心实现 (Implementation)
- **`rt-core`**:
    - 新增 `PluginTool` 结构体，实现 `Tool` trait。
    - 负责调用子进程、序列化/反序列化 JSON、处理超时与错误。
- **`rt-cli` / `rt-gui`**:
    - 启动时扫描 `plugins/` 目录。
    - 为发现的每个有效可执行文件创建一个 `PluginTool` 实例并注册。

