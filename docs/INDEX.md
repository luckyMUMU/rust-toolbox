# 文档索引

> **Rust工作流工具包完整导航指南**  
> **版本**: v0.1.0  
> *最后更新：2026-02-05*

---

## 📚 文档架构

本文档采用四级分层架构，基于 [LLM 文档规范](参考/document_llm_GUIDE.md)：

```
docs/
├── 01_concept_overview/      # L1: 核心概念层
├── 02_logical_workflow/      # L2: 逻辑流转层（伪代码）
├── 03_technical_spec/        # L3: 技术规格层
├── 04_context_reference/     # L4: 决策参考层
├── api/                      # API 参考
├── dev/                      # 开发文档
├── guides/                   # 用户指南
└── plugins/                  # 插件文档
```

---

## 🎯 按层级导航

### L1: 核心概念层
一句话定义系统，解决的核心痛点。

- **[README.md](../README.md)** - 项目概述和价值主张

### L2: 逻辑流转层
使用结构化伪代码描述核心流程，与实现无关。

- **[workflow_execution.pseudo](02_logical_workflow/workflow_execution.pseudo)** - 工作流执行流程
- **[tool_execution.pseudo](02_logical_workflow/tool_execution.pseudo)** - 工具执行流程
- **[plugin_loading.pseudo](02_logical_workflow/plugin_loading.pseudo)** - 插件加载流程
- **[error_handling.pseudo](02_logical_workflow/error_handling.pseudo)** - 错误处理流程

### L3: 技术规格层
接口契约、数据模型、错误码定义。

- **[interfaces.md](03_technical_spec/interfaces.md)** - 接口契约规范
- **[VERSION_MAPPING.md](VERSION_MAPPING.md)** - 版本映射表

### L4: 决策参考层
架构决策记录、性能基准、迁移说明。

- **[architecture_decision.md](04_context_reference/architecture_decision.md)** - 架构决策记录 (ADR)

---

## 📖 按主题导航

### 🚀 入门指南
面向各级用户的实用指南。

- **[用户指南](guides/USER_GUIDE.md)** - 新用户完整手册
- **[速查表](guides/CHEATSHEET.md)** - 快速命令参考
- **[教程](guides/TUTORIAL.md)** - 分步骤学习路径
- **[故障排除](guides/TROUBLESHOOTING.md)** - 常见问题和解决方案

### 💻 API与参考
CLI和Rust SDK的技术规范。

- **[CLI参考](api/CLI_REFERENCE.md)** - 命令行接口文档
- **[Rust SDK参考](api/RUST_SDK_REFERENCE.md)** - 开发者核心库API

### 🛠️ 开发文档
面向贡献者和插件开发者的资源。

- **[开发指南](dev/DEVELOPMENT_GUIDE.md)** - 贡献和设置
- **[插件开发](dev/PLUGIN_DEVELOPMENT.md)** - 创建自定义扩展
- **[项目概览](dev/PROJECT_OVERVIEW.md)** - 架构和设计
- **[文档标准](dev/DOCUMENTATION_STANDARDS.md)** - 编写规范

### 📁 专题文档
特定插件子系统的文档。

- **[文件管理工具](plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md)** - 全面的文件操作套件
- **[迁移指南](guides/MIGRATION_GUIDE.md)** - 从Python/旧版本迁移

---

## 🔍 按角色查找

### "我是..."

#### 新用户
1. 阅读 **[用户指南](guides/USER_GUIDE.md)**
2. 跟随 **[教程](guides/TUTORIAL.md)**
3. 常备 **[速查表](guides/CHEATSHEET.md)**

#### 系统管理员
1. 查看 **[CLI参考](api/CLI_REFERENCE.md)** 用于自动化
2. 参考 **[故障排除](guides/TROUBLESHOOTING.md)** 用于维护

#### 开发者/贡献者
1. 从 **[开发指南](dev/DEVELOPMENT_GUIDE.md)** 开始
2. 在 **[项目概览](dev/PROJECT_OVERVIEW.md)** 中了解架构
3. 阅读 **[Rust SDK参考](api/RUST_SDK_REFERENCE.md)**
4. 查看 **[architecture_decision.md](04_context_reference/architecture_decision.md)** 了解设计决策

#### 架构师
1. 阅读 **[workflow_execution.pseudo](02_logical_workflow/workflow_execution.pseudo)** 了解执行流程
2. 查看 **[architecture_decision.md](04_context_reference/architecture_decision.md)** 了解架构决策
3. 参考 **[interfaces.md](03_technical_spec/interfaces.md)** 了解接口契约

#### 插件作者
1. 掌握 **[插件开发](dev/PLUGIN_DEVELOPMENT.md)**
2. 参考 **[plugin_loading.pseudo](02_logical_workflow/plugin_loading.pseudo)** 了解加载流程
3. 阅读 **[Rust SDK参考](api/RUST_SDK_REFERENCE.md)** 了解traits

---

## 📞 获取帮助

- **问题：** 首先查看 [故障排除](guides/TROUBLESHOOTING.md)。
- **CLI：** 运行 `workflow-toolkit --help` 或特定命令帮助。
- **代码：** 运行 `cargo doc --open` 查看自动生成的源码文档。
- **版本信息：** 查看 [VERSION_MAPPING.md](VERSION_MAPPING.md)。

---

## 📝 文档维护

- **版本**: v0.1.0
- **维护者**: Workflow Toolkit Team
- **更新频率**: 每个版本更新

### 文档规范

- L1-L4 分层遵循 [LLM 文档规范](参考/document_llm_GUIDE.md)
- 所有文档包含版本标记
- 代码引用使用相对路径

### 贡献文档

1. 确定文档层级（L1-L4）
2. 遵循对应层级的编写规范
3. 添加版本标记
4. 更新本文档索引
5. 更新 [VERSION_MAPPING.md](VERSION_MAPPING.md)
