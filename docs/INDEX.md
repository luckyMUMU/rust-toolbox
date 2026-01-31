# 文档索引

> **Rust工作流工具包完整导航指南**  
> *最后更新：2026-01-14*

---

## 📚 快速导航

### 🚀 入门指南
- **[用户指南](guides/USER_GUIDE.md)** - 新用户完整手册
- **[速查表](guides/CHEATSHEET.md)** - 快速命令参考
- **[教程](guides/TUTORIAL.md)** - 分步骤学习路径
- **[故障排除](guides/TROUBLESHOOTING.md)** - 常见问题和解决方案

### 💻 API与参考
- **[CLI参考](api/CLI_REFERENCE.md)** - 命令行接口文档
- **[Rust SDK参考](api/RUST_SDK_REFERENCE.md)** - 开发者核心库API

### 🛠️ 开发文档
- **[开发指南](dev/DEVELOPMENT_GUIDE.md)** - 贡献和设置
- **[插件开发](dev/PLUGIN_DEVELOPMENT.md)** - 创建自定义扩展
- **[项目概览](dev/PROJECT_OVERVIEW.md)** - 架构和设计
- **[文档标准](dev/DOCUMENTATION_STANDARDS.md)** - 编写规范

### 📁 专题文档
- **[文件管理工具](plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md)** - 全面的文件操作套件
- **[迁移指南](guides/MIGRATION_GUIDE.md)** - 从Python/旧版本迁移

---

## 📖 文档结构

文档分为四个主要类别：

### 1. 指南 (`docs/guides/`)
面向各级用户的实用指南。
- **从这里开始：** `USER_GUIDE.md`
- **快速帮助：** `CHEATSHEET.md`, `TROUBLESHOOTING.md`

### 2. API参考 (`docs/api/`)
CLI和Rust SDK的技术规范。
- **CLI用户：** `CLI_REFERENCE.md`
- **Rust开发者：** `RUST_SDK_REFERENCE.md`

### 3. 开发文档 (`docs/dev/`)
面向贡献者和插件开发者的资源。
- **贡献者：** `DEVELOPMENT_GUIDE.md`
- **扩展开发者：** `PLUGIN_DEVELOPMENT.md`

### 4. 插件文档 (`docs/plugins/`)
特定插件子系统的文档。
- **文件管理：** `file_management/`

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

#### 插件作者
1. 掌握 **[插件开发](dev/PLUGIN_DEVELOPMENT.md)**
2. 参考 **[Rust SDK参考](api/RUST_SDK_REFERENCE.md)** 了解traits

---

## 📞 获取帮助

- **问题：** 首先查看 `guides/TROUBLESHOOTING.md`。
- **CLI：** 运行 `workflow-toolkit --help` 或特定命令帮助。
- **代码：** 运行 `cargo doc --open` 查看自动生成的源码文档。
