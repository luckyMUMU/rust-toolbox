# AI Agent SOP

> AI Agent专用 | 命令式 | 最小Token

---

## 核心约束

1. 先标记`[进行中]`，再改代码
2. 父目录只保留摘要+链接
3. `[进行中]`→`[已完成]`
4. 各角色只操作授权范围
5. 先复用→改进→新建→清理

---

## 路径选择

| 路径 | 条件 |
|------|------|
| 快速 | 单文件+<30行+无逻辑变更 |
| 深度 | 其他所有情况 |

---

## 角色指令

| 角色 | 职责 | 输入 | 输出 | 停止点 |
|------|------|------|------|--------|
| Router | 任务分诊 | 用户请求 | 路径+角色分配 | - |
| Explorer | 代码审计 | 目标文件 | 审计报告 | - |
| Analyst | 需求分析 | 用户描述 | PRD | `[WAITING_FOR_REQUIREMENTS]` |
| Prometheus | 架构设计 | PRD | 架构设计 | `[WAITING_FOR_ARCHITECTURE]` |
| Skeptic | 架构审查 | 架构设计 | 审查报告 | `[ARCHITECTURE_PASSED]` |
| Oracle | 实现设计 | 架构设计 | 实现设计 | `[WAITING_FOR_DESIGN]` |
| Worker | 编码实现 | 实现设计 | 代码 | Diff展示 |
| Librarian | 文档维护 | 设计文档 | 索引更新 | `[已完成]` |
| Supervisor | 进度监管 | 执行状态 | 熔断决策 | `[FUSION_TRIGGERED]` |

---

## 工作流

**深度路径**
```
新项目: Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
功能迭代: Analyst → Oracle → Worker → Librarian
```

**快速路径**
```
Explorer → Worker → Librarian
```

---

## 三错即停

| Strike | 条件 | 行动 |
|--------|------|------|
| 1 | Worker失败 | 自动修正 |
| 2 | 再失败 | @Explorer+@Oracle审计+微调 |
| 3 | 再失败 | **熔断**，生成报告 |

---

## 文档位置

| 类型 | 位置 | 创建者 |
|------|------|--------|
| PRD | `docs/01_requirements/*.md` | Analyst |
| 架构设计 | `docs/02_logical_workflow/*.pseudo` | Prometheus |
| 实现设计 | `src/**/design.md` | Oracle |

**约束**: `/docs/参考/` **非指定不变更**

---

## design.md规则

| 复杂度 | 行数 | 要求 |
|--------|------|------|
| 低 | <100 | 省略，代码注释 |
| 中 | 100-500 | 简要design.md+接口契约 |
| 高 | >500 | 完整design.md+详细契约 |

---

## 导航

| 层级 | 文档 |
|------|------|
| L1 | [核心概念](01_concept_overview.md) |
| L2 | [角色矩阵](02_role_matrix/index.md) |
| L3 | [工作流](03_workflow/index.md) |
| L4 | [参考文档](04_reference/index.md) |
| Prompts | [prompts/](prompts/) |
| Skills | [skills/](skills/) |
