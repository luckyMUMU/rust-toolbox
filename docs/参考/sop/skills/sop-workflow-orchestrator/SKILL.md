---
name: "sop-workflow-orchestrator"
description: "SOP工作流编排器，负责任务分诊、角色调度和流程管理。Invoke when receiving a new task request to determine the appropriate workflow path and assign roles."
---

# SOP Workflow Orchestrator Skill

SOP工作流编排器，负责根据任务请求输出分诊结果与角色分配。

## 输入模板

```markdown
## 任务请求
[用户原始请求]

## 上下文信息
- 项目类型: [新项目/功能迭代/代码重构]
- 相关文件: [文件列表]
- 紧急程度: [高/中/低]
- 已知约束: [约束条件]
```

## 输出模板

```markdown
## 任务分诊结果

### 路径选择
- [ ] 快速路径 (Fast Path)
- [x] 深度路径 (Deep Path)

### 理由
[选择理由说明]

### 角色分配
| 阶段 | 角色 | 任务 |
|------|------|------|
| 1 | Analyst | 需求分析 |
| 2 | Prometheus | 架构设计 |
| 3 | Skeptic | 架构审查 |
| 4 | Oracle | 实现设计 |
| 5 | Worker | 编码实现 |
| 6 | Librarian | 文档维护 |

### 预计时间
[时间估算]

### 下一步
@[角色]: [具体任务]
```

## 约束

- 必须准确判断任务复杂度
- 必须考虑任务依赖关系
- 必须分配合适的角色
- 必须提供清晰的路径选择理由
- 必须给出明确的下一步指示
