---
version: v2.12.0
updated: 2026-02-25
---

# 工作流规范

## 质量门控机制

> 每个阶段完成后必须执行门控检查，确保质量：

| 阶段 | 门控检查项 | 通过条件 | 失败处理 |
|------|-----------|----------|----------|
| 需求阶段 | 需求边界清晰、技术方案对齐、验收标准具体、关键假设确认 | 全部通过 | 返回需求分析修正 |
| 架构阶段 | 架构图清晰、接口定义完整、与现有系统无冲突、设计可行 | 全部通过 | 返回架构设计修正 |
| 实现设计阶段 | 任务覆盖完整、依赖无循环、每个任务可独立验证 | 全部通过 | 返回实现设计修正 |
| 代码实现阶段 | 代码规范、测试通过、文档同步 | 全部通过 | 返回代码实现修正 |
| 文档同步阶段 | 需求实现、验收满足、质量达标 | 全部通过 | 返回相应阶段修正 |

**门控失败约束**：
- 门控失败不累计，每次失败都需要用户决策
- 用户可选择：修复后重试、回滚到上一阶段、终止任务
- 不与三错即停机制关联
- 必须记录失败原因和检查项

## 编排入口

- 入口 Skill：`sop-workflow-orchestrator`
- 仅当用户请求包含 `ultrawork` 或明确要求"全自动/持续执行直到完成"时 → 允许连续调用多个 Skill
- 否则 → 每次仅执行一个 Skill，并在输出中给出下一步可调用的 Skill

## 能力选择协议（必须）

- 必须输出"选择清单"：本次准备调用的 Skill 与理由
- 必须输出"排除清单"：未选择但可能相关的 Skill 与不选择理由（例如：不满足触发条件/无输入/不在当前路径）
- 不得跳过停止点：当输入不足/冲突/依赖缺口时必须进入 `[USER_DECISION]`

## 路径选择

| 路径 | 条件 |
|------|------|
| 快速 | 单文件+<30行+无逻辑变更 |
| 深度 | 其他所有情况 |
| TDD | 深度+启用TDD(可选) |

---

## 目录维度执行

### 核心原则

实现类 Skill（如 `sop-code-implementation`）以 `design.md` 所在目录为工作范围，按目录深度自底向上并行执行：

CMD: `LIST_DESIGN_MD(root) -> design_list`
CMD: `SCHEDULE_DIRS(design_list) -> dir_map`
CMD: `RUN_DIR_BATCH(depth_desc)`（同 depth 并行）
CMD: `WAIT_DEP(dir,deps)` / `COMPLETE_DIR(dir)`

参见：05_constraints/command_dictionary.md

**多目录时实现设计确认粒度**：默认**整批确认**（sop-implementation-designer 按目录产出全部 design.md 后，进行一次用户确认 `DESIGN_CONFIRM()`，再进入 sop-code-explorer → sop-progress-supervisor）。若需每目录或仅关键目录确认，在 05_constraints/state_dictionary.md 与实现设计中约定。

### 并行执行规则

| 场景 | 执行方式 | 说明 |
|------|----------|------|
| 同深度无依赖 | **并行** | 多个目录批次同时执行 `sop-code-implementation` |
| 同深度有依赖 | 串行 | 按依赖顺序执行 |
| 父子目录 | 串行 | 子目录完成后父目录才能开始 |
| 跨模块依赖 | 协调 | 通过 `sop-progress-supervisor` 调度与唤醒 |

👉 [目录维度工作策略详情](04_reference/design_directory_strategy.md)

---

## 快速路径

```
sop-code-explorer → sop-code-implementation → sop-code-review → sop-document-sync
```

| 阶段 | 输入 | 输出 | 停止点 |
|------|------|------|--------|
| sop-code-explorer | 目标文件/范围 | 审计报告 | `[USER_DECISION]` |
| sop-code-implementation | 审计报告/Scope | 代码修改 + 验证结果 | `[WAITING_FOR_CODE_REVIEW]` / `[DIR_WAITING_DEP]` |
| sop-code-review | Diff+设计依据 | 审查报告 | `[USER_DECISION]` |
| sop-document-sync | 变更集 | 文档同步变更 | `[USER_DECISION]` |

来源与依赖准则：
- `sop-code-implementation` / `sop-code-review` 必须声明来源与依赖（模板：`04_reference/interaction_formats/source_dependency.md`）
- 当找不到来源或依赖时，必须进入 `[USER_DECISION]` 并落盘决策记录

**注意**：快速路径不涉及多目录并行，单文件修改直接执行。

👉 [快速路径详情](fast_path.md)

---

## 深度路径

### 新项目/大重构（目录维度）

```
sop-requirement-analyst
→ sop-architecture-design
→ sop-architecture-reviewer
→ sop-implementation-designer (按目录)
→ sop-code-explorer (LIST_DESIGN_MD → design_list)
→ sop-progress-supervisor (SCHEDULE_DIRS(design_list) → dir_map)
→ sop-code-implementation (按目录并行)
→ sop-code-review
→ sop-document-sync
```

### 功能迭代（目录维度）

```
sop-requirement-analyst
→ [架构影响评估检查点]
→ sop-implementation-designer (按目录)
→ sop-code-explorer (LIST_DESIGN_MD → design_list)
→ sop-progress-supervisor (SCHEDULE_DIRS(design_list) → dir_map)
→ sop-code-implementation (按目录并行)
→ sop-code-review
→ sop-document-sync
```

**架构影响评估检查点**：
- 在功能迭代入口，评估变更是否涉及架构影响
- 评估项：接口变更、数据模型变更、跨模块依赖、性能影响
- 仅当评估结果显示架构影响 → 必须插入 `sop-architecture-design` 和 `sop-architecture-reviewer` 阶段
- 当评估结果显示无架构影响 → 直接进入实现设计阶段

阶段合约（触发条件/输入输出/停止点/落盘交付物）以 [Skill 矩阵（SSOT）](02_skill_matrix/index.md) 与各 `skills/*/SKILL.md` 为准。

👉 [深度路径详情](deep_path.md)

---

## TDD深度路径 (可选)

**多目录时**（与标准深度路径一致，需目录调度）：
```
... 深度路径调用链 ...
→ sop-test-design-csv
→ sop-test-implementation
→ sop-progress-supervisor (dir_map)
→ sop-code-implementation (运行验收 + 修正代码)
→ sop-code-review
→ sop-document-sync
```

**单目录时**：可省略 sop-progress-supervisor，直接 `sop-test-implementation → sop-code-implementation → sop-code-review → sop-document-sync`。

分层验收门禁与停止点以 `05_constraints/acceptance_criteria.md` 与 `05_constraints/state_dictionary.md` 为准。

**启用条件**: 核心业务/复杂逻辑/高覆盖要求

**测试用例来源**: 仅基于设计文档，不参考代码

**测试代码来源**: 主要基于CSV，仅参考代码接口

👉 [TDD工作流详情](skills/sop-tdd-workflow/SKILL.md)

---

## 三错即停

| Strike | 条件 | 行动 |
|--------|------|------|
| 1 | 同一 Skill 同一步骤失败 | 自动修正（同 Skill 内） |
| 2 | 再失败 | 调用 `sop-code-explorer` + 设计类 Skill 复核并微调 |
| 3 | 再失败 | **熔断**：由 `sop-progress-supervisor` 生成报告并停止自动推进 |

👉 [三错即停详情](three_strike_rule.md)

---

## 目录调度状态机

多目录并行执行时，目录状态按以下状态机流转：

```
[DIR_WAITING_DEP] ←── 依赖未就绪 ──┐
       │                          │
       ↓ 依赖就绪（自动触发）       │
[DIR_WORKING] ──→ 处理完成 ──→ [DIR_COMPLETED]
       │                          │
       ↓ 处理失败                 │
[DIR_FAILED] ─────────────────────┘
```

### 状态转移规则

| 当前状态 | 触发条件 | 目标状态 | 执行动作 |
|----------|----------|----------|----------|
| `[DIR_WAITING_DEP]` | 依赖目录进入 `[DIR_COMPLETED]` | `[DIR_WORKING]` | 自动唤醒，继续执行 |
| `[DIR_WORKING]` | 任务完成并通过验证 | `[DIR_COMPLETED]` | 更新 dir_map，调度下一批 |
| `[DIR_WORKING]` | 任务失败且可恢复 | `[DIR_WORKING]` | 重试（计入迭代计数） |
| `[DIR_WORKING]` | 任务失败且不可恢复 | `[DIR_FAILED]` | 触发熔断检查 |
| `[DIR_FAILED]` | 用户决策继续 | `[DIR_WORKING]` | 重置计数器，重新执行 |

### 调度状态保存格式

中断恢复时，调度状态保存为 JSON 格式：

```json
{
  "version": "1.0",
  "timestamp": "2026-02-24T10:30:00Z",
  "directories": [
    {
      "path": "src/module-a",
      "state": "DIR_COMPLETED",
      "design_md": "src/module-a/design.md",
      "completed_at": "2026-02-24T10:00:00Z"
    },
    {
      "path": "src/module-b",
      "state": "DIR_WAITING_DEP",
      "dependencies": ["src/module-a"],
      "design_md": "src/module-b/design.md"
    }
  ],
  "current_batch": 1,
  "total_batches": 3
}
```

保存位置：`.trae/scheduler_state.json`

---

## 中断与再执行

流程支持**中断 → 重建 → 再执行**的循环：在任意停止点中断后，可经用户决策与方案调整完成“重建”，再从某一可验证检查点安全地再执行。

- **中断点**：任意停止点（含 `[USER_DECISION]`、`[FUSION_TRIGGERED]`）。
- **重建**：用户决策 + 方案调整 + 可选 Scope/设计/验收变更 + 重置计数器（熔断恢复时参见 [三错即停](three_strike_rule.md)）。
- **再执行**：从可恢复检查点继续，使用 [续跑与恢复请求](04_reference/interaction_formats/continuation_request.md) 交接。

**可恢复检查点**：允许作为再执行起点的状态及所需最小输入见 [state_dictionary.md](05_constraints/state_dictionary.md#可恢复检查点recoverable-checkpoints)。从 `[USER_DECISION]` / `[FUSION_TRIGGERED]` 续跑时，须在 continuation_request 中写明"建议下一步"对应的检查点及该清单所列最小输入。

**状态机**：`[USER_DECISION]` / `[FUSION_TRIGGERED]` 后可选 (1) 重新分诊 → `ROUTE(task)` 或 (2) 从检查点续跑 → 输出 continuation_request → 对应 Skill 再执行。参见 [sop_state_machine.md](sop_state_machine.md)。

---

## 停止点

- SSOT: 05_constraints/state_dictionary.md（所有状态标记与停止点定义以该文件为准）
- 命令契约: 05_constraints/command_dictionary.md
- 人工审批点: Diff 展示（代码审查通过后，展示变更 Diff 等待用户审批）
