---
version: v2.12.0
updated: 2026-02-25
---

# SOP 版本变更历史

---

## 版本号规则

```
v[主版本].[次版本].[修订版本]
```

| 版本位 | 变更类型 | 示例 |
|--------|----------|------|
| **主版本** | 体系重大变更（Skill/Prompt Pack/SSOT 重构） | v1→v2 |
| **次版本** | 新增/调整 Skill、工作流、文档类型 | v2.0→v2.1 |
| **修订版本** | 文档修正、错误修复、格式统一 | v2.0.0→v2.0.1 |

**版本同步检查**：版本更新时必须执行 [版本同步检查清单](05_constraints/version_sync_checklist.md)。

---

## 版本历史

### v2.12.0 (2026-02-25)

**系统性审查与版本同步** - 完成SOP体系全面审查，统一版本号，清理冗余文档

#### 关键变更

- **版本号同步**：
  - 同步18个文档版本号至v2.12.0
  - 确立版本基准文档：state_dictionary.md、command_dictionary.md、02_skill_matrix/index.md
- **内容去重**：
  - 识别质量门控机制重复定义（3处）
  - 识别三错即停规则重复定义（4处）
  - 识别路径选择规则重复定义（3处）
- **模拟场景验证**：
  - 完成7个典型编程场景验证
  - 验证入口导航流畅性（L1→L2→L3→L4）
  - 验证跨文档引用正确性
  - 验证模板引用完整性
- **文档清理**：
  - 移除 `04_context_reference/adr_Spec_001-004.md`（内容已整合到对应文档）
  - 移除 `05_constraints/sop_existing_optimization.md`（已整合到improvement_plan.md）
  - 移除 `05_constraints/sop_optimization_proposal.md`（已整合到improvement_plan.md）
  - 移除 `05_constraints/automation_check_proposal.md`（方案阶段文档）
  - 清理 `.trae/specs/` 临时产物（保留关键报告）
  - 清理 `.trae/documents/` 临时文档

#### 主要更新文件

- 核心文档：`AGENT_SOP.md`、`01_concept_overview.md`、`CHANGELOG.md`
- 约束文档：`constraint_matrix.md`、`coding_principles.md`、`acceptance_criteria.md`、`05_constraints/index.md`
- 工作流文档：`03_workflow/index.md`、`fast_path.md`、`deep_path.md`、`three_strike_rule.md`
- 参考文档：`04_reference/index.md`、`design_guide.md`、`spec_interactive_guide.md`、`document_directory_mapping.md`
- 交互格式模板：`continuation_request.md`、`code_review.md`、`worker_execution_result.md`

---

### v2.9.2 (2026-02-24)

**逻辑缺口补全** - 根据用户决策补全 9 个逻辑缺口

#### 关键变更

- **状态字典增强**：
  - 新增 `[ARCHITECTURE_FAILED]` 状态（架构审查失败处理）
  - 完善 `[GATE_FAILED]` 状态（门控失败等待用户决策）
  - 新增 `[已完成]` 状态排除说明（不可作为续跑起点）
- **命令字典增强**：
  - 新增 `ARCH_REPAIR(reason)` 命令（架构修复）
  - 新增 `ARCH_ROLLBACK(reason)` 命令（架构回滚）
  - 新增 `GATE_RETRY(fix_description)` 命令（门控重试）
  - 新增 `GATE_ROLLBACK(reason)` 命令（门控回滚）
- **流程文档增强**：
  - 功能迭代新增架构影响评估检查点
  - 新增目录调度状态机章节
  - 新增调度状态保存格式（JSON）
- **约束矩阵增强**：
  - 门控失败处理规则更新（每次失败需用户决策）
  - 门控失败与三错即停机制独立
- **模板更新**：
  - `continuation_request.md` 新增 `[已完成]` 排除说明

#### 用户决策记录

| 问题 | 决策 |
|------|------|
| 架构审查失败处理 | 新增 `[ARCHITECTURE_FAILED]` 状态，用户选择修复/回滚/终止 |
| 架构变更检测 | 功能迭代入口增加架构影响评估检查点 |
| 依赖唤醒机制 | 依赖目录完成后自动触发下游目录 |
| 状态机位置 | 在 `03_workflow/index.md` 中增加状态机章节 |
| 并行批次大小 | 由 LLM 根据系统资源判断，不固定限制 |
| 调度状态格式 | JSON 格式保存到 `.trae/scheduler_state.json` |
| 门控与三错关系 | 门控失败独立于三错即停，每次失败需用户决策 |
| 门控重试机制 | 进入 `[GATE_FAILED]` 状态，等待用户决策 |
| 已完成排除说明 | 在状态字典和续跑模板两处都增加说明 |

#### 主要更新文件

- 状态字典：`05_constraints/state_dictionary.md`
- 命令字典：`05_constraints/command_dictionary.md`
- 工作流：`03_workflow/index.md`
- 约束矩阵：`05_constraints/constraint_matrix.md`
- 续跑模板：`04_reference/interaction_formats/continuation_request.md`

---

### v2.9.1 (2026-02-24)

**系统性审查修复** - 根据 sop_GUIDE.md 进行全面审查并修复问题

#### 关键变更

- **SSOT 一致性修复**：
  - 新增 `[WAITING_ADR_CONFIRM]` 状态定义
  - 修复 `Diff展示` 为 `[DIFF_APPROVAL]`（3处）
  - 修复 `ASK_USER_DECISION` 命令格式（4处）
  - 更新命令字典参数定义（`TEST_DESIGN_CSV`、`TEST_IMPLEMENT`、`RUN_DIR_BATCH`、`FAST_PATH_CHECK`）
- **表达规范修复**：
  - 修复含混词"尽量"为命令式表达（3处）
  - 修复规则格式问题（2处）
  - 精简重复定义为引用（三错即停、路径选择、版本号管理）
- **版本同步**：
  - 32+ 核心文件版本号同步至 v2.9.0
- **链接修复**：
  - 修复 AGENT_SOP.md 无效链接（5处）
  - 修复 01_concept_overview.md 无效链接（2处）
  - 修复 Skill 合约中的过时引用（5处）

#### 主要更新文件

- 状态字典：`05_constraints/state_dictionary.md`
- 命令字典：`05_constraints/command_dictionary.md`
- 入口文档：`AGENT_SOP.md`
- 概念文档：`01_concept_overview.md`
- 审查标准：`04_reference/review_standards/test_code.standard.md`、`source_dependency.standard.md`、`context_handoff.standard.md`
- 验收标准：`05_constraints/acceptance_criteria.md`
- Skill 合约：`skills/sop-code-review/SKILL.md`、`sop-fast-path/SKILL.md`、`sop-deep-path/SKILL.md` 等

---

### v2.9.0 (2026-02-24)

**6A工作流融合优化** - 将6A方法论融入现有SOP体系

#### 关键变更

- **质量门控机制**：
  - 新增各阶段质量门控检查清单
  - 定义门控失败处理流程
  - 新增门控状态定义（`[GATE_PASSED]`、`[GATE_FAILED]`）
- **文档模板增强**：
  - `implementation_design.md` 新增质量门控检查清单、输入/输出契约定义章节
  - `architecture_design.md` 新增质量门控检查清单、架构图建议章节
- **Skill合约增强**：
  - 6个Skill合约新增质量门控检查要求
  - 新增中断恢复状态保存要求
  - 新增评估阶段产出建议
- **约束文档增强**：
  - `constraint_matrix.md` 新增质量门控约束章节
  - `state_dictionary.md` 新增门控状态定义

#### 主要更新文件

- 入口文档：`AGENT_SOP.md`
- 工作流：`03_workflow/index.md`
- Skill矩阵：`02_skill_matrix/index.md`
- 约束矩阵：`05_constraints/constraint_matrix.md`
- 状态字典：`05_constraints/state_dictionary.md`
- 文档模板：`04_reference/document_templates/implementation_design.md`、`architecture_design.md`
- Skill合约：`skills/sop-requirement-analyst/SKILL.md`、`sop-architecture-design/SKILL.md`、`sop-implementation-designer/SKILL.md`、`sop-code-implementation/SKILL.md`、`sop-progress-supervisor/SKILL.md`、`sop-document-sync/SKILL.md`

---

### v2.8.1 (2026-02-23)

**持续改进机制建立** - 根据全面审查结果建立改进机制

#### 关键变更

- **新增版本同步检查清单**：
  - 新增 `05_constraints/version_sync_checklist.md`
  - 定义版本更新检查流程
  - 定义检查结果模板
- **更新约束索引**：
  - `05_constraints/index.md` 新增版本同步检查清单引用
- **更新审查指南**：
  - `sop_GUIDE.md` 新增审查触发条件章节
  - 新增 SSOT 漂移监控方法

#### 主要更新文件

- 版本同步检查清单：`05_constraints/version_sync_checklist.md`（新增）
- 约束索引：`05_constraints/index.md`
- 审查指南：`sop_GUIDE.md`

---

### v2.8.0 (2026-02-23)

**prompts 目录删除与 SSOT 合并** - 消除内容重复，符合 SSOT 原则

#### 关键变更

- **🔴 BREAKING：删除 prompts 目录**：
  - 删除 `prompts/` 整个目录（20个文件）
  - 全局不变量合并到 `05_constraints/constraint_matrix.md`
  - 编排规则合并到 `03_workflow/index.md`
  - 侧重点合并到各 `skills/*/SKILL.md`
- **Skill 合约更新**：
  - 所有 17 个 Skill 合约新增"侧重点"章节
  - 版本号统一更新为 v2.8.0
- **参考文档更新**：
  - `prompt_pack.standard.md` 标记为废弃
  - `02_skill_matrix/index.md` 删除"默认 Prompt 模块"列
- **工作流更新**：
  - `03_workflow/index.md` 新增"编排入口"和"能力选择协议"章节

#### 删除文件

- `prompts/packs/default/00_system.md`（已合并）
- `prompts/packs/default/01_operator.md`（已合并）
- `prompts/packs/default/index.md`（已删除）
- `prompts/packs/default/skills/*.md`（17个文件，已合并）

#### 主要更新文件

- 约束矩阵：`05_constraints/constraint_matrix.md`
- 工作流入口：`03_workflow/index.md`
- Skill 合约：`skills/*/SKILL.md`（17个）
- Skill 矩阵：`02_skill_matrix/index.md`
- Prompt Pack 规范：`04_reference/prompt_pack.standard.md`（标记废弃）

---

### v2.7.1 (2026-02-23)

**文件架构整理** - 清理过时和临时性内容，同步版本号

#### 关键变更

- **删除过时审查报告**：
  - 删除 `SOP_REVIEW_REPORT.md`（v2.0.0，内容已过时）
  - 删除 `SKILL_REVIEW_REPORT.md`（v2.2.0，问题已修复）
  - 删除 `PROMPT_SKILL_CONSISTENCY_REPORT.md`（v2.0.0，内容已过时）
- **删除已废弃文件**：
  - 删除 `ROLE_CHEATSHEET.md`（已标记 deprecated）
- **清理已完成 specs 目录**：
  - 删除 `.trae/specs/` 下 6 个已完成的任务目录
- **版本号同步**：
  - 更新 `command_dictionary.md` 版本号（v2.4.0 → v2.7.0）
  - 更新 `state_dictionary.md` 版本号（v2.4.0 → v2.7.0）

---

### v2.7.0 (2026-02-23)

**文档精简与整合** - 消除重复内容，建立清晰的引用层次

#### 关键变更

- **文档整合**：
  - 合并 `spec_design_questioning.md`、`conflict_detection_rules.md`、`questioning_checklist.md` 为 `spec_interactive_guide.md`（184行）
  - 合并 `design_decision_rules.md`、`design_directory_strategy.md` 为 `design_guide.md`（284行）
  - 精简 `document_directory_mapping.md`（119行→82行）
- **ADR 整理**：
  - ADR-Spec-003 标记为"历史决策记录"
  - ADR-Spec-004 更新引用关系
- **引用关系优化**：
  - 核心决策引用 ADR 作为唯一真源
  - 参考文档精简为操作指南
  - 消除循环引用

#### 删除文件

- `04_reference/spec_design_questioning.md`（已合并）
- `04_reference/conflict_detection_rules.md`（已合并）
- `04_reference/questioning_checklist.md`（已合并）
- `04_reference/design_decision_rules.md`（已合并）
- `04_reference/design_directory_strategy.md`（已合并）

#### 主要更新文件

- 新增：`04_reference/spec_interactive_guide.md`
- 新增：`04_reference/design_guide.md`
- 精简：`04_reference/document_directory_mapping.md`
- 更新：`04_context_reference/adr_Spec_003_version_sync.md`
- 更新：`04_context_reference/adr_Spec_004_interactive_questioning.md`
- 更新：`AGENT_SOP.md`、`04_reference/index.md`、`05_constraints/constraint_matrix.md`

---

### v2.6.0 (2026-02-22)

**审查改进实施** - 根据系统性审查报告实施改进，完善 Skill 合约与 ADR 模板

#### 关键变更

- **Skill 合约 Spec 约束**：
  - 为全部 17 个 Skill 合约添加 Spec 模式约束章节
  - 包含：规划阶段只读、交互式提问、冲突检测、决策记录、ADR 引用
  - sop-fast-path 新增 Spec 模式升级规则
- **ADR 模板完善**：
  - 合并"决策变更记录"和"决策记录"为统一的"决策记录"章节
  - 完善用户确认机制，扩展为四个选项：更新现有 ADR、创建新 ADR、请求讨论、跳过更新
- **快速路径与 Spec 模式交互**：
  - 明确升级触发条件（用户决策、跨文件影响、ADR 冲突、复杂度超限）
  - 定义升级目标（Spec 模式交互式提问 / 深度路径）
  - 规范升级流程

#### 主要更新文件

- ADR 模板：`04_reference/document_templates/adr.md`
- 入口文档：`AGENT_SOP.md`
- Skill 合约（17个）：
  - 核心：`sop-fast-path`、`sop-deep-path`、`sop-architecture-design`、`sop-implementation-designer`、`sop-code-implementation`
  - 辅助：`sop-code-review`、`sop-architecture-reviewer`、`sop-test-design-csv`、`sop-test-implementation`、`sop-tdd-workflow`
  - 协调：`sop-workflow-orchestrator`、`sop-progress-supervisor`、`sop-document-sync`、`sop-design-placement`
  - 探索：`sop-code-explorer`、`sop-requirement-analyst`、`sop-capability-reuse`

---

### v2.4.0 (2026-02-22)

**Spec 生命周期与版本同步** - 定义 Spec 产物生命周期、建立版本同步机制

#### 关键变更

- **Spec 产物生命周期**：
  - 定义执行期→归档期→清理三阶段
  - 明确归档判断标准
  - 设计先行原则：持久化设计 vs 临时规范
- **Spec 与 design.md 映射关系**：
  - 任务划分原则：单目录任务 = 单个 design.md
  - 执行顺序：自底向上（depth_desc）
  - 任务声明字段：design_path、depth、dependencies、scope
- **版本同步机制**：
  - 批量更新 22+ 文件版本号
  - 建立版本一致性检查流程
- **ADR 记录**：
  - 新增 `adr_Spec_001_lifecycle.md`：Spec 生命周期管理
  - 新增 `adr_Spec_002_design_relation.md`：Spec 与 design.md 关系定义

#### 主要更新文件

- 目录映射：`04_reference/document_directory_mapping.md`
- 目录策略：`04_reference/design_directory_strategy.md`
- 决策规则：`04_reference/design_decision_rules.md`
- ADR：`04_context_reference/adr_Spec_001_lifecycle.md`（新增）
- ADR：`04_context_reference/adr_Spec_002_design_relation.md`（新增）
- 全部 Skill 合约：版本同步更新
- 全部参考文档：版本同步更新

---

### v2.2.0 (2026-02-21)

**基于审查结果的系统性改进** - 统一术语、增强任务管理、补充缺失机制

#### 关键变更

- **术语统一**：
  - 状态图 `DONE` 改为 `[已完成]`
  - `Diff展示` 改为 `[DIFF_APPROVAL]`
  - 新增 `RESUME` 伪状态说明
- **任务管理机制重构**：
  - 新增 `TASK_SPEC_CREATE`、`TASK_SPEC_SYNC` 命令
  - design.md 支持与 trae spec 对齐的 spec/tasks/checklist 结构
  - 新增任务粒度指导（单任务建议不超过 4 小时）
- **目录依赖表**：implementation_design.md 模板新增"目录依赖"章节
- **决策记录规范**：
  - 新增决策记录模板 `decision_record.md`
  - 明确决策记录路径规范 `docs/04_context_reference/decisions/YYYY-MM-DD_[topic].md`
- **依赖循环检测**：
  - 新增 `CYCLE_CHECK` 命令
  - 新增 `[CYCLE_DETECTED]` 状态
  - sop-progress-supervisor 增加循环检测步骤
- **迭代反馈机制**：
  - 新增 `ITERATION_COUNT`、`ITERATION_RESET` 命令
  - 定义迭代阈值（正常≤3，警告=4，熔断≥5）
- **快速路径量化**：新增 AST 变化检测说明和量化判断标准表
- **覆盖率阈值**：补充 L2-L4 覆盖率建议（L2≥70%，L3≥60%，L4 关键路径 100%）

#### 主要更新文件

- 状态机：`sop_state_machine.md`
- 状态字典：`05_constraints/state_dictionary.md`
- 命令字典：`05_constraints/command_dictionary.md`
- 模板：`04_reference/document_templates/implementation_design.md`
- 模板：`04_reference/document_templates/decision_record.md`（新增）
- 目录映射：`04_reference/document_directory_mapping.md`
- 快速路径：`03_workflow/fast_path.md`
- 验收标准：`05_constraints/acceptance_criteria.md`
- Skill合约：`skills/sop-progress-supervisor/SKILL.md`

---

### v2.1.0 (2026-02-12)

**入口统一与任务管理增强** - 合并入口文档，增强 design.md 任务管理能力

#### 关键变更

- **入口统一**：合并 `LLM_INDEX.md` 到 `AGENT_SOP.md` 作为唯一入口
- **角色定位**：`sop_for_human.md` 标记为仅供参考
- **任务状态管理**：新增任务状态（待处理/进行中/已完成/已阻塞/已归档）
- **任务命令**：新增 `TASK_START`、`TASK_COMPLETE`、`TASK_BLOCK`、`TASK_ARCHIVE` 命令
- **归档机制**：`sop-document-sync` 支持目录归档时自动清除已完成任务

#### 主要更新文件

- 入口：`AGENT_SOP.md`（合并原 `LLM_INDEX.md`）
- 模板：`04_reference/document_templates/implementation_design.md`
- 状态字典：`05_constraints/state_dictionary.md`
- 命令字典：`05_constraints/command_dictionary.md`
- Skill合约：`skills/sop-document-sync/SKILL.md`

#### 删除文件

- `LLM_INDEX.md`（已合并至 `AGENT_SOP.md`）

---

### v2.0.0 (2026-02-12)

**Skill-first 体系上线** - 以 Skill 作为唯一执行单元，Prompt Pack 作为偏好层，SSOT 收敛

#### 关键变更

- **Skill 矩阵 SSOT**：新增/完善 `02_skill_matrix/index.md`，作为 Skill 清单与边界唯一真源
- **Prompt Pack 规范化**：Prompts 以 `prompts/packs/<pack>/skills/<skill>.md` 组织，默认 pack 为 `default`
- **测试资产隔离**：引入 `sop-test-design-csv` 与 `sop-test-implementation`，并在约束中固化 CSV 与测试代码隔离规则
- **字典与约束收敛**：状态/命令/红线统一以 `05_constraints/*` 为准，工作流直接引用 SSOT
- **交付物模板统一**：交互格式与文档模板统一引用 `04_reference/*`，便于审查与落盘

#### 主要更新文件（摘要）

- SSOT：`02_skill_matrix/index.md`
- 工作流：`03_workflow/*`
- 约束：`05_constraints/*`
- 模板与标准：`04_reference/*`
- Prompt Pack：`prompts/packs/default/*`
- Skill 合约：`skills/*/SKILL.md`

---

## 备注

- 若需要追溯 v1.x 版本历史，请以版本控制系统中的历史记录为准。
