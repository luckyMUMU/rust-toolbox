# Rust 工具箱项目 AI 工作规范 (AI Work Protocol)

**生效对象**：所有参与本项目的 AI 辅助工具。
**最后更新**：2025-12-13

本规范旨在确保 AI 在协助开发 `Rust Toolbox` 项目时，能够保持高质量的代码输出、一致的架构风格以及完善的文档管理。

## 1. 核心原则 (Core Principles)

### 1.1 语言规范
- **默认语言**：所有交互、文档、代码注释、Commit Message 默认使用 **中文 (简体)**。
- **例外**：代码中的标识符（变量名、函数名等）必须使用标准的 **英语**。

### 1.2 设计优先 (Design First)
- **绝对准则**：严禁在没有设计文档的情况下直接编写实现代码。
- **执行流程**：
    1.  **Task**: 接收开发任务。
    2.  **Design**: 在对应模块目录下创建或更新 `DESIGN.md`。
        - **必须包含设计文档的层级**：
            - **根级 Crate**：(如 `rt-core`, `rt-tools`, `rt-cli`)
            - **具体工具模块**：(如 `rt-tools/src/text/convert_chinese/`)
            - **复杂子模块**：(任何包含复杂逻辑的独立子模块)
            - **插件项目**：(所有开发的插件)
        - **内容必须包含**：模块职责、公开 API (Traits/Structs) 定义、关键逻辑流程。
    3.  **Review**: 使用 `notify_user` 请求用户审查 `DESIGN.md`。
    4.  **Implement**: 只有在设计通过后，才开始编写 `src/*.rs` 代码。

### 1.3 文档闭环 (Documentation Loop)
- **同步更新**：代码的任何行为变更（API 修改、配置项增减、CLI 参数变化），必须同步修改对应的 `DESIGN.md`、`README.md` 和 `USER_GUIDE.md`。
- **自检机制**：每次完成代码写入后，必须检查："我的修改是否让文档过时了？"如果是，立即更新文档。

## 2. 编码规范 (Coding Standards)

### 2.1 Rust 风格
- **Idiomatic Rust**：遵循 Rust 官方惯用写法。优先使用标准库功能。
- **Formatting**：严格遵守 `rustfmt` 标准。
- **Linting**：代码必须通过 `clippy` 检查，消除所有 Warnings。
- **Error Handling**：
    - 禁止使用 `unwrap()` 或 `expect()` 在生产逻辑中（Test 除外）。
    - 必须使用 `Result<T, E>` 传播错误。
    - 使用 `thiserror` 或 `anyhow` 进行错误管理。

### 2.2 项目结构
- 遵循 Cargo Workspace 标准结构：
    ```
    / (Root)
    ├── Cargo.toml          # Workspace definition
    ├── AI_WORK_PROTOCOL.md
    ├── README.md
    ├── USER_GUIDE.md
    ├── plugins/            # 外部插件目录
    ├── rt-core/            # 核心库: Tool trait, Plugin system
    ├── rt-tools/           # 内置工具集
    ├── rt-cli/             # 命令行入口
    └── rt-gui/             # 图形界面入口
    ```

### 2.3 模块化与解耦
- **Core-Logic 分离**：业务逻辑严禁耦合在 CLI 或 GUI 层。所有逻辑必须在 `rt-core` 或 `rt-tools` 中实现。
- **UI 无状态**：CLI 和 GUI 仅作为"皮"，负责收集输入、调用 Core、展示输出。
- **动态注册**：工具通过 `rt_tools::get_all_tools()` 统一注册，避免硬编码。

### 2.4 工具开发规范
每个新工具必须遵循以下结构：

```
rt-tools/src/{category}/{tool_name}/
├── DESIGN.md       # [必须] 工具详细设计文档
├── mod.rs          # 核心逻辑，实现 Tool trait
└── locales/        # [必须] 多语言资源目录
    ├── tool.en.json
    └── tool.zh-CN.json
```

**Tool trait 实现要求**：
- `name()`: 返回唯一标识符 (格式: `category.tool_name`)
- `display_name(locale)`: 使用 `ToolI18n` 获取 localized string
- `description(locale)`: 使用 `ToolI18n` 获取 localized string
- `user_guide(locale)`: 使用 `ToolI18n` 获取 localized markdown
- `input_schema(locale)`: 使用 `ToolI18n` 为 schema 注入 localized title
- `output_schema(locale)`: 使用 `ToolI18n` 为 schema 注入 localized title
- `run(input)`: 异步执行逻辑

**多语言资源文件 (JSON) 要求**：
- 必须包含 `display_name`, `description`, `user_guide`。
- `input_schema` 和 `output_schema` 字段用于定义字段标题。
- `extra` 字段用于定义枚举值标签等额外信息。

### 2.5 依赖选择原则
- **优先纯 Rust**：避免依赖需要 C/C++ 编译的库，确保跨平台编译顺畅。
- **示例**：使用 `ferrous-opencc` 而非 `opencc-rust`（后者需要 pkg-config 和 C++ 库）。

## 3. 工作流规范 (Workflow Guidelines)

### 3.1 任务管理
- 使用 `TodoWrite` 工具跟踪进度。
- 每次开始一大块工作前，先根据任务设定边界。

### 3.2 提交策略
- 代码变更应按逻辑单元分批写入，避免一次性生成无法调试的巨型文件。
- 对于复杂的重构，先创建新文件验证，再替换旧文件。

### 3.3 新工具开发流程
1. **更新 DESIGN.md**: 
   - 在 `rt-tools/DESIGN.md` 中添加工具规格。
   - 在 `rt-tools/src/{category}/{tool_name}/` 下创建该工具专属的 `DESIGN.md`。
2. **创建实施计划**: 编写 `implementation_plan.md` (如果任务复杂)。
3. **用户评审**: 使用 `notify_user` 请求审查。
4. **实现代码**:
   - 创建工具目录和 `locales/` JSON 文件。
   - 实现 `mod.rs` 中的 Tool trait，使用 `i18n_utils::ToolI18n` 加载资源。
   - 在 `rt-tools/src/lib.rs` 中注册工具。
5. **验证**: 运行 `cargo build`、`rt-cli list`、`rt-gui` 测试。

## 4. 插件系统 (Plugin System)

### 4.1 插件协议
外部插件必须是可执行文件，且必须在源码库中包含 `DESIGN.md` 文档。
支持以下命令：
- `plugin spec`: 输出 JSON 格式的工具元数据
- `plugin run`: 从 stdin 读取 JSON 输入，向 stdout 输出 JSON 结果

### 4.2 插件发现
- 插件放置在项目根目录的 `plugins/` 文件夹
- 文件名建议以 `rt-plugin-` 为前缀
- 启动时自动扫描并加载

## 5. 紧急制动 (Emergency Stop)
- 如果发现当前的架构设计无法满足新需求，**立即停止编码**。
- 回退到 **PLANNING** 模式，修改全局设计文档 `implementation_plan.md`，直到路径清晰。

## 6. 变更日志 (Changelog)

### [2025-12-13]
- **架构重构**: `rt-tools` 多语言实现从硬编码的 `i18n.rs` 迁移至独立的 JSON 资源文件 (`locales/*.json`)。
- **新增模块**: `rt-tools/src/i18n_utils.rs` 用于运行时加载和解析多语言配置。
- **依赖移除**: 移除了 `rt-i18n-codegen` 及其相关构建依赖。
- **规范更新**: 更新了工具开发规范，明确了 JSON 资源文件的结构和用途。
- **文档同步**: 更新了 `AI_WORK_PROTOCOL.md` 以反映最新的项目结构和开发流程。

---
*请严格遵守以上规范执行开发任务。*
