---
name: "sop-tdd-workflow"
description: "TDD工作流定义。Invoke when 用户启用TDD测试驱动开发流程。"
version: v2.12.0
updated: 2026-02-25
layer: "路径宏"
load_policy:
  tier: 3
  auto_load_states: ["[ROUTE_TDD]"]
  depends_on: ["sop-deep-path", "sop-test-design-csv", "sop-test-implementation", "sop-code-implementation", "sop-code-review"]
---

# TDD 工作流（可选 / Skill-first）

**类型**: 可选项  
**触发**: `sop-workflow-orchestrator` 判断启用 TDD 时  
**位置**: `sop/skills/sop-tdd-workflow/SKILL.md`

## 侧重点

- 测试资产隔离：CSV 仅 `sop-test-design-csv` 可写；测试代码仅 `sop-test-implementation` 可写
- 测试用例来源：仅基于设计文档，不从代码倒推

---

## 触发条件

- 仅当深度路径且要求高覆盖/复杂逻辑时 → 允许启用本 Skill
- 仅当测试资产隔离规则无法满足或输入缺口影响测试生成时 → 必须进入 `[USER_DECISION]`

## 输入

- deep path 调用链（或当前任务上下文）
- L2/L3 设计依据（`.md` / `design.md`）
- 分层验收门禁：`05_constraints/acceptance_criteria.md`

## 概述

TDD（测试驱动开发）作为深度路径的可选增强：在编码前先落盘测试用例与测试代码，以保证测试覆盖设计场景。

**核心原则**：测试独立、资产隔离、版本管理

---

## 工作流对比

### 标准深度路径
```
... deep path ...
→ sop-code-implementation
→ sop-code-review
→ sop-document-sync
```

### TDD深度路径

**多目录时**（需目录调度，与标准深度路径一致）：
```
... deep path ...
→ sop-test-design-csv
→ sop-test-implementation
→ sop-progress-supervisor (dir_map)
→ sop-code-implementation（运行验收 + 修正代码）
→ sop-code-review
→ sop-document-sync
```

**单目录时**：可省略 sop-progress-supervisor，直接 test-implementation → code-implementation → code-review → document-sync。

---

## 测试资产隔离（关键）

| 资产 | 维护 Skill | 禁止 |
|------|-----------|------|
| CSV 测试用例 | `sop-test-design-csv` | `sop-test-implementation` / `sop-code-implementation` 禁止修改 |
| 测试代码 | `sop-test-implementation` | 其他 Skill 禁止修改 |

---

## 测试用例位置

`docs/03_technical_spec/test_cases/[module]_test_cases.csv`

与分层验收的关系（参见 05_constraints/acceptance_criteria.md）：
- CSV 是 **测试设计载体**（TDD 路径下的唯一用例来源）
- `sop-test-implementation` 将 CSV 用例实现为验收测试代码（必须落地到 `tests/acceptance/l1-l4/`）
- `sop-code-implementation` 仅运行测试并根据失败结果修正代码，不创建/修改测试资产

---

## CSV格式
模板：04_reference/interaction_formats/test_case_csv.md

---

## 关键约束

### 测试用例来源
- **仅基于设计文档** (L2 `.md` + L3 `design.md`)
- **不参考代码实现**
- `sop-test-design-csv` 禁止从代码推导用例

### 测试代码来源
- **主要基于CSV测试用例**
- 仅参考代码实现获取接口细节
- **`sop-test-implementation` 禁止修改 CSV**

### 追溯关系
```
L2原子操作 ←→ CSV测试用例 ←→ 测试代码
```

### 版本管理
- CSV必须包含版本号
- 每次变更更新版本号和日期
- 变更记录写在CSV头部注释

---

## 测试用例变更流程

当测试用例需要变更时：

```
发现需求 → `sop-test-design-csv` 评估 → 更新 CSV → 版本+1 → 触发 `sop-test-implementation` 同步测试代码
```

### 变更场景

| 场景 | 处理流程 |
|------|----------|
| 设计变更 | `sop-test-design-csv` 根据新设计更新 CSV → 版本+1 → 触发 `sop-test-implementation` |
| 发现遗漏 | `sop-test-design-csv` 补充用例 → 版本+1 → 触发 `sop-test-implementation` |
| 用例错误 | `sop-test-design-csv` 修正 → 版本+1 → 触发 `sop-test-implementation` |
| 发现 CSV 问题 | `sop-test-implementation` 报告问题 → `sop-test-design-csv` 评估 → 如确认则更新 CSV |

### 问题报告模板

```markdown
**CSV问题报告**

**位置**: TC001, 输入数据字段
**问题**: 输入数据缺少必要字段"user_id"
**建议**: 补充"user_id"字段
**影响**: 测试代码无法正确执行
```

---

## 来源与依赖准则

- TDD 路径下的关键产物（CSV测试用例/测试代码/审查报告）必须包含“来源与依赖声明”（标准：04_reference/review_standards/source_dependency.standard.md），并优先用 `TRACE_SOURCES(inputs)` 固化
- 当关键来源/依赖缺口无法消解时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录

## Workflow Steps

### Step 1: Test Design (CSV)

CMD: `TEST_DESIGN_CSV(design) -> [WAITING_FOR_TEST_DESIGN]`

### Step 2: Test Implementation

CMD: `TEST_IMPLEMENT(test_design) -> [WAITING_FOR_TEST_IMPLEMENTATION]`

### Step 3: Run Acceptance & Fix

CMD: `RUN_ACCEPTANCE(level) -> [WAITING_FOR_ACCEPTANCE_REVIEW]`

## 停止点

| 标记 | 触发 | 等待 |
|------|------|------|
| `[WAITING_FOR_TEST_DESIGN]` | `sop-test-design-csv` 完成 | 用户确认测试设计 |
| `[WAITING_FOR_TEST_IMPLEMENTATION]` | `sop-test-implementation` 完成 | `sop-code-review` 进行测试代码审查 |
| `[WAITING_FOR_ACCEPTANCE_REVIEW]` | 验收测试通过 | `sop-code-review` 进行验收审查（level参数指定L1-L4） |

**统一验收审查说明**：
- 使用统一状态 `[WAITING_FOR_ACCEPTANCE_REVIEW]`，通过参数 `level` 指定验收级别
- 旧状态 `[WAITING_FOR_L1_REVIEW]` ~ `[WAITING_FOR_L4_REVIEW]` 保留兼容性支持
- 命令 `RUN_ACCEPTANCE(level)` 输出统一状态

---

## 启用条件

`sop-workflow-orchestrator` 在以下场景启用 TDD（可选推荐）：
- 核心业务模块
- 复杂逻辑场景
- 需要高测试覆盖度
- 团队有TDD实践

---

## 模板

- [测试用例CSV模板](04_reference/document_templates/test_cases.csv)

## 输出

- 交付物：测试用例 CSV + 测试代码 + 运行/验收证据
- 状态：`[WAITING_FOR_TEST_DESIGN]` / `[WAITING_FOR_TEST_IMPLEMENTATION]` / `[WAITING_FOR_ACCEPTANCE_REVIEW]`

## 约束

- 测试资产隔离：CSV 仅 `sop-test-design-csv` 可写；测试代码仅 `sop-test-implementation` 可写
- 测试用例来源：仅基于设计，不从代码倒推
- 目录边界与状态引用：以 `05_constraints/*_dictionary.md` 为准

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）
