# AI 项目通用规约 (Minimalist v5.6)

> **使用对象**: AI Agent / 人类用户 | **目标**: 最小Token消耗 + 清晰导航 | **版本**: v5.6

**一句话定义**：一套用于规范 AI 辅助开发的通用工作流标准，采用渐进式披露的信息架构。

---

## 核心原则

- **准度 > 速度**：严禁跳步，重复失败立即熔断
- **文档先行**：先变更文档（标记变更），再修改代码
- **渐进披露**：按需获取信息，避免信息过载
- **少即是多**：先复用已有能力，优先改进而非新建，新建后清理过时代码

👉 [查看核心概念详解](./01_concept_overview.md)

---

## 快速开始

```
接收任务 → Router分诊 → 选择路径 → 执行 → 完成
```

**路径选择**:
- 快速路径: 单文件 + <30行 + 无逻辑变更
- 深度路径: 其他所有情况

---

## 快速导航

| 层级 | 内容 | 适用场景 |
|------|------|----------|
| **L1** | [核心概念与价值](./01_concept_overview.md) | 快速了解规约全貌 |
| **L2** | [角色矩阵](./02_role_matrix/index.md) | 了解各角色职责与权限 |
| **L3** | [工作流规范](./03_workflow/index.md) | 执行具体开发任务 |
| **L4** | [参考文档](./04_reference/index.md) | 查阅模板和技术细节 |

---

## 角色总览（9个）

👉 [查看完整角色矩阵](./02_role_matrix/index.md) | [角色速查卡](./ROLE_CHEATSHEET.md)

| 角色 | 层级 | 职责 | 详情 |
|------|------|------|------|
| **Router** | 规划 | 任务分诊与调度 | [→ Prompt](prompts/router_prompt.md) |
| **Explorer** | 规划 | 代码审计与影响评估 | [→ 详情](02_role_matrix/explorer.md) |
| **Analyst** | 需求 | 需求分析与 PRD 生成 | [→ Prompt](prompts/analyst_prompt.md) |
| **Prometheus** | 设计 | 架构设计与伪代码编写 | [→ Prompt](prompts/prometheus_prompt.md) |
| **Skeptic** | 设计 | 架构审查与质疑 | [→ Prompt](prompts/skeptic_prompt.md) |
| **Oracle** | 设计 | 实现设计与方案对比 | [→ Prompt](prompts/oracle_prompt.md) |
| **Worker** | 实现 | 物理编码与质量检查 | [→ Prompt](prompts/worker_prompt.md) |
| **Librarian** | 监管 | 文档索引与渐进披露维护 | [→ 详情](02_role_matrix/librarian.md) |
| **Supervisor** | 监管 | 进度监管与熔断决策 | [→ 详情](02_role_matrix/supervisor.md) |

---

## 工作流概览

👉 [查看工作流详情](./03_workflow/index.md)

### 深度路径
```
新项目/大重构: Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
功能迭代:      Analyst → Oracle → Worker → Librarian
```

### 快速路径
```
Explorer → Worker → Librarian
```

---

## 停止点速查

| 阶段 | 停止点标记 | 等待内容 |
|------|-----------|----------|
| Analyst | `[WAITING_FOR_REQUIREMENTS]` | 用户确认PRD |
| Prometheus | `[WAITING_FOR_ARCHITECTURE]` | 架构审批 |
| Skeptic | `[ARCHITECTURE_PASSED]` | 审查通过 |
| Oracle | `[WAITING_FOR_DESIGN]` | 设计审批 |
| Worker | Diff展示 | 用户审批代码 |

---

## 三错即停

| Strike | 触发条件 | 行动 |
|--------|----------|------|
| 1 | Worker失败 | 自动修正 |
| 2 | 再次失败 | @Explorer+@Oracle 审计+微调 |
| 3 | 第三次失败 | **熔断**,生成FAILURE_REPORT |

👉 [查看详细规则](./03_workflow/three_strike_rule.md)

---

## 文档位置

### 项目设计文档

| 文档类型 | 位置 | 创建者 | 特点 |
|----------|------|--------|------|
| PRD | `docs/01_requirements/*.md` | Analyst | 业务导向 |
| 架构设计 | `docs/02_logical_workflow/*.pseudo` | Prometheus | 技术无关 |
| 实现设计 | `src/**/design.*` 或 `docs/**/design.md` | Oracle | 项目特定，基于模块划分 |

### 目录权限

| 目录 | 用途 | 变更权限 |
|------|------|----------|
| `/docs` | 项目设计文档 | 开发过程中动态创建和更新 |
| `/docs/参考/` | SOP 参考文档、模板 | **非指定不变更**，仅 Librarian 维护 |

### design.md 创建规则

- **基于模块划分**：每个独立模块在根目录创建 `design.md`
- **基于复杂度**：低复杂度可省略，中高复杂度必须创建
- **接口契约**：必须包含输入/输出/依赖接口定义

👉 [查看详细规则](./04_reference/index.md)

---

## 关键约束

1. **文档先行**: 先标记`[进行中]`,再改代码
2. **渐进披露**: 父目录只保留摘要+链接
3. **状态标记**: `[进行中]`→`[已完成]`
4. **权限隔离**: 各角色只操作授权范围
5. **少即是多**: 先复用已有能力→优先改进→新建后清理过时/相似代码

---

## 开始使用

### 新用户
1. 阅读 [L1: 核心概念](./01_concept_overview.md) 了解全貌
2. 查看 [L2: 角色矩阵](./02_role_matrix/index.md) 了解分工
3. 根据任务类型选择 [L3: 工作流](./03_workflow/index.md)

### 老用户
- 直接跳转到需要的层级查阅细节
- 使用 [L4: 参考文档](./04_reference/index.md) 查找模板

---

## 版本信息

- **当前版本**: v5.6
- **最后更新**: 2026-02-07
- **主要变更**: 重构为渐进式披露结构，合并COMPACT版本

---

## 完整文档索引

```
docs/参考/sop/
├── AGENT_SOP.md                    # 本文件：统一入口
├── ROLE_CHEATSHEET.md              # 角色速查卡
├── 01_concept_overview.md          # L1: 核心概念与价值
├── 02_role_matrix/                 # L2: 角色定义与职责
│   ├── index.md                   # 角色矩阵总览
│   ├── router.md                  # 任务分诊角色
│   ├── explorer.md                # 代码审计角色
│   ├── analyst.md                 # 需求分析角色
│   ├── prometheus.md              # 架构设计角色
│   ├── skeptic.md                 # 架构审查角色
│   ├── oracle.md                  # 实现设计角色
│   ├── worker.md                  # 编码实现角色
│   ├── supervisor.md              # 进度监管角色
│   └── librarian.md               # 文档管理角色
├── 03_workflow/                    # L3: 工作流详细规范
│   ├── index.md                   # 工作流总览
│   ├── fast_path.md               # 快速路径
│   ├── deep_path.md               # 深度路径
│   └── three_strike_rule.md       # 三错即停机制
├── 04_reference/                   # L4: 参考文档与模板
│   ├── index.md                   # 参考文档首页
│   ├── document_templates/        # 文档模板
│   │   ├── prd.md
│   │   ├── architecture_design.md
│   │   └── implementation_design.md
│   └── interaction_formats/       # 交互格式
│       ├── design_review.md
│       └── supervisor_report.md
├── prompts/                        # AI Agent提示词
│   ├── router_prompt.md
│   ├── analyst_prompt.md
│   ├── prometheus_prompt.md
│   ├── skeptic_prompt.md
│   ├── oracle_prompt.md
│   └── worker_prompt.md
└── skills/                         # AI Agent Skill定义
    ├── sop-workflow-orchestrator/
    ├── sop-requirement-analyst/
    ├── sop-architecture-reviewer/
    ├── sop-progress-supervisor/
    ├── sop-document-sync/
    └── sop-capability-reuse/
```

---

**人类阅读版**: [sop_for_human.md](../sop_for_human.md) | **编写指南**: [sop_GUIDE.md](../sop_GUIDE.md)
