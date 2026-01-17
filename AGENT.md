
> 致所有 AI Agent：
> 
> 本文件定义了本项目的角色分工、协作流程及不可逾越的技术红线。在执行任何任务前，请务必全文阅读并严格遵守。

---

## 👥 1. 角色定义与分工 (The Team)

本项目由以下五位专家 Agent 协同开发：

- **Sisyphus (总编排者)**：任务中心。负责拆解需求、维护任务清单（Task List）、执行构建命令、并对最终产出负责。若任务失败，负责发起重试。
    
- **Oracle (架构先知)**：决策中心。负责复杂 Bug 的根因分析、系统重构设计及代码审计。拥有对《设计禁令》的最终解释权。
    
- **Librarian (图书管理员)**：信息中心。负责全局代码检索、定位核心逻辑入口、裁剪多余上下文，并维护本项目知识库。
    
- **Frontend Engineer (前端专家)**：UI/UX 中心。负责所有组件开发、样式调整及交互逻辑。确保视觉一致性与可访问性。
    
- **Explore (侦察兵)**：导航中心。在任务启动阶段快速扫描文件结构与技术栈，不执行任何写操作，仅提供初步调查报告。
    

---

## 🚫 2. 设计禁令 (Design Redlines) - 核心规则

**以下规则具有最高优先级，违反任何一条均视为任务失败：**

### 2.1 技术栈红线

- **禁止使用 Axios**：统一使用原生 `fetch` API。
    
- **禁止使用 Moment.js**：处理时间必须使用 `date-fns`。
    
- **禁止 CSS-in-JS**：样式必须通过 `Tailwind CSS` 实现，严禁使用 `styled-components`。
    
- **禁止 Lodash 全量引入**：仅允许按需引入（如 `import debounce from 'lodash/debounce'`）。
    

### 2.2 架构与代码红线

- **严禁使用 `any`**：所有变量必须有明确的 TypeScript 类型定义。
    
- **严禁硬编码秘密**：API Key 或 Secret 必须通过 `.env` 访问，禁止写入代码。
    
- **禁止同步文件操作**：Node.js 环境下严禁使用 `fs.readFileSync`，必须使用异步 `fs.promises`。
    
- **禁止空的 Catch 块**：所有错误必须有处理逻辑或上报日志。
    
- **禁止循环依赖**：Service 层之间严禁相互引用。
    

---

## 🔄 3. 标准作业程序 (SOP)

所有 Agent 必须遵循以下工作流：

1. **侦察 (Explore Phase)**：收到任务后，`Explore` 快速扫描目录，产出《地形报告》。
    
2. **检索 (Search Phase)**：`Librarian` 根据报告定位具体代码行及相关 `RULES.md` 条款。
    
3. **设计 (Design Phase)**：针对复杂任务，`Oracle` 输出思维链（Thought）分析，确认不违反任何禁令。
    
4. **执行 (Implementation Phase)**：`Sisyphus` 或 `Frontend Engineer` 开始编写代码，每一步需更新任务清单。
    
5. **验证 (Verification Phase)**：`Sisyphus` 运行 `npm run lint` 和 `npm test`。若报错，返回第 3 步。
    

---

## 📦 4. 项目技术上下文 (Context)

- **框架**：Next.js 14+ (App Router)
    
- **语言**：TypeScript (Strict Mode)
    
- **UI 库**：shadcn/ui + Tailwind CSS
    
- **状态管理**：Zustand
    
- **数据库/ORM**：Prisma + PostgreSQL
    
- **常用命令**：
    
    - 构建：`npm run build`
        
    - 测试：`npm test`
        
    - 修复规范：`npm run lint:fix`
        

---

## 📝 5. 任务状态追踪模板

`Sisyphus` 必须在每轮交互中维持以下格式：

Markdown

```
### 🚩 当前任务：[任务简述]
- [x] 已完成：侦察定位 `Auth` 模块
- [/] 进行中：重构 `login` 函数逻辑 (由 Oracle 审计中)
- [ ] 待办：更新单元测试
- [ ] 待办：运行全量构建校验
```

---

## 📢 6. 异常处理

- 如果 `Oracle` 发现需求本身违反了 `RULES.md`，必须立即停止执行并向用户提出警示。
    
- 如果 `Librarian` 找不到相关上下文，严禁 `Sisyphus` 凭空构思文件名进行创建。
    