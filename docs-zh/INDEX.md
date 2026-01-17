# 文档索引

> **Rust Workflow Toolkit 完整导航指南**  
> *最后更新时间：2026-01-14*

---

## 📚 快速导航

### 🚀 入门指南
- **[用户指南](guides/USER_GUIDE.md)** - 针对新用户的全面手册
- **[速查表](guides/CHEATSHEET.md)** - 快速命令参考
- **[教程](guides/TUTORIAL.md)** - 逐步学习路径
- **[故障排除](guides/TROUBLESHOOTING.md)** - 常见问题及解决方案

### 💻 API 与参考
- **[CLI 参考](api/CLI_REFERENCE.md)** - 命令行界面文档
- **[Rust SDK 参考](api/RUST_SDK_REFERENCE.md)** - 供开发者使用的核心库 API

### 🛠️ 开发
- **[开发指南](dev/DEVELOPMENT_GUIDE.md)** - 贡献与环境搭建
- **[插件开发](dev/PLUGIN_DEVELOPMENT.md)** - 创建自定义扩展
- **[项目概览](dev/PROJECT_OVERVIEW.md)** - 架构与设计
- **[文档标准](dev/DOCUMENTATION_STANDARDS.md)** - 编写指南

### 📁 专题
- **[文件管理工具](plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md)** - 全面的文件操作套件
- **[迁移指南](guides/MIGRATION_GUIDE.md)** - 从 Python 或以前版本迁移

---

## 📖 文档结构

文档分为四个主要类别：

### 1. 指南 (`docs/guides/`)
面向各级别用户的实用指南。
- **从这里开始：** `USER_GUIDE.md`
- **快速帮助：** `CHEATSHEET.md`, `TROUBLESHOOTING.md`

### 2. API 参考 (`docs/api/`)
CLI 和 Rust SDK 的技术规范。
- **CLI 用户：** `CLI_REFERENCE.md`
- **Rust 开发者：** `RUST_SDK_REFERENCE.md`

### 3. 开发 (`docs/dev/`)
面向贡献者和插件开发者的资源。
- **贡献者：** `DEVELOPMENT_GUIDE.md`
- **扩展者：** `PLUGIN_DEVELOPMENT.md`

### 4. 插件 (`docs/plugins/`)
特定插件子系统的文档。
- **文件管理：** `file_management/`

---

## 🔍 按角色搜索

### "我是..."

#### 新用户
1. 阅读 **[用户指南](guides/USER_GUIDE.md)**
2. 遵循 **[教程](guides/TUTORIAL.md)**
3. 随身携带 **[速查表](guides/CHEATSHEET.md)**

#### 系统管理员
1. 检查 **[CLI 参考](api/CLI_REFERENCE.md)** 以实现自动化
2. 查看 **[故障排除](guides/TROUBLESHOOTING.md)** 以进行维护

#### 开发者 / 贡献者
1. 从 **[开发指南](dev/DEVELOPMENT_GUIDE.md)** 开始
2. 在 **[项目概览](dev/PROJECT_OVERVIEW.md)** 中了解架构
3. 阅读 **[Rust SDK 参考](api/RUST_SDK_REFERENCE.md)**

#### 插件作者
1. 掌握 **[插件开发](dev/PLUGIN_DEVELOPMENT.md)**
2. 参考 **[Rust SDK 参考](api/RUST_SDK_REFERENCE.md)** 了解 Trait

---

## 📞 获取帮助

- **问题：** 首先检查 `guides/TROUBLESHOOTING.md`。
- **CLI：** 运行 `workflow-toolkit --help` 或特定命令帮助。
- **代码：** 运行 `cargo doc --open` 获取自动生成的源码文档。
