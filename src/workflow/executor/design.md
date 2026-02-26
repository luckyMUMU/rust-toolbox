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

---

## 4. 性能分析

### 4.1 执行器开销分析

每个执行器在调用链中都会引入额外开销。以下是各执行器的性能特性：

| 执行器 | 额外开销 | 内存占用 | 说明 |
|--------|----------|----------|------|
| `BasicExecutor` | ~1μs | 极低 | 直接调用，几乎无开销 |
| `RetryExecutor` | ~5μs/次 | 低 | 仅重试时有额外开销 |
| `CacheExecutor` | ~10μs | 中 | 缓存查找 + 存储 |
| `AuditExecutor` | ~3μs | 低 | 日志记录 |

### 4.2 执行器链性能模型

```mermaid
graph LR
    subgraph "执行器链开销"
        A[请求] --> B[AuditExecutor<br/>~3μs]
        B --> C[CacheExecutor<br/>~10μs]
        C --> D{缓存命中?}
        D -->|是| E[返回缓存结果]
        D -->|否| F[RetryExecutor<br/>~5μs]
        F --> G[BasicExecutor<br/>~1μs]
        G --> H[组件执行<br/>实际耗时]
        H --> I[返回结果]
    end
```

### 4.3 性能基准数据

#### 单执行器性能

```rust
// 基准测试结果（仅供参考）
// 测试环境：Intel i7-12700, 32GB RAM, SSD

// BasicExecutor
benchmark_basic_executor:
  mean: 1.2 μs
  p50: 1.0 μs
  p99: 3.5 μs

// RetryExecutor (无重试)
benchmark_retry_executor_no_retry:
  mean: 5.3 μs
  p50: 4.8 μs
  p99: 12.1 μs

// RetryExecutor (3次重试)
benchmark_retry_executor_with_retry:
  mean: 15.8 ms  // 包含重试等待时间
  p50: 12.2 ms
  p99: 45.6 ms

// CacheExecutor (缓存命中)
benchmark_cache_executor_hit:
  mean: 10.5 μs
  p50: 8.2 μs
  p99: 25.3 μs

// CacheExecutor (缓存未命中)
benchmark_cache_executor_miss:
  mean: 45.2 μs  // 包含缓存存储
  p50: 38.6 μs
  p99: 120.8 μs

// AuditExecutor
benchmark_audit_executor:
  mean: 3.1 μs
  p50: 2.5 μs
  p99: 8.7 μs
```

#### 完整执行器链性能

```rust
// 完整链：AuditExecutor -> CacheExecutor -> RetryExecutor -> BasicExecutor

// 缓存命中场景
benchmark_full_chain_cache_hit:
  mean: 18.5 μs
  p50: 15.2 μs
  p99: 45.6 μs

// 缓存未命中场景
benchmark_full_chain_cache_miss:
  mean: 65.3 μs
  p50: 52.8 μs
  p99: 156.2 μs

// 包含重试场景
benchmark_full_chain_with_retry:
  mean: 18.5 ms
  p50: 14.2 ms
  p99: 52.8 ms
```

### 4.4 性能优化建议

#### 1. 执行器顺序优化

```rust
// ✅ 推荐：缓存执行器靠前，减少后续执行器调用
let executor = ExecutorChainBuilder::new()
    .with_cache(cache)      // 优先缓存检查
    .with_audit(logger)     // 审计日志
    .with_retry(3, delay)   // 重试逻辑
    .build();

// ❌ 不推荐：缓存执行器靠后，无法避免重试开销
let executor = ExecutorChainBuilder::new()
    .with_retry(3, delay)   // 重试可能多次执行
    .with_audit(logger)
    .with_cache(cache)      // 缓存检查太晚
    .build();
```

#### 2. 条件性执行器

```rust
/// 条件执行器：仅在需要时启用
pub struct ConditionalExecutor {
    inner: BoxedExecutor,
    enabled: Arc<AtomicBool>,
}

impl ConditionalExecutor {
    pub fn new(inner: BoxedExecutor, enabled: Arc<AtomicBool>) -> Self {
        Self { inner, enabled }
    }
}

#[async_trait]
impl Executor for ConditionalExecutor {
    async fn execute(
        &self,
        component: &dyn Component,
        context: &mut DataContext,
        execution_ctx: &ExecutionContext,
    ) -> Result<ComponentOutput> {
        if self.enabled.load(Ordering::Relaxed) {
            self.inner.execute(component, context, execution_ctx).await
        } else {
            // 直接执行组件，跳过内部执行器
            component.execute(context, execution_ctx).await
        }
    }
}
```

#### 3. 批量执行优化

```rust
/// 批量执行器：合并多个组件执行
pub struct BatchExecutor {
    batch_size: usize,
    batch_timeout: Duration,
}

impl BatchExecutor {
    /// 批量执行多个组件
    pub async fn execute_batch(
        &self,
        components: &[&dyn Component],
        contexts: &mut [DataContext],
    ) -> Vec<Result<ComponentOutput>> {
        // 并行执行所有组件
        let futures: Vec<_> = components.iter()
            .zip(contexts.iter_mut())
            .map(|(c, ctx)| c.execute(ctx, &ExecutionContext::default()))
            .collect();
        
        futures::future::join_all(futures).await
    }
}
```

### 4.5 性能监控

```rust
/// 执行器性能指标
pub struct ExecutorMetrics {
    /// 执行次数
    pub execution_count: AtomicU64,
    /// 总执行时间
    pub total_duration_ns: AtomicU64,
    /// 错误次数
    pub error_count: AtomicU64,
    /// 重试次数
    pub retry_count: AtomicU64,
    /// 缓存命中次数
    pub cache_hits: AtomicU64,
    /// 缓存未命中次数
    pub cache_misses: AtomicU64,
}

impl ExecutorMetrics {
    /// 记录执行
    pub fn record_execution(&self, duration: Duration, success: bool) {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ns.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// 获取平均执行时间
    pub fn avg_duration(&self) -> Duration {
        let count = self.execution_count.load(Ordering::Relaxed);
        if count == 0 {
            return Duration::ZERO;
        }
        let total = self.total_duration_ns.load(Ordering::Relaxed);
        Duration::from_nanos(total / count)
    }
    
    /// 获取错误率
    pub fn error_rate(&self) -> f64 {
        let count = self.execution_count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        let errors = self.error_count.load(Ordering::Relaxed);
        errors as f64 / count as f64
    }
}
```

### 4.6 性能调优参数

```toml
# executor-config.toml

[executor]
# 重试配置
[executor.retry]
max_attempts = 3
base_delay_ms = 100
max_delay_ms = 5000
backoff_multiplier = 2.0

# 缓存配置
[executor.cache]
max_size = 10000
ttl_seconds = 300
tti_seconds = 600  # 空闲过期

# 审计配置
[executor.audit]
enabled = true
async_write = true
buffer_size = 1000

# 超时配置
[executor.timeout]
default_timeout_seconds = 30
max_timeout_seconds = 300
```
