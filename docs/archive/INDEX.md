# 文档索引

> **Workflow Toolkit 完整导航指南**  
> **版本**: v0.1.0  
> *最后更新：2026-02-06*

---

## 文档架构

本文档采用四级分层架构，遵循 [document_llm_GUIDE.md](./参考/document_llm_GUIDE.md) 规范：

```
docs/
├── 01_concept_overview.md    # L1: 核心概念层（价值与痛点）
├── 02_logical_workflow/      # L2: 逻辑流转层（伪代码）
├── 03_technical_spec/        # L3: 技术规格层（接口契约）
├── 04_context_reference/     # L4: 决策参考层（ADR）
├── api/                      # API 参考
├── dev/                      # 开发者资源
├── guides/                   # 用户指南
└── plugins/                  # 插件文档
```

---

## 按层级导航

### L1: 核心概念层
理解项目价值定位与核心术语。

- [01_concept_overview.md](01_concept_overview.md) - 核心概念与价值主张

### L2: 逻辑流转层
使用结构化伪代码描述核心流程，与实现无关。

- [workflow_execution.pseudo](02_logical_workflow/workflow_execution.pseudo) - 工作流执行流程
- [tool_execution.pseudo](02_logical_workflow/tool_execution.pseudo) - 工具执行流程
- [plugin_loading.pseudo](02_logical_workflow/plugin_loading.pseudo) - 插件加载流程
- [error_handling.pseudo](02_logical_workflow/error_handling.pseudo) - 错误处理流程

### L3: 技术规格层
接口契约、数据模型、错误码定义。

- [interfaces.md](03_technical_spec/interfaces.md) - 接口契约规范
- [VERSION_MAPPING.md](VERSION_MAPPING.md) - 版本映射表

### L4: 决策参考层
架构决策记录、性能基准、迁移说明。

- [architecture_decision.md](04_context_reference/architecture_decision.md) - 架构决策记录 (ADR)

---

## 按主题导航

### API与参考
CLI和Rust SDK的技术规范。

- [CLI参考](api/CLI_REFERENCE.md) - 命令行接口文档
- [Rust SDK参考](api/RUST_SDK_REFERENCE.md) - 开发者核心库API
- [工具参考](TOOLS_REFERENCE.md) - 可用工具清单
- [工具设计标准](TOOL_DESIGN_STANDARDS.md) - 工具开发规范

### 开发者资源
面向开发者的技术文档。

- [PROJECT_OVERVIEW.md](dev/PROJECT_OVERVIEW.md) - 系统架构与设计总览
- [DEVELOPMENT_GUIDE.md](dev/DEVELOPMENT_GUIDE.md) - 贡献者指南
- [PLUGIN_DEVELOPMENT.md](dev/PLUGIN_DEVELOPMENT.md) - 插件开发指南
- [DOCUMENTATION_STANDARDS.md](dev/DOCUMENTATION_STANDARDS.md) - 文档编写规范

### 用户指南
面向最终用户的使用文档。

- [USER_GUIDE.md](guides/USER_GUIDE.md) - 全面的用户手册
- [CHEATSHEET.md](guides/CHEATSHEET.md) - 快速参考
- [TUTORIAL.md](guides/TUTORIAL.md) - 分步骤教程
- [TROUBLESHOOTING.md](guides/TROUBLESHOOTING.md) - 问题解决
- [MIGRATION_GUIDE.md](guides/MIGRATION_GUIDE.md) - 从旧版本迁移

### 专题文档
特定插件子系统的文档。

- [文件管理工具](plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md) - 全面的文件操作套件

---

## 按角色查找

### 新用户
1. 阅读 [01_concept_overview.md](01_concept_overview.md) 了解核心价值
2. 阅读 [工具设计标准](TOOL_DESIGN_STANDARDS.md) 了解基本概念
3. 查看 [CLI参考](api/CLI_REFERENCE.md) 学习命令使用

### 系统管理员
1. 查看 [CLI参考](api/CLI_REFERENCE.md) 用于自动化
2. 参考 [TROUBLESHOOTING.md](guides/TROUBLESHOOTING.md) 解决问题

### 开发者/贡献者
1. 阅读 [PROJECT_OVERVIEW.md](dev/PROJECT_OVERVIEW.md) 了解架构
2. 阅读 [Rust SDK参考](api/RUST_SDK_REFERENCE.md)
3. 查看 [architecture_decision.md](04_context_reference/architecture_decision.md) 了解设计决策

### 架构师
1. 阅读 [01_concept_overview.md](01_concept_overview.md) 了解价值定位
2. 阅读 [workflow_execution.pseudo](02_logical_workflow/workflow_execution.pseudo) 了解执行流程
3. 查看 [architecture_decision.md](04_context_reference/architecture_decision.md) 了解架构决策
4. 参考 [interfaces.md](03_technical_spec/interfaces.md) 了解接口契约

### 插件作者
1. 参考 [plugin_loading.pseudo](02_logical_workflow/plugin_loading.pseudo) 了解加载流程
2. 阅读 [PLUGIN_DEVELOPMENT.md](dev/PLUGIN_DEVELOPMENT.md) 了解插件开发
3. 阅读 [Rust SDK参考](api/RUST_SDK_REFERENCE.md) 了解traits
4. 查看 [文件管理工具文档](plugins/file_management/) 作为示例

---

## 模块级设计文档

各模块的详细设计文档位于 `src/` 目录下：

- [领域层](../src/domain/design.md) - 业务逻辑与领域模型
- [应用层](../src/application/design.md) - 用例编排与应用服务
- [基础设施层](../src/infrastructure/design.md) - 技术实现细节
- [接口层](../src/interfaces/design.md) - 用户交互接口
- [工作流引擎](../src/workflow/design.md) - 工作流定义、调度与执行
- [插件系统](../src/plugins/design.md) - 多类型插件管理
- [工具系统](../src/tools/design.md) - 工具注册与执行
- [存储系统](../src/storage/design.md) - 状态持久化与备份
- [性能优化](../src/performance/design.md) - 性能监控与优化

---

## 获取帮助

- **CLI：** 运行 `workflow-toolkit --help` 或特定命令帮助。
- **代码：** 运行 `cargo doc --open` 查看自动生成的源码文档。
- **版本信息：** 查看 [VERSION_MAPPING.md](VERSION_MAPPING.md)。

---

## 文档维护

- **版本**: v0.1.0
- **维护者**: Workflow Toolkit Team
- **更新频率**: 每个版本更新

### 文档规范

- L1-L4 分层遵循 [document_llm_GUIDE.md](./参考/document_llm_GUIDE.md) 规范
- 所有文档包含版本标记
- 代码引用使用相对路径

### 贡献文档

1. 确定文档层级（L1-L4）
2. 遵循对应层级的编写规范
3. 添加版本标记
4. 更新本文档索引
5. 更新 [VERSION_MAPPING.md](VERSION_MAPPING.md)
