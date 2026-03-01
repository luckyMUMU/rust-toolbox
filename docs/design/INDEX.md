# 设计文档索引

> **版本**: v1.0.0  
> **最后更新**: 2026-03-01  
> **状态**: Active

---

## 概述

本文档是设计文档的总入口，提供所有设计相关文档的导航。

---

## 设计文档结构

```
design/
├── INDEX.md                        # 本文档
├── concept-overview.md             # 核心概念
├── root-design.md                  # 根设计文档（原 design.md）
├── logical-workflow/               # 逻辑工作流
│   ├── workflow_execution.pseudo   # 工作流执行流程
│   ├── tool_execution.pseudo       # 工具执行流程
│   ├── plugin_loading.pseudo       # 插件加载流程
│   └── error_handling.pseudo       # 错误处理流程
├── architecture/                   # 架构设计
│   └── architecture-decision.md    # 架构决策总览
├── adr/                            # 架构决策记录
│   ├── adr_di_001_自建DI容器选择.md
│   ├── adr_workflow_002_采用JoinSet并行执行.md
│   ├── adr_plugin_003_WASM沙箱隔离策略.md
│   └── adr_tools_004_Schema验证策略.md
└── modules/                        # 模块设计
    ├── application-design.md       # 应用层设计
    ├── domain-design.md            # 领域层设计
    ├── infrastructure-design.md    # 基础设施层设计
    ├── interfaces-design.md        # 接口层设计
    ├── workflow-design.md          # 工作流引擎设计
    ├── plugins-design.md           # 插件系统设计
    ├── tools-design.md             # 工具系统设计
    ├── storage-design.md           # 存储系统设计
    ├── performance-design.md       # 性能优化设计
    ├── workflow-component-design.md    # 工作流组件设计
    ├── workflow-context-design.md      # 工作流上下文设计
    ├── workflow-executor-design.md     # 工作流执行器设计
    ├── workflow-state-design.md        # 工作流状态设计
    └── plugins-file-management-design.md # 文件管理插件设计
```

---

## 快速导航

### 理解系统架构
1. [核心概念](./concept-overview.md) - 项目概述与术语
2. [根设计文档](./root-design.md) - 整体架构设计
3. [架构决策](./architecture/architecture-decision.md) - 技术决策记录

### 了解业务流程
- [逻辑工作流](./logical-workflow/) - 业务流程伪代码描述

### 查看模块设计
- [模块设计目录](./modules/) - 各模块详细设计文档

### 查看技术决策
- [ADR 目录](./adr/) - 具体架构决策记录

---

## 设计文档分层

### L1: 概念层
- [核心概念](./concept-overview.md) - 项目价值、术语、场景

### L2: 逻辑层
- [逻辑工作流](./logical-workflow/) - 业务流程伪代码

### L3: 架构层
- [根设计文档](./root-design.md) - 整体架构
- [架构决策](./architecture/architecture-decision.md) - 技术决策

### L4: 模块层
- [模块设计](./modules/) - 各模块详细设计

---

## 相关文档

- [文档中心](../INDEX.md) - 返回主索引
- [系统规范](../specs/system-spec.md) - 功能需求
- [API 契约](../specs/api-contract.md) - 接口定义
- [P0 宪章](../01_constitution/) - 工程宪章

---

*最后更新：2026-03-01*
