# rt-gui Design Document

## 1. 模块概述 (Module Overview)
`rt-gui` 提供了一个图形化界面，允许用户直观地发现、配置和运行工具。

## 2. 技术选型 (Tech Stack)
- **GUI Framework**: `eframe` (egui wrapper). 纯 Rust，即时模式 GUI，适合工具类应用。
- **Runtime**: `tokio` (用于异步执行工具)。

## 3. 界面设计 (UI Design)

### 3.1 主布局
- **Left Panel (Sidebar)**: 工具列表。
- **Central Panel**: 当前选中工具的配置与执行区域。
- **Bottom Panel (Log)**: 执行日志/输出结果。

### 3.2 交互逻辑
1. **Tool Discovery**: 启动时注册所有可用工具。
2. **Tool Selection**: 点击侧边栏工具名称，中间面板显示该工具的输入表单。
3. **Execution**:
   - 用户填写表单（对于 `file.move_folder`，提供 Source, Dest, Overwrite 输入框）。
   - 点击 "Run" 按钮。
   - 触发异步任务执行 `tool.run(json)`.
   - 结果显示在底部面板。

## 4. 详细设计 (Detailed Design)

### 4.1 数据结构
```rust
struct ToolkitApp {
    tools: HashMap<String, Box<dyn Tool>>,
    selected_tool: Option<String>,
    input_buffer: String, // 暂时使用 JSON 字符串作为通用输入，后续可根据 Tool Schema 渲染表单
    output_log: String,
}
```

### 4.2 异步与 UI 通信
由于 `eframe` 是同步渲染循环，工具执行需在后台线程（Tokio Runtime）中运行。
使用 `std::sync::mpsc` 或 `tokio::sync::mpsc` 传递执行结果回主线程。

```rust
enum AppMessage {
    ToolFinished(Result<Value, String>),
}
```

## 5. MVP 范围
本次仅实现 `file.move_folder` 的基础支持：
- 侧边栏列出工具。
- 中间面板提供一个简单的 JSON 文本编辑框供输入参数。
- 运行按钮。
- 结果展示。
*(未来优化：根据 Tool Input Schema 动态生成特定的 UI 表单)*
