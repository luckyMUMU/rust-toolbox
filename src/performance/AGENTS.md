# src/performance/ - Performance Optimization

## OVERVIEW
Caching, concurrency control, memory optimization, metrics, and profiling. 6 files with comprehensive optimization capabilities.

## CACHING
**Cache** (moka-based):
- **TTL**: Time-based expiration
- **LRU**: Least-recently-used eviction
- **Async**: Non-blocking operations
- **Metrics**: Hit/miss rates, size tracking

**ResultCache**: Workflow execution results
- Cache key generation
- Invalidation strategies
- Configurable TTL

**CacheConfig**:
```rust
pub struct CacheConfig {
    pub max_capacity: u64,
    pub ttl: Option<Duration>,
    pub enable_metrics: bool,
}
```

**Eviction Policies**:
- `LRU`: Least Recently Used
- `LFU`: Least Frequently Used
- `FIFO`: First In, First Out
- `TTL`: Time To Live based
- `Random`: Random eviction

## CONCURRENCY
**ConcurrencyConfig**: Controls parallelism
- **Max Workflows**: Concurrent workflow limit
- **Semaphore**: Resource limiting
- **Backpressure**: Queue management

**Control Structures**:
- `Arc<Semaphore>` for execution limits
- `DashMap` for concurrent access
- `RwLock` for mutable shared state

**Load Balancing Strategies**:
- `RoundRobin`: Distribute evenly
- `LeastConnections`: Least busy worker
- `WeightedRoundRobin`: Weighted distribution
- `ResourceBased`: Based on resource usage
- `Adaptive`: Dynamic adjustment

**Rejection Strategies**:
- `DropOldest`: Remove oldest task
- `DropNewest`: Remove newest task
- `Reject`: Return error immediately

## MEMORY
**MemoryConfig**: Optimization settings
- **Allocation limits**: Per-workflow caps
- **Garbage collection**: Periodic cleanup
- **Leak detection**: Debug mode

**MemoryOptimizationType**:
- `ReduceBufferSize`: Smaller buffers
- `EnablePooling`: Object pooling
- `IncreaseCleanupFrequency`: More frequent GC
- `OptimizeDataStructures`: Better data structures
- `ReduceCacheSize`: Smaller caches
- `CompressData`: Data compression
- `LazyLoading`: Load on demand
- `StreamProcessing`: Process in streams

**PressureLevel**:
- `Normal`: Low memory usage
- `Medium`: Moderate usage
- `High`: High usage, caution
- `Critical`: Critical, immediate action needed

## METRICS
**MetricsConfig**: Collection settings
- **Enabled**: Toggle metrics
- **Export**: Prometheus format (optional)
- **Tracking**: Execution times, resource usage

**MetricsCollector**:
- Execution duration
- Tool invocation counts
- Error rates
- Cache performance
- System metrics

**MetricType**:
- `Counter`: Monotonically increasing
- `Gauge`: Can go up and down
- `Histogram`: Distribution of values
- `Summary`: Quantile statistics

**Custom Metrics**:
```rust
pub struct CustomMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
}
```

## PROFILING
**Profiler**: Built-in performance analysis
- **Start/Stop**: Manual profiling
- **Report**: Detailed breakdown
- **Hotspots**: Identify bottlenecks
- **Flame graphs**: Visual representation

**ProfilingConfig**:
- **Enabled**: Toggle profiling
- **Sampling**: Frequency settings
- **Output**: File or stdout
- **CPU/Memory/IO**: Selective profiling

**Export Formats**:
- `Json`: Machine-readable
- `FlameGraph`: Visual representation
- `CallGraph`: Function call hierarchy

**Profile Data**:
```rust
pub struct ProfileData {
    pub session_id: String,
    pub component: String,
    pub duration: Duration,
    pub samples: Vec<ProfileSample>,
    pub call_stack: Vec<CallInfo>,
    pub memory_usage: MemoryProfile,
    pub cpu_usage: CpuProfile,
}
```

## PERFORMANCE MANAGER
**PerformanceManager**: Central coordinator
- **MemoryManager**: Memory optimization
- **ConcurrencyManager**: Concurrency control
- **MetricsCollector**: Metrics collection
- **CacheManager**: Cache management
- **Profiler**: Performance profiling

**PerformanceStats**:
```rust
pub struct PerformanceStats {
    pub execution_count: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_usage: MemoryUsage,
    pub concurrency_stats: ConcurrencyStats,
    pub cache_stats: CacheStats,
}
```

## OPTIMIZATION REPORTS
**OptimizationReport**: Generated recommendations
- Memory optimizations
- Concurrency optimizations
- Cache optimizations
- Timestamped for tracking

**RecommendationPriority**:
- `Low`: Nice to have
- `Medium`: Should implement
- `High`: Important
- `Critical`: Must implement

## USAGE
```rust
use workflow_toolkit::performance::{PerformanceManager, PerformanceConfig};

let config = PerformanceConfig::default();
let manager = PerformanceManager::new(config);

// Start profiling
manager.start_profiling();

// Execute workload
// ...

// Get report
let report = manager.get_profiling_report();

// Get optimization recommendations
let optimizations = manager.analyze_performance();
```

## MONITORING
**PerformanceMonitor**: Individual operation tracking
- Component-specific monitoring
- Start/stop timing
- Memory snapshotting
- Profile session management

**PerformanceResult**:
```rust
pub struct PerformanceResult {
    pub component: String,
    pub duration: Duration,
    pub memory_usage: MemoryUsage,
    pub profile_data: Option<ProfileData>,
}
```

## OPTIMIZATION TIPS
1. **Cache**: Use appropriate TTL for data freshness
2. **Concurrency**: Tune semaphore limits for workload
3. **Memory**: Monitor allocation patterns
4. **Profiling**: Run before optimization attempts
5. **Metrics**: Collect and analyze regularly
6. **Load Balancing**: Choose strategy based on workload
7. **Memory Pooling**: Reuse allocations for frequent operations
8. **Streaming**: Process large data in streams

## TESTING
**Performance Benchmarks**:
- Cache performance tests
- Concurrency stress tests
- Memory usage tests
- Profiling accuracy tests

**See**: `tests/AGENTS.md` for test details
