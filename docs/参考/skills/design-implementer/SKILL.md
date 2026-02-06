---
name: "design-implementer"
description: "根据 design.md 中的设计实现代码。Invoke when design is approved and ready for implementation. Follows design specifications strictly and validates against design.md."
---

# Design Implementer Skill

根据 design.md 中的设计实现代码，严格按照设计规范执行。

## 使用场景

- Oracle 完成设计方案并获得审批后
- Librarian 创建/更新 design.md 后
- 需要按照设计文档执行具体编码任务时

## 工作流程

### Step 1: 读取设计文档

```rust
Read(file_path: "src/module/design.md")
```

确认：
- 状态为 `[进行中]`
- 任务清单已定义
- 接口契约已明确

### Step 2: 建立 Checkpoint

记录当前代码状态，便于回滚。

### Step 3: 按任务清单实现

逐个完成任务：

```markdown
## 2.2 任务清单
- [x] Task 0: 渐进式重构  ← 先完成
- [ ] Task 1: [进行中]     ← 当前进行
- [ ] Task 2: [待开始]
```

### Step 4: 编码规范检查

每完成一个任务，执行：

```bash
cargo fmt                    # 格式化
cargo clippy                 # 静态检查
cargo check                  # 编译检查
```

### Step 5: 运行测试

```bash
cargo test [相关测试]        # 运行测试
```

### Step 6: 展示 Diff

输出变更摘要等待审批。

## 输入格式

```markdown
## 实现任务
[design.md 路径]

## 当前任务
[Task X: 任务描述]

## 接口契约
```rust
[关键接口定义]
```
```

## 输出格式

```markdown
## 实现完成

### 变更文件
- [文件路径] ([新增/修改])

### Diff
[代码差异展示]

### 验证结果
- 编译: [✅/❌]
- 测试: [X/Y 通过]
- Lint: [✅/❌ (警告数)]

### 状态更新
- 已完成: [Task X]
- 待进行: [Task Y]

请审批以上变更
```

## 示例

### 实现接口 Trait

**输入**:
```markdown
## 实现任务
src/workflow/executor/design.md

## 当前任务
Task 1: 实现 RetryExecutor Trait

## 接口契约
```rust
#[async_trait]
pub trait RetryExecutor {
    async fn execute_with_retry<F, Fut>(
        &self,
        operation: F,
        policy: RetryPolicy,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>;
}
```
```

**输出**:
```markdown
## 实现完成

### 变更文件
- src/workflow/executor/retry.rs (新增)
- src/workflow/executor/mod.rs (修改)

### Diff
[代码差异展示]

### 验证结果
- 编译: ✅
- 测试: 8/8 通过
- Lint: ✅

### 状态更新
- 已完成: Task 1
- 待进行: Task 2

请审批以上变更
```

## 三错即停机制

### Strike 1
- **触发**: 编译错误或测试失败
- **行动**: 分析报错，执行逻辑修正

### Strike 2
- **触发**: 再次失败
- **行动**: 停止编码，@Explorer 审计环境，@Oracle 微调方案

### Strike 3 (熔断)
- **触发**: 第三次失败
- **行动**: 停止工具调用，生成 `FAILURE_REPORT.md`，锁定权限等待人工干预

## 约束

- 严格按照 design.md 实现，不擅自变更设计
- 每次修改后必须运行测试
- 必须通过 clippy 和 fmt
- 展示 Diff 等待审批后才算完成
- 三次失败后必须熔断
