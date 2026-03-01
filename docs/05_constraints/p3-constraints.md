# P3 级约束定义 (P3 Constraints)

> **版本**: v1.0.0  
> **创建日期**: 2026-03-01  
> **最后更新**: 2026-03-01  
> **状态**: Active  
> **级别**: P3 级（自动化验证）

---

## 1. 概述

本文档定义了 Workflow Toolkit 项目的 P3 级约束，这些是**实现层面**的规范，通过自动化工具进行验证。

---

## 2. 编码规范

### 2.1 Rust 代码风格

**工具**: `rustfmt`, `clippy`

**配置**:
```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
```

**验证命令**:
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
```

### 2.2 命名规范

**规则**:
- 结构体/枚举：PascalCase (`WorkflowExecutor`)
- 函数/方法：snake_case (`execute_workflow`)
- 常量：UPPER_SNAKE_CASE (`MAX_RETRY_COUNT`)
-  trait: PascalCase (`Plugin`)
- 类型参数：PascalCase (`T`, `E`)

### 2.3 函数复杂度

**规则**:
- 函数行数不超过 50 行
- 圈复杂度不超过 10

**验证工具**: `cargo clippy` with `too_many_lines`, `cognitive_complexity`

---

## 3. 注释规范

### 3.1 文档注释

**规则**: 所有公共 API 必须有文档注释

**格式**:
```rust
/// 函数简短描述
///
/// # 参数
/// * `param_name` - 参数描述
///
/// # 返回
/// 返回值描述
///
/// # 错误
/// * `Error::Type` - 错误描述
///
/// # 示例
/// ```
/// let result = some_function();
/// ```
pub fn some_function(param_name: Type) -> Result<ReturnType> {
    // ...
}
```

### 3.2 行内注释

**规则**: 注释解释"为什么"而非"是什么"

**示例**:
```rust
// ✅ 推荐：解释原因
// 使用指数退避，因为网络请求失败率随重试次数递减
let delay = base_delay * 2u64.pow(retry_count as u32);

// ❌ 避免：重复代码
// 增加重试计数
retry_count += 1;
```

### 3.3 TODO 注释

**规则**: TODO 注释必须包含责任人和截止日期

**格式**:
```rust
// TODO(@username, 2026-03-15): 实现缓存失效逻辑
// FIXME(@username, 2026-03-10): 修复并发竞争条件
// HACK(@username, 2026-03-01): 临时解决方案，需重构
```

---

## 4. 测试规范

### 4.1 测试命名

**规则**: 测试函数名应描述测试场景

**格式**: `test_{功能}_{场景}_{预期结果}`

**示例**:
```rust
#[test]
fn test_workflow_execution_with_parallel_nodes_should_complete_successfully() {
    // ...
}

#[test]
fn test_tool_validation_with_invalid_input_should_return_error() {
    // ...
}
```

### 4.2 测试结构

**规则**: 遵循 AAA 模式（Arrange-Act-Assert）

**示例**:
```rust
#[test]
fn test_add_user() {
    // Arrange
    let mut repo = InMemoryUserRepository::new();
    let user = User::new("test@example.com");
    
    // Act
    let result = repo.add(user);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(repo.count(), 1);
}
```

### 4.3 测试覆盖率

**规则**: 单元测试覆盖率不低于 80%

**验证命令**:
```bash
cargo tarpaulin --out Html --output-dir ./coverage
```

---

## 5. 错误处理规范

### 5.1 错误类型定义

**规则**: 使用 thiserror 定义错误类型

**示例**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("工作流未找到：{workflow_id}")]
    NotFound { workflow_id: String },
    
    #[error("工作流验证失败：{message}")]
    Validation { message: String },
    
    #[error("工作流执行失败：节点 {node_id} - {reason}")]
    Execution { node_id: String, reason: String },
}
```

### 5.2 错误传播

**规则**: 使用 `?` 操作符，避免 `.unwrap()`

**示例**:
```rust
// ✅ 推荐
pub fn process(&self) -> Result<()> {
    let data = self.load()?;
    self.validate(&data)?;
    self.save(data)?;
    Ok(())
}

// ❌ 禁止
pub fn process(&self) {
    let data = self.load().unwrap();
    self.validate(&data).unwrap();
}
```

---

## 6. 日志规范

### 6.1 日志级别

**规则**: 正确使用日志级别

| 级别 | 使用场景 |
|------|----------|
| ERROR | 系统错误，需要立即处理 |
| WARN | 警告，不影响系统运行 |
| INFO | 重要状态变更 |
| DEBUG | 调试信息 |
| TRACE | 详细追踪信息 |

### 6.2 结构化日志

**规则**: 使用结构化日志格式

**示例**:
```rust
// ✅ 推荐：结构化
tracing::info!(
    workflow_id = %workflow.id,
    status = "started",
    node_count = workflow.nodes.len(),
    "Workflow execution started"
);

// ❌ 避免：字符串拼接
tracing::info!("Workflow {} started with {} nodes", workflow.id, workflow.nodes.len());
```

### 6.3 日志上下文

**规则**: 日志必须包含上下文信息

**必选字段**:
- `request_id`: 请求 ID
- `user_id`: 用户 ID（如适用）
- `workflow_id`: 工作流 ID（如适用）

---

## 7. 配置规范

### 7.1 配置层次

**规则**: 遵循配置优先级

**优先级**（从高到低）:
1. 命令行参数
2. 环境变量
3. 配置文件
4. 默认值

### 7.2 配置命名

**规则**: 环境变量使用统一前缀

**示例**:
```bash
WORKFLOW_TOOLKIT_MAX_CONCURRENT_WORKFLOWS=100
WORKFLOW_TOOLKIT_DATABASE_URL=postgres://localhost
WORKFLOW_TOOLKIT_LOG_LEVEL=debug
```

---

## 8. 性能规范

### 8.1 性能基准

**规则**: 关键操作必须满足性能要求

| 操作 | 目标 | 容忍 |
|------|------|------|
| 工作流提交 | < 10ms | < 100ms |
| 节点调度 | < 1ms | < 10ms |
| 状态查询 | < 5ms | < 50ms |

### 8.2 性能测试

**规则**: 性能测试自动化

**验证命令**:
```bash
cargo bench --bench workflow_bench
```

---

## 9. 文档规范

### 9.1 模块文档

**规则**: 每个模块必须有 `mod.rs` 文档注释

**示例**:
```rust
//! 工作流引擎模块
//!
//! 提供工作流定义、调度和执行功能。
//!
//! # 架构
//!
//! ```text
//! WorkflowEngine
//!     ├── WorkflowDefinition
//!     ├── WorkflowExecutor
//!     └── ExecutionContext
//! ```
//!
//! # 使用示例
//!
//! ```rust
//! let engine = WorkflowEngine::new(config);
//! let result = engine.execute(workflow).await?;
//! ```

pub mod engine;
pub mod definition;
```

### 9.2 README 文档

**规则**: 每个目录必须有 README.md

**内容**:
- 目录说明
- 核心组件
- 使用示例
- 相关链接

---

## 10. 版本控制规范

### 10.1 提交信息

**规则**: 遵循 Conventional Commits

**格式**:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

**示例**:
```
feat(workflow): 添加并行节点支持

- 实现并行节点调度器
- 添加最大并发数配置
- 更新相关文档

Closes #123
```

### 10.2 分支命名

**规则**: 分支名包含类型和描述

**格式**: `<type>/<description>`

**示例**:
- `feat/parallel-execution`
- `fix/memory-leak`
- `docs/api-reference`

---

## 11. 自动化验证

### 11.1 CI 检查清单

**规则**: 所有 PR 必须通过 CI 检查

**检查项**:
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test --all`
- [ ] 代码覆盖率 > 80%
- [ ] 性能测试通过

### 11.2 预提交钩子

**配置**:
```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running rustfmt..."
cargo fmt -- --check

echo "Running clippy..."
cargo clippy -- -D warnings

echo "Running tests..."
cargo test --all

echo "All checks passed!"
```

---

## 12. 变更历史

| 版本 | 日期 | 变更人 | 变更描述 |
|------|------|--------|----------|
| v1.0.0 | 2026-03-01 | Workflow Toolkit Team | 初始版本 |

---

*本文档是 P3 级约束，通过自动化工具验证。所有代码必须遵循这些规范。*
