# 模块名称：rt-cli 命令行入口

## 1. 目标 (Goal)
- **核心功能**：Rust Toolbox 的命令行入口，负责加载工具、解析用户命令并调用核心逻辑执行。

## 2. 核心定义 (Definitions)
- **ToolRegistry**：动态工具注册中心，使用 `HashMap<String, Box<dyn Tool>>` 存储所有可用工具。
- **CommandArgs**：命令行参数结构，由 `clap` 自动生成。
- **ToolResult**：工具执行结果，包含 JSON 格式的输出数据。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心流程
1. **工具加载**：
   - 调用 `rt_tools::get_all_tools()` 获取内置工具。
   - 调用 `rt_core::plugin::load_plugins("plugins")` 扫描并加载外部插件。
   - 将所有工具注册到 `ToolRegistry` 中。

2. **命令解析**：
   - 使用 `clap` 解析命令行参数。
   - 根据命令类型分发到对应的处理函数。

3. **命令执行**：
   - `list` 命令：遍历 `ToolRegistry` 并打印工具信息。
   - `run` 命令：查找指定工具，解析 JSON 输入，调用工具执行，打印结果。
   - `workflow` 命令：(未来功能) 运行工作流文件。

4. **错误处理**：
   - 统一捕获 `anyhow::Result`。
   - 使用 `tracing` 或 `eprintln!` 打印错误信息。

### 复杂度分析
- 工具加载：O(n)，n 为工具数量。
- 命令解析：O(1)，由 `clap` 高效处理。
- 工具查找：O(1)，基于哈希表实现。

## 4. 接口与边界 (Interface & Boundary)

### 命令行结构
```bash
rt-cli [OPTIONS] <COMMAND>

Commands:
  list              列出所有可用工具
  run <TOOL_NAME>   运行指定工具
    --input <JSON>  JSON 格式的输入参数
  workflow <FILE>   运行工作流文件 (Future)
```

### 依赖关系
- `clap`: 参数解析 (Features: `derive`)
- `tokio`: 异步运行时 (Features: `full`)
- `serde_json`: JSON 解析
- `rt-core`: 核心接口
- `rt-tools`: 工具实现

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一命名规范 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20