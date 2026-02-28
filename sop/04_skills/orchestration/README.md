---
version: v3.0.0
updated: 2026-02-28
skill_type: orchestration
---

# 编排类 Skill

> **职责**: 管理规范版本和流程编排

---

## 概述

编排类 Skill 负责管理规范的版本和演化，编排工作流程。

---

## Skill 列表

### sop-workflow-orchestrator

**职责**: 流程编排，管理规范的版本和演化

**输入契约**:
```yaml
required_inputs:
  - name: "weight_decision"
    type: json
    path: "contracts/stage-0-decision.json"
  - name: "constitution_docs"
    type: files
    path: "01_constitution/"
```

**输出契约**:
```yaml
required_outputs:
  - name: "workflow_state"
    type: json
    path: "contracts/workflow-state.json"
    format:
      current_stage: "stage-N"
      completed_stages: ["stage-0", "stage-1"]
      pending_stages: ["stage-2", "stage-3", "stage-4"]
```

**行为契约**:
```yaml
preconditions:
  - "规范重量决策已确认"
postconditions:
  - "工作流状态已更新"
invariants:
  - "阶段必须按顺序执行"
```

---

### sop-document-sync

**职责**: 文档同步，确保规范与实现同步

**输入契约**:
```yaml
required_inputs:
  - name: "code_changes"
    type: git_diff
    path: "git commit"
  - name: "design_document"
    type: file
    path: "src/{module}/design.md"
```

**输出契约**:
```yaml
required_outputs:
  - name: "documentation_updates"
    type: files
    path: "docs/ 或相关文档路径"
    format: "更新的文档列表"
```

**行为契约**:
```yaml
preconditions:
  - "代码变更已提交"
postconditions:
  - "文档与代码同步更新"
invariants:
  - "文档必须反映最新实现"
```

---

### sop-progress-supervisor

**职责**: 进度监管，目录并行调度

**输入契约**:
```yaml
required_inputs:
  - name: "workflow_state"
    type: json
    path: "contracts/workflow-state.json"
  - name: "active_tasks"
    type: json
    path: "contracts/active-tasks.json"
```

**输出契约**:
```yaml
required_outputs:
  - name: "progress_report"
    type: json
    path: "contracts/progress-report.json"
    format:
      overall_progress: 0-100
      stage_progress:
        stage_0: 100
        stage_1: 80
        stage_2: 0
      blocking_issues: ["阻塞问题"]
```

**行为契约**:
```yaml
preconditions:
  - "工作流已启动"
postconditions:
  - "进度报告已生成"
invariants:
  - "进度报告必须准确"
```

---

## 工作流编排

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    工作流编排                                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ sop-workflow-orchestrator                                            │   │
│  │                                                                      │   │
│  │   阶段 0 ──▶ 阶段 1 ──▶ 阶段 2 ──▶ 阶段 3 ──▶ 阶段 4                │   │
│  │     │          │          │          │          │                   │   │
│  │     ▼          ▼          ▼          ▼          ▼                   │   │
│  │   规范选择   设计审查   代码审查   文档同步   归档更新               │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ sop-progress-supervisor                                              │   │
│  │                                                                      │   │
│  │   监控各阶段进度，报告阻塞问题                                       │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ sop-document-sync                                                    │   │
│  │                                                                      │   │
│  │   确保规范与实现同步更新                                             │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 相关文档

- [Skill 索引](../index.md)
- [工作流程](../../03_workflow/)
- [约束规范](../../05_constraints/)

---

**文档所有者**: Skill 团队  
**最后审核**: 2026-02-28
