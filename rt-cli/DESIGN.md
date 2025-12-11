# rt-cli Design Document

## 1. 模块概述 (Module Overview)
`rt-cli` 是 Rust Toolbox 的命令行入口。它负责加载工具、解析用户命令并调用核心逻辑执行。

## 2. 命令行结构 (Command Structure)
使用 `clap` 定义子命令：

```bash
rt-cli [OPTIONS] <COMMAND>

Commands:
  list              列出所有可用工具
  run <TOOL_NAME>   运行指定工具
    --input <JSON>  JSON 格式的输入参数
  workflow <FILE>   运行工作流文件 (Future)
```

## 3. 详细设计 (Detailed Design)

### 3.1 依赖 (Dependencies)
- `clap`: 参数解析 (Features: `derive`)
- `tokio`: 异步运行时 (Features: `full`)
- `serde_json`: JSON 解析
- `rt-core`: 核心接口
- `rt-tools`: 工具实现

### 3.2 核心逻辑 (`main.rs`)

1. **Tool Registry**:
   需要一个简单的注册机制，将所有可用工具注册到一个 `HashMap<String, Box<dyn Tool>>` 中。
   ```rust
   fn register_tools() -> Vec<Box<dyn Tool>> {
       vec![
           Box::new(rt_tools::file::MoveFolder),
           // Future tools...
       ]
   }
   ```

2. **Command Handlers**:
   - `handle_list(tools)`: 遍历注册的工具并打印名称和描述。
   - `handle_run(tool_name, input_json)`: 
     - 查找工具。
     - 解析 JSON 输入。
     - 调用 `tool.run(input)`.
     - 打印结果 (JSON Pretty Print).

## 4. 错误处理
统一捕获 `anyhow::Result` 并打印红色错误信息 (使用 `tracing` 或 `eprintln!`)。
