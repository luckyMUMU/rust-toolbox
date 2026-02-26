# Executor 模块设计文档

> 模块路径: `src/workflow/executor/`

---

## 1. 核心定义 (Stable)

### 1.1 模块职责

Executor 层负责组件执行的横切关注点，采用**责任链模式**实现可组合的执行器链。

核心职责：
- 封装组件执行逻辑
- 提供可插拔的横切关注点（重试、缓存、审计）
- 支持执行器链的灵活组合

### 1.2 架构设计

执行器链调用顺序：

```text
AuditExecutor -> CacheExecutor -> RetryExecutor -> BasicExecutor -> Component
```

每个执行器可：
1. 执行前置逻辑（如记录开始时间）
2. 委托给下一个执行器
3. 执行后置逻辑（如缓存结果、记录日志）

### 1.3 核心类型

| 类型 | 文件 | 说明 |
|------|------|------|
| `Executor` trait | [mod.rs:55-78](mod.rs#L55-L78) | 执行器抽象，定义 `execute` 方法 |
| `BoxedExecutor` | [mod.rs:81](mod.rs#L81) | `Arc<dyn Executor>` 类型别名 |
| `ExecutorChainBuilder` | [mod.rs:94-141](mod.rs#L94-L141) | 执行器链构建器，支持链式配置 |

### 1.4 子模块结构

| 模块 | 结构体 | 职责 |
|------|--------|------|
| [basic.rs](basic.rs) | `BasicExecutor` | 基础执行器，直接调用组件 `execute` 方法 |
| [retry.rs](retry.rs) | `RetryExecutor` | 重试执行器，支持 Fixed/Linear/Exponential 退避策略 |
| [cache.rs](cache.rs) | `CacheExecutor` | 缓存执行器，基于 `moka::future::Cache` 缓存结果 |
| [audit.rs](audit.rs) | `AuditExecutor` | 审计执行器，记录执行开始/完成/错误事件 |

### 1.5 关键接口

```rust
#[async_trait]
pub trait Executor: Send + Sync {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput>;

    fn name(&self) -> &str { "Executor" }
}
```

### 1.6 使用示例

```rust
let executor = ExecutorChainBuilder::new()
    .with_retry(3, Duration::from_secs(1))
    .with_cache(cache)
    .with_audit(audit_logger)
    .build();

let output = executor.execute(component, &mut context, &exec_ctx).await?;
```

---

## 2. 待实现方案 (In Progress)

### 2.1 决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| - | 使用责任链模式 | 横切关注点可独立实现、灵活组合 |
| - | 使用 `moka` 缓存库 | 高性能异步缓存，支持 TTL/TTI |
| - | 审计日志独立于 tracing | 合规需求，需要持久化存储 |

### 2.2 任务清单

- [x] `BasicExecutor` - 基础执行器实现
- [x] `RetryExecutor` - 重试执行器实现（含多种退避策略）
- [x] `CacheExecutor` - 缓存执行器实现（含 CacheBuilder）
- [x] `AuditExecutor` - 审计执行器实现
- [x] `ExecutorChainBuilder` - 链式构建器实现
- [ ] 指标收集执行器 `MetricsExecutor`
- [ ] 超时执行器 `TimeoutExecutor`
- [ ] 熔断执行器 `CircuitBreakerExecutor`

---

## 3. 状态记录

### 3.1 实现状态

| 组件 | 状态 | 测试覆盖 |
|------|------|----------|
| BasicExecutor | ✅ 完成 | ✅ 有测试 |
| RetryExecutor | ✅ 完成 | ✅ 有测试 |
| CacheExecutor | ✅ 完成 | ✅ 有测试 |
| AuditExecutor | ✅ 完成 | ⚠️ 无测试 |

### 3.2 依赖关系

```text
executor/
├── mod.rs          → component, context, core::ExecutionContext
├── basic.rs        → mod.rs (Executor trait)
├── retry.rs        → mod.rs (BoxedExecutor, Executor)
├── cache.rs        → mod.rs, moka::future::Cache
└── audit.rs        → mod.rs, AuditLogger, chrono
```

### 3.3 变更历史

| 日期 | 变更内容 |
|------|----------|
| - | 初始实现：BasicExecutor, RetryExecutor, CacheExecutor, AuditExecutor |
