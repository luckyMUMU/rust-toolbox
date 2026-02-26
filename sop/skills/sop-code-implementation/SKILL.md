---
name: "sop-code-implementation"
description: "Code implementation workflow for physical coding. Invoke when implementation design is approved and ready for coding."
version: v2.12.0
updated: 2026-02-25
layer: "实现"
load_policy:
  tier: 2
  auto_load_states: ["[WAITING_FOR_DESIGN]"]
  depends_on: ["sop-code-explorer", "sop-implementation-designer"]
---

# Code Implementation Workflow

## 侧重点

- 仅当 `design.md` 存在且 Scope 明确 → 才能进入 `[DIR_WORKING]` 并开始改代码
- 优先复用：先复用→改进→新建→清理（必要时调用 `sop-capability-reuse`）
- 跨目录依赖 → 必须进入 `[DIR_WAITING_DEP]` 并请求 `sop-progress-supervisor` 调度

## 质量门控检查

> 完成代码实现后，必须执行以下门控检查：

| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 代码规范 | lint/type check通过 | [ ] |
| 测试通过 | 所有测试用例通过 | [ ] |
| 文档同步 | 相关文档已更新 | [ ] |

**门控失败处理**：若任一检查项未通过，应记录失败原因并返回修正。

## 完成情况记录建议

> 当目录实现完成时，建议记录以下完成情况：

- 完成的任务列表
- 遗留问题（如有）
- 测试覆盖率
- 性能指标（如适用）

## 触发条件

- 仅当存在已确认的实现设计（design.md）且 Scope 明确时 → 必须调用本 Skill
- 仅当出现跨目录依赖或 Scope 争议时 → 必须进入 `[DIR_WAITING_DEP]` 或 `[USER_DECISION]`

## Input

- Implementation Design（design.md / link）
- Directory Scope（dir + depth + deps）
- 可执行命令契约（test/lint/typecheck；参见 05_constraints/acceptance_criteria.md）

## Workflow Steps

### Step 1: Directory Scope Check

**Purpose**: Confirm implementation scope boundary

**Actions**:
1. Read design.md in target directory
2. Confirm directory boundary (current dir + subdirs without nested design.md)
3. Check dependency directory status
4. Mark `[DIR_WORKING]`
CMD: `WAIT_DEP(dir,deps)` / `COMPLETE_DIR(dir)`

### Step 2: Checkpoint

**Purpose**: Create rollback point

**Actions**:
1. Note current state
2. Prepare rollback plan

### Step 3: Code Development (Within Directory Boundary)

**Purpose**: Implement by design within directory scope

**Actions**:
1. Follow design doc strictly
2. **Only modify files within current directory scope**
3. **Do NOT modify files in other design.md directories**
4. Add code comments

**Cross-Directory Change Handling**:
CMD: `REQUEST_CROSS_DIR(src_dir, target_dir, change) -> appended_request`

### Step 4: Testing

**Purpose**: Verify correctness

**Actions**:
1. Run unit tests within directory
2. Run integration tests
3. Fix failures

### Step 5: Quality Check

**Purpose**: Ensure code quality

**Actions**:
1. Run linter
2. Run type checker
3. Fix issues

### Step 6: Completion

**Purpose**: Prepare for code review and completion

**Actions**:
1. Generate diff for review
2. Mark `[WAITING_FOR_CODE_REVIEW]`
3. Wait `sop-code-review` result
4. If passed: mark `[DIR_COMPLETED]` and notify `sop-progress-supervisor`

### Step 7: Gate Check

**Purpose**: Execute quality gate check for implementation phase

**Actions**:
CMD: `GATE_CHECK(code_changes, gate='GATE_IMPLEMENTATION')`

**Gate Check Items**:
| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 代码规范 | lint/type check通过 | [ ] |
| 测试通过 | 所有测试用例通过 | [ ] |
| 文档同步 | 相关文档已更新 | [ ] |

**State Transition**:
- 通过 → `[WAITING_FOR_CODE_REVIEW]`
- 失败 → `[GATE_FAILED]` → 用户决策

## 来源与依赖准则

- 必须声明输入来源与依赖（design.md/验收标准/红线约束等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当找不到来源或依赖时必须中断：进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## Output

- 模板：04_reference/interaction_formats/worker_execution_result.md
- CMD: `IMPLEMENT(dir, design)`

## Constraints

- **Directory Boundary**: Only modify files within current design.md directory
- **No Cross-Directory Changes**: **Strictly Prohibited** to modify other design.md files
- **Dependency Wait**: Must wait for dependencies to complete
- **Follow design strictly**: No design changes during implementation
- **Must pass tests**: All tests must pass
- **Must pass quality checks**: Lint and type check must pass

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## 3-Strike Rule

| Strike | Condition | Action |
|--------|-----------|--------|
| 1 | Test/quality fail | Auto-fix |
| 2 | Fail again | Audit + redesign |
| 3 | Fail again | **Break**, user decision |

## Stop Points

- `[DIR_WAITING_DEP]`: 发现跨目录依赖且需要调度
- `[WAITING_FOR_CODE_REVIEW]`: 已产出 Diff，等待 `sop-code-review`
- `[USER_DECISION]`: 设计依据/依赖缺口无法消解

## Failure Handling

- 当无法在 Scope 内完成设计要求且跨目录依赖无法通过追加“待处理变更”表达时，必须进入 `[USER_DECISION]` 并落盘决策记录
