# 文档目录

> **`docs/`文件夹结构元指南**  
> *最后更新：2026-02-06*

---

## 📂 目录结构

此文件夹包含 Workflow Toolkit 的所有文档，采用 **四级分层架构** 组织：

```
docs/
├── 01_concept_overview.md    # L1: 核心概念层（价值与痛点）
├── 02_logical_workflow/      # L2: 逻辑流转层（伪代码）
│   ├── workflow_execution.pseudo
│   ├── tool_execution.pseudo
│   ├── plugin_loading.pseudo
│   └── error_handling.pseudo
├── 03_technical_spec/        # L3: 技术规格层（接口契约）
│   └── interfaces.md
├── 04_context_reference/     # L4: 决策参考层（ADR）
│   └── architecture_decision.md
│
├── INDEX.md                  # 所有文档的主入口
├── DOCS_README.md            # 本文件（文件夹指南）
│
├── api/                      # API 参考
│   ├── CLI_REFERENCE.md      # 命令行接口参考
│   └── RUST_SDK_REFERENCE.md # 核心库API参考
│
├── guides/                   # 用户手册和指南
│   ├── USER_GUIDE.md         # 全面的用户手册
│   ├── CHEATSHEET.md         # 快速参考
│   ├── TUTORIAL.md           # 分步骤教程
│   ├── TROUBLESHOOTING.md    # 问题解决
│   └── MIGRATION_GUIDE.md    # 从旧版本迁移
│
├── dev/                      # 开发者资源
│   ├── DEVELOPMENT_GUIDE.md  # 贡献者指南
│   ├── PLUGIN_DEVELOPMENT.md # 扩展开发
│   ├── PROJECT_OVERVIEW.md   # 架构和设计
│   └── DOCUMENTATION_STANDARDS.md # 编写规范
│
├── plugins/                  # 插件特定文档
│   └── file_management/      # 文件管理工具套件
│       └── ...
│
└── 参考/                     # AI 工作流规约（非明确不变更）
    ├── AGENT_SOP.md
    ├── document_llm_GUIDE.md
    └── ...
```

---

## 📚 文档分层规范

遵循 [document_llm_GUIDE.md](./参考/document_llm_GUIDE.md) 的分级存储架构：

| 层级 | 目录 | 内容 | 禁止 |
|------|------|------|------|
| L1 | `01_concept_overview.md` | 核心概念、价值主张、术语 | 代码、路径、配置 |
| L2 | `02_logical_workflow/` | 伪代码描述流程 | 具体语法、实现细节 |
| L3 | `03_technical_spec/` | 接口契约、数据模型 | 业务逻辑描述 |
| L4 | `04_context_reference/` | ADR、限制、历史 | - |

---

## 📝 编写文档

### 添加新文档

1. **确定层级：** 根据内容选择 L1-L4 层级
2. **确定类别：** 选择适当的子目录（`api`、`guides`、`dev`、`plugins`）
3. **遵循标准：** 参见 [DOCUMENTATION_STANDARDS.md](dev/DOCUMENTATION_STANDARDS.md)
4. **更新索引：** 在 [INDEX.md](INDEX.md) 中添加新文件的链接
5. **链接返回：** 确保您的文件有链接返回到 `INDEX.md` 或其父类别

### 风格指南

- **清晰简洁：** 使用直接的语言
- **代码优先：** 为每个功能提供示例
- **交叉链接：** 使用相对路径链接到相关文档
- **元数据：** 在顶部包含"最后更新"日期
- **语言：** 中文为主，术语/变量除外

---

## 🔗 关键链接

- **[L1 核心概念](01_concept_overview.md)**：从这里开始了解项目
- **[主索引](INDEX.md)**：完整文档导航
- **[项目根目录](../README.md)**：返回代码库根目录
- **[根设计文档](../design.md)**：全局设计文档索引

---

## 🗄️ 归档文档

过时文档已归档至 `.backup/` 目录：

- `.backup/docs/archive/` - 历史版本文档
- `.backup/plans/` - 过时的实现计划
