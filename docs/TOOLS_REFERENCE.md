# 工具与插件参考指南

本文档详细介绍了 Rust Tool V2 系统中可用的内置工具和运行时插件。

## 1. 内置工具 (File Management Plugin)

这些工具是 `FileManagementPlugin` 的一部分，直接集成在引擎中，提供高效的文件操作和流程控制能力。

### 1.1 基础工具

#### ac-matcher (模式匹配工具)
- **描述**: 基于 Aho-Corasick 算法的高性能多模式字符串匹配工具，支持并行搜索多个关键字。
- **用途**: 快速检查文本中是否包含敏感词、特定标记或分类关键词。
- **参数 Schema**:
  ```json
  {
    "text": "待搜索的文本内容",
    "patterns": [
      {
        "pattern": "搜索关键字",
        "category": "类别标记",
        "score": 1.0
      }
    ],
    "case_sensitive": false,
    "find_overlapping": false
  }
  ```

#### directory-scanner (目录扫描工具)
- **描述**: 扫描目录结构，列出文件和子目录。
- **参数 Schema**:
  ```json
  {
    "path": "扫描路径",
    "recursive": false,
    "scan_type": "files" // "files" | "directories" | "both"
  }
  ```
- **输出**: 返回包含文件路径、名称、类型、大小等信息的列表。

#### text-processor (文本处理工具)
- **描述**: 提供文本规范化、清洗和转换功能。
- **用途**: 在处理文件内容前进行预处理。

### 1.2 文件操作工具

#### file-mover (文件移动/复制工具)
- **描述**: 执行文件或文件夹的移动、复制、链接操作，内置多种冲突解决策略。
- **参数 Schema**:
  ```json
  {
    "operations": [
      {
        "source": "源路径",
        "destination": "目标路径",
        "operation_type": "Move" // "Move" | "Copy" | "Link" | "HardLink"
      }
    ],
    "conflict_resolution": "Rename", // "Skip" | "Overwrite" | "Rename" | "Fail" | "Ask" | "Merge" | "KeepBoth" ...
    "check_disk_space": true,
    "create_directories": true
  }
  ```

#### folder-merger (文件夹合并工具)
- **描述**: 智能合并文件夹，支持基于大小、日期或智能策略的重复文件处理。
- **参数 Schema**:
  ```json
  {
    "source_directories": ["目录1", "目录2"],
    "merge_strategy": "SizeBased", // "SizeBased" | "DateBased" | "Intelligent"
    "duplicate_handling": "Rename",
    "min_confidence_threshold": 0.7
  }
  ```

### 1.3 高级流程工具

#### folder-classifier (文件夹分类工具)
- **描述**: 基于预定义规则对文件夹内容进行分析和分类。
- **参数 Schema**:
  ```json
  {
    "folder_path": "待分类文件夹路径",
    "classification_rules": { ... }, // 规则对象
    "output_format": "Detailed" // "Simple" | "Detailed" | "Full"
  }
  ```

#### batch-processor (批量处理器)
- **描述**: 对一组输入执行批量处理逻辑。

### 1.4 人机交互与确认工具

#### human-decision (人工决策工具)
- **描述**: 暂停工作流执行，等待人工用户输入决策或数据。
- **用途**: 需要人工审核的关键步骤。

#### batch-confirmer (批量确认工具)
- **描述**: 在执行大批量操作前，汇总信息并请求确认。

#### result-reviewer (结果审查工具)
- **描述**: 展示中间结果供用户审查。

#### result-confirmer (综合结果确认工具)
- **描述**: 工作流结束前的最终确认步骤。

---

## 2. 运行时插件 (Runtime Plugins)

运行时插件提供了在隔离环境或不同语言运行时中执行代码的能力。

### 2.1 Docker Plugin
- **类型**: `PluginType::Docker`
- **能力**: 
  - 在 Docker 容器中执行命令。
  - 管理容器生命周期（创建、启动、停止、删除）。
  - 支持自定义镜像、环境变量、卷挂载和网络配置。
- **配置项**:
  - `image`: Docker 镜像名称。
  - `command`: 执行命令。
  - `mounts`: 文件挂载映射。
  - `environment`: 环境变量注入。
  - `resource_limits`: CPU/内存限制。

### 2.2 Node.js Plugin
- **类型**: `PluginType::NodeJs`
- **能力**: 
  - 执行 JavaScript/TypeScript 代码。
  - 管理 npm 依赖。
  - 提供 Node.js 运行时环境。

### 2.3 Python Plugin
- **类型**: `PluginType::Python`
- **能力**: 
  - 执行 Python 脚本。
  - 管理 pip 依赖。
  - 提供 Python 虚拟环境支持。

### 2.4 Native Plugin
- **类型**: `PluginType::Native`
- **能力**: 
  - 直接在宿主机操作系统上执行 Shell 命令。
  - **注意**: 具有最高权限，需谨慎使用。
