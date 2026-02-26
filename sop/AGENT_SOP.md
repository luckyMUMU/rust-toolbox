---
version: v2.12.0
updated: 2026-02-25
sop_path: sop/
---

# SOP（Skill-first / Agent 执行版）

> **SOP 文件夹路径**：`sop/` - 所有 SOP 相关文件的根目录，所有引用路径均基于此目录计算。
>
> **Skill 引用机制**：Skill 文件中的引用应优先读取本文件的 `sop_path` 字段获取 SOP 路径，然后根据相对路径获取目标文件。
>
> **唯一入口**：本文档是 SOP 的唯一入口，包含快速分诊、核心约束、工作流和导航。
> 
> **人类阅读版**：本文档为唯一入口，概念详情参见 [01_concept_overview.md](01_concept_overview.md)。

---

## 使用方式

- **目标**：不加载引用正文也能判断"是否需要继续加载"，并保证从本入口到任意 SOP 文档最短跳数 ≤3。
- **规则**：本页只提供最小摘要 + 直达链接，不复制正文。

---

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
- 最多3次门控重试
- 3次失败后进入`[USER_DECISION]`
- 必须记录失败原因和检查项

---

## 快速分诊

| 需求 | 目标文档 |
|------|----------|
| 直接执行任务（约束/停止点/权限） | 继续阅读本文档 |
| 理解体系概念（L1-L4、渐进披露、SSOT） | [01_concept_overview.md](01_concept_overview.md) |
| 查看 Skill 清单与边界（唯一真源） | [02_skill_matrix/index.md](02_skill_matrix/index.md) |
| 选择路径与执行流程（fast/deep/TDD） | [03_workflow/index.md](03_workflow/index.md) |
| 状态机速查（状态/转移/子流程图） | [状态字典](05_constraints/state_dictionary.md) |
| 模板/交互格式/审查标准入口 | [04_reference/index.md](04_reference/index.md) |
| 全局约束/红线/状态与命令字典 | [05_constraints/index.md](05_constraints/index.md) |

### 工程质量速查

| 需求 | 目标文档 |
|------|----------|
| 基础编码原则（六大原则 + 方法层级一致性） | [coding_principles.md](05_constraints/coding_principles.md) |
| 代码审查格式（报告模板） | [code_review.md](04_reference/interaction_formats/code_review.md) |
| 代码 Diff 审查标准（硬门槛） | [code_diff.standard.md](04_reference/review_standards/code_diff.standard.md) |

### 合约与偏好

| 需求 | 目标文档 |
|------|----------|
| Skill 合约（SKILL.md）索引 | [skills/index.md](skills/index.md) |

### 审查归档

| 需求 | 目标文档 |
|------|----------|
| 审查归档入口 | [reviews/](reviews/) |

---

## 核心约束

1. 先标记`[DIR_WORKING]`，再改代码
2. 父目录只保留摘要+链接
3. `[DIR_WORKING]`→`[DIR_COMPLETED]`
4. 各 Skill 只操作合约范围（Scope）
5. 先复用→改进→新建→清理
6. **实现类 Skill 按目录工作**: 以 design.md 所在目录为工作范围
7. **自底向上并行**: 按目录深度从深到浅并行执行
8. **无出处不决断**: 无法追溯到既定依据的判断/决策须带建议询问用户（`ASK_USER_DECISION(topic, options)`），不得自行决断推进
9. **审查须确认**: 各审查环节结论须通过对用户的明确提问完成确认，审查输出须包含可操作确认项（是否通过/是否修订/选项）
10. **设计冲突检测**: Spec 编写过程中实时检测与 ADR/设计文档/约束矩阵的冲突
11. **即时提问机制**: 发现冲突时立即暂停，向用户提问获取决策
12. **决策分级记录**: 重要决策记录到 ADR，一般决策记录到 spec.md

**禁止项矩阵**: [查看完整黑白名单](05_constraints/constraint_matrix.md)

**中断与再执行**：流程支持在任意停止点中断后，经用户决策与方案调整（重建），再从可恢复检查点再执行。可恢复检查点清单与续跑格式见 [03_workflow/index.md#中断与再执行](03_workflow/index.md#中断与再执行)、[state_dictionary.md](05_constraints/state_dictionary.md#可恢复检查点recoverable-checkpoints)、[continuation_request](04_reference/interaction_formats/continuation_request.md)。

---

## 路径选择

👉 [路径选择详情](03_workflow/index.md)

---

## Skill 指令（SSOT）

以 [Skill 矩阵（SSOT）](02_skill_matrix/index.md) 为准（Skill 清单、触发条件、输入/输出、停止点、落盘交付物、默认 Prompt Pack 映射）。

---

## 目录维度工作范围

### 实现类 Skill 工作范围定义

实现类 Skill（如 `sop-code-implementation`）以 `design.md` 所在目录为工作边界：

CMD: `DIR_SCOPE(dir_with_design_md) = dir/** - {subdir/** | subdir contains design.md}`

参见：04_reference/design_guide.md + 05_constraints/command_dictionary.md

### 目录层级处理顺序

CMD: `LIST_DESIGN_MD(root) -> design_list`
CMD: `SCHEDULE_DIRS(design_list) -> dir_map`
CMD: `RUN_DIR_BATCH(depth_desc)`（同 depth 并行；父目录等待子目录 `DIR_COMPLETED`）

👉 [目录维度工作策略详情](04_reference/design_guide.md)

---

## 工作流

### 目录维度深度路径

**核心流程** (带并行执行)
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

design_list 由 **sop-code-explorer** 产出，**sop-progress-supervisor** 接收后执行 SCHEDULE_DIRS 创建 dir_map。参见：05_constraints/command_dictionary.md（LIST_DESIGN_MD 主体为 sop-code-explorer）。

**目录并行执行流程**
```
1. `sop-code-explorer` 扫描目录结构，识别所有 design.md，产出 design_list
2. `sop-progress-supervisor` 接收 design_list，按目录深度排序，创建 dir_map
3. `sop-progress-supervisor` 按深度降序分批调度 `sop-code-implementation`（同深度并行）
4. `sop-code-implementation` 在 Scope 内执行；遇到跨目录依赖则进入 `[DIR_WAITING_DEP]`
5. `sop-progress-supervisor` 监控状态并唤醒等待依赖的目录批次
6. `sop-code-review` 审查 Diff（只输出报告）；失败则回到 `sop-code-implementation` 返工
7. 全部目录完成且审查放行后，`sop-document-sync` 更新文档与索引并归档已完成任务
```

### 标准深度路径（单目录）
```
新项目:
sop-requirement-analyst → sop-architecture-design → sop-architecture-reviewer
→ sop-implementation-designer → sop-code-implementation → sop-code-review → sop-document-sync

功能迭代:
sop-requirement-analyst → sop-implementation-designer → sop-code-implementation
→ sop-code-review → sop-document-sync
```

### 分层验收深度路径 (推荐)
```
sop-requirement-analyst
→ sop-architecture-design
→ sop-architecture-reviewer
→ sop-implementation-designer
→ sop-test-design-csv
→ sop-test-implementation
→ sop-progress-supervisor (dir_map)
→ sop-code-implementation (运行验收 + 修正代码)
→ sop-code-review
→ sop-document-sync
```

**验收流程** (由实现类 Skill 驱动)
CMD: `RUN_ACCEPTANCE(L1) -> [WAITING_FOR_L1_REVIEW] -> REVIEW_ACCEPTANCE(L1)`
CMD: `RUN_ACCEPTANCE(L2) -> [WAITING_FOR_L2_REVIEW] -> REVIEW_ACCEPTANCE(L2)`
CMD: `RUN_ACCEPTANCE(L3) -> [WAITING_FOR_L3_REVIEW] -> REVIEW_ACCEPTANCE(L3)`
CMD: `RUN_ACCEPTANCE(L4) -> [WAITING_FOR_L4_REVIEW] -> REVIEW_ACCEPTANCE(L4)`

参见：05_constraints/acceptance_criteria.md + 05_constraints/command_dictionary.md

**快速路径**
```
sop-code-explorer → sop-code-implementation → sop-code-review → sop-document-sync
```

适用条件与升级红线参见：[快速路径](03_workflow/fast_path.md)

### 快速路径与 Spec 模式交互

当快速路径任务执行过程中出现以下情况时，必须升级处理：

1. **升级触发条件**：
   - 检测到需要用户决策
   - 发现跨文件/跨目录影响
   - 存在与现有 ADR/设计文档的冲突
   - 任务复杂度超出快速路径限制

2. **升级目标**：
   - 优先升级到 Spec 模式交互式提问
   - 复杂任务升级到深度路径

3. **升级流程**：
   - 记录当前任务状态
   - 通过 AskUserQuestion 向用户说明升级原因
   - 等待用户确认后切换模式

---

## Spec 模式交互式提问

### 提问触发时机

Spec 编写过程中，在以下章节级检测点触发冲突检测：

| 检测点 | 检测内容 | 触发条件 |
|--------|----------|----------|
| 架构章节 | 与现有 ADR 冲突 | 技术选型、架构模式变更 |
| 接口章节 | 与现有设计文档冲突 | 接口签名、数据结构变更 |
| 约束章节 | 与约束矩阵冲突 | 权限、安全、性能约束 |
| 依赖章节 | 与模块边界冲突 | 跨模块依赖、循环依赖 |

### 冲突优先级处理

按影响范围确定冲突优先级，高优先级冲突必须先解决：

| 优先级 | 影响范围 | 处理方式 |
|--------|----------|----------|
| P0-阻断 | 架构决策、安全合规 | 立即暂停，必须用户确认 |
| P1-高 | 跨模块影响、性能边界 | 暂停并提问，建议解决方案 |
| P2-中 | 单模块内设计变更 | 提示用户，可继续推进 |
| P3-低 | 风格、命名等非关键 | 记录到 spec.md，不暂停 |

### ADR 确认更新机制

当检测到与 ADR 相关的冲突时：

1. **暂停当前编写**：标记当前章节状态为 `[WAITING_ADR_CONFIRM]`
2. **展示冲突详情**：列出冲突的 ADR 条目、当前设计、冲突点
3. **提问用户决策**：
   - 选项 A：遵循现有 ADR，调整当前设计
   - 选项 B：更新 ADR，记录变更理由
   - 选项 C：创建新 ADR，记录新决策
4. **执行用户选择**：更新相关文档后继续

### 决策分级记录

根据决策重要性选择记录位置：

| 决策类型 | 记录位置 | 模板 |
|----------|----------|------|
| 架构变更、技术选型 | ADR | [adr.md](04_reference/document_templates/adr.md) |
| 接口设计、数据结构 | spec.md | 设计章节 |
| 实现细节、代码风格 | spec.md | 实现说明章节 |
| 临时决策、待定事项 | spec.md | 待办/备注章节 |

**ADR 记录触发条件**：
- 涉及架构层面的技术选型变更
- 影响多个模块的接口变更
- 安全、性能、可用性等非功能性决策
- 与既有 ADR 存在冲突的决策

👉 [ADR 模板详情](04_reference/document_templates/adr.md)

---

## 三错即停

👉 [三错即停详情](03_workflow/three_strike_rule.md)

---

## 文档位置

参见 [document_directory_mapping.md](04_reference/document_directory_mapping.md)（逻辑目录 → 项目实际目录映射）。

### 需求文档
| 类型 | 位置 | 层级 | 产出 Skill |
|------|------|------|--------|
| 项目PRD | `docs/01_requirements/project_prd.md` | L1 | sop-requirement-analyst |
| 模块MRD | `docs/01_requirements/modules/[module]_mrd.md` | L2 | sop-requirement-analyst |
| 功能FRD | `docs/01_requirements/modules/[module]/[feature]_frd.md` | L3 | sop-requirement-analyst |
| 原型 | `docs/01_requirements/prototypes/[module]/` | L3 | sop-requirement-analyst |

### 设计文档
| 类型 | 位置 | 层级 | 产出 Skill |
|------|------|------|--------|
| 架构设计 | `docs/02_logical_workflow/*.md` | L2 | sop-architecture-design |
| 实现设计 | `src/**/design.md` | L3 | sop-implementation-designer |
| 测试用例 | `docs/03_technical_spec/test_cases/*.csv` | L3 | sop-test-design-csv |
| 测试代码 | `tests/*.test.[ext]` | L3 | sop-test-implementation |

**约束**: `/docs/参考/` **非指定不变更**

---

## 需求分层（sop-requirement-analyst）

| 层级 | 文档 | 内容 | 触发条件 |
|------|------|------|----------|
| L1 | Project PRD | 项目愿景、模块清单 | 新项目 |
| L2 | Module MRD | 模块功能、边界 | 新模块 |
| L3 | Feature FRD | 功能详情、交互 | 新功能 |
| L3 | Prototype | 界面原型 | UI项目 |

👉 [需求分层详情](04_reference/index.md#l1-l3-需求分层-analyst)

---

## design.md规则

| 复杂度 | 行数 | 要求 |
|--------|------|------|
| 低 | <100 | 创建极简design.md（仅接口契约），快速路径可省略 |
| 中 | 100-500 | 简要design.md+接口契约+任务清单 |
| 高 | >500 | 完整design.md+详细契约+全部章节 |

### 任务管理

design.md 中的任务清单支持以下状态：

| 状态 | 标记 | 含义 |
|------|------|------|
| 待处理 | `[ ]` | 任务尚未开始 |
| 进行中 | `[-]` | 任务正在执行 |
| 已完成 | `[x]` | 任务已完成并通过验证 |
| 已阻塞 | `[!]` | 任务被阻塞，需外部依赖 |
| 已归档 | `[archived]` | 任务已归档，不在活跃清单显示 |

**归档规则**：当目录状态变为 `[DIR_COMPLETED]` 时，`sop-document-sync` 自动将已完成任务移入归档章节。

👉 [design.md模板详情](04_reference/document_templates/implementation_design.md)

---

## TDD规则 (可选)

**启用条件**: 核心业务/复杂逻辑/高覆盖要求

**测试用例来源**: 仅基于设计文档 (L2+L3)，不参考代码

**测试代码来源**: 主要基于CSV，仅参考代码接口

👉 [TDD工作流详情](skills/sop-tdd-workflow/SKILL.md)

---

## 版本号管理

👉 [版本号规则](CHANGELOG.md#版本号规则)

---

## 导航

| 层级 | 文档 |
|------|------|
| L1 | [核心概念](01_concept_overview.md) |
| L2 | [Skill矩阵](02_skill_matrix/index.md) |
| L3 | [工作流](03_workflow/index.md) |
| L4 | [参考文档](04_reference/index.md) |
| L4-ADR | [架构决策](04_reference/document_templates/adr.md) |
| L4-RAG | [参考资料管理](04_reference/knowledge_management.md) |
| Skills | [skills/](skills/) |
| 版本历史 | [CHANGELOG.md](CHANGELOG.md) |
