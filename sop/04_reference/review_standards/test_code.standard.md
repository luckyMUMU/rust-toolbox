---
version: v2.4.0
updated: 2026-02-22
artifact: Test Code
---

# 测试代码审查标准

## Scope

- 适用产物：测试代码 Diff（单测/集成/验收/E2E），由 `sop-test-implementation` 产出
- 审查者：`sop-code-review`

## SSOT

- 测试设计与权限隔离：`02_skill_matrix/index.md`、`prompts/packs/default/skills/sop-test-implementation.md`
- 分层验收标准：`05_constraints/acceptance_criteria.md`
- 状态/命令：`05_constraints/state_dictionary.md`、`05_constraints/command_dictionary.md`

## 严重等级与通过规则

- 🔴：与测试设计不一致/不可运行/不稳定/引入安全问题或泄露数据，必须修复
- 🟡：可读性差/维护成本高/过度耦合实现细节，建议修复
- 🟢：优化建议，可选采纳
- 通过门槛：🔴=0 且“必检项”全部满足

## 必检项（Hard requirements）

- 设计一致性：测试场景覆盖与测试设计/CSV 一致；缺失必须说明并回流到 `sop-test-design-csv`
- 可运行与稳定：可重复运行、无随机性、不依赖外部不稳定资源（除非明确声明并隔离）
- 断言质量：断言业务结果而非实现细节；避免脆弱断言
- 隔离性：Mock/Stub 边界合理，不污染全局状态，不产生跨用例耦合
- 数据与隐私：测试数据不包含真实敏感信息，不输出密钥或隐私

## 推荐项（Soft requirements）

- 命名与结构：用例命名可追溯到 TC_ID/场景，结构清晰
- 运行成本：避免不必要的慢测试；必要时标注分组与执行策略
- 失败可诊断：失败时输出足够上下文，便于定位

## 证据要求（Evidence）

- 对“已覆盖关键场景”的声明必须给出用例映射（列表或矩阵）
- 若无法在当前环境执行测试，必须给出可复制的命令与预期结果，并记录为 🟡 或 🔴（按项目要求）

## 项目可配置项（Project knobs）

- 允许的外部依赖类型（默认：必须隔离）
- 必须达到的覆盖/层级门槛（按项目风险与工具链调整）
- 测试运行超时与分组策略（按 CI/本地约束调整）
