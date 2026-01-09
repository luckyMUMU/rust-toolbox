# src/performance/ - Performance Optimization

## OVERVIEW
Caching, concurrency control, memory optimization, metrics, and profiling.

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

## CONCURRENCY
**ConcurrencyConfig**: Controls parallelism
- **Max Workflows**: Concurrent workflow limit
- **Semaphore**: Resource limiting
- **Backpressure**: Queue management

**Control Structures**:
- `Arc<Semaphore>` for execution limits
- `DashMap` for concurrent access
- `RwLock` for mutable shared state

## MEMORY
**MemoryConfig**: Optimization settings
- **Allocation limits**: Per-workflow caps
- **Garbage collection**: Periodic cleanup
- **Leak detection**: Debug mode

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
```

## OPTIMIZATION TIPS
1. **Cache**: Use appropriate TTL for data freshness
2. **Concurrency**: Tune semaphore limits for workload
3. **Memory**: Monitor allocation patterns
4. **Profiling**: Run before optimization attempts
