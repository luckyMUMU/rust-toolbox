# 设计文档全面审查 Spec

## Why
设计文档是项目开发的核心指导，需要确保其完整性、准确性、一致性和规范性。当前项目文档存在多处不一致、缺失和过时的问题，影响开发效率和代码质量。

## What Changes
- 审查所有 design.md 文档的完整性和准确性
- 审查架构设计文档的一致性
- 审查接口设计文档的规范性
- 审查数据模型设计文档的技术可行性
- 生成详细的审查报告，列出问题和改进建议

## Impact
- Affected specs: 所有 design.md 文件、docs/ 目录下的技术文档
- Affected code: 无代码变更，仅文档审查

## ADDED Requirements

### Requirement: 文档完整性审查
系统 SHALL 确保所有设计文档包含必要的章节和内容。

#### Scenario: design.md 必要章节检查
- **WHEN** 审查 design.md 文件
- **THEN** 必须包含以下章节：
  - 核心定义 (Stable)
  - 待实现方案 (In Progress)
  - 状态记录
  - 版本信息（可选但推荐）

#### Scenario: 技术规格文档必要内容检查
- **WHEN** 审查技术规格文档
- **THEN** 必须包含：
  - 版本号和更新日期
  - 接口定义（含前置/后置条件）
  - 错误码定义
  - 性能约束

### Requirement: 文档准确性审查
系统 SHALL 确保文档描述与代码实现一致。

#### Scenario: 接口签名一致性检查
- **WHEN** 审查接口文档
- **THEN** 文档中的接口签名必须与实际代码实现一致

#### Scenario: 数据结构一致性检查
- **WHEN** 审查数据模型文档
- **THEN** 文档中的字段定义必须与代码中的结构体一致

### Requirement: 文档一致性审查
系统 SHALL 确保跨文档的描述一致。

#### Scenario: 跨文档术语一致性
- **WHEN** 同一概念在多个文档中出现
- **THEN** 使用相同的术语和定义

#### Scenario: 架构层次一致性
- **WHEN** 描述模块所属架构层
- **THEN** 各文档描述必须一致（如 workflow 属于基础设施层）

### Requirement: 文档规范性审查
系统 SHALL 确保文档符合项目既定的设计标准和规范。

#### Scenario: 版本信息规范
- **WHEN** 文档包含版本信息
- **THEN** 格式应为 `vX.Y.Z`，并包含更新日期

#### Scenario: 状态标记规范
- **WHEN** 文档描述任务或功能状态
- **THEN** 使用 `[进行中]` / `[已完成]` / `[待开始]` 标记

### Requirement: 技术可行性审查
系统 SHALL 确保文档描述的技术方案可行。

#### Scenario: 依赖可行性检查
- **WHEN** 文档引用外部依赖
- **THEN** 依赖必须存在且版本可用

#### Scenario: 架构决策可行性检查
- **WHEN** 文档描述架构决策
- **THEN** 决策必须有明确的理由和风险评估

## MODIFIED Requirements
无

## REMOVED Requirements
无

---

## 审查范围

### 1. 架构设计文档
| 文档路径 | 审查重点 |
|---------|---------|
| `design.md` (根目录) | 整体架构、分层定义、模块依赖 |
| `src/domain/design.md` | 领域模型、端口定义、事件系统 |
| `src/application/design.md` | 用例定义、服务编排、事务管理 |
| `src/infrastructure/design.md` | 存储实现、插件运行时、缓存策略 |
| `src/interfaces/design.md` | CLI/TUI/MCP 接口定义 |

### 2. 功能模块设计文档
| 文档路径 | 审查重点 |
|---------|---------|
| `src/workflow/design.md` | 引擎架构、调度算法、检查点机制 |
| `src/workflow/component/design.md` | 组件系统、Component trait |
| `src/workflow/executor/design.md` | 执行器链、中间件模式 |
| `src/workflow/context/design.md` | 数据上下文、数据传递 |
| `src/workflow/state/design.md` | 状态管理、执行追踪 |
| `src/tools/design.md` | 工具系统、Enum 架构 |
| `src/plugins/design.md` | 插件系统、多语言支持 |
| `src/storage/design.md` | 存储后端、备份恢复 |
| `src/performance/design.md` | 性能监控、缓存策略 |

### 3. 接口设计文档
| 文档路径 | 审查重点 |
|---------|---------|
| `docs/03_technical_spec/interfaces.md` | 接口契约、错误码定义 |
| `docs/03_technical_spec/api/CLI_REFERENCE.md` | CLI 命令定义 |
| `docs/03_technical_spec/api/RUST_SDK_REFERENCE.md` | SDK API 定义 |

### 4. 架构决策文档
| 文档路径 | 审查重点 |
|---------|---------|
| `docs/04_context_reference/architecture_decision.md` | ADR 完整性、决策理由 |
| `docs/04_context_reference/adr/*.md` | 各 ADR 详细内容 |

---

## 已发现的问题清单

### 高优先级问题

#### P1-001: WorkflowEngine trait 定义不一致
- **位置**: `src/workflow/design.md` vs `src/application/design.md` vs `docs/03_technical_spec/interfaces.md`
- **描述**: 三处文档对 WorkflowEngine 的方法签名描述不一致
  - workflow/design.md: `execute(definition, params)`
  - application/design.md: `start/pause/resume/stop/get_status`
  - interfaces.md: `execute(definition, initial_params)`
- **影响**: 开发人员可能使用错误的接口

#### P1-002: NodeType 枚举定义不一致
- **位置**: `src/workflow/design.md` vs `src/workflow/component/design.md`
- **描述**: NodeType 枚举值在不同文档中不一致
  - workflow/design.md: `Tool/Parallel/Condition/SubWorkflow`
  - component/design.md: `Tool/Parallel/Condition/Loop`
- **影响**: 组件实现可能使用错误的节点类型

#### P1-003: WorkflowNode 结构定义不一致
- **位置**: `src/workflow/design.md` vs `docs/03_technical_spec/interfaces.md`
- **描述**: WorkflowNode 字段定义不一致
  - workflow/design.md: `id, node_type, config`
  - interfaces.md: `id, name, node_type, component, config`
- **影响**: 数据结构实现可能出错

### 中优先级问题

#### P2-001: 缺少版本信息
- **位置**: 多个 design.md 文件
- **描述**: 部分文档缺少版本号和更新日期
- **影响**: 无法追踪文档变更历史

#### P2-002: ADR 缺少决策日期
- **位置**: `docs/04_context_reference/architecture_decision.md`
- **描述**: 部分 ADR 缺少决策日期和决策者信息
- **影响**: 无法追溯决策背景

#### P2-003: WASM 插件状态不明确
- **位置**: `src/plugins/design.md`
- **描述**: WASM 被标记为"暂时禁用"，但仍列在 PluginType 中
- **影响**: 用户可能误以为 WASM 可用

### 低优先级问题

#### P3-001: 时间格式不统一
- **位置**: 多个文档
- **描述**: 部分使用 `2026-02-07`，部分使用 `2026年2月7日`
- **影响**: 文档风格不统一

#### P3-002: 图表说明缺失
- **位置**: 部分架构图
- **描述**: 架构图缺少文字说明
- **影响**: 新成员理解困难

---

## 审查方法

### 1. 自动化检查
- 使用脚本检查文档必要章节是否存在
- 检查版本号格式是否正确
- 检查链接是否有效

### 2. 人工审查
- 对比代码实现验证文档准确性
- 跨文档对比验证一致性
- 评估技术方案可行性

### 3. 代码交叉验证
- 读取实际代码文件
- 对比文档描述与代码实现
- 标记不一致之处
