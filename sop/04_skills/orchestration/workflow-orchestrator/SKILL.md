---
name: sop-workflow-orchestrator
description: 编排工作流程，管理规范版本和工作流状态转换
version: v3.0.0
skill_type: orchestration
---

# sop-workflow-orchestrator

## 描述

工作流编排 Skill 负责管理整个工作流程的状态转换。该 Skill 是编排层的核心，确保工作流按正确的顺序执行。

主要职责：
- 管理工作流状态
- 编排阶段转换
- 管理规范版本
- 协调 Skill 执行

## 使用场景

触发此 Skill 的条件：

1. **工作流启动**：用户提交新需求，启动工作流
2. **阶段转换**：一个阶段完成，需要转换到下一阶段
3. **状态查询**：需要查询当前工作流状态
4. **异常处理**：工作流执行出现异常，需要处理

## 指令

### 步骤 1: 初始化工作流

1. 读取权重决策（Stage 0）
2. 创建工作流状态文件
3. 初始化阶段队列
4. 记录工作流元数据

### 步骤 2: 编排阶段执行

1. 确定当前阶段
2. 加载阶段契约
3. 调度相关 Skill
4. 监控执行状态

### 步骤 3: 管理阶段转换

1. 验证当前阶段完成条件
2. 执行质量门控检查
3. 更新工作流状态
4. 启动下一阶段

### 步骤 4: 处理异常情况

1. 检测执行异常
2. 触发熔断机制（三击熔断）
3. 记录异常信息
4. 通知相关人员

### 步骤 5: 更新工作流状态

1. 记录阶段完成状态
2. 更新进度信息
3. 保存工作流状态文件
4. 生成状态报告

## 契约

### 输入契约

```yaml
required_inputs:
  - name: "weight_decision"
    type: json
    path: "contracts/stage-0-decision.json"
    description: "权重决策结果"
  
  - name: "constitution_docs"
    type: files
    path: "01_constitution/"
    description: "工程宪章文档"

optional_inputs:
  - name: "workflow_state"
    type: json
    path: "contracts/workflow-state.json"
    description: "当前工作流状态"
```

### 输出契约

```yaml
required_outputs:
  - name: "workflow_state"
    type: json
    path: "contracts/workflow-state.json"
    format:
      current_stage: "stage-N"
      completed_stages: ["stage-0", "stage-1"]
      pending_stages: ["stage-2", "stage-3", "stage-4"]
    guarantees:
      - "工作流状态已更新"
      - "阶段按顺序执行"
```

### 行为契约

```yaml
preconditions:
  - "规范重量决策已确认"
  - "工程宪章存在"

postconditions:
  - "工作流状态已更新"
  - "阶段按顺序执行"
  - "工作流状态文件已保存"

invariants:
  - "阶段必须按顺序执行"
  - "每个阶段必须通过质量门控"
  - "三击熔断机制必须生效"
```

## 示例

### 输入示例

```json
{
  "weight_decision": {
    "complexity": "high",
    "path": "deep",
    "stages": ["stage-0", "stage-1", "stage-2", "stage-3", "stage-4"]
  }
}
```

### 输出示例

```json
{
  "workflow_id": "wf-20260301-001",
  "current_stage": "stage-1",
  "completed_stages": ["stage-0"],
  "pending_stages": ["stage-2", "stage-3", "stage-4"],
  "stage_details": {
    "stage-0": {
      "status": "completed",
      "completed_at": "2026-03-01T09:00:00Z",
      "decision": "deep"
    },
    "stage-1": {
      "status": "in_progress",
      "started_at": "2026-03-01T09:05:00Z"
    }
  }
}
```

## 相关文档

- [Skill 索引](../index.md)
- [文档同步 Skill](../document-sync/SKILL.md)
- [进度监管 Skill](../progress-supervisor/SKILL.md)
- [工作流程](../../../03_workflow/)
