# Workflow Toolkit 设计文档

## 1. 核心定义 (Stable)

### 1.1 项目概述

Workflow Toolkit 是一个基于 Rust 的多接口工作流执行系统，支持 CLI、TUI 和 MCP Server 三种交互方式。采用 DDD 分层架构，实现高内聚低耦合的设计目标。

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────────────┐
│                      接口层 (Interfaces)                      │
│         CLI        TUI        MCP Server                     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    应用层 (Application)                       │
│         UseCase    Service    Workflow Orchestration         │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    领域层 (Domain)                            │
│         Model      Port (Repository/Service Interface)       │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                  基础设施层 (Infrastructure)                  │
│    Persistence   Plugin   Cache   External   Config          │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 核心领域模型

| 领域实体 | 职责 | 关键属性 |
|---------|------|---------|
| Workflow | 工作流定义 | id, name, nodes, edges, config |
| Execution | 执行记录 | id, workflow_id, status, context |
| Tool | 工具定义 | name, version, input/output schema |
| Plugin | 插件信息 | name, type, metadata |

### 1.4 模块依赖关系

```
interfaces → application → domain ← infrastructure
                ↓              ↑
            workflow ←→ tools ←→ plugins
                ↓              ↓
            storage ←→ performance
```

---

## 2. 子模块索引

### 核心模块
- [领域层](./src/domain/design.md) - 业务逻辑与领域模型
- [应用层](./src/application/design.md) - 用例编排与应用服务
- [基础设施层](./src/infrastructure/design.md) - 技术实现细节
- [接口层](./src/interfaces/design.md) - 用户交互接口

### 功能模块
- [工作流引擎](./src/workflow/design.md) - 工作流定义、调度与执行
- [插件系统](./src/plugins/design.md) - 多类型插件管理
- [工具系统](./src/tools/design.md) - 工具注册与执行
- [存储系统](./src/storage/design.md) - 状态持久化与备份
- [性能优化](./src/performance/design.md) - 性能监控与优化

---

## 3. 技术决策 (ADR)

### ADR-001: 分层架构选择
- **决策**: 采用 DDD 分层架构（Domain/Application/Infrastructure/Interfaces）
- **理由**: 清晰的职责分离，便于测试和维护
- **风险**: 初期开发成本较高

### ADR-002: 异步运行时
- **决策**: 使用 Tokio 作为异步运行时
- **理由**: 生态成熟，性能优秀，与 Rust 异步生态兼容
- **风险**: 学习曲线较陡

### ADR-003: 错误处理
- **决策**: 使用 thiserror + anyhow 组合
- **理由**: thiserror 用于库代码定义错误类型，anyhow 用于应用代码简化错误处理
- **风险**: 需要团队统一规范

### ADR-004: 插件系统架构
- **决策**: 支持多类型插件（Native/Python/Node.js/Docker）
- **理由**: 最大化扩展性，支持不同技术栈
- **风险**: 沙箱隔离和安全管理复杂度

### ADR-005: 工作流引擎架构
- **决策**: 采用 LiteFlow 风格的组件 + 执行器链架构
- **理由**: 更好的扩展性，支持 AOP 风格的横切关注点
- **风险**: 学习成本，需要理解组件和执行器链的概念

---

## 4. 状态记录

- `[已完成]` | 设计文档体系建立 | 2026-02-06
- `[已完成]` | 基础架构搭建 | 2026-01-20
- `[已完成]` | 工作流引擎核心实现 | 2026-02-01
- `[已完成]` | MCP Server 实现 | 2026-01-28
- `[已完成]` | EL 表达式引擎 | 2026-01-28
- `[进行中]` | 检查点机制完善 | 2026-02-06
- `[进行中]` | 事务管理实现 | 2026-02-06
- `[待开始]` | 工具原子化重构
- `[待开始]` | WASM 插件恢复
- `[待开始]` | LanceDB 深度集成

---

## 5. 快速导航

### 开发人员入口
1. 新功能开发 → 从 [应用层](./src/application/design.md) 开始
2. Bug 修复 → 根据错误类型定位到具体模块
3. 架构理解 → 阅读本文件和各模块 design.md

### 设计规范
- 所有模块必须包含 design.md
- 父级文档只保留摘要和链接
- 使用 `[进行中]` / `[已完成]` 标记状态

### 相关文档
- [项目文档索引](./docs/INDEX.md) - 完整文档导航
- [核心概念](./docs/01_concept_overview.md) - L1 层概念说明
- [产品设计](./PRODUCT_DESIGN.md) - 产品级设计文档
