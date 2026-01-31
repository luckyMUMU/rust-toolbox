# 文档标准

> **编写和维护文档的完整指南**  
> *最后更新：2026-01-14*

---

## 📋 概述

### 目的
本文档定义了 rust-tool-v2 项目中所有文档的标准。遵循这些标准可确保：
- 所有文档的一致性
- 对所有受众的清晰沟通
- 易于维护和更新
- 专业质量

### 受众
- **技术文档撰写者**：创建新文档
- **开发者**：更新API文档
- **维护者**：审查贡献
- **用户**：阅读指南

---

## 🎯 文档类型

### 1. 用户指南
**目的**：帮助用户完成任务  
**受众**：最终用户、操作员  
**语气**：友好、直接、任务导向  
**示例**：USER_GUIDE.md, TUTORIAL.md

### 2. 开发者指南
**目的**：解释架构和开发实践  
**受众**：贡献者、维护者  
**语气**：技术、精确、全面  
**示例**：DEVELOPMENT_GUIDE.md, PLUGIN_DEVELOPMENT.md

### 3. API参考
**目的**：记录命令、函数和接口  
**受众**：开发者、高级用户  
**语气**：正式、结构化、完整  
**示例**：API_REFERENCE.md, API_INDEX.md

### 4. 教程
**目的**：分步骤学习  
**受众**：新用户  
**语气**：鼓励性、渐进式、实践性  
**示例**：TUTORIAL.md

### 5. 故障排除
**目的**：解决常见问题  
**受众**：所有用户  
**语气**：共情、解决方案导向  
**示例**：TROUBLESHOOTING.md

### 6. 迁移指南
**目的**：帮助用户从其他工具迁移  
**受众**：其他工具的用户  
**语气**：比较性、鼓励性、详细  
**示例**：MIGRATION_GUIDE.md

---

## 📝 写作标准

### 语言与语气

#### 1. 使用清晰、简单的语言
✅ **好的示例：**
```markdown
运行 `cargo run -- workflow execute workflow.yaml` 来执行工作流。
```

❌ **避免：**
```markdown
工作流的执行可以通过调用命令 `cargo run -- workflow execute workflow.yaml` 来启动，
该命令将开始处理指定的工作流定义文件。
```

#### 2. 直接且主动
✅ **好的示例：**
```markdown
创建工作流文件。
运行命令。
检查输出。
```

❌ **避免：**
```markdown
应该创建工作流文件。
应该运行命令。
应该检查输出。
```

#### 3. 使用一致的术语
| 概念 | 使用 | 避免 |
|---------|-----|-------|
| 执行工作流 | `execute`, `run` | `invoke`, `trigger`, `start` |
| 命令 | `command`, `CLI` | `tool`, `utility`, `program` |
| 工作流文件 | `workflow file`, `YAML file` | `script`, `config`, `definition` |
| 参数 | `parameter`, `option` | `argument`, `flag` (除非特定) |

### 结构与组织

#### 1. 层级标题
```markdown
# 标题 (H1) - 文档名称
## 章节 (H2) - 主要主题
### 小节 (H3) - 具体项目
#### 细节 (H4) - 详细内容
```

**规则：**
- 每个文档只有一个H1
- H2用于主要章节
- H3-H4用于子章节
- 不要跳过级别 (H1 → H3)

#### 2. 逻辑流程
```
1. 简介 (是什么 & 为什么)
2. 前置条件
3. 快速开始 / 基本用法
4. 详细解释
5. 高级主题
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
使用 `--verbose` 标志获取详细输出。
```

**列表：**
```markdown
- 使用连号表示无序列表
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

### Markdown约定

#### 1. 链接
```markdown
# 相对链接（推荐）
[USER_GUIDE.md](USER_GUIDE.md)

# 带锚点
[快速开始](#quick-start)

# 外部链接
[Rust文档](https://doc.rust-lang.org/)
```

**规则：**
- 在docs/内使用相对链接
- 使用锚点进行内部导航
- 始终使用描述性文本
- 不要使用"点击这里"

#### 2. 代码示例
```markdown
**好的示例：**
```bash
# 解释这是做什么的注释
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

**更好的示例：**
```bash
# 执行简单工作流
cargo run -- workflow execute workflows/basic/hello-world.yaml

# 预期输出：
# 工作流成功完成
```

**最佳示例：**
```bash
# 执行简单工作流
cargo run -- workflow execute workflows/basic/hello-world.yaml

# 预期输出：
# 工作流成功完成

# 常见错误：
# - 文件未找到：检查工作流文件路径
# - 语法错误：验证YAML结构
```
```

#### 3. 警告与注意
```markdown
> **⚠️ 警告：** 此操作无法撤销。始终先备份。

> **ℹ️ 注意：** 此功能需要 `lancedb` 功能标志。

> **💡 提示：** 使用 `--dry-run` 在执行前预览更改。
```

#### 4. 表情符号（可选）
谨慎使用以增强视觉层次：
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

### 模板1：用户指南

```markdown
# [功能名称] 指南

> **目的**：[此功能的作用]  
> **受众**：[应该使用此功能的人]  
> **前置条件**：[他们需要了解的内容]

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

| 选项 | 描述 | 默认值 | 必需 |
|--------|-------------|---------|----------|
| `--source` | 源目录 | 无 | 是 |
| `--dest` | 目标目录 | 无 | 是 |

## 🎯 示例

### 示例 1：[场景]
```bash
# 命令
cargo run -- [command] [options]

# 解释
这将[发生什么]。
```

### 示例 2：[场景]
```bash
# 命令
cargo run -- [command] [options]

# 解释
这将[发生什么]。
```

## 🔧 故障排除

### 问题：[常见问题]
**解决方案：** [解决方案]

### 问题：[常见问题]
**解决方案：** [解决方案]

## 📚 相关
- [INDEX.md](INDEX.md) - 完整文档索引
- [API_INDEX.md](API_INDEX.md) - 命令参考
```

### 模板2：API参考

```markdown
# [组件] API参考

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

## Traits

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

### 模板3：故障排除

```markdown
# [功能] 故障排除

> **常见问题和解决方案**

---

## 🔍 快速修复

先尝试这些：

1. **检查版本**
   ```bash
   cargo --version
   ```

2. **验证安装**
   ```bash
   cargo run -- --help
   ```

## 🚨 常见问题

### 问题 1：[问题描述]

**症状：**
```
错误消息或行为
```

**原因：**
解释为什么会发生

**解决方案：**
```bash
# 修复命令或步骤
```

**预防：**
如何避免再次发生

---

**← 返回 [INDEX.md](INDEX.md)**
```

---

## ✅ 写作检查清单

在提交文档之前，验证：

- [ ] 标题遵循层级结构
- [ ] 所有代码块都有语言标识符
- [ ] 链接有效且描述性强
- [ ] 术语一致
- [ ] 包含示例
- [ ] 包含故障排除（如适用）
- [ ] 已校对拼写和语法
- [ ] 已更新日期

---

## 🎨 风格指南

### 语气

#### 主动语态
```markdown
✅ 运行命令来执行工作流。
❌ 命令被运行来执行工作流。
```

#### 第二人称（你）
```markdown
✅ 你可以使用 --verbose 运行命令。
❌ 用户可以使用 --verbose 运行命令。
```

#### 积极语言
```markdown
✅ 使用 --verbose 获取详细输出。
❌ 除非需要详细信息，否则不要使用 --verbose。
```

### 格式规则

#### 代码块
- 始终指定语言
- 保持行长度在80个字符以内
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
| Execute | 运行工作流 | `execute a workflow` |
| Run | 运行命令 | `run the command` |
| Parameter | 命令选项 | `--source parameter` |
| Flag | 布尔选项 | `--verbose flag` |
| Option | 任何命令行选择 | `available options` |
| Workflow | 工作流文件 | `create a workflow` |
| Tool | 单个工具 | `file-classifier tool` |
| Command | CLI命令 | `cargo run -- command` |

---

## 📚 现有文档示例

### 好示例：USER_GUIDE.md
```markdown
## 🚀 快速开始

### 执行你的第一个工作流
```bash
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

这将执行一个打印"Hello World"的简单工作流。
```

### 好示例：DEVELOPMENT_GUIDE.md
```markdown
## 代码风格

### 导入顺序
```rust
// 1. 标准库
use std::sync::Arc;

// 2. 外部crate（按字母顺序）
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 3. 内部模块
use crate::core::WorkflowId;
```
```

### 好示例：TROUBLESHOOTING.md
```markdown
### Cargo构建失败

**症状：**
```
error: could not compile `rust-tool-v2`
```

**解决方案：**
```bash
# 1. 检查Rust版本
rustc --version

# 2. 更新Rust
rustup update
```
```

---

## 🎯 常见错误避免

### ❌ 不要这样做
```markdown
# 太模糊
## 选项
使用 --verbose 获取更多信息。

# 没有示例
运行命令。

# 链接损坏
[点击这里](some-file.md)

# 术语不一致
在一个地方使用"run"，在另一个地方使用"execute"

# 缺少前置条件
没有提及所需的Rust版本

# 没有错误处理
假设一切都正常工作
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
[DEBUG] 正在解析YAML...
[INFO] 正在执行步骤1...
```

**常见错误：**
- 文件未找到：检查路径
- 语法错误：验证YAML
```

---

## 📖 参考链接

### 内部文档
- **[INDEX.md](INDEX.md)** - 完整导航
- **[README.md](../README.md)** - 项目概述
- **[USER_GUIDE.md](USER_GUIDE.md)** - 使用手册
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - 开发指南
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - 解决方案

### 外部资源
- **Rust文档**：https://doc.rust-lang.org/
- **Markdown指南**：https://www.markdownguide.org/
- **技术写作**：https://developers.google.com/tech-writing

---

## ✅ 快速参考

### 基本命令
```bash
# 检查所有文档的损坏链接
grep -r "\[.*\](.*\.md)" docs/ | grep -v "http"

# 查找所有markdown文件
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
command
\`\`\`

# 警告
> **⚠️ 警告：** 重要信息

# 注意
> **ℹ️ 注意：** 有帮助的信息

# 提示
> **💡 提示：** 专业提示
```

---

## 🔄 更新本文档

### 何时更新
- 添加新文档类型
- 新工具或功能
- 用户对清晰度的反馈
- 现有文档的模式变化

### 如何更新
1. 添加新章节或模式
2. 提供清晰的示例
3. 更新版本/日期
4. 向团队宣布
5. 更新INDEX.md

---

**← 返回 [INDEX.md](INDEX.md)** | **顶部** ↑
