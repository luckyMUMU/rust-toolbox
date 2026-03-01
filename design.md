# Workflow Toolkit 设计文档

> **版本**: v1.0  
> **最后更新**: 2026-03-01  
> **维护者**: Workflow Toolkit Team

---

## 重要说明

**本文档已迁移至 [docs/design/](./docs/design/) 目录**

- **新入口**: [docs/design/root-design.md](./docs/design/root-design.md) - 原文档内容
- **文档索引**: [docs/design/INDEX.md](./docs/design/INDEX.md) - 设计文档导航
- **主索引**: [docs/INDEX.md](./docs/INDEX.md) - 完整文档中心

---

## 快速导航

### 按角色查找

| 角色 | 推荐入口 | 说明 |
|------|----------|------|
| 新开发者 | [docs/design/modules/application-design.md](./docs/design/modules/application-design.md) | 理解用例和业务流程 |
| 架构师 | [docs/design/INDEX.md](./docs/design/INDEX.md) | 整体架构和技术决策 |
| 工具开发者 | [docs/design/modules/tools-design.md](./docs/design/modules/tools-design.md) | 工具开发和扩展 |
| 插件开发者 | [docs/design/modules/plugins-design.md](./docs/design/modules/plugins-design.md) | 插件开发和集成 |
| 运维人员 | [docs/design/modules/infrastructure-design.md](./docs/design/modules/infrastructure-design.md) | 部署和配置 |

---

## 文档结构

```
docs/design/
├── INDEX.md                        # 设计文档索引
├── concept-overview.md             # 核心概念
├── root-design.md                  # 根设计文档（本文档迁移后）
├── logical-workflow/               # 逻辑工作流
├── architecture/                   # 架构设计
├── adr/                            # 架构决策记录
└── modules/                        # 模块设计
    ├── application-design.md       # 应用层
    ├── domain-design.md            # 领域层
    ├── infrastructure-design.md    # 基础设施层
    ├── interfaces-design.md        # 接口层
    ├── workflow-design.md          # 工作流引擎
    ├── plugins-design.md           # 插件系统
    ├── tools-design.md             # 工具系统
    ├── storage-design.md           # 存储系统
    └── performance-design.md       # 性能优化
```

---

## 相关文档

- [项目章程](./docs/01_constitution/project-charter.md) - 项目愿景
- [系统规范](./docs/specs/system-spec.md) - 功能需求
- [API 契约](./docs/specs/api-contract.md) - 接口定义
- [质量红线](./docs/01_constitution/quality-redlines.md) - 质量要求
- [架构原则](./docs/01_constitution/architecture-principles.md) - 设计原则
- [文档中心](./docs/INDEX.md) - 完整文档导航
