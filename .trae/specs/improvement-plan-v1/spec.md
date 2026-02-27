# 全面改进计划 Spec

## Why

当前代码库在完成基础功能后，存在以下需要系统性改进的问题：

1. **功能缺口**：30+ 处 TODO 标记待实现，核心功能未完成（Python/Node.js/Docker/WASM 执行器、Switch/Loop 控制流）
2. **架构冗余**：`adapter/` 和 `interfaces/` 目录职责重叠，空模块占用项目结构
3. **代码质量**：虽有改进但仍有 unwrap/expect 调用残留
4. **安全增强**：native 插件签名验证、运行时安全配置待完善

本改进计划旨在对齐 PRD 定义的功能需求，填补功能缺口，优化架构，为后续迭代奠定坚实基础。

## What Changes

### 功能实现（对齐 PRD）

- 实现 `tools/types.rs` 中的 Python/Node.js/Docker/WASM 执行器
- 实现 `workflow/engine.rs` 中的 Switch/Loop 控制流
- 实现 `workflow/engine.rs` 中的工作流停止功能

### 架构整理

- 评估 `adapter/` 模块迁移方案，消除 `adapter/` 和 `interfaces/` 职责重叠
- 合并 `application/port/` 和 `domain/port/` 重复端口定义
- 统一 metrics 收集到 `performance/` 模块

### 安全增强

- 设计并实现 native 插件签名验证机制
- 强化生产环境 JWT 密钥配置要求
- 完善 WASM 沙箱安全级别配置

## Impact

- Affected specs: 工作流引擎(FR-W001-W012)、插件系统(FR-P001-P015)、工具系统(FR-T001-T006)
- Affected code:
  - `src/tools/types.rs` - 多种执行器实现
  - `src/workflow/engine.rs` - 控制流实现
  - `src/adapter/` - 架构整理
  - `src/plugins/` - 安全增强
  - `src/performance/` - metrics 统一

## ADDED Requirements

### Requirement: 工具执行器完整性

系统 SHALL 支持以下工具执行器类型：
- Native (Rust) 执行器
- Python 脚本执行器
- Node.js 执行器
- Docker 容器执行器
- WASM 执行器

#### Scenario: Python 工具执行
- **WHEN** 用户执行 Python 编写的工具
- **THEN** 系统调用 Python 解释器执行脚本，返回执行结果

### Requirement: 工作流控制流

系统 SHALL 支持以下控制流：
- 顺序执行
- 并行执行（已实现）
- 条件分支 (Switch)
- 循环 (Loop)

#### Scenario: 条件分支执行
- **WHEN** 工作流包含 Switch 节点
- **THEN** 根据条件表达式选择对应分支执行

### Requirement: 架构一致性

系统 SHALL 保持清晰的模块边界：
- 无空模块占位
- 无职责重叠
- 端口接口统一管理

## MODIFIED Requirements

### Requirement: 架构分层优化

原有：多个层次存在端口定义
修改为：端口接口统一放置在 `domain/port/`，其他层通过依赖注入使用

## REMOVED Requirements

### Requirement: 空模块占位
**Reason**: 空模块增加维护负担且无实际功能
**Migration**: 实现功能或删除模块

---

## 决策记录

### ADR-IMP-001: 改进计划执行策略
- **决策**: 采用渐进式改进，优先实现 PRD 中 P0 功能
- **理由**: 保持功能稳定性，避免大规模重构风险
- **风险**: 部分技术债务需要后续迭代清理
