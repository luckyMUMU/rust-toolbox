# 性能模块 (Performance Module)

> **版本**: v1.0  
> **创建日期**: 2026-01-20  
> **最后更新**: 2026-02-27  
> **维护者**: Workflow Toolkit Team

## 1. 核心定义 (Stable)

### 1.1 模块职责

性能模块提供性能监控、优化和调优功能，包括内存管理、并发控制、缓存优化和性能分析。

### 1.2 模块结构

```
performance/
├── cache.rs        # 缓存管理
├── concurrency.rs  # 并发控制
├── memory.rs       # 内存管理
├── metrics.rs      # 指标收集
└── profiler.rs     # 性能分析
```

### 1.3 核心组件

#### 性能管理器

```rust
/// 性能管理器
pub struct PerformanceManager {
    config: Arc<RwLock<PerformanceConfig>>,
    memory_manager: Arc<MemoryManager>,
    concurrency_manager: Arc<ConcurrencyManager>,
    metrics_collector: Arc<MetricsCollector>,
    cache_manager: Arc<CacheManager>,
    profiler: Arc<Profiler>,
    stats: Arc<DashMap<String, PerformanceStats>>,
}

impl PerformanceManager {
    /// 创建性能管理器
    pub fn new(config: PerformanceConfig) -> Self;
    
    /// 获取性能统计
    pub fn get_stats(&self, component: &str) -> Option<PerformanceStats>;
    
    /// 更新配置
    pub async fn update_config(&self, config: PerformanceConfig) -> Result<()>;
}

/// 性能配置
pub struct PerformanceConfig {
    pub memory: MemoryConfig,
    pub concurrency: ConcurrencyConfig,
    pub cache: CacheConfig,
    pub metrics: MetricsConfig,
    pub profiling: ProfilingConfig,
}
```

#### 内存管理

```rust
/// 内存管理器
pub struct MemoryManager {
    config: MemoryConfig,
    usage: Arc<AtomicUsize>,
}

impl MemoryManager {
    /// 分配内存
    pub fn allocate(&self, size: usize) -> Result<MemoryAllocation>;
    
    /// 释放内存
    pub fn deallocate(&self, allocation: MemoryAllocation);
    
    /// 获取当前使用
    pub fn current_usage(&self) -> MemoryUsage;
    
    /// 触发垃圾回收（GC）
    pub async fn gc(&self) -> Result<()>;
}

/// 内存配置
pub struct MemoryConfig {
    pub max_memory_mb: usize,
    pub gc_threshold_percent: f64,
    pub allocation_limit_mb: usize,
}

/// 内存使用统计
pub struct MemoryUsage {
    pub used_bytes: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub fragmentation_percent: f64,
}
```

#### 并发管理

```rust
/// 并发管理器
pub struct ConcurrencyManager {
    config: ConcurrencyConfig,
    semaphores: DashMap<String, Arc<Semaphore>>,
}

impl ConcurrencyManager {
    /// 获取执行许可
    pub async fn acquire_permit(&self, resource: &str) -> Result<Permit>;
    
    /// 获取当前并发数
    pub fn current_concurrency(&self, resource: &str) -> usize;
    
    /// 调整限制
    pub fn adjust_limit(&self, resource: &str, new_limit: usize);
}

/// 并发配置
pub struct ConcurrencyConfig {
    pub max_concurrent_workflows: usize,
    pub max_concurrent_tasks: usize,
    pub max_concurrent_tools: usize,
}

/// 并发统计
pub struct ConcurrencyStats {
    pub current: usize,
    pub peak: usize,
    pub limit: usize,
    pub wait_time_avg_ms: u64,
}
```

#### 缓存管理

```rust
/// 缓存管理器
pub struct CacheManager {
    config: CacheConfig,
    caches: DashMap<String, Box<dyn Cache>>,
}

impl CacheManager {
    /// 获取缓存
    pub fn get_cache(&self, name: &str) -> Option<Arc<dyn Cache>>;
    
    /// 注册缓存
    pub fn register_cache(&self, name: &str, cache: Box<dyn Cache>);
    
    /// 清空所有缓存
    pub async fn clear_all(&self);
    
    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats;
}

/// 缓存配置
pub struct CacheConfig {
    pub default_ttl: Duration,
    pub max_size_mb: usize,
    pub eviction_policy: EvictionPolicy,
}

pub enum EvictionPolicy {
    Lru,    // 最近最少使用（Least Recently Used）
    Lfu,    // 最少使用频率（Least Frequently Used）
    Fifo,   // 先进先出（First In First Out）
    Random, // 随机
}

/// 缓存统计
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size_bytes: usize,
    pub entry_count: usize,
}
```

#### 指标收集

```rust
/// 指标收集器
pub struct MetricsCollector {
    config: MetricsConfig,
    metrics: Arc<DashMap<String, MetricValue>>,
}

impl MetricsCollector {
    /// 记录计数器
    pub fn counter(&self, name: &str, value: u64);
    
    /// 记录仪表盘
    pub fn gauge(&self, name: &str, value: f64);
    
    /// 记录直方图
    pub fn histogram(&self, name: &str, value: f64);
    
    /// 获取指标
    pub fn get_metric(&self, name: &str) -> Option<MetricValue>;
}

/// 指标配置
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval_secs: u64,
    pub retention_hours: u64,
}

pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}
```

#### 性能分析器

```rust
/// 性能分析器（Profiler）
pub struct Profiler {
    config: ProfilingConfig,
    sessions: Arc<DashMap<String, ProfileSession>>,
}

impl Profiler {
    /// 开始分析会话
    pub fn start_session(&self, name: &str) -> ProfileSession;
    
    /// 结束分析会话
    pub fn end_session(&self, session_id: &str) -> Result<ProfileReport>;
    
    /// 生成火焰图（Flamegraph）
    pub fn generate_flamegraph(&self, session_id: &str) -> Result<String>;
}

/// 分析配置
pub struct ProfilingConfig {
    pub enabled: bool,
    pub sample_rate: f64,
    pub max_depth: usize,
}

/// 分析报告
pub struct ProfileReport {
    pub session_id: String,
    pub duration_ms: u64,
    pub samples: Vec<Sample>,
    pub hotspots: Vec<Hotspot>,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-PE001: 内存管理策略
- **决策**: 使用显式内存分配跟踪 + 定期 GC（垃圾回收）
- **理由**: Rust 内存安全，但需要监控大对象
- **风险**: 跟踪开销

#### ADR-PE002: 并发控制
- **决策**: 使用信号量（Semaphore）进行细粒度控制
- **理由**: 精确控制资源使用
- **风险**: 死锁（Deadlock）风险

#### ADR-PE003: 缓存策略
- **决策**: 可插拔驱逐策略（LRU/LFU/FIFO）
- **理由**: 适应不同场景
- **风险**: 配置复杂度

### 2.2 任务清单

- [x] Task 0: 性能监控基础
- [x] Task 1: 内存跟踪
- [x] Task 2: 并发控制
- [x] Task 3: 高级缓存策略
- [ ] Task 4: 性能分析完善

### 2.3 接口契约

```rust
/// 性能统计
pub struct PerformanceStats {
    pub execution_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_usage: MemoryUsage,
    pub concurrency_stats: ConcurrencyStats,
    pub cache_stats: CacheStats,
    pub last_updated_timestamp: u64,
}
```

### 2.4 测试策略

- **基准测试（Benchmark Test）**: 各组件性能基准
- **压力测试（Stress Test）**: 高并发场景
- **内存测试（Memory Test）**: 内存泄漏（Memory Leak）检测

## 3. 状态记录

- `[进行中]` | 性能分析完善 | 2026-02-07
- `[已完成]` | 并发控制 | 2026-02-01
- `[已完成]` | 内存跟踪 | 2026-01-28
- `[已完成]` | 高级缓存策略 | 2026-02-07

## 4. 优化建议

### 内存优化
- 使用对象池（Object Pool）减少分配
- 大对象使用 Arena 分配
- 定期触发 GC（垃圾回收）

### 并发优化
- 动态调整并发限制
- 使用工作窃取队列（Work Stealing Queue）
- 优先级调度

### 缓存优化
- 多级缓存（L1/L2）
- 预加载热点数据
- 智能驱逐策略

---

## 5. 性能基准与优化目标

### 5.1 关键路径性能基准

#### 工作流执行关键路径

| 路径 | 描述 | 目标延迟 (P99) | 当前基线 | 优化目标 |
|------|------|----------------|----------|----------|
| 工作流启动 | 从请求到第一个节点执行 | < 50ms | 30ms | 20ms |
| 节点调度 | DAG 解析到节点就绪 | < 10ms | 8ms | 5ms |
| 组件执行 | 组件 execute 调用 | < 5ms | 3ms | 2ms |
| 状态保存 | 检查点持久化 | < 100ms | 80ms | 50ms |
| 工作流完成 | 最后节点到结果返回 | < 20ms | 15ms | 10ms |

#### 工具执行关键路径

| 路径 | 描述 | 目标延迟 (P99) | 当前基线 | 优化目标 |
|------|------|----------------|----------|----------|
| 工具查找 | 注册表查询 | < 1ms | 0.5ms | 0.3ms |
| 参数验证 | Schema 验证 | < 5ms | 3ms | 2ms |
| 工具执行 | 实际执行时间 | 取决于工具 | - | - |
| 结果处理 | 输出转换 | < 2ms | 1ms | 0.5ms |

#### 插件执行关键路径

| 路径 | 描述 | 目标延迟 (P99) | 当前基线 | 优化目标 |
|------|------|----------------|----------|----------|
| 插件加载 | 首次加载 | < 500ms | 400ms | 300ms |
| 进程获取 | 从池中获取进程 | < 10ms | 8ms | 5ms |
| IPC 调用 | 进程间通信 | < 20ms | 15ms | 10ms |
| 插件卸载 | 资源清理 | < 100ms | 80ms | 50ms |

### 5.2 吞吐量基准

#### 工作流吞吐量

| 场景 | 目标吞吐量 | 当前基线 | 测试条件 |
|------|------------|----------|----------|
| 简单工作流 | > 1000/s | 800/s | 5 节点线性流程 |
| 复杂工作流 | > 100/s | 80/s | 50 节点 DAG |
| 并行工作流 | > 500/s | 400/s | 10 节点并行 |
| 大规模工作流 | > 10/s | 8/s | 1000 节点 |

#### 工具吞吐量

| 场景 | 目标吞吐量 | 当前基线 | 测试条件 |
|------|------------|----------|----------|
| Native 工具 | > 10000/s | 8000/s | 无 IO 操作 |
| Python 工具 | > 500/s | 400/s | 简单计算 |
| Node.js 工具 | > 500/s | 350/s | 简单计算 |
| Docker 工具 | > 100/s | 80/s | 简单命令 |

### 5.3 资源使用基准

#### 内存使用

| 组件 | 目标内存 | 当前基线 | 限制 |
|------|----------|----------|------|
| 核心引擎 | < 50MB | 40MB | 100MB |
| 工具注册表 | < 10MB | 8MB | 20MB |
| 插件管理器 | < 30MB | 25MB | 50MB |
| 状态管理器 | < 20MB | 15MB | 50MB |
| 缓存系统 | < 100MB | 80MB | 可配置 |

#### CPU 使用

| 场景 | 目标 CPU | 当前基线 | 说明 |
|------|----------|----------|------|
| 空闲状态 | < 1% | 0.5% | 无工作流执行 |
| 单工作流 | < 10% | 8% | 简单工作流 |
| 高负载 | < 80% | 70% | 100 并发工作流 |
| 峰值处理 | < 95% | 90% | 短时峰值 |

### 5.4 性能优化策略

#### 5.4.1 内存优化

```rust
/// 对象池配置
pub struct ObjectPoolConfig {
    /// 初始容量
    pub initial_capacity: usize,
    /// 最大容量
    pub max_capacity: usize,
    /// 空闲超时
    pub idle_timeout: Duration,
}

/// 内存优化建议
pub struct MemoryOptimization {
    /// 对象池化：减少频繁分配
    pub object_pooling: bool,
    /// Arena 分配：大对象批量分配
    pub arena_allocation: bool,
    /// 零拷贝：避免不必要的数据复制
    pub zero_copy: bool,
    /// 内存预分配：预留空间
    pub pre_allocation: bool,
}
```

**优化效果预估**：

| 优化项 | 预期收益 | 实现复杂度 |
|--------|----------|------------|
| 对象池化 | 减少 30% 分配开销 | 中 |
| Arena 分配 | 减少 50% 大对象开销 | 高 |
| 零拷贝 | 减少 20% 内存复制 | 中 |
| 预分配 | 减少 10% 动态扩容 | 低 |

#### 5.4.2 并发优化

```rust
/// 并发优化配置
pub struct ConcurrencyOptimization {
    /// 工作窃取
    pub work_stealing: bool,
    /// 优先级队列
    pub priority_queue: bool,
    /// 动态调整
    pub dynamic_adjustment: bool,
    /// 批处理
    pub batching: bool,
}

/// 动态并发调整器
pub struct DynamicConcurrencyAdjuster {
    /// 当前并发限制
    current_limit: AtomicUsize,
    /// 最小限制
    min_limit: usize,
    /// 最大限制
    max_limit: usize,
    /// 调整间隔
    adjustment_interval: Duration,
}

impl DynamicConcurrencyAdjuster {
    /// 根据负载调整并发限制
    pub fn adjust(&self, metrics: &LoadMetrics) {
        let current_load = metrics.cpu_usage;
        let current_latency = metrics.p99_latency;
        
        if current_load < 0.5 && current_latency < self.target_latency {
            // 负载低，增加并发
            self.current_limit.fetch_add(1, Ordering::Relaxed);
        } else if current_load > 0.8 || current_latency > self.target_latency * 1.5 {
            // 负载高，减少并发
            self.current_limit.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
```

#### 5.4.3 缓存优化

```rust
/// 多级缓存配置
pub struct MultiLevelCacheConfig {
    /// L1 缓存（本地内存）
    pub l1: L1CacheConfig,
    /// L2 缓存（共享内存/Redis）
    pub l2: Option<L2CacheConfig>,
}

pub struct L1CacheConfig {
    /// 最大条目数
    pub max_entries: usize,
    /// 最大内存（MB）
    pub max_memory_mb: usize,
    /// TTL
    pub ttl: Duration,
    /// 驱逐策略
    pub eviction: EvictionPolicy,
}

/// 缓存预热策略
pub struct CacheWarmupStrategy {
    /// 启动时预热
    pub warmup_on_start: bool,
    /// 预热数据源
    pub warmup_sources: Vec<WarmupSource>,
    /// 预热并发数
    pub warmup_concurrency: usize,
}
```

### 5.5 性能测试配置

```toml
# benchmark-config.toml

[workflow]
# 工作流基准测试配置
node_counts = [5, 10, 50, 100, 500, 1000]
concurrency_levels = [1, 10, 50, 100]
duration_secs = 60

[tool]
# 工具基准测试配置
tool_types = ["native", "python", "nodejs", "docker"]
execution_counts = [100, 1000, 10000]
warmup_iterations = 100

[plugin]
# 插件基准测试配置
plugin_types = ["native", "python", "nodejs", "docker"]
pool_sizes = [2, 5, 10, 20]
lifecycle_tests = ["load", "execute", "unload"]

[monitoring]
# 监控配置
sample_interval_ms = 100
report_interval_secs = 10
metrics = ["latency", "throughput", "memory", "cpu", "gc"]
```

### 5.6 性能回归检测

```yaml
# .github/workflows/performance.yml
name: Performance Regression
on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # 每日凌晨 2 点

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run benchmarks
        run: |
          cargo bench --bench workflow_benchmark
          cargo bench --bench tool_benchmark
          cargo bench --bench plugin_benchmark
      - name: Compare with baseline
        run: |
          cargo install critcmp
          critcmp main baseline
      - name: Check regression
        run: |
          # 如果性能下降超过 10%，则失败
          ./scripts/check_regression.sh 10
```

### 5.7 性能优化路线图

| 阶段 | 目标 | 时间 | 关键指标 |
|------|------|------|----------|
| 第一阶段 | 建立基准 | 1 周 | 完成所有基准测试 |
| 第二阶段 | 内存优化 | 2 周 | 内存使用降低 20% |
| 第三阶段 | 并发优化 | 2 周 | 吞吐量提升 30% |
| 第四阶段 | 缓存优化 | 1 周 | 延迟降低 15% |
| 第五阶段 | 持续监控 | 持续 | 防止性能回归 |
