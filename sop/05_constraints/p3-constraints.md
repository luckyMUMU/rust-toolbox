---
version: v3.0.2
level: P3
---

# P3级约束

constraint_strength: 实现细节，IDE实时提示

## 编码规范

```yaml
P3-CODE-001:
  name: 缩进规范
  desc: 使用2空格缩进(或项目配置)
  verify: IDE插件(Prettier, EditorConfig)
  handle: 自动格式化
  exception: 无例外

P3-CODE-002:
  name: 行长度限制
  desc: 每行不超过100字符
  verify: IDE插件
  handle: 自动换行
  exception: URL可例外

P3-CODE-003:
  name: 空行规范
  desc: 函数之间空一行，类之间空两行
  verify: IDE插件
  handle: 自动格式化
  exception: 无例外

P3-CODE-004:
  name: 引号规范
  desc: 优先使用单引号
  verify: IDE插件
  handle: 自动格式化
  exception: 字符串包含单引号时可使用双引号
```

## 注释规范

```yaml
P3-COMMENT-001:
  name: 注释语言
  desc: 注释使用中文(或项目配置)
  verify: 代码审查
  handle: 提示，建议修改
  exception: 引用外部文档时可使用英文

P3-COMMENT-002:
  name: TODO格式
  desc: TODO注释应包含负责人和日期
  verify: 代码风格检查
  handle: 提示，建议补充
  exception: 无例外
  format: // TODO(负责人): 描述 - YYYY-MM-DD
```

## 测试规范

```yaml
P3-TEST-001:
  name: 测试文件位置
  desc: 测试文件与源文件同目录或tests/目录
  verify: 文件结构检查
  handle: 提示，建议移动
  exception: 无例外

P3-TEST-002:
  name: 测试文件命名
  desc: 测试文件名为{源文件名}.test.{ext}或{源文件名}_test.{ext}
  verify: 文件结构检查
  handle: 提示，建议重命名
  exception: 无例外

P3-TEST-003:
  name: 测试覆盖率显示
  desc: 测试运行后显示覆盖率报告
  verify: 测试工具配置
  handle: 提示，建议配置
  exception: 无例外
```

## 文档规范

```yaml
P3-DOC-001:
  name: Markdown格式
  desc: Markdown文件使用标准格式
  verify: Markdown linter
  handle: 自动格式化
  exception: 无例外

P3-DOC-002:
  name: 文档标题层级
  desc: 文档标题层级不超过4级
  verify: Markdown linter
  handle: 提示，建议重构
  exception: 复杂文档可例外
```

## Git规范

```yaml
P3-GIT-001:
  name: 提交信息格式
  desc: 提交信息遵循Conventional Commits
  verify: commitlint
  handle: 提示，建议修改
  exception: 无例外
  format: type(scope): 描述

P3-GIT-002:
  name: 分支命名
  desc: 分支名遵循{type}/{issue-id}-{description}
  verify: 分支命名检查
  handle: 提示，建议重命名
  exception: 无例外
```

## 验证工具

```yaml
tools:
  - type: 代码格式化
    names: [Prettier, EditorConfig]
    integration: IDE插件
  - type: 代码风格
    names: [ESLint, Pylint]
    integration: IDE插件
  - type: Markdown
    names: [markdownlint]
    integration: IDE插件
  - type: Git
    names: [commitlint, husky]
    integration: pre-commit hook
```
