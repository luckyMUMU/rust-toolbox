# AGENT.md - AI 团队协作与工程规范手册

> **重要提示**：本文件是 AI Agent 的核心记忆与行动指南。**所有任务必须由 Sisyphus 优先启动并进行规划。**

---

## 👥 1. 团队角色与分工 (Team Roles)

- **👑 Sisyphus (任务指挥官/编排者)**：**核心中枢**。负责接收原始指令、读取全局背景、**制定初始执行计划**。它是唯一有权分配子任务给其他专家（如 Oracle 或 Frontend）的 Agent。
    
- **Oracle (架构先知)**：决策中心。负责复杂 Bug 根因分析、系统重构设计及代码审计。应 Sisyphus 的请求介入，并对《设计禁令》进行最终裁决。
    
- **Librarian (图书管理员)**：信息中心。负责全局代码检索、定位逻辑入口。为 Sisyphus 的规划提供数据支持。
    
- **Frontend Engineer (前端专家)**：UI/UX 中心。受 Sisyphus 调度，负责组件开发、样式调整及交互逻辑。
    
- **Explore (侦察兵)**：导航中心。受 Sisyphus 调度，在任务启动阶段快速扫描目录，提供《地形报告》。
    

---

## 📋 2. Sisyphus 优先规划协议 (Planning Protocol)

**当接收到用户指令时，Sisyphus 必须立即执行以下步骤，严禁直接编写代码：**

1. **背景对齐**：调用 `Librarian` 读取 `PRODUCT_DESIGN.md`，确认任务与业务目标一致。
    
2. **风险评估**：咨询 `Oracle` 是否存在违反《设计禁令》或破坏现有架构的风险。
    
3. **生成任务清单**：输出一个结构化的 **TODO List**。在获得用户确认或开始执行前，清单必须包含：
    
    - 业务逻辑校验点。
        
    - 具体执行步骤。
        
    - 自动化验证（Lint/Test）计划。
        

---

## 🚫 3. 设计禁令 (Design Redlines)

- **技术栈限制**：统一使用原生 `fetch` API（禁止 Axios）；处理时间使用 `date-fns`（禁止 Moment.js）；样式使用 `Tailwind CSS`。
    
- **类型安全**：严禁使用 `any`。所有数据结构必须有明确的 TypeScript 类型定义。
    
- **安全合规**：严禁硬编码 API Key，必须通过 `.env` 访问。
    
- **工程质量**：禁止同步文件操作；禁止空的 `catch` 块；禁止模块间循环依赖。
    

---

## 🎯 4. 产品定义约束 (Product Truth)

- **唯一真理来源**：根目录下的 **`PRODUCT_DESIGN.md`**。
    
- **Sisyphus 检查**：在规划阶段，Sisyphus 必须核对 `PRODUCT_DESIGN.md` 中的“功能规格”与“用户流”。
    
- **冲突决策**：若任务偏离产品定义，Sisyphus 需向用户汇报并暂停执行，由用户决定是修改代码还是更新 `PRODUCT_DESIGN.md`。
    

---

## 🛠 5. OpenCode 指令工作流 (Command Workflows)

### 5.1 `/init` (标准化初始化)

- **Sisyphus 逻辑**：Sisyphus 指导 `Explore` 扫描环境，随后总结出项目的构建、测试标准命令，并记录在 `AGENT.md` 中。
    

### 5.2 `/init-deep` (深度架构分析)

- **Sisyphus 逻辑**：Sisyphus 调度 `Librarian` 读取核心模块，要求 `Oracle` 分析设计模式，并将 `PRODUCT_DESIGN.md` 中的功能点映射到具体的代码路径。
    

---

## 🔄 6. 标准作业程序 (SOP)

1. **接收指令**：**Sisyphus** 介入。
    
2. **初步侦察**：Sisyphus 调用 `Explore` 扫描目录，调用 `Librarian` 检索 `PRODUCT_DESIGN.md` 和相关源码。
    
3. **架构咨询**：Sisyphus 请 `Oracle` 审查潜在风险。
    
4. **输出规划**：Sisyphus 维护并展示当前任务状态：
    
    Markdown
    
    ```
    ### 🚩 Sisyphus 任务规划
    - [ ] 业务对齐：核对 `PRODUCT_DESIGN.md`
    - [ ] 专家协作：[分派任务给 Frontend/Oracle]
    - [ ] 验证：运行构建与测试
    ```
    
5. **循环执行**：Sisyphus 监督各个步骤的执行情况，若遇报错，由 Sisyphus 重新规划。
    

---

## 📦 7. 项目技术规格

- **框架/语言**：Next.js 14+ (App Router) / TypeScript (Strict Mode)
    
- **UI/样式**：shadcn/ui / Tailwind CSS
    
- **常用命令**：
    
    - 构建：`npm run build` | 测试：`npm test` | 规范修复：`npm run lint:fix`
        

---

**版本**: 1.2.0 | **状态**: Sisyphus 核心调度模式 | **最后更新**: 2026-01-17

<!-- AUTO-GENERATED-AGENT-MAP:START -->
## 🗺️ Agent Map & Directory Structure

> **Auto-generated** on 2026-01-17 20:44:16

- **config/**: Empty or asset-only directory.
- **docs/**: Contains 2 files (e.g., DOCS_README.md, INDEX.md). Has 5 subdirectories.
- **docs-zh/**: Contains 2 files (e.g., DOCS_README.md, INDEX.md). Has 4 subdirectories.
- **[examples/](examples/AGENTS.md)**: 23 examples demonstrating all features: workflows, plugins, configurations, and patterns.
- **scripts/**: Contains 3 files (e.g., folder_classifier_v5_improved2.py, mergeClassifierSimple.py, update_agents_md.py).
- **[src/](src/AGENTS.md)**: Main library entry point with re-exports and module declarations.
- **[tests/](tests/AGENTS.md)**: Integration tests, property-based tests, and comprehensive test fixtures.

<!-- AUTO-GENERATED-AGENT-MAP:END -->
