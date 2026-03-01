# Workflow Toolkit 文档中心

> **版本**: v1.0.0  
> **最后更新**: 2026-03-01  
> **状态**: Active

---

## 项目概述

Workflow Toolkit 是一个**工具能力编排与复用平台**，通过对工具的工作流编排实现固定流程的复用，支持 AGENT 动态扩展能力构建复杂功能，同时提供 CLI/TUI 接口让人类用户便捷调用并监测运行状况。

---

## 文档架构

本文档体系遵循 AGENT_SOP.md 规范，采用分层结构组织：

```
docs/
├── INDEX.md                        # 本文档（唯一入口）
├── specs/                          # P1/P2 级规范文档
│   ├── system-spec.md              # 系统规范
│   └── api-contract.md             # API 契约
├── design/                         # 设计文档
│   ├── concept-overview.md         # 核心概念
│   ├── logical-workflow/           # 逻辑工作流
│   ├── architecture/               # 架构设计
│   ├── adr/                        # 架构决策记录
│   └── modules/                    # 模块设计
├── 01_constitution/                # P0 级工程宪章
│   ├── project-charter.md          # 项目章程
│   ├── quality-redlines.md         # 质量红线
│   ├── architecture-principles.md  # 架构原则
│   └── security-baseline.md        # 安全基线
├── 05_constraints/                 # P3 级约束
│   └── p3-constraints.md           # 编码规范
├── contracts/                      # 契约文件
└── 参考/                           # SOP 规范（保护目录）
```

---

## 快速导航

### 按角色查找

#### 新加入开发者
1. [核心概念](./design/concept-overview.md) - 了解项目价值
2. [项目章程](./01_constitution/project-charter.md) - 了解项目目标
3. [架构原则](./01_constitution/architecture-principles.md) - 理解架构设计
4. [系统规范](./specs/system-spec.md) - 详细功能需求

#### 架构师
1. [架构决策](./design/architecture/architecture-decision.md) - 技术决策记录
2. [架构原则](./01_constitution/architecture-principles.md) - 设计原则
3. [模块设计](./design/modules/) - 各模块详细设计
4. [ADR 目录](./design/adr/) - 具体架构决策

#### 开发人员
1. [质量红线](./01_constitution/quality-redlines.md) - 不可违背的约束
2. [P3 约束](./05_constraints/p3-constraints.md) - 编码规范
3. [API 契约](./specs/api-contract.md) - 接口定义
4. [模块设计](./design/modules/) - 实现细节

#### 测试人员
1. [系统规范](./specs/system-spec.md) - 功能需求
2. [质量红线](./01_constitution/quality-redlines.md) - 质量要求
3. [API 契约](./specs/api-contract.md) - 接口测试依据

#### 产品经理
1. [项目章程](./01_constitution/project-charter.md) - 项目愿景
2. [系统规范](./specs/system-spec.md) - 功能详情
3. [核心概念](./design/concept-overview.md) - 产品定位

---

## 文档分层说明

### P0 级：工程宪章 (01_constitution/)

**不可违背**的顶层规范，违反即熔断：

- [项目章程](./01_constitution/project-charter.md) - 项目愿景、目标、范围
- [质量红线](./01_constitution/quality-redlines.md) - 质量底线要求
- [架构原则](./01_constitution/architecture-principles.md) - 架构设计原则
- [安全基线](./01_constitution/security-baseline.md) - 安全最低标准

### P1/P2 级：规范文档 (specs/)

跨模块/单模块的规范要求：

- [系统规范](./specs/system-spec.md) - 完整功能需求（原 PRD）
- [API 契约](./specs/api-contract.md) - 接口定义与数据模型

### 设计文档 (design/)

实现层面的设计说明：

- [核心概念](./design/concept-overview.md) - 项目概述与术语
- [逻辑工作流](./design/logical-workflow/) - 业务流程伪代码
- [架构设计](./design/architecture/) - 整体架构设计
- [架构决策记录](./design/adr/) - 具体技术决策
- [模块设计](./design/modules/) - 各模块详细设计

### P3 级：约束定义 (05_constraints/)

自动化工具验证的实现规范：

- [P3 约束](./05_constraints/p3-constraints.md) - 编码、测试、文档规范

---

## 核心文档索引

### 需求与设计
- [系统规范](./specs/system-spec.md) - 完整功能需求
- [核心概念](./design/concept-overview.md) - 项目概述
- [逻辑工作流](./design/logical-workflow/) - 业务流程
- [架构决策](./design/architecture/architecture-decision.md) - 技术决策

### 接口与实现
- [API 契约](./specs/api-contract.md) - 接口定义
- [模块设计](./design/modules/) - 实现细节
  - [应用层](./design/modules/application-design.md)
  - [领域层](./design/modules/domain-design.md)
  - [基础设施层](./design/modules/infrastructure-design.md)
  - [接口层](./design/modules/interfaces-design.md)
  - [工作流引擎](./design/modules/workflow-design.md)
  - [插件系统](./design/modules/plugins-design.md)
  - [工具系统](./design/modules/tools-design.md)
  - [存储系统](./design/modules/storage-design.md)
  - [性能优化](./design/modules/performance-design.md)

### 规范与约束
- [项目章程](./01_constitution/project-charter.md)
- [质量红线](./01_constitution/quality-redlines.md)
- [架构原则](./01_constitution/architecture-principles.md)
- [安全基线](./01_constitution/security-baseline.md)
- [P3 约束](./05_constraints/p3-constraints.md)

---

## 文档维护

### 文档更新流程

1. 修改文档前确认文档层级（P0-P3）
2. P0/P1 级文档修改需技术负责人审批
3. 更新后验证链接有效性
4. 更新本文档索引（如需要）

### 文档版本控制

- 所有文档包含版本号和最后更新日期
- 重大变更需更新版本号
- 废弃文档移至归档目录

### 文档质量保证

- 定期审查文档准确性
- 文档与代码保持一致
- 链接无死链

---

## 相关资源

- [SOP 规范](../sop/AGENT_SOP.md) - 工作流规范
- [参考目录](./参考/) - SOP 相关资料
- [设计文档](./design/root-design.md) - 根设计文档（原 design.md）

---

## 帮助与支持

如有问题，请：
1. 首先查阅相关文档
2. 查看架构决策记录了解背景
3. 联系技术团队

---

*最后更新：2026-03-01*
