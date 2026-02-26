---
version: v2.12.0
updated: 2026-02-25
---

# 状态字典

---

## 目的

本文件定义 SOP 全部状态标记的**唯一来源**（Single Source of Truth）。

约束：
- 文档、Prompt、技能说明中的状态标记必须引用本字典
- 新增/变更状态必须先更新本字典，再更新引用方

---

## 质量门控状态（Quality Gate States）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[GATE_PASSED]` | 各阶段Skill | 门控检查通过 | 进入下一阶段 |
| `[GATE_FAILED]` | 各阶段Skill | 门控检查失败，等待用户决策 | 用户选择：修复后重试 / 回滚 / 终止 |

**门控失败处理规则**：
- 门控失败不累计，每次失败都需要用户决策
- 用户可选择：修复后重试、回滚到上一阶段、终止任务
- 不与三错即停机制关联

### 门控检查点定义

| 阶段 | 检查点状态 | 检查项 |
|------|-----------|--------|
| 需求阶段 | `[GATE_REQUIREMENTS]` | 需求边界清晰、技术方案对齐、验收标准具体、关键假设确认 |
| 架构阶段 | `[GATE_ARCHITECTURE]` | 架构图清晰、接口定义完整、与现有系统无冲突、设计可行 |
| 实现设计阶段 | `[GATE_DESIGN]` | 任务覆盖完整、依赖无循环、每个任务可独立验证 |
| 代码实现阶段 | `[GATE_IMPLEMENTATION]` | 代码规范、测试通过、文档同步 |
| 文档同步阶段 | `[GATE_SYNC]` | 需求实现、验收满足、质量达标 |

### 门控状态转移规则

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[GATE_REQUIREMENTS]` | sop-requirement-analyst | 需求门控检查 | 通过→`[WAITING_FOR_REQUIREMENTS]`，失败→`[GATE_FAILED]` |
| `[GATE_ARCHITECTURE]` | sop-architecture-design | 架构门控检查 | 通过→`[WAITING_FOR_ARCHITECTURE]`，失败→`[GATE_FAILED]` |
| `[GATE_DESIGN]` | sop-implementation-designer | 设计门控检查 | 通过→`[WAITING_FOR_DESIGN]`，失败→`[GATE_FAILED]` |
| `[GATE_IMPLEMENTATION]` | sop-code-implementation | 实现门控检查 | 通过→`[WAITING_FOR_CODE_REVIEW]`，失败→`[GATE_FAILED]` |
| `[GATE_SYNC]` | sop-document-sync | 同步门控检查 | 通过→`[已完成]`，失败→`[GATE_FAILED]` |

### 门控失败标准处理流程

```
[GATE_FAILED] → ASK_USER_DECISION("门控检查失败", [
  "修复后重试",
  "回滚到上一阶段",
  "终止任务"
])
→ 用户选择后执行：
  - GATE_RETRY(fix_description) → 重新执行门控检查
  - GATE_ROLLBACK(reason) → 回滚到上一阶段
  - 终止流程
```

---

## 全局停止点（Global Stop Points）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[WAITING_FOR_REQUIREMENTS]` | sop-requirement-analyst | 需求文档已完成，等待确认 | 用户确认需求 |
| `[WAITING_FOR_ARCHITECTURE]` | sop-architecture-design | 架构设计已完成，等待确认 | 用户确认架构 |
| `[WAITING_FOR_DESIGN]` | sop-implementation-designer | 实现设计已完成，等待确认 | 用户确认设计 |

多目录场景下 **设计确认粒度**：默认**整批确认**（所有目录的 design.md 产出后，进行一次 `DESIGN_CONFIRM()`，用户确认后进入 SCHEDULE_DIRS）。若项目需每目录或仅关键目录确认，在实现设计中约定并执行对应停止点。

---

## 架构审查结果（Architecture Review Result）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[ARCHITECTURE_PASSED]` | sop-architecture-reviewer | 架构审查通过 | 进入实现设计阶段（sop-implementation-designer） |
| `[ARCHITECTURE_FAILED]` | sop-architecture-reviewer | 架构审查失败，等待用户决策 | 用户选择：修复架构 / 回滚到需求阶段 / 终止 |

**架构审查失败处理规则**：
- 触发 `ASK_USER_DECISION("架构审查失败", ["修复架构设计", "回滚到需求阶段", "终止任务"])`
- 用户选择后执行对应命令：`ARCH_REPAIR()` / `ARCH_ROLLBACK(reason)` / 终止流程

---

## 异常恢复路径定义（Exception Recovery Paths）

### `[ARCHITECTURE_FAILED]` 恢复路径

```
[ARCHITECTURE_FAILED] → ASK_USER_DECISION("架构审查失败", [
  "修复架构设计",
  "回滚到需求阶段",
  "终止任务"
])
→ 用户选择后执行：
  - ARCH_REPAIR(reason) → 返回架构设计阶段
  - ARCH_ROLLBACK(reason) → 回滚到需求阶段
  - 终止流程
```

### `[DIR_FAILED]` 恢复路径

```
[DIR_FAILED] → ASK_USER_DECISION("目录处理失败", [
  "重试当前目录",
  "跳过当前目录",
  "终止任务"
])
→ 用户选择后执行：
  - DIR_RETRY() → 重新执行当前目录
  - DIR_SKIP(reason) → 跳过当前目录，继续下一目录
  - 终止流程
```

### `[FUSION_TRIGGERED]` 恢复路径

```
[FUSION_TRIGGERED] → ASK_USER_DECISION("熔断触发", [
  "重置并继续",
  "人工介入",
  "终止任务"
])
→ 用户选择后执行：
  - FUSION_RESET() → 重置熔断计数器，继续执行
  - MANUAL_INTERVENTION() → 等待人工处理
  - 终止流程
```

### `[CYCLE_DETECTED]` 恢复路径

```
[CYCLE_DETECTED] → ASK_USER_DECISION("检测到循环依赖", [
  "打破循环（指定打破点）",
  "人工介入",
  "终止任务"
])
→ 用户选择后执行：
  - BREAK_CYCLE(point) → 打破循环依赖
  - MANUAL_INTERVENTION() → 等待人工处理
  - 终止流程
```

### `[GATE_FAILED]` 恢复路径

```
[GATE_FAILED] → ASK_USER_DECISION("门控检查失败", [
  "修复后重试",
  "回滚到上一阶段",
  "终止任务"
])
→ 用户选择后执行：
  - GATE_RETRY(fix_description) → 修复后重新执行门控检查
  - GATE_ROLLBACK(reason) → 回滚到上一阶段
  - 终止流程
```

---

## 失败处理规范（Failure Handling Standards）

### 错误类型分类

| 错误类型 | 描述 | 恢复策略 |
|----------|------|----------|
| `UNKNOWN_ERROR` | 未知错误 | 记录错误日志，查询用户如何处理 |
| `TOOL_UNAVAILABLE` | 工具不可用 | 尝试替代工具或降级方案 |
| `CONTEXT_OVERFLOW` | 上下文溢出 | 卸载非必要Skills，压缩上下文 |
| `DEPENDENCY_MISSING` | 依赖缺失 | 等待依赖完成或进入用户决策 |
| `VALIDATION_FAILED` | 验证失败 | 修复后重试 |
| `CONFLICT_DETECTED` | 冲突检测 | 进入冲突解决流程 |

### 三错即停机制详解

**触发条件**：
- "同一Skill同一步骤"：相同的Skill + 相同的action
- 连续3次失败触发熔断
- 不同失败原因分别计数

**熔断级别**：

| 级别 | 连续失败次数 | 影响 | 处理方式 |
|------|-------------|------|----------|
| 警告 | 2 | 提示即将熔断 | 记录警告，继续执行 |
| 熔断 | 3 | 暂停技能执行 | 进入`[FUSION_TRIGGERED]` |

**恢复流程**：
```
[FUSION_TRIGGERED] → 
  1. 记录熔断详情（Skill、步骤、失败原因、时间戳）
  2. 保存当前检查点
  3. ASK_USER_DECISION([
      "重置并继续（计数器归零）",
      "回滚到上一检查点",
      "人工介入",
      "终止任务"
    ])
  4. 根据用户选择执行对应命令
```

### 冲突检测机制

**触发时机**：
1. 每次保存设计文档时
2. 进入下一阶段前
3. 用户明确触发

**检测粒度**：
- **语义冲突**：接口签名变更、逻辑变更
- **依赖冲突**：循环依赖、缺失依赖
- **设计冲突**：与现有ADR/设计文档冲突

**冲突解决流程**：
```
[CONFLICT_DETECTED] →
  1. 生成冲突报告（冲突位置、冲突内容、影响范围）
  2. ASK_USER_DECISION([
      "采纳当前修改",
      "保留冲突，由后续处理",
      "人工介入解决"
    ])
  3. 更新设计文档状态
  4. 记录冲突解决决策
```

**冲突优先级**：
- 安全相关冲突：最高优先级，必须解决
- 接口兼容性冲突：高优先级，建议解决
- 设计风格冲突：低优先级，可记录待处理

---

## 代码审查停止点（Code Review Stop Points）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[WAITING_FOR_CODE_REVIEW]` | sop-code-implementation | 代码变更已就绪，等待代码审查 | sop-code-review 输出审查结论（通过/需修改/僵局→用户决策） |

### 统一审查状态（Unified Review State）

| 状态 | 触发者 | 含义 | 审查者 | 参数 |
|------|--------|------|--------|------|
| `[WAITING_FOR_REVIEW]` | sop-code-implementation / sop-test-implementation | 产物已就绪，等待审查 | sop-code-review | `type: code/test` |

**使用说明**：
- 新流程优先使用统一状态 `[WAITING_FOR_REVIEW]`，通过参数 `type` 指定审查类型
- 旧状态 `[WAITING_FOR_CODE_REVIEW]` 和 `[WAITING_FOR_TEST_IMPLEMENTATION]` 保留兼容性支持
- 审查类型参数：`code`（代码审查）、`test`（测试代码审查）

### 第二意见审查状态（Second Opinion Review States）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[REVIEW_CONFLICT]` | sop-code-review | 多审结论冲突，需要解决 | 用户选择解决方案后继续 |
| `[REVIEW_MERGING]` | sop-code-review | 正在合并多份审查报告 | 合并完成→`[DIFF_APPROVAL]`或`[REVIEW_CONFLICT]` |

**审查冲突处理规则**：
- 安全审查结论优先于其他审查结论
- 关键问题必须修复，非关键问题可记录为改进建议
- 无法自动解决的冲突进入`[USER_DECISION]`

---

## 测试相关停止点（Test Stop Points）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[WAITING_FOR_TEST_DESIGN]` | sop-test-design-csv | 测试设计（用例）已完成，等待确认 | 用户确认测试设计 |
| `[WAITING_FOR_TEST_IMPLEMENTATION]` | sop-test-implementation | 测试代码已完成，等待审查 | sop-code-review 输出审查结论（通过/需修改/僵局→用户决策） |
| `[WAITING_FOR_TEST_CREATION]` | sop-code-implementation → 用户 | 测试不充分，暂停实现等待决策 | 用户选择：补充测试/继续/暂停 |

---

## 历史别名与弃用状态（Deprecated States）

以下状态为历史别名或已弃用状态，**新文档禁止使用**：

| 弃用状态 | 替代状态 | 弃用原因 | 清理计划 |
|----------|----------|----------|----------|
| `[WAITING_FOR_TEST_REVIEW]` | `[WAITING_FOR_TEST_DESIGN]` | 命名不准确 | 已完成替换 |
| `[USER_DECISION_REQUIRED]` | `[USER_DECISION]` | 命名冗余 | 已完成替换 |

**兼容性处理规则**：
- 解析时自动将弃用状态映射到替代状态
- 输出时统一使用新状态名称
- 遇到弃用状态时输出警告日志

---

## 分层验收审查点（L1-L4 Review Points）

| 状态 | 触发者 | 含义 | 审查者 |
|------|--------|------|--------|
| `[WAITING_FOR_L1_REVIEW]` | sop-code-implementation | L1 验收测试通过，等待审查 | sop-code-review |
| `[WAITING_FOR_L2_REVIEW]` | sop-code-implementation | L2 验收测试通过，等待审查 | sop-code-review |
| `[WAITING_FOR_L3_REVIEW]` | sop-code-implementation | L3 验收测试通过，等待审查 | sop-code-review |
| `[WAITING_FOR_L4_REVIEW]` | sop-code-implementation | L4 验收测试通过，等待审查 | sop-code-review |

### 统一验收审查状态（Unified Acceptance Review）

| 状态 | 触发者 | 含义 | 审查者 | 参数 |
|------|--------|------|--------|------|
| `[WAITING_FOR_ACCEPTANCE_REVIEW]` | sop-code-implementation | 验收测试通过，等待审查 | sop-code-review | `level: L1/L2/L3/L4` |

**使用说明**：
- 新流程优先使用统一状态 `[WAITING_FOR_ACCEPTANCE_REVIEW]`，通过参数 `level` 指定验收级别
- 旧状态 `[WAITING_FOR_L1_REVIEW]` ~ `[WAITING_FOR_L4_REVIEW]` 保留兼容性支持
- 命令 `RUN_ACCEPTANCE(level)` 可输出统一状态或旧状态（根据项目配置）

---

## 目录执行状态（Directory Execution States）

| 状态 | 触发者 | 含义 | 典型场景 |
|------|--------|------|----------|
| `[DIR_WORKING]` | sop-code-implementation | 正在处理当前目录 | 开始执行当前目录任务 |
| `[DIR_WAITING_DEP]` | sop-code-implementation | 等待依赖目录完成 | 依赖目录尚未 `[DIR_COMPLETED]` |
| `[DIR_COMPLETED]` | sop-code-implementation | 当前目录处理完成 | 变更完成并通过必要验证 |
| `[DIR_FAILED]` | sop-code-implementation | 当前目录处理失败 | 失败无法恢复或熔断前置 |

---

## 跨目录依赖请求状态（Dependency Request States）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[WAITING_FOR_WORKER]` | sop-progress-supervisor | 依赖请求已发出，等待目标目录完成处理 | 目标目录完成处理并回报 |

---

## 调度协调状态（Scheduling Coordination States）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[SCHEDULING]` | sop-progress-supervisor | 正在创建目录-skill 映射与调度计划 | 进入并行执行或依赖等待 |
| `[PARALLEL_EXECUTING]` | sop-progress-supervisor | 多目录并行执行中 | 所有目录进入完成/等待 |
| `[WAITING_DEPENDENCY]` | sop-progress-supervisor | 存在目录依赖等待中 | 依赖目录完成后继续 |
| `[ALL_COMPLETED]` | sop-progress-supervisor | 所有目录处理完成 | 进入收尾/文档同步 |

---

## 人工确认点（Manual Approval Points）

| 标记 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[DIFF_APPROVAL]` | sop-code-implementation / sop-code-review | 展示变更 Diff（经审查通过后），等待人工审批 | 用户审批通过 |


## ADR 确认点（ADR Confirmation Points）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| [WAITING_ADR_CONFIRM] | Spec 编写过程 | 检测到与 ADR 冲突，等待用户确认 | 用户选择：遵循现有 ADR / 更新 ADR / 创建新 ADR |
---

## 伪状态（Pseudo States）

伪状态用于状态机图中表示控制流，不作为实际执行状态：

| 标记 | 含义 | 用途 |
|------|------|------|
| `RESUME` | 从检查点续跑的元状态 | 表示从可恢复检查点恢复执行的入口点，实际执行时需选择具体检查点 |

---

## 用户决策点（User Decision Points）

| 状态 | 触发者 | 含义 | 继续条件 |
|------|--------|------|----------|
| `[USER_DECISION]` | 任意 Skill → 用户 | 当前存在冲突/风险/分歧，需要用户做出决策 | 用户选择方案或给出新方案 |

决策记录要求：
- 当触发原因是"找不到来源或依赖"（例如 SOURCE_MISSING / DEPENDENCY_MISSING / CONFLICT）时，必须同时：
  - 使用 `ASK_USER_DECISION(topic, options)` 输出选项
  - 使用 `RECORD_DECISION(topic, decision)` 落盘决策记录文件
  - 在后续产物中引用该决策记录路径

### 用户决策退出路径

**标准退出路径**：
```
[USER_DECISION] → 用户选择继续 → 返回原状态继续执行
[USER_DECISION] → 用户选择回滚 → 返回上一阶段检查点
[USER_DECISION] → 用户选择终止 → 进入[已完成]（标记为终止）
```

**决策记录要求**：
每次用户决策必须：
1. 使用 `ASK_USER_DECISION(topic, options)` 输出选项
2. 使用 `RECORD_DECISION(topic, decision)` 落盘决策记录
3. 在后续产物中引用该决策记录路径

**决策记录模板**：
```markdown
# 决策记录

## 决策主题
{topic}

## 可选方案
{options}

## 用户选择
{selected_option}

## 决策时间
{timestamp}

## 影响范围
{affected_scope}

## 后续动作
{next_actions}
```

**决策类型与退出路径映射**：

| 决策类型 | 典型选项 | 退出路径 |
|----------|----------|----------|
| 门控失败 | 修复后重试 / 回滚 / 终止 | `[GATE_RETRY]` / `[GATE_ROLLBACK]` / `[已完成]` |
| 架构审查失败 | 修复架构 / 回滚到需求 / 终止 | `[ARCH_REPAIR]` / `[ARCH_ROLLBACK]` / `[已完成]` |
| 目录处理失败 | 重试 / 跳过 / 终止 | `[DIR_RETRY]` / `[DIR_SKIP]` / `[已完成]` |
| 熔断触发 | 重置并继续 / 人工介入 / 终止 | `[FUSION_RESET]` / `[MANUAL_INTERVENTION]` / `[已完成]` |
| 循环依赖 | 打破循环 / 人工介入 / 终止 | `[BREAK_CYCLE]` / `[MANUAL_INTERVENTION]` / `[已完成]` |
| 路径选择冲突 | 选择路径A / 选择路径B / 终止 | 对应路径入口 / `[已完成]` |
| 来源缺失 | 补充信息 / 跳过 / 终止 | 继续执行 / 跳过当前步骤 / `[已完成]` |

---

## 熔断状态（Fusion State）

| 状态 | 触发者 | 含义 | 恢复条件 |
|------|--------|------|----------|
| `[FUSION_TRIGGERED]` | sop-progress-supervisor | 连续失败触发熔断，停止执行 | 用户决策 + 方案调整 + 重置计数器 |

---

## Skills加载状态（Skill Loading States）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[SKILL_LOADING]` | LOAD_SKILL命令 | 正在加载Skill及资源 | 资源加载成功→`[SKILL_LOADED]` |
| `[SKILL_LOADED]` | 资源加载完成 | Skill加载完成 | 继续执行 |
| `[SKILL_UNLOADING]` | UNLOAD_SKILL命令 | 正在卸载Skill | 卸载完成→继续执行 |
| `[CONTEXT_WARNING]` | 上下文监控 | 上下文接近阈值 | token使用量超过80%→提示用户 |

**Skills分层加载规则**：
- **Tier 1（核心）**：始终加载，包括 sop-workflow-orchestrator, sop-code-explorer, sop-progress-supervisor, sop-code-review
- **Tier 2（阶段）**：按需加载，根据当前状态自动触发
- **Tier 3（增强）**：手动加载，通过 LOAD_SKILL 命令显式调用

---

## 极速路径状态（Micro Path States）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[DIRECT_EXECUTE]` | sop-code-implementation | 直接执行模式，跳过门控检查 | 执行完成后→`[AUTO_VERIFY]` |
| `[AUTO_VERIFY]` | sop-code-implementation | 自动验证中（lint+语法检查） | 验证通过→`[AUTO_SYNC]`，失败→`[DIR_WORKING]` |
| `[AUTO_SYNC]` | sop-document-sync | 自动同步中（仅更新索引） | 同步完成→`[已完成]` |

**极速路径适用条件**（全部满足）：
- 单文件修改
- 行数变化 ≤ 5行
- 仅涉及格式/注释/命名调整
- 无测试依赖
- 无文档依赖

---

## 轻量深度路径状态（Light Deep Path States）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[ROUTE_LIGHT_DEEP]` | sop-workflow-orchestrator | 路由到轻量深度路径 | 进入轻量需求分析 |
| `[LIGHT_REQUIREMENTS]` | sop-requirement-analyst | 轻量需求分析中 | 分析完成→`[WAITING_FOR_REQUIREMENTS]` |
| `[LIGHT_DESIGN]` | sop-implementation-designer | 轻量设计进行中 | 设计完成→`[WAITING_FOR_DESIGN]` |

**轻量深度路径适用条件**（任一满足）：
- 跨文件修改（但影响范围可界定，文件数1-3）
- 行数变化 30-100行
- 存在接口变更（但影响范围可界定）
- 存在逻辑变更（但可局部验证）

**轻量版特点**：
- 需求和设计阶段采用简化模板
- 不要求完整的文档产出
- 支持单轮快速确认
- 简化审查清单，聚焦变更区域

---

## 循环检测状态（Cycle Detection States）

| 状态 | 触发者 | 含义 | 处理方式 |
|------|--------|------|----------|
| `[CYCLE_DETECTED]` | sop-progress-supervisor | 检测到目录依赖循环 | 进入 `[USER_DECISION]` 并报告循环路径 |

---

## 迭代监控（Iteration Monitoring）

| 阈值 | 说明 | 触发动作 |
|------|------|----------|
| 迭代次数 ≤ 3 | 正常范围 | 继续执行 |
| 迭代次数 = 4 | 警告阈值 | 输出警告，建议检查收敛性 |
| 迭代次数 ≥ 5 | 熔断阈值 | 进入 `[USER_DECISION]`，提供收敛建议 |

**迭代计数规则**：
- 同一状态的转移计为一次迭代
- 用户决策后重置迭代计数器
- CMD: `ITERATION_COUNT(state)` 查询当前迭代次数
- CMD: `ITERATION_RESET(state)` 重置迭代计数器

---

## 指标采集状态（Metrics Collection States）

| 状态 | 触发者 | 含义 | 后续动作 |
|------|--------|------|----------|
| `[METRICS_COLLECTING]` | sop-progress-supervisor | 正在采集指标数据 | 采集完成→存储 |
| `[ALERT_TRIGGERED]` | sop-progress-supervisor | 告警触发 | 根据告警级别处理 |
| `[HEALTH_REPORT_READY]` | sop-progress-supervisor | 健康报告已生成 | 等待用户查看 |

**指标采集规则**：
- 状态转移时自动记录timestamp
- 关键事件（路径升级、审查失败等）记录事件日志
- 定期生成健康报告
- 告警触发时自动通知

**指标卡片模板**：
```markdown
# .trae/metrics/skill_execution_time.md
---
metric: "skill_execution_time"
skill: "sop-code-implementation"
period: "weekly"
updated: "2026-02-25"
---

## 本周统计
- 平均执行时长: 45分钟
- 中位数: 38分钟
- P95: 72分钟
- 样本数: 23

## 趋势
📈 较上周 +12%（因复杂需求增加）

## 建议
关注执行时长超过P95的案例，分析瓶颈
```

---

## 任务状态（Task States）

| 状态 | 标记 | 触发者 | 含义 | 继续条件 |
|------|------|--------|------|----------|
| `[TASK_PENDING]` | `[ ]` | sop-implementation-designer | 任务待处理 | 任务开始 |
| `[TASK_IN_PROGRESS]` | `[-]` | sop-code-implementation | 任务执行中 | 任务完成或阻塞 |
| `[TASK_COMPLETED]` | `[x]` | sop-code-implementation | 任务已完成 | 自动进入下一任务或归档 |
| `[TASK_BLOCKED]` | `[!]` | sop-code-implementation | 任务被阻塞 | 依赖解决后手动解除 |
| `[TASK_ARCHIVED]` | `[archived]` | sop-document-sync | 任务已归档 | - |

### 任务状态转移

```
[TASK_PENDING] → TASK_START() → [TASK_IN_PROGRESS]
[TASK_IN_PROGRESS] → TASK_COMPLETE() → [TASK_COMPLETED]
[TASK_IN_PROGRESS] → TASK_BLOCK() → [TASK_BLOCKED]
[TASK_BLOCKED] → TASK_UNBLOCK() → [TASK_IN_PROGRESS]
[TASK_COMPLETED] → TASK_ARCHIVE() → [TASK_ARCHIVED]
```

---

## 全局终态（Global Terminal State）

| 状态 | 触发者 | 含义 | 备注 |
|------|--------|------|------|
| `[已完成]` | sop-document-sync / sop-progress-supervisor | 全流程收尾完成 | 用于对用户声明任务结束（非目录级别）；不作为再执行起点 |

**排除说明**：`[已完成]` 状态**不作为可恢复检查点**，不可用于 continuation_request 的恢复起点。若需重新执行，必须从需求阶段开始新流程。

---

## 可恢复检查点（Recoverable Checkpoints）

以下状态可作为“再执行”的起点；再执行前须具备对应最小输入/落盘物，并由 continuation_request 声明（模板：04_reference/interaction_formats/continuation_request.md）。

| 检查点状态 | 再执行所需最小输入/落盘物 | 建议下一步 Skill |
|------------|---------------------------|------------------|
| `[WAITING_FOR_REQUIREMENTS]` | PRD/MRD/FRD 草稿路径、用户确认结论或决策记录 | sop-requirement-analyst（修订）或进入下一阶段 |
| `[WAITING_FOR_ARCHITECTURE]` | PRD/MRD 路径、架构设计草稿路径、用户确认结论 | sop-architecture-design（修订）或 sop-architecture-reviewer |
| `[ARCHITECTURE_PASSED]` | L2 架构文档路径（已审查通过） | sop-implementation-designer |
| `[WAITING_FOR_DESIGN]` | L2 架构路径、各目录 design.md 路径、用户确认结论 | sop-implementation-designer（修订）或 sop-code-explorer + sop-progress-supervisor |
| `[SCHEDULING]` | design_list（path + depth）、dir_map 草稿 | sop-progress-supervisor |
| `[PARALLEL_EXECUTING]` / `[WAITING_DEPENDENCY]` | dir_map、temp/scheduler_state.md、各目录状态 | sop-progress-supervisor |
| `[DIR_WAITING_DEP]` | 当前目录 scope、依赖目录列表、依赖目录完成情况 | sop-progress-supervisor 唤醒后 sop-code-implementation |
| `[WAITING_FOR_CODE_REVIEW]` | Diff、design/验收依据路径、当前目录 scope | sop-code-review |
| `[DIR_COMPLETED]`（单目录） | dir_map、已完成目录列表、剩余目录 | sop-progress-supervisor 调度下一批或 sop-code-review / sop-document-sync |
| `[WAITING_FOR_TEST_DESIGN]` | 测试设计 CSV/文档路径、用户确认结论 | sop-test-design-csv（修订）或 sop-test-implementation |
| `[WAITING_FOR_TEST_IMPLEMENTATION]` | CSV 路径、测试代码路径、审查结论 | sop-code-review（测试代码审查）或 sop-code-implementation |
| `[WAITING_FOR_L1_REVIEW]` / `[WAITING_FOR_L2_REVIEW]` / `[WAITING_FOR_L3_REVIEW]` / `[WAITING_FOR_L4_REVIEW]` | 对应层级验收结果、design/验收依据路径 | sop-code-review（REVIEW_ACCEPTANCE） |
| `[USER_DECISION]` / `[FUSION_TRIGGERED]` | 决策记录路径、方案调整说明、重置计数器 | 选择：重新分诊（ROUTE）或从本表上述某一检查点续跑 |

说明：从 `[USER_DECISION]` / `[FUSION_TRIGGERED]` 续跑时，须在 continuation_request 中写明"建议下一步"对应的检查点及上表所列最小输入。

---

## SOP 术语表（Terminology）

### 设计相关术语

| 标准名称 | 历史别名 | 使用场景 |
|----------|----------|----------|
| 设计依据 | 设计参考、设计引用 | FRD/设计文档 |
| 设计决策 | 设计选择 | 设计文档 |
| 设计约束 | 约束条件 | 全局 |
| 实现设计 | L3设计、详细设计 | 目录级设计文档 |
| 架构设计 | L2设计、技术设计 | 项目级设计文档 |

### 流程相关术语

| 标准名称 | 历史别名 | 使用场景 |
|----------|----------|----------|
| 代码审查 | Code Review、CR | 全局 |
| 验收审查 | 交付审查、Acceptance Review | L1-L4 |
| 门控检查 | Gate Check、质量门控 | 各阶段 |
| 熔断 | Fuse、Circuit Breaker | 异常处理 |
| 三错即停 | 3-Strike Rule | 失败处理 |

### 状态相关术语

| 标准名称 | 历史别名 | 说明 |
|----------|----------|------|
| `[USER_DECISION]` | `[USER_DECISION_REQUIRED]` | 用户决策点 |
| `[WAITING_FOR_ACCEPTANCE_REVIEW]` | `[WAITING_FOR_Lx_REVIEW]` | 统一验收审查状态 |
| `[WAITING_FOR_REVIEW]` | `[WAITING_FOR_CODE_REVIEW]` / `[WAITING_FOR_TEST_IMPLEMENTATION]` | 统一审查状态 |
| `[已完成]` | `[COMPLETED]` | 全局终态 |

### 路径相关术语

| 标准名称 | 别名 | 适用场景 |
|----------|------|----------|
| 极速路径 | Micro Path | ≤5行变更 |
| 快速路径 | Fast Path | <30行单文件变更 |
| 轻量深度路径 | Light Deep Path | 30-100行跨文件变更 |
| 标准深度路径 | Deep Path | >100行复杂变更 |

### 引用路径规范

- **内部引用**：使用基于 `SOP_ROOT` 的相对路径
  - 正确：`05_constraints/state_dictionary.md`
  - 错误：`./state_dictionary.md` 或 `../constraints/state_dictionary.md`

- **跨模块引用**：使用完整路径
  - 正确：`sop/05_constraints/command_dictionary.md`
  - 错误：`command_dictionary.md`（缺少上下文）
