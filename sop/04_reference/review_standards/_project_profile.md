---
version: v2.4.0
updated: 2026-02-22
type: Review Profile Template
---

# 项目审查 Profile（模板）

## 定位

- Profile 用于 **项目级调整** 审查标准的“可配置项（Project knobs）”
- Profile 不重写标准全文，不引入新的 SSOT

## 使用方式

- 建议路径：`sop/04_reference/review_standards/profiles/<project>.md`
- 审查报告中必须声明：
  - Standard：使用了哪些 `*.standard.md`
  - Profile：是否启用该项目 Profile
  - Overrides：使用了哪些覆写项

## Profile 元信息

- 项目：<project_name>
- 适用范围：<scope>
- 生效日期：YYYY-MM-DD

## 覆写项（Overwrites）

### 通用

- 审查轮次上限：3
- 允许豁免：否
- 豁免记录要求：若允许，必须包含风险、期限、回滚策略

### 代码 Diff（code_diff.standard.md）

- 强制测试层级：L1 + L2
- 安全审查强度：红线强制 + 高风险点额外检查
- 允许跨目录例外：否

### 测试设计（test_design.standard.md）

- 必须覆盖的场景集合：主流程 + 边界 + 异常 + 权限/安全
- 覆盖矩阵格式：表格（需求/场景/层级/用例ID/备注）

### 测试代码（test_code.standard.md）

- 允许外部依赖：否（如必须，需隔离并标记）
- 覆盖/层级门槛：与测试设计一致
- 超时门槛：<project_defined>

### 架构（architecture_design.standard.md）

- 必须覆盖维度：完整性/一致性/可行性/性能/安全/可扩展
- 强制 ADR 类型：关键依赖/核心接口/数据模型/安全边界

## 不适用项（裁剪）

- <列出本项目明确不适用的条目，并说明原因与替代机制>
