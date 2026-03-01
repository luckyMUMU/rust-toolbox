# 文档归档与重建 Spec

## Why
现有文档分散在多个位置，需要按照 SOP 规范进行归档整理，并根据代码实现现状重建完整的文档体系。

## What Changes
- 将 `docs/` 目录下的所有文档归档到 `.temp/`
- 将 `src/` 目录下所有的 `design.md` 文档归档到 `.temp/`
- 将根目录的 `design.md` 归档到 `.temp/`
- 提取决策记录(ADR)和核心理念到 `.temp/ADR/`
- 根据 SOP 和代码现状创建完整文档体系
- 基于 ADR 优化文档并请求用户决策

## Impact
- Affected specs: 文档体系结构
- Affected code: 无代码变更，仅文档变更

## ADDED Requirements

### Requirement: 文档归档
系统 SHALL 将现有文档按类别归档到 `.temp/` 目录，保留原有目录结构。

#### Scenario: 归档成功
- **WHEN** 执行归档操作
- **THEN** 所有 `docs/` 下的文档被移动到 `.temp/docs-archive/`
- **AND** 原有目录结构被保留

### Requirement: design.md 归档
系统 SHALL 将所有 `design.md` 文档归档到 `.temp/` 目录。

#### Scenario: design.md 归档成功
- **WHEN** 执行归档操作
- **THEN** 根目录 `design.md` 被移动到 `.temp/design-archive/`
- **AND** `src/` 下所有 `design.md` 被移动到 `.temp/design-archive/src/`
- **AND** 原有目录结构被保留

### Requirement: ADR 提取
系统 SHALL 从现有文档中提取决策记录和核心理念。

#### Scenario: ADR 提取成功
- **WHEN** 分析现有文档
- **THEN** ADR 文档被复制到 `.temp/ADR/`
- **AND** 核心设计决策被整理记录

### Requirement: 文档重建
系统 SHALL 根据 SOP 规范和代码现状创建完整文档体系。

#### Scenario: 文档创建成功
- **WHEN** 分析代码实现
- **THEN** 按照 SOP 创建各类规范文档
- **AND** 文档符合 P0-P3 约束

### Requirement: 用户决策确认
系统 SHALL 在关键决策点请求用户确认。

#### Scenario: 决策确认
- **WHEN** 文档优化涉及重要变更
- **THEN** 向用户展示选项并请求确认
