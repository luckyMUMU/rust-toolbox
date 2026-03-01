---
name: sop-progress-supervisor
description: |
  Use when:
    - 用户需要了解当前工作流进度
    - 需要检测和处理阻塞问题
    - 定期生成进度报告
    - 需要协调多个并行任务
  Don't use when:
    - 需要启动工作流 → 使用 sop-workflow-orchestrator
    - 需要执行具体任务 → 调用对应的 Skill
    - 需要修改代码 → 使用 sop-code-implementation
    - 需要同步文档 → 使用 sop-document-sync
  Inputs:
    - workflow_state: 工作流状态文件（contracts/workflow-state.json）
    - active_tasks: 活动任务列表
    - previous_report: 上次进度报告（可选）
  Outputs:
    - contracts/progress-report.json: 进度报告
  Success criteria:
    - 进度报告已生成
    - 进度报告准确
    - 阻塞问题已记录
---

# sop-progress-supervisor

## 描述

进度监管 Skill 负责监控工作流执行进度，报告阻塞问题。该 Skill 提供工作流的可视化进度信息。

主要职责：
- 监控阶段进度
- 检测阻塞问题
- 生成进度报告
- 协调并行任务

## 使用场景

触发此 Skill 的条件：

1. **进度查询**：用户需要了解当前工作流进度
2. **阻塞检测**：需要检测和处理阻塞问题
3. **定期报告**：定期生成进度报告
4. **并行协调**：需要协调多个并行任务

## 指令

### 步骤 1: 收集进度信息

1. 读取工作流状态文件
2. 收集各阶段执行状态
3. 统计已完成任务
4. 统计待处理任务

### 步骤 2: 计算进度指标

1. 计算整体进度百分比
2. 计算各阶段进度
3. 估算剩余时间
4. 识别关键路径

### 步骤 3: 检测阻塞问题

1. 检查长时间未完成的任务
2. 识别依赖阻塞
3. 检测资源冲突
4. 记录阻塞原因

### 步骤 4: 生成进度报告

1. 汇总进度信息
2. 列出阻塞问题
3. 提供解决建议
4. 生成可视化报告

### 步骤 5: 协调并行任务

1. 识别可并行执行的任务
2. 分配任务资源
3. 监控并行执行状态
4. 处理任务冲突

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "workflow_state"
    type: json
    path: "contracts/workflow-state.json"
    description: "工作流状态文件"
  
  - name: "active_tasks"
    type: json
    path: "contracts/active-tasks.json"
    description: "活动任务列表"

optional_inputs:
  - name: "previous_report"
    type: json
    path: "contracts/progress-report.json"
    description: "上次进度报告"
```

### 输出契约

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
    guarantees:
      - "进度报告已生成"
      - "进度报告准确"
```

### 行为契约

```yaml
preconditions:
  - "工作流已启动"
  - "工作流状态文件存在"

postconditions:
  - "进度报告已生成"
  - "进度报告准确"
  - "阻塞问题已记录"

invariants:
  - "进度报告必须准确"
  - "阻塞问题必须及时报告"
  - "进度报告必须实时更新"
```

## 常见坑

### 坑 1: 进度计算不准确

- **现象**: 报告显示进度 80%，但实际剩余工作量远超预期。
- **原因**: 进度计算仅基于任务数量，未考虑任务的复杂度和实际工作量。
- **解决**: 进度计算应综合考虑任务数量、任务权重和已完成工作量，而非简单计数。

### 坑 2: 阻塞问题上报延迟

- **现象**: 任务阻塞多时才被发现，导致整体进度严重延误。
- **原因**: 未实时监控任务状态，仅在生成报告时才检查阻塞情况。
- **解决**: 设置阻塞检测阈值，任务阻塞超过指定时间后立即上报并通知相关人员。

### 坑 3: 并行任务资源冲突

- **现象**: 多个并行任务争夺同一资源，导致任务失败或数据损坏。
- **原因**: 协调并行任务时未检查资源依赖和冲突情况。
- **解决**: 在启动并行任务前，检查资源依赖关系，对共享资源进行锁定或排队处理。

## 示例

### 输入示例

```json
{
  "workflow_state": {
    "current_stage": "stage-1",
    "completed_stages": ["stage-0"],
    "pending_stages": ["stage-2", "stage-3", "stage-4"]
  },
  "active_tasks": [
    { "id": "task-1", "status": "in_progress", "progress": 80 },
    { "id": "task-2", "status": "blocked", "reason": "等待 task-1 完成" }
  ]
}
```

### 输出示例

```json
{
  "report_date": "2026-03-01T10:00:00Z",
  "overall_progress": 35,
  "stage_progress": {
    "stage_0": 100,
    "stage_1": 80,
    "stage_2": 0,
    "stage_3": 0,
    "stage_4": 0
  },
  "blocking_issues": [
    {
      "task_id": "task-2",
      "reason": "等待 task-1 完成",
      "suggested_action": "优先完成 task-1"
    }
  ],
  "estimated_completion": "2026-03-01T18:00:00Z"
}
```

## 相关文档

- [Skill 索引](../index.md)
- [工作流编排 Skill](../workflow-orchestrator/SKILL.md)
- [文档同步 Skill](../document-sync/SKILL.md)