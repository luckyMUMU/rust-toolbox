# Rust 工具箱项目 AI 工作规范 (AI Work Protocol)

**生效对象**：Antigravity Agent 及所有参与本项目的 AI 辅助工具。
**最后更新**：2025-12-10

本规范旨在确保 AI 在协助开发 `Rust Toolbox` 项目时，能够保持高质量的代码输出、一致的架构风格以及完善的文档管理。

## 1. 核心原则 (Core Principles)

### 1.1 语言规范
- **默认语言**：所有交互、文档、代码注释、Commit Message 默认使用 **中文 (简体)**。
- **例外**：代码中的标识符（变量名、函数名等）必须使用标准的 **英语**。

### 1.2 设计优先 (Design First)
- **绝对准则**：严禁在没有设计文档的情况下直接编写实现代码。
- **执行流程**：
    1.  **Task**: 接收开发任务。
    2.  **Design**: 在对应模块目录下创建或更新 `DESIGN.md`。内容必须包含：模块职责、公开 API (Traits/Structs) 定义、关键逻辑流程。
    3.  **Review**: 使用 `notify_user` 请求用户审查 `DESIGN.md`。
    4.  **Implement**: 只有在设计通过后，才开始编写 `src/*.rs` 代码。

### 1.3 文档闭环 (Documentation Loop)
- **同步更新**：代码的任何行为变更（API 修改、配置项增减、CLI 参数变化），必须同步修改对应的 `DESIGN.md` 和 `README.md`。
- **自检机制**：每次完成代码写入后，必须检查：“我的修改是否让文档过时了？”如果是，立即更新文档。

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
    ├── Cargo.toml (Workspace definition)
    ├── AI_WORK_PROTOCOL.md
    ├── rt-core/      (核心库)
    ├── rt-tools/     (工具集)
    ├── rt-cli/       (命令行入口)
    └── rt-gui/       (图形界面入口)
    ```

### 2.3 模块化与解耦
- **Core-Logic 分离**：业务逻辑严禁耦合在 CLI 或 GUI 层。所有逻辑必须在 `rt-core` 或 `rt-tools` 中实现。
- **UI 无状态**：CLI 和 GUI 仅作为“皮”，负责收集输入、调用 Core、展示输出。

## 3. 工作流规范 (Workflow Guidelines)

### 3.1 任务管理
- 使用 `task.md` 跟踪进度。
- 每次开始一大块工作前，先根据 `task.md` 设定 `task_boundary`。

### 3.2 提交策略
- 代码变更应按逻辑单元分批写入，避免一次性生成无法调试的巨型文件。
- 对于复杂的重构，先创建新文件验证，再替换旧文件。

## 4. 紧急制动 (Emergency Stop)
- 如果发现当前的架构设计无法满足新需求，**立即停止编码**。
- 回退到 **PLANNING** 模式，修改全局设计文档 `implementation_plan.md`，直到路径清晰。

---
*请严格遵守以上规范执行开发任务。*
