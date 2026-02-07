# 深度路径

**适用**: 跨文件/新功能/重构/API变更

---

## 流程

### 新项目/大重构
```
Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
```

### 功能迭代
```
Analyst → Oracle → Worker → Librarian
```

---

## 步骤

| 阶段 | 输入 | 输出 | 停止点 |
|------|------|------|--------|
| Analyst | 用户描述 | PRD | `[WAITING_FOR_REQUIREMENTS]` |
| Prometheus | PRD | 架构设计 | `[WAITING_FOR_ARCHITECTURE]` |
| Skeptic | 架构设计 | 审查报告 | `[ARCHITECTURE_PASSED]` |
| Oracle | 架构设计 | 实现设计 | `[WAITING_FOR_DESIGN]` |
| Worker | 实现设计 | 代码 | Diff展示 |
| Librarian | 代码 | 文档更新 | `[已完成]` |

---

## 审查循环

```
Prometheus设计 → Skeptic审查 → Prometheus回复 → ...
```

**终止条件**:
- 正常: 设计完善
- 异常: 3次无法回复/僵局/需用户决策

---

## 约束

- 必须遵循所有阶段
- 必须通过审查
- 三错即停适用
- 文档必须同步
