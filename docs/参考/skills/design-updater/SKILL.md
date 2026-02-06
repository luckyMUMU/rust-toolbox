---
name: "design-updater"
description: "根据设计变更更新 design.md 文档。Invoke when design changes during implementation, when adding new tasks, or when updating ADR records. Maintains document consistency with code changes."
---

# Design Updater Skill

根据设计变更更新 design.md 文档，保持文档与代码变更同步。

## 使用场景

- 实现过程中设计发生变更时
- 需要添加新的任务到任务清单时
- 需要更新 ADR (架构决策记录) 时
- 需要修改接口契约时
- 需要更新状态记录时

## 工作流程

### Step 1: 读取当前 design.md

```rust
Read(file_path: "src/module/design.md")
```

### Step 2: 分析变更类型

| 变更类型 | 操作 |
|----------|------|
| 新增任务 | 添加到任务清单 |
| 修改接口 | 更新接口契约 |
| 变更决策 | 更新 ADR |
| 完成阶段 | 更新状态记录 |

### Step 3: 执行更新

使用 `SearchReplace` 进行精确修改：

```markdown
## 2. 待实现方案 (In Progress) 🟢

### 2.2 任务清单
- [x] Task 0: 渐进式重构
- [x] Task 1: [已完成任务]
- [ ] Task 2: [进行中任务]  ← 更新此处
- [ ] Task 3: [新增任务]    ← 添加此处
```

### Step 4: 更新状态记录

```markdown
## 3. 状态记录
- `[进行中]` | Task 2 开发中 | 2026-02-05
- `[已完成]` | Task 0, Task 1 | 2026-02-04
```

### Step 5: 同步父级索引（如需要）

## 输入格式

```markdown
## 更新目标
[design.md 路径]

## 变更内容
- 类型: [新增任务/修改接口/更新ADR/状态变更]
- 详情: [具体变更描述]

## 原因
[为什么需要这个变更]
```

## 输出格式

```markdown
## 更新完成

- 文件: [design.md 路径]
- 变更类型: [类型]
- 变更摘要: [简要说明]
- 状态: [进行中/已完成]

## 变更详情
[具体修改内容]
```

## 示例

### 添加新任务

**输入**:
```markdown
## 更新目标
src/workflow/executor/design.md

## 变更内容
- 类型: 新增任务
- 详情: 添加 Task 3: 实现指数退避算法

## 原因
设计评审后决定支持指数退避
```

**输出**:
```markdown
## 更新完成

- 文件: src/workflow/executor/design.md
- 变更类型: 新增任务
- 变更摘要: 添加 Task 3: 实现指数退避算法
- 状态: `[进行中]`

## 变更详情
任务清单已更新，新增 Task 3
```

### 标记任务完成

**输入**:
```markdown
## 更新目标
src/workflow/executor/design.md

## 变更内容
- 类型: 状态变更
- 详情: Task 1 已完成

## 原因
代码已实现并通过测试
```

**输出**:
```markdown
## 更新完成

- 文件: src/workflow/executor/design.md
- 变更类型: 状态变更
- 变更摘要: Task 1 标记为已完成
- 状态: `[进行中]`

## 变更详情
- Task 1: [x] 已完成
- 状态记录已更新
```

## 约束

- 只修改 `.md` 文件，不修改代码
- 状态变更必须记录日期
- 接口契约变更必须说明原因
- ADR 更新必须保留历史记录
