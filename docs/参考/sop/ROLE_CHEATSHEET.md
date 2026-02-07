# 角色速查

---

## 角色索引

| 角色 | 层级 | 职责 | 停止点 |
|------|------|------|--------|
| Router | 规划 | 任务分诊 | - |
| Explorer | 规划 | 代码审计 | - |
| Analyst | 需求 | 需求分析，PRD生成 | `[WAITING_FOR_REQUIREMENTS]` |
| Prometheus | 设计 | 架构设计 | `[WAITING_FOR_ARCHITECTURE]` |
| Skeptic | 设计 | 架构审查 | `[ARCHITECTURE_PASSED]` |
| Oracle | 设计 | 实现设计 | `[WAITING_FOR_DESIGN]` |
| Worker | 实现 | 编码实现 | Diff展示 |
| Librarian | 监管 | 文档维护 | `[已完成]` |
| Supervisor | 监管 | 进度监管，熔断 | `[FUSION_TRIGGERED]` |

---

## 路径

### 快速路径
```
Explorer → Worker → Librarian
```

### 深度路径
```
新项目: Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
功能迭代: Analyst → Oracle → Worker → Librarian
```

---

## 文档类型

| 类型 | 位置 | 创建者 |
|------|------|--------|
| PRD | `docs/01_requirements/*.md` | Analyst |
| 架构设计 | `docs/02_logical_workflow/*.pseudo` | Prometheus |
| 实现设计 | `src/**/design.md` | Oracle |

---

## 三错即停

| Strike | 条件 | 行动 |
|--------|------|------|
| 1 | Worker失败 | 自动修正 |
| 2 | 再失败 | @Explorer+@Oracle审计+微调 |
| 3 | 再失败 | **熔断**，Supervisor介入 |
