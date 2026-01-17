# 文档目录

> **`docs/` 文件夹结构的元指南**  
> *最后更新时间：2026-01-14*

## 📂 目录结构

此文件夹包含 Rust Workflow Toolkit 的所有文档，按用途和受众进行组织。

```
docs/
├── INDEX.md                 # 所有文档的主入口
├── DOCS_README.md           # 此文件（文件夹指南）
│
├── api/                     # 技术规范
│   ├── CLI_REFERENCE.md     # 命令行界面参考
│   └── RUST_SDK_REFERENCE.md# 核心库 API 参考
│
├── guides/                  # 用户手册和指南
│   ├── USER_GUIDE.md        # 全面用户手册
│   ├── CHEATSHEET.md        # 快速参考
│   ├── TUTORIAL.md          # 逐步教程
│   ├── TROUBLESHOOTING.md   # 问题解决
│   └── MIGRATION_GUIDE.md   # 从以前版本的迁移
│
├── dev/                     # 开发者资源
│   ├── DEVELOPMENT_GUIDE.md # 贡献者指南
│   ├── PLUGIN_DEVELOPMENT.md# 扩展开发
│   ├── PROJECT_OVERVIEW.md  # 架构和设计
│   └── DOCUMENTATION_STANDARDS.md # 编写指南
│
└── plugins/                 # 特定插件文档
    └── file_management/     # 文件管理工具套件
        ├── FILE_MANAGEMENT_TOOLS_INDEX.md
        └── ...
```

## 📝 编写文档

### 添加新文档
1. **确定类别：** 选择合适的子目录（`api`、`guides`、`dev`、`plugins`）。
2. **遵循标准：** 参见 `dev/DOCUMENTATION_STANDARDS.md`（如果可用）。
3. **更新索引：** 在 `INDEX.md` 中添加指向新文件的链接。
4. **反向链接：** 确保您的文件有返回 `INDEX.md` 或其父类别的链接。

### 风格指南
- **清晰简洁：** 使用直接的语言。
- **代码优先：** 为每个特性提供示例。
- **交叉链接：** 使用相对路径链接到相关文档（例如 `[指南](../guides/USER_GUIDE.md)`）。
- **元数据：** 在顶部包含“最后更新时间”日期。

## 🔗 关键链接
- **[主索引](INDEX.md)**：从这里开始导航。
- **[项目根目录](../README.md)**：返回代码库根目录。
