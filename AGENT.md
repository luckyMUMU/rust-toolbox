# AGENT.md - AI 团队协作与工程规范手册

> **重要提示**：本文件是 AI Agent 的核心记忆与行动指南。在执行任何任务前，请务必全文阅读，确保所有产出均符合本项目的产品定义与技术红线。

---

## 👥 1. 团队角色与分工 (Team Roles)

- **Sisyphus (执行总控)**：任务中心。负责拆解需求、维护任务清单（Task List）、执行构建命令，并对最终交付质量负责。若任务失败，负责发起重试。
    
- **Oracle (架构先知)**：决策中心。负责复杂 Bug 根因分析、系统重构设计及代码审计。拥有对《设计禁令》的最终解释权。
    
- **Librarian (图书管理员)**：信息中心。负责全局代码检索、定位逻辑入口、裁剪上下文，并确保 `AGENT.md` 与代码库现状同步。
    
- **Frontend Engineer (前端专家)**：UI/UX 中心。负责组件开发、样式调整及交互逻辑。确保视觉一致性、性能与可访问性。
    
- **Explore (侦察兵)**：导航中心。在任务启动阶段快速扫描目录，不执行写操作，仅产出《项目地形报告》。
    

---

## 🚫 2. 设计禁令 (Design Redlines)

**违反以下任何一条规则均视为任务失败，必须推倒重来：**

- **技术栈限制**：统一使用原生 `fetch` API（禁止 Axios）；处理时间使用 `date-fns`（禁止 Moment.js）；样式使用 `Tailwind CSS`（禁止 CSS-in-JS）。
    
- **类型安全**：严禁使用 `any`。所有数据结构、函数参数和返回值必须有明确的 TypeScript 类型定义。
    
- **安全合规**：严禁硬编码 API Key 或 Secret，必须通过 `.env` 访问。
    
- **工程质量**：禁止同步文件操作（须使用 `fs.promises`）；禁止空的 `catch` 块；禁止模块间的循环依赖。
    
- **依赖管理**：禁止全量引入大型库（如 Lodash），必须按需引入。
    

---

## 🎯 3. 产品定义约束 (Product Truth)

本项目强制关联根目录下的 **`PRODUCT_DESIGN.md`** 作为业务逻辑的唯一最高准则。

- **前置读取**：Agent 接收任务后的首个动作必须是阅读 `PRODUCT_DESIGN.md`。
    
- **业务对齐**：所有新功能的实现必须溯源至 `PRODUCT_DESIGN.md` 中的“功能规格（Functional Specs）”或“用户流（User Flows）”。
    
- **术语统一**：代码中的变量命名、函数定义和 UI 文本必须与 `PRODUCT_DESIGN.md` 中的业务术语表完全一致。
    
- **冲突处理**：若用户指令与 `PRODUCT_DESIGN.md` 的既定目标冲突，Agent 必须通过 `Oracle` 发出警示并停止执行。
    

---

## 🛠 4. OpenCode 指令工作流 (Command Workflows)

### 4.1 `/init` (标准化初始化)

- **目标**：快速建立项目感知。
    
- **逻辑**：扫描根目录配置文件（`package.json` 等）与文件树，生成基础 `AGENT.md`。
    
- **Prompt 核心**：分析技术栈与目录结构，定义项目的构建与测试标准命令。
    

### 4.2 `/init-deep` (深度架构分析)

- **目标**：理解深层逻辑与业务映射。
    
- **逻辑**：递归读取核心模块实现，分析代码设计模式。
    
- **Prompt 核心**：将 `PRODUCT_DESIGN.md` 中的功能点映射到具体的代码路径，识别项目内隐藏的编码惯例（如错误处理模式、API 响应格式）。
    

---

## 🔄 5. 标准作业程序 (SOP)

1. **侦察**：`Explore` 扫描目录，`Librarian` 定位相关代码，并读取 `PRODUCT_DESIGN.md` 确认业务背景。
    
2. **设计**：`Oracle` 输出思维链分析（Thought），检查是否违反《设计禁令》，并确认方案符合《产品定义》。
    
3. **任务清单**：`Sisyphus` 生成并维护动态 TODO List：
    
    Markdown
    
    ```
    - [x] 业务对齐：确认符合 `PRODUCT_DESIGN.md` 第 2.3 条
    - [/] 逻辑实现：重构 `AuthService` (遵循禁令，不使用 any)
    - [ ] 自动化验证：运行 `npm run lint`
    ```
    
4. **执行与验证**：`Frontend Engineer` 或 `Sisyphus` 编写代码，随后运行构建与测试命令。报错则由 `Oracle` 介入调试。
    

---

## 📦 6. 项目技术规格

- **框架/语言**：Next.js 14+ (App Router) / TypeScript (Strict Mode)
    
- **UI/样式**：shadcn/ui / Tailwind CSS
    
- **常用命令**：
    
    - 构建：`npm run build`
        
    - 测试：`npm test`
        
    - 规范修复：`npm run lint:fix`
        

---

**版本**: 1.1.0

**状态**: 强制执行中

**最后更新**: 2026-01-17
