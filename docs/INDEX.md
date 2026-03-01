# Workflow Toolkit 文档索引

## 文档元数据

- **版本**: v1.0.0
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 文档结构

```
docs/
├── 01_constitution/          # P0 级工程宪章
│   ├── project-charter.md    # 项目宪章
│   ├── architecture-principles.md # 架构原则
│   ├── quality-redlines.md   # 质量红线
│   └── security-baseline.md  # 安全基线
├── 02_specifications/        # P1-P2 级规范
│   ├── system-spec.md        # P1 系统规范
│   ├── domain-model.md       # P2 领域模型
│   ├── api-contract.md       # P2 API 契约
│   └── data-model.md         # P2 数据模型
├── 02_logical_workflow/      # 设计文档
│   ├── architecture-design.md # 架构设计
│   ├── adr-index.md          # ADR 索引
│   └── adr/                  # ADR 文档
│       ├── adr-di-001-di-container.md
│       ├── adr-workflow-002-joinset.md
│       ├── adr-plugin-003-wasm-sandbox.md
│       └── adr-tools-004-schema-validation.md
├── 03_guides/                # 指南文档
│   ├── module-design-guide.md # 模块设计指南
│   ├── plugin-development-guide.md # 插件开发指南
│   └── workflow-writing-guide.md # 工作流编写指南
└── INDEX.md                  # 本文件
```

## 2. 规范分层

### 2.1 P0 级 - 工程宪章

不可违背的约束，违反即熔断。

| 文档 | 描述 |
|------|------|
| [项目宪章](01_constitution/project-charter.md) | 项目目标、范围、约束 |
| [架构原则](01_constitution/architecture-principles.md) | 架构设计原则 |
| [质量红线](01_constitution/quality-redlines.md) | 质量标准和红线 |
| [安全基线](01_constitution/security-baseline.md) | 安全要求和基线 |

### 2.2 P1 级 - 系统规范

跨模块约束，技术负责人审批。

| 文档 | 描述 |
|------|------|
| [系统规范](02_specifications/system-spec.md) | 系统边界、模块职责、接口规范 |

### 2.3 P2 级 - 模块规范

单模块约束，模块负责人审批。

| 文档 | 描述 |
|------|------|
| [领域模型](02_specifications/domain-model.md) | 聚合、实体、值对象定义 |
| [API 契约](02_specifications/api-contract.md) | CLI、TUI、MCP API 定义 |
| [数据模型](02_specifications/data-model.md) | 数据存储格式和结构 |

### 2.4 设计文档

| 文档 | 描述 |
|------|------|
| [架构设计](02_logical_workflow/architecture-design.md) | 分层架构、核心模块设计 |
| [ADR 索引](02_logical_workflow/adr-index.md) | 架构决策记录索引 |

### 2.5 指南文档

| 文档 | 描述 |
|------|------|
| [模块设计指南](03_guides/module-design-guide.md) | 模块设计原则和规范 |
| [插件开发指南](03_guides/plugin-development-guide.md) | 插件开发完整指南 |
| [工作流编写指南](03_guides/workflow-writing-guide.md) | 工作流编写完整指南 |

## 3. 相关文档

### 3.1 SOP 文档

| 文档 | 描述 |
|------|------|
| [AGENT_SOP.md](../sop/AGENT_SOP.md) | SOP 入口文档 |
| [工作流阶段](../sop/03_workflow/) | 5 阶段工作流定义 |
| [约束定义](../sop/05_constraints/) | P0-P3 约束定义 |

### 3.2 ADR 文档

| 文档 | 描述 |
|------|------|
| [ADR-DI-001](02_logical_workflow/adr/adr-di-001-di-container.md) | 自建 DI 容器选择 |
| [ADR-Workflow-002](02_logical_workflow/adr/adr-workflow-002-joinset.md) | JoinSet 并行执行 |
| [ADR-Plugin-003](02_logical_workflow/adr/adr-plugin-003-wasm-sandbox.md) | WASM 沙箱隔离 |
| [ADR-Tools-004](02_logical_workflow/adr/adr-tools-004-schema-validation.md) | Schema 验证策略 |

## 4. 快速导航

### 4.1 按角色导航

**架构师**:
- [架构原则](01_constitution/architecture-principles.md)
- [架构设计](02_logical_workflow/architecture-design.md)
- [ADR 索引](02_logical_workflow/adr-index.md)

**开发者**:
- [系统规范](02_specifications/system-spec.md)
- [领域模型](02_specifications/domain-model.md)
- [API 契约](02_specifications/api-contract.md)
- [模块设计指南](03_guides/module-design-guide.md)
- [插件开发指南](03_guides/plugin-development-guide.md)

**测试工程师**:
- [质量红线](01_constitution/quality-redlines.md)
- [系统规范 - 测试要求](02_specifications/system-spec.md#10-测试要求)

**运维工程师**:
- [安全基线](01_constitution/security-baseline.md)
- [数据模型 - 备份](02_specifications/data-model.md#7-备份数据模型)

### 4.2 按主题导航

**工作流**:
- [系统规范 - 工作流模块](02_specifications/system-spec.md#35-工作流模块-srcworkflow)
- [领域模型 - 工作流聚合](02_specifications/domain-model.md#21-工作流聚合)
- [数据模型 - 工作流定义](02_specifications/data-model.md#2-工作流定义数据模型)
- [工作流编写指南](03_guides/workflow-writing-guide.md)

**插件**:
- [系统规范 - 插件模块](02_specifications/system-spec.md#36-插件模块-srcplugins)
- [领域模型 - 插件聚合](02_specifications/domain-model.md#23-插件聚合)
- [数据模型 - 插件数据](02_specifications/data-model.md#4-插件数据模型)
- [插件开发指南](03_guides/plugin-development-guide.md)

**工具**:
- [系统规范 - 工具模块](02_specifications/system-spec.md#38-工具模块-srctools)
- [领域模型 - 工具聚合](02_specifications/domain-model.md#24-工具聚合)
- [数据模型 - 工具数据](02_specifications/data-model.md#5-工具数据模型)

## 5. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
