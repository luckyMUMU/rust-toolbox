# rt-gui Design Document

## 1. 模块概述 (Module Overview)
`rt-gui` 提供了一个图形化界面，允许用户直观地发现、配置和运行工具。

## 2. 技术选型 (Tech Stack)
- **GUI Framework**: `eframe` (egui wrapper). 纯 Rust，即时模式 GUI，适合工具类应用。
- **Runtime**: `tokio` (用于异步执行工具)。
- **Helpers**: `egui_commonmark` (渲染 Markdown 帮助文档)。

## 3. 界面设计 (UI Design)

### 3.1 主布局
- **Top Panel**: 标题栏，包含 Locale 切换 (En/Zh)。
- **Left Panel (Sidebar)**: 工具列表，显示本地化名称和简短描述。
- **Central Panel**: 当前选中工具的配置与执行区域。
  - **Tool Header**: 工具名和 "See Help" 按钮。
  - **Form Area**: 根据 `input_schema` 动态渲染的输入表单。
  - **Raw Input**: (调试用) 原始 JSON 视图。
- **Right Panel (Collapsible)**: 帮助文档 (`user_guide`)。
- **Bottom Panel (Log)**: 执行输出 (`output_schema` 渲染) 或错误信息。

### 3.2 交互逻辑
1. **Tool Discovery**: 
   - 启动时初始化 Tokio Runtime。
   - 阻塞调用 `load_plugins` 加载插件。
   - 合并 `get_all_tools` 结果。
2. **Tool Selection**: 点击侧边栏工具名称，重置 I/O 状态，加载新 Schema。
3. **Execution**:
   - 用户在生成的表单中输入。
   - 点击 "Run" 按钮。
   - 使用 `tokio::spawn` 异步执行 `tool.run(input)`.
   - 通过 `mpsc` 发送结果回主线程。

## 4. 详细设计 (Detailed Design)

### 4.1 数据结构
```rust
struct ToolkitApp {
    tools: Arc<HashMap<String, Box<dyn Tool>>>,
    selected_tool_name: Option<String>,
    
    // UI State
    input_value: Value,       // 当前输入 JSON
    current_schema: Option<Value>, // 当前工具的 Input Schema
    
    output_value: Option<Value>,
    output_schema: Option<Value>,
    
    // I18n
    locale: Locale,
    
    // Runtime
    runtime: tokio::runtime::Runtime,
}
```

### 4.2 动态表单渲染
使用递归函数 `render_schema` 遍历 JSON Schema：
- `type: object`: 递归渲染 properties。
- `type: string`: 渲染文本框。
- `type: boolean`: 渲染 Checkbox。
- `title`: 优先显示 Schema 中的 `title` (支持 I18n 注入)。

## 5. MVP 范围
已实现：
- 完整的工具动态发现。
- 基于 JSON Schema 的动态表单生成。
- 多语言切换支持。
- 插件系统集成。
