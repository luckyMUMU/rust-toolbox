---
name: "sop-progress-supervisor"
description: "Progress monitoring workflow for tracking execution and triggering circuit breakers. Invoke when monitoring task execution or detecting anomalies."
version: v2.12.0
updated: 2026-02-25
layer: "编排"
load_policy:
  tier: 1
  auto_load_states: []
  depends_on: []
---

# Progress Monitoring Workflow

**位置**: `sop/skills/sop-progress-supervisor/SKILL.md`

## 侧重点

- 目录并行调度以状态机驱动：`[DIR_WORKING]` → `[DIR_COMPLETED]`，等待以 `[DIR_WAITING_DEP]` 表达
- 三错即停：到达熔断条件必须产出报告并停止自动推进

## 门控失败处理流程

> 当检测到门控失败时，应执行以下流程：

1. **记录失败**：记录失败的目录、阶段、检查项
2. **分析原因**：判断是设计问题、实现问题还是依赖问题
3. **通知相关方**：通知对应的Skill进行修正
4. **跟踪修正**：等待修正完成后重新检查

## 状态保存增强

> 当调度中断时，应保存以下完整状态：

- dir_map（目录-skill映射）
- 各目录当前状态
- 依赖关系图
- 失败记录
- 迭代计数

## 触发条件

- 进入目录并行执行，需要创建/更新调度映射并监督执行
- 任意实现类 Skill 出现阻塞、重复失败、依赖死锁、跨目录冲突等异常

## Input

- task_id/name
- stage/skill/elapsed
- dir_map（目录/深度/skill/状态/依赖）
- failures（0-3）

## Workflow Steps

### Step 1: Directory Mapping Creation

**Purpose**: Create directory-skill mapping for parallel execution

**Actions**:
- 接收 **design_list**（由 sop-code-explorer 执行 `LIST_DESIGN_MD(root)` 产出；命令归属见 05_constraints/command_dictionary.md）
- CMD: `CYCLE_CHECK(design_list) -> cycle_report`（检测依赖循环）
- 若检测到循环：进入 `[CYCLE_DETECTED]` → `[USER_DECISION]`
- CMD: `SCHEDULE_DIRS(design_list) -> dir_map`（必须持久化 `temp/scheduler_state.md`）

### Step 2: Implementation Skill Scheduling

**Purpose**: Schedule implementation skills by directory depth

**Actions**:
CMD: `RUN_DIR_BATCH(depth_desc)`（deps 全部 `[DIR_COMPLETED]` 才可启动）

### Step 3: State Collection

**Purpose**: Gather current status from all directories/skills

**Actions**:
CMD: collect skill_status -> update dir_map -> persist `temp/scheduler_state.md`

### Step 4: Deviation Detection

**Purpose**: Identify issues

**Actions**:
1. Compare to planned schedule
2. Check for delays
3. Identify dependency deadlocks
4. Detect cross-directory conflicts

### Step 5: Dependency Coordination

**Purpose**: Handle cross-directory dependencies

**Actions**:
CMD: `WAIT_DEP(dir,deps)` / notify resume

### Step 5.1: Dependency Wake Mechanism

**Purpose**: Detect and wake directories waiting for dependencies

**Detection Methods**:
1. **Event-driven (Preferred)**: Subscribe to directory state change events
2. **Polling (Fallback)**: Check dependency status every N seconds

**Wake Sequence**:
```yaml
dependency_wake:
  detection_method: "event_driven"
  fallback: "polling"
  polling_interval: 30  # seconds
  
  wake_sequence:
    - step: 1
      action: "检测依赖目录状态"
      cmd: "CHECK_DEP_STATUS(dir_list)"
    - step: 2
      condition: "all_deps_completed"
      action: "唤醒等待目录"
      cmd: "WAKE_DIR(waiting_dir)"
      output: "[DIR_WORKING]"
    - step: 3
      condition: "has_pending_deps"
      action: "继续等待依赖"
      cmd: "WAIT_FOR_DEP(pending_list)"
      output: "[DIR_WAITING_DEP]"
```

**Wake Commands**:
| CMD | Description |
|-----|-------------|
| `CHECK_DEP_STATUS(dir_list: path[])` | 检查依赖目录状态 |
| `WAKE_DIR(waiting_dir: path)` | 唤醒等待中的目录 |
| `WAIT_FOR_DEP(pending_list: path[])` | 继续等待依赖完成 |

### Step 6: Iteration Monitoring

**Purpose**: Monitor iteration count and convergence

**Actions**:
CMD: `ITERATION_COUNT(state) -> iteration_count`
- 若迭代次数 = 4：输出警告
- 若迭代次数 ≥ 5：进入 `[USER_DECISION]` 并提供收敛建议

### Step 7: Risk Assessment

**Purpose**: Evaluate severity

**Severity**:
- 🔴 Critical: Blocked, needs immediate action
- 🟡 Warning: Delayed, needs attention
- 🟢 Normal: On track

### Step 8: Decision

**Purpose**: Determine next action

**Options**:
- Continue: Normal progress
- Alert: Warning, notify stakeholders
- Break: Critical, trigger circuit breaker

## 来源与依赖准则

- 必须声明调度与熔断的输入来源与依赖（dir_map/失败记录/依赖关系/约束等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当依据缺失或冲突无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- 交付物（模板）：04_reference/interaction_formats/supervisor_report.md
- 交付物：`temp/scheduler_state.md`（目录-skill 映射、目录状态、依赖、批次）
- CMD: `STRIKE(record)` / `FUSE(reason)` / `ASK_USER_DECISION(topic, options)`

## Stop Points

- `[SCHEDULING]`: 生成目录-skill 映射与调度计划
- `[PARALLEL_EXECUTING]`: 多目录并行执行中
- `[WAITING_DEPENDENCY]`: 存在目录依赖等待
- `[ALL_COMPLETED]`: 所有目录进入完成态
- `[FUSION_TRIGGERED]`: 连续失败触发熔断，必须停止推进并进入用户决策
- `[CYCLE_DETECTED]`: 检测到目录依赖循环，进入用户决策

## Constraints

- Read all: code, docs, status
- Write status only
- Trigger `[FUSION_TRIGGERED]` when needed
- No implementation
- Must reference SSOT when using states/commands: 05_constraints/state_dictionary.md, 05_constraints/command_dictionary.md
- **Maintain directory-skill mapping table**
- **Coordinate cross-directory dependencies**
- **Schedule parallel execution by depth**

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Failure Handling

- 失败计数到达阈值时必须执行 `STRIKE(record)`，并在第 3 次失败触发 `FUSE(reason)` 与 `[FUSION_TRIGGERED]`

## 3-Strike Rule

| Strike | Condition | Action |
|--------|-----------|--------|
| 1 | Implementation fails | Log, allow retry |
| 2 | Fails again | Audit + redesign |
| 3 | Fails again | **Break**, `[FUSION_TRIGGERED]` |
