---
version: v3.0.0
updated: 2026-03-01
---

# 三错即停

**定义**: 同一 Skill 同一步骤连续失败 3 次时触发的熔断机制

---

## 流程

| Strike | 条件 | 行动 | 后果 |
|--------|------|------|------|
| 1 | 同一 Skill 失败 | 同 Skill 内自检与修正 | 允许继续 |
| 2 | 再失败 | 调用 `sop-code-explorer` + **设计类 Skill** 复核并微调 | 暂停自动推进 |

**设计类 Skill**（由 sop-progress-supervisor 根据失败环节选择）：sop-implementation-designer、sop-architecture-design、sop-architecture-reviewer、sop-design-placement。完整层级见 [02_skill_matrix/index.md](02_skill_matrix/index.md)。
| 3 | 再失败 | **熔断**：由 `sop-progress-supervisor` 生成报告并停止 | 必须人工决策 |

---

## 熔断恢复

1. 用户决策（进入 `[USER_DECISION]` 并落盘决策记录）
2. 方案调整（更新设计/验收/Scope 约束）
3. 知识沉淀（若引用了外部规范/最佳实践作为决策依据，必须沉淀到 RAG：`04_reference/knowledge_management.md`）
4. 重置计数器
5. 继续执行（从最近一个可验证停止点继续）

---

## 失败报告

```markdown
## 失败分析

### 基本信息
- 任务: [描述]
- 失败次数: 3/3

### 经过
- Strike 1: [问题] → [处理] → 失败
- Strike 2: [审计] → [调整] → 失败
- Strike 3: [最终问题]

### 建议
- A: [方案]
- B: [方案]

### 决策
- [ ] 方案A
- [ ] 方案B
- [ ] 重新设计
- [ ] 暂停
```
