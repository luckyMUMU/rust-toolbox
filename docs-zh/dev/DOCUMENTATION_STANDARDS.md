# 文档标准

> **编写和维护文档的完整指南**  
> *最后更新日期：2026-01-14*

---

## 📋 概述

### 目的
本文档定义了 rust-tool-v2 项目中所有文档的标准。遵循这些标准可确保：
- 所有文档保持一致性
- 为所有受众提供清晰的沟通
- 易于维护和更新
- 专业质量

### 受众
- **技术作家**：创建新文档
- **开发人员**：更新 API 文档
- **维护者**：审核贡献
- **用户**：阅读指南

---

## 🎯 文档类型

### 1. 用户指南
**目的**：帮助用户完成任务  
**受众**：最终用户、操作员  
**语调**：友好、直接、以任务为导向  
**示例**：USER_GUIDE.md, TUTORIAL.md

### 2. 开发人员指南
**目的**：解释架构和开发实践  
**受众**：贡献者、维护者  
**语调**：技术性、精确、全面  
**示例**：DEVELOPMENT_GUIDE.md, PLUGIN_DEVELOPMENT.md

### 3. API 参考
**目的**：记录命令、函数和接口  
**受众**：开发人员、高级用户  
**语调**：正式、结构化、完整  
**示例**：API_REFERENCE.md, API_INDEX.md

### 4. 教程
**目的**：分步学习  
**受众**：新用户  
**语调**：鼓励性、渐进式、动手实践  
**示例**：TUTORIAL.md

### 5. 故障排除
**目的**：解决常见问题  
**受众**：所有用户  
**语调**：同理心、以解决方案为中心  
**示例**：TROUBLESHOOTING.md

### 6. 迁移指南
**目的**：帮助用户从替代方案迁移  
**受众**：其他工具的用户  
**语调**：对比性、鼓励性、详细  
**示例**：MIGRATION_GUIDE.md

---

## 📝 编写标准

### 语言与语调

#### 1. 使用清晰、简单的语言
✅ **好：**
```markdown
运行 `cargo run -- workflow execute workflow.yaml` 来执行工作流。
```

❌ **避免：**
```markdown
工作流的执行可以通过调用命令 `cargo run -- workflow execute workflow.yaml` 来启动，该命令将开始处理指定的工作流定义文件。
```

#### 2. 直接且使用主动语态
✅ **好：**
```markdown
创建一个工作流文件。
运行命令。
检查输出。
```

❌ **避免：**
```markdown
应该创建一个工作流文件。
命令应该被运行。
输出应该被检查。
```

#### 3. 使用一致的术语
| 概念 | 使用 | 避免 |
|---------|-----|-------|
| 执行工作流 | `execute`, `run` | `invoke`, `trigger`, `start` |
| 命令 | `command`, `CLI` | `tool`, `utility`, `program` |
| 工作流文件 | `workflow file`, `YAML file` | `script`, `config`, `definition` |
| 参数 | `parameter`, `option` | `argument`, `flag` (除非特定) |

### 结构与组织

#### 1. 分级标题
```markdown
# 标题 (H1) - 文档名称
## 章节 (H2) - 主要话题
### 子章节 (H3) - 具体条目
#### 细节 (H4) - 微小细节
```

**规则：**
- 每个文档只能有一个 H1
- H2 用于主要章节
- H3-H4 用于子章节
- 永远不要跳级（如 H1 → H3）

#### 2. 逻辑流
```
1. 简介 (是什么 & 为什么)
2. 前提条件
3. 快速开始 / 基本用法
4. 详细说明
5. 高级话题
6. 示例
7. 故障排除
8. 相关资源
```

#### 3. 一致的格式

**代码块：**
```markdown
```bash
# 使用语言标识符
cargo run -- --help
```
```

**行内代码：**
```markdown
使用 `--verbose` 标志查看详细输出。
```

**列表：**
```markdown
- 使用连字符表示无序列表
- 保持项目结构平行
- 使用数字表示有序列表

1. 第一步
2. 第二步
3. 第三步
```

**表格：**
```markdown
| 选项 | 描述 | 默认值 |
|--------|-------------|---------|
| `--verbose` | 显示详细输出 | `false` |
```

### Markdown 约定

#### 1. 链接
```markdown
# 相对链接 (推荐)
[USER_GUIDE.md](USER_GUIDE.md)

# 带锚点
[快速开始](#quick-start)

# 外部链接
[Rust 文档](https://doc.rust-lang.org/)
```

**规则：**
- 在 docs/ 目录下使用相对链接
- 使用锚点进行内部导航
- 始终使用描述性文本
- 永远不要使用 "点击这里"

#### 2. 代码示例
```markdown
**好：**
```bash
# 注释解释其功能
cargo run -- workflow execute examples/hello-world.yaml
```

**更好：**
```bash
# 执行一个简单的工作流
cargo run -- workflow execute examples/hello-world.yaml

# 预期输出：
# 工作流成功完成
```

**最好：**
```bash
# 执行一个简单的工作流
cargo run -- workflow execute examples/hello-world.yaml

# 预期输出：
# 工作流成功完成

# 常见错误：
# - 文件未找到：检查工作流文件的路径
# - 语法错误：验证 YAML 结构
```
```

#### 3. 警告与注释
```markdown
> **⚠️ 警告：** 此操作无法撤销。请务必先备份。

> **ℹ️ 注意：** 此功能需要 `lancedb` 功能标志。

> **💡 提示：** 在执行前使用 `--dry-run` 预览更改。
```

#### 4. 表情符号 (可选)
适量使用以增强视觉层次感：
- ✅ 成功/完成
- ❌ 错误/避免
- ⚠️ 警告
- ℹ️ 信息
- 💡 提示
- 🚀 快速开始
- 🔧 技术
- 📚 参考
- 🎯 目标

---

## 📄 文档模板

### 模板 1：用户指南

```markdown
# [功能名称] 指南

> **目的**：[此功能的作用]  
> **受众**：[谁应该使用此功能]  
> **前提条件**：[他们需要了解什么]

---

## 🚀 快速开始

```bash
# 最简单的示例
cargo run -- [command] --help
```

## 📖 基本用法

### [常见用例 1]
```bash
# 带解释的示例
cargo run -- [command] [options]
```

### [常见用例 2]
```bash
# 另一个示例
cargo run -- [command] [options]
```

## ⚙️ 选项参考

| 选项 | 描述 | 默认值 | 是否必填 |
|--------|-------------|---------|----------|
| `--source` | 源目录 | 无 | 是 |
| `--dest` | 目标目录 | 无 | 是 |

## 🎯 示例

### 示例 1：[场景]
```bash
# 命令
cargo run -- [command] [options]

# 解释
这将 [会发生什么]。
```

### 示例 2：[场景]
```bash
# 命令
cargo run -- [command] [options]

# 解释
这将 [会发生什么]。
```

## 🔧 故障排除

### 问题：[常见问题]
**解决方案：** [解决方案]

### 问题：[常见问题]
**解决方案：** [解决方案]

## 📚 相关内容
- [INDEX.md](INDEX.md) - 完整文档索引
- [API_INDEX.md](API_INDEX.md) - 命令参考
```

### 模板 2：API 参考

```markdown
# [组件] API 参考

> **最后更新**：[日期]  
> **版本**：[版本]

---

## 概述

[组件的简要描述]

## 类型

### [类型名称]
```rust
pub struct TypeName {
    pub field: Type,
}
```

**字段：**
- `field`: 描述

## 函数

### `function_name()`
```rust
pub fn function_name(param: Type) -> Result<Output>
```

**参数：**
- `param`: 描述

**返回：**
- `Result<Output>`: 描述

**示例：**
```rust
let result = function_name(value)?;
```

## Trait

### `TraitName`
```rust
#[async_trait]
pub trait TraitName: Send + Sync {
    async fn method(&self, param: Type) -> Result<Output>;
}
```

**方法：**
- `method()`: 描述

## 枚举

### `EnumName`
```rust
pub enum EnumName {
    Variant1,
    Variant2(Type),
}
```

**变体：**
- `Variant1`: 描述
- `Variant2(Type)`: 描述

---

**← 返回 [INDEX.md](INDEX.md)**
```

### 模板 3：故障排除

```markdown
# 故障排除 [功能]

> **常见问题及解决方案**

---

## 🔍 快速修复

先尝试这些：
1. [修复 1]
2. [修复 2]
3. [修复 3]

## 🚨 常见问题

### 问题：[问题描述]
**症状：** [你看到的现象]

**原因：**
- [原因 1]
- [原因 2]

**解决方案：**
```bash
# 解决方案 1
修复命令

# 解决方案 2
备选命令
```

### 问题：[问题描述]
**症状：** [你看到的现象]

**原因：**
- [原因 1]

**解决方案：**
```bash
修复命令
```

## 🔧 高级调试

### 启用日志
```bash
RUST_LOG=debug cargo run -- [command]
```

### 检查系统
```bash
# 检查 Rust 版本
rustc --version

# 检查磁盘空间
df -h

# 检查权限
ls -la /path/to/directory
```

## 📞 获取更多帮助

- [INDEX.md](INDEX.md) - 查找相关文档
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - 调试技术
- [README.md](../README.md) - 项目概述

---

**← 返回 [INDEX.md](INDEX.md)**
```

---

## 🔍 质量检查清单

### 发布前
- [ ] **拼写**：运行拼写检查
- [ ] **语法**：检查错误
- [ ] **链接**：所有链接均有效
- [ ] **代码**：所有示例均可运行
- [ ] **一致性**：遵循标准
- [ ] **完整性**：涵盖所有情况
- [ ] **清晰度**：易于理解
- [ ] **语调**：适合受众

### 技术审核
- [ ] **准确性**：信息正确
- [ ] **时效性**：与代码同步
- [ ] **示例**：所有示例均有效
- [ ] **命令**：命令有效
- [ ] **选项**：所有选项均已记录
- [ ] **错误**：涵盖常见错误

### 用户体验
- [ ] **导航**：易于查找信息
- [ ] **可读性**：结构清晰
- [ ] **可扫描性**：合理使用标题/列表
- [ ] **可操作性**：明确的后续步骤
- [ ] **完整性**：无缺失信息

---

## 📊 文档结构

### 项目布局
```
docs/
├── INDEX.md                    # 主索引
├── USER_GUIDE.md              # 用户手册
├── DEVELOPMENT_GUIDE.md       # 开发实践
├── API_INDEX.md               # 命令参考
├── TROUBLESHOOTING.md         # 问题解决
├── MIGRATION_GUIDE.md         # Python → Rust
├── DOCUMENTATION_STANDARDS.md # 本文件
├── CHEATSHEET.md              # 快速参考
├── TUTORIAL.md                # 分步教程
├── PROJECT_OVERVIEW.md        # 架构
├── PLUGIN_DEVELOPMENT.md      # 扩展
├── API_REFERENCE.md           # 技术规范
├── API_USAGE_GUIDE.md         # 用法示例
└── FILE_MANAGEMENT/           # 文件管理文档
    ├── TOOLS_GUIDE.md
    ├── API_REFERENCE.md
    ├── HUMAN_DECISION_GUIDE.md
    ├── TOOLS_INDEX.md
    └── WORKFLOW_TEMPLATES.md
```

### 交叉引用
```markdown
# 在任何文档中，链接到：
- [INDEX.md](INDEX.md) - 查找任何文档
- [USER_GUIDE.md](USER_GUIDE.md) - 用法示例
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - 开发实践
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - 解决方案
- [API_INDEX.md](API_INDEX.md) - 命令
```

---

## 🎯 编写流程

### 步骤 1：规划
```markdown
1. 确定受众
2. 定义目的
3. 概述结构
4. 收集示例
5. 检查现有文档
```

### 步骤 2：起草
```markdown
1. 编写 H1 和简介
2. 创建主要章节 (H2)
3. 添加子章节 (H3-H4)
4. 编写示例
5. 添加交叉引用
```

### 步骤 3：审核
```markdown
1. 检查质量检查清单
2. 验证所有示例均有效
3. 测试所有命令
4. 检查所有链接
5. 进行同行评审
```

### 步骤 4：发布
```markdown
1. 更新 INDEX.md
2. 如果需要，更新 README.md
3. 检查断开的链接
4. 验证导航
5. 宣布更新
```

---

## 🔄 维护标准

### 更新计划
- **每周**：检查过时信息
- **每月**：审核所有示例
- **每季度**：全面文档审计

### 更新触发因素
- 添加新功能
- API 更改
- 命令更改
- 用户反馈
- 影响用法的错误修复

### 版本跟踪
```markdown
> **最后更新**：2026-01-14  
> **版本**：1.0.0  
> **更改**：[简要描述]
```

---

## 📝 风格指南

### 语声与语调

#### 主动语态 (推荐)
```markdown
✅ 运行命令以执行工作流。
❌ 命令被运行以执行工作流。
```

#### 第二人称 (你)
```markdown
✅ 你可以使用 --verbose 运行命令。
❌ 用户可以使用 --verbose 运行命令。
```

#### 正面语言
```markdown
✅ 使用 --verbose 查看详细输出。
❌ 除非你需要详细信息，否则不要使用 --verbose。
```

### 格式规则

#### 代码块
- 始终指定语言
- 每行保持在 80 个字符以内
- 添加注释以提高清晰度
- 在有帮助时显示预期输出

#### 行内代码
- 用于命令、选项、文件路径
- 用于代码片段
- 用于错误消息

#### 列表
- 使用平行结构
- 保持项目简洁
- 使用一致的标点符号

#### 表格
- 文本左对齐
- 使用清晰的标题
- 保持简洁

### 术语一致性

| 术语 | 用于 | 示例 |
|------|---------|---------|
| 执行 | 运行工作流 | `执行工作流` |
| 运行 | 运行命令 | `运行命令` |
| 参数 | 命令选项 | `--source 参数` |
| 标志 | 布尔选项 | `--verbose 标志` |
| 选项 | 任何命令行选择 | `可用选项` |
| 工作流 | 工作流文件 | `创建一个工作流` |
| 工具 | 单个工具 | `文件分类工具` |
| 命令 | CLI 命令 | `cargo run -- 命令` |

---

## 📚 现有文档示例

### 优秀示例：USER_GUIDE.md
```markdown
## 🚀 快速开始

### 执行你的第一个工作流
```bash
cargo run -- workflow execute examples/hello-world.yaml
```

这将执行一个打印 "Hello World" 的简单工作流。
```

### 优秀示例：DEVELOPMENT_GUIDE.md
```markdown
## 代码风格

### 导入顺序
```rust
// 1. 标准库
use std::sync::Arc;

// 2. 外部 crate (按字母顺序)
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 3. 内部模块
use crate::core::WorkflowId;
```
```

### 优秀示例：TROUBLESHOOTING.md
```markdown
### Cargo 构建失败

**症状：**
```
error: could not compile `rust-tool-v2`
```

**解决方案：**
```bash
# 1. 检查 Rust 版本
rustc --version

# 2. 更新 Rust
rustup update
```
```

---

## 🎯 应避免的常见错误

### ❌ 不要这样做
```markdown
# 太模糊
## 选项
使用 --verbose 获取更多信息。

# 无示例
运行命令。

# 断开的链接
[点击这里](some-file.md)

# 术语不一致
一处使用 "运行"，另一处使用 "执行"

# 缺少前提条件
未提及所需的 Rust 版本

# 无错误处理
假设一切正常
```

### ✅ 应该这样做
```markdown
## 选项参考

| 选项 | 描述 | 默认值 |
|--------|-------------|---------|
| `--verbose` | 显示详细执行日志 | `false` |

**示例：**
```bash
cargo run -- workflow execute workflow.yaml --verbose
```

**预期输出：**
```
[INFO] 正在加载工作流...
[DEBUG] 正在解析 YAML...
[INFO] 正在执行步骤 1...
```

**常见错误：**
- 文件未找到：检查路径
- 语法错误：验证 YAML
```

---

## 📖 参考链接

### 内部文档
- **[INDEX.md](INDEX.md)** - 完整导航
- **[README.md](../README.md)** - 项目概述
- **[USER_GUIDE.md](USER_GUIDE.md)** - 用户手册
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - 开发指南
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - 解决方案

### 外部资源
- **Rust 文档**：https://doc.rust-lang.org/
- **Markdown 指南**：https://www.markdownguide.org/
- **技术写作**：https://developers.google.com/tech-writing

---

## ✅ 快速参考

### 必备命令
```bash
# 检查所有文档中是否有断开的链接
grep -r "\[.*\](.*\.md)" docs/ | grep -v "http"

# 查找所有 markdown 文件
find docs/ -name "*.md"

# 统计文档行数
wc -l docs/*.md
```

### 常见模式
```markdown
# 链接到另一个文档
[链接文本](FILENAME.md)

# 带锚点的链接
[链接文本](FILENAME.md#section-name)

# 代码块
\`\`\`bash
命令
\`\`\`

# 警告
> **⚠️ 警告：** 重要信息

# 注意
> **ℹ️ 注意：** 有用的信息

# 提示
> **💡 提示：** 专家建议
```

---

## 🔄 更新本文档

### 何时更新
- 添加了新的文档类型
- 新工具或功能
- 用户对清晰度的反馈
- 现有文档中的模式更改

### 如何更新
1. 添加新章节或模式
2. 提供清晰的示例
3. 更新版本/日期
4. 向团队宣布
5. 更新 INDEX.md

---

**← 返回 [INDEX.md](INDEX.md)** | **置顶** ↑
