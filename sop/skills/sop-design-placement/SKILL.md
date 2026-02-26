---
name: "sop-design-placement"
description: "指导AI正确放置设计文档和创建design.md。Invoke when deciding design/doc placement and design.md granularity in a skill-first workflow."
version: v2.12.0
updated: 2026-02-25
layer: "规约"
load_policy:
  tier: 3
  auto_load_states: []
  depends_on: ["sop-code-explorer"]
---

# 设计文档放置指南

**位置**: `sop/skills/sop-design-placement/SKILL.md`

## 侧重点

- 必须按目录策略选择 design.md 落点，避免跨域耦合
- 当落点存在冲突或影响重大 → 进入 `[USER_DECISION]` 并落盘决策记录

## 触发条件

- 需要创建/更新设计文档，且需要判断正确放置位置
- 需要决定是否创建 design.md 以及 design.md 的深度与拆分粒度

## 目录结构规范

### 文档放置位置

参见：04_reference/document_directory_mapping.md

### 重要约束

⚠️ **`/docs/参考/` 非指定不变更**
- 该目录包含 SOP 标准文档
- 仅 `sop-document-sync` 可在明确任务下维护
- 其他 Skill **禁止**修改此目录

## design.md 创建规则

### 1. 基于目录层级划分

**目录层级定义**:
- 以项目根目录为基准（深度 0）
- 每深入一级，深度 +1
- `design.md` 的深度 = 其所在目录的深度

**放置位置**:
`<module_dir>/design.md`

**路径选择**:
| 模块位置 | design.md 位置 | 深度示例 |
|----------|----------------|----------|
| `src/module/` | `src/module/design.md` | depth 2 |
| `packages/package/` | `packages/package/design.md` | depth 2 |
| 顶层模块 | `docs/module/design.md` | depth 2 |
| 子模块 | `src/module/sub/design.md` | depth 3 |

### 2. 基于复杂度判断

**复杂度评估**:

| 复杂度 | 代码行数 | 功能特点 | design.md 要求 |
|--------|----------|----------|----------------|
| **低** | <100行 | 单一功能，无外部依赖 | 可省略，代码注释说明 |
| **中** | 100-500行 | 多函数，有外部依赖 | 创建简要 design.md，含接口契约 |
| **高** | >500行 | 多模块交互，复杂逻辑 | 创建完整 design.md，含详细设计 |

**决策流程**:
参见：AGENT_SOP.md（design.md 规则）+ skills/sop-implementation-designer/SKILL.md

### 3. 目录层级与并行执行

**目录深度与执行顺序**:
参见：04_reference/design_guide.md + 05_constraints/command_dictionary.md

**实现分配**:
- 每个 design.md 目录对应一个实现 Scope（供实现类 Skill 执行）
- 同深度无依赖的目录并行执行
- 父目录等待子目录完成后才能开始

### 4. 接口契约规范

每个 design.md 必须包含接口契约章节：
模板：04_reference/document_templates/implementation_design.md

### 5. 跨目录依赖声明

在 design.md 中声明跨目录依赖：
写入位置：design.md 的“目录依赖”章节

## 工作流程

### Step 1: 确定文档类型

| 文档类型 | 创建者 | 放置位置 |
|----------|--------|----------|
| PRD | sop-requirement-analyst | `docs/01_requirements/*.md` |
| 架构设计 | sop-architecture-design | `docs/02_logical_workflow/*.md` |
| 实现设计 | sop-implementation-designer | `src/**/design.md` 或 `docs/**/design.md` |

CMD: `IMPL_DESIGN(l2, dir) -> design.md`

## 来源与依赖准则

- 必须声明放置决策的来源与依赖（目录结构/复杂度判定依据/约束等），并优先用 `TRACE_SOURCES(inputs)` 固化“来源与依赖声明”
- 当文档放置存在争议或关键依据缺失时，必须进入 `[USER_DECISION]`，并使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
- 标准：04_reference/review_standards/source_dependency.standard.md

## 输入
- module_name/path
- depth(optional)
- complexity(low/medium/high)
- deps / used_by

## 输出
- 交付物：design.md 路径（落盘至 `<module_dir>/design.md`）
- 交付物：design.md 必备章节（接口契约/目录依赖/任务清单）
- 交付物（模板）：04_reference/document_templates/implementation_design.md

## Stop Points

- `[USER_DECISION]`: 文档放置存在争议（目录边界/模块划分不确定）

## 约束

1. **禁止修改 `/docs/参考/`** - 仅 `sop-document-sync` 可维护
2. **基于目录划分** - 每个独立目录创建独立 design.md
3. **基于复杂度判断** - 低复杂度可省略，中高复杂度必须创建
4. **必须包含接口契约** - 输入/输出/依赖必须明确定义
5. **渐进式披露** - 复杂度越高，设计文档越详细
6. **目录深度标记** - 记录目录深度用于并行执行调度
7. **跨目录依赖声明** - 必须声明与其他目录的依赖关系
8. 必须引用SSOT：05_constraints/state_dictionary.md、05_constraints/command_dictionary.md

## Spec 模式约束

- **规划阶段只读**: 在 Spec 模式规划阶段，本 Skill 仅执行只读分析，不进行实际代码修改
- **交互式提问**: 当检测到决策点时，必须通过 AskUserQuestion 向用户提问
- **冲突检测**: 执行前必须检测与现有 ADR/设计文档的冲突，参考 04_reference/spec_interactive_guide.md
- **决策记录**: 重要决策必须记录到 spec.md 的决策记录章节
- **ADR 引用**: 本 Skill 涉及的 ADR 文档：ADR-Spec-001（生命周期）、ADR-Spec-002（设计关系）、ADR-Spec-004（交互式提问）

## Failure Handling

- 当发现需要修改 `/docs/参考/` 且任务未包含该范围时，必须停止并转为 `[USER_DECISION]` 或调用 `sop-document-sync` 处理

## 快速参考
参见：AGENT_SOP.md（design.md 规则）+ 04_reference/design_guide.md
