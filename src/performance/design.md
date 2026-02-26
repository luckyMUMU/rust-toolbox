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
