# src/storage/ - Storage Layer

## OVERVIEW
Storage backends, state management, backup system, and persistence. 5 files with comprehensive storage capabilities.

## BACKENDS
**StorageBackend** (trait):
- `save(key, data)` - Persist data
- `load(key)` - Retrieve data
- `delete(key)` - Remove data
- `exists(key)` - Check existence
- `list()` - List all keys

**Implementations**:
- **FileStorage**: File-based persistence (129 lines)
  - Directory-based organization
  - Atomic file operations
  - Metadata support
  
- **SimpleMemoryCache**: In-memory cache (moka-based)
  - Basic HashMap wrapper
  - Thread-safe access
  
- **LocalMemoryCache**: High-performance moka cache (445 lines)
  - LRU eviction policy
  - TTL support
  - Metrics tracking
  
- **LanceDbStorage** (optional): Vector database backend
  - Feature-flagged (`lancedb`)
  - Advanced querying capabilities
  - Structured data storage

## STATE MANAGEMENT
**StateManager**: Orchestrates storage and caching
- **Persistence**: Long-term storage via backends
- **Caching**: Fast access via moka cache
- **TTL**: Time-to-live for cache entries
- **Checkpoint**: Periodic state snapshots

**Workflow State**:
- Execution records
- Node states
- Global variables
- Checkpoint data
- Audit logs

**ExecutionStatistics**:
```rust
pub struct ExecutionStatistics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub cancelled_executions: usize,
    pub success_rate: f64,
    pub total_duration: chrono::Duration,
    pub average_duration: Option<chrono::Duration>,
    pub min_duration: Option<chrono::Duration>,
    pub max_duration: Option<chrono::Duration>,
}
```

## BACKUP SYSTEM
**BackupManager**: Automated backup handling (785 lines)
- **Interval**: Configurable (default 24h)
- **Retention**: Configurable days (default 30)
- **Compression**: Gzip support
- **Restoration**: Recovery from backup
- **Verification**: Checksum validation

**Backup Format**:
```rust
pub struct BackupData {
    pub metadata: BackupMetadata,
    pub workflow_states: HashMap<WorkflowId, WorkflowState>,
    pub execution_records: Vec<ExecutionRecord>,
    pub system_data: HashMap<String, Value>,
}
```

**Backup Types**:
- `Full`: Complete system snapshot
- `Incremental`: Changes since last backup

**Backup Configuration**:
```rust
pub struct BackupConfig {
    pub backup_directory: PathBuf,
    pub compression_level: u32,
    pub include_execution_history: bool,
    pub max_backup_count: Option<usize>,
    pub retention_policy: RetentionPolicy,
    pub enable_incremental: bool,
    pub verify_backup: bool,
}
```

**Retention Policy**:
```rust
pub struct RetentionPolicy {
    pub max_age: chrono::Duration,
    pub max_count: Option<usize>,
}
```

## CACHING
**CacheConfig**:
```rust
pub struct CacheConfig {
    pub max_capacity: u64,
    pub ttl: Option<Duration>,
    pub enable_metrics: bool,
}
```

**Cache Backends**:
- **SimpleMemoryCache**: Basic in-memory cache
- **LocalMemoryCache**: High-performance moka cache
  - Async operations
  - TTL-based expiration
  - Size-based eviction
  - Metrics collection

**Cache Statistics**:
```rust
pub struct CacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
}
```

## RETENTION AND CLEANUP
**Automatic Cleanup**:
- Configurable cleanup interval
- Age-based retention
- Count-based retention
- Manual cleanup triggers

**Cleanup Strategies**:
- Remove old execution records
- Archive old workflows
- Delete expired checkpoints
- Clean up temporary files

## USAGE
```rust
use workflow_toolkit::storage::{FileStorage, StateManager, LocalMemoryCache, BackupManager};

// Create storage backend
let storage = Arc::new(FileStorage::new("./data/storage")?);

// Create cache
let cache_config = CacheConfig {
    max_capacity: 100_000_000, // 100MB
    ttl: Some(Duration::from_secs(3600)),
    enable_metrics: true,
};
let cache = Arc::new(LocalMemoryCache::new(cache_config));

// Create state manager
let state_manager = Arc::new(StateManager::new(storage, cache));

// Create backup manager
let backup_config = BackupConfig {
    backup_directory: "./backups".into(),
    compression_level: 6,
    include_execution_history: true,
    max_backup_count: Some(30),
    retention_policy: RetentionPolicy {
        max_age: chrono::Duration::days(30),
        max_count: Some(100),
    },
    enable_incremental: true,
    verify_backup: true,
};
let backup_manager = Arc::new(BackupManager::new(state_manager.clone(), backup_config));
```

## PERFORMANCE
- **Moka Cache**: High-performance LRU cache (445 lines)
- **Async I/O**: Non-blocking file operations
- **Batching**: Efficient bulk operations
- **Metrics**: Cache hit/miss tracking
- **Compression**: Optional gzip compression
- **Incremental Backups**: Only changed data

## ERROR HANDLING
**Storage Errors**:
- `NotFound`: Key not found
- `PermissionDenied`: Access denied
- `CorruptedData`: Data integrity check failed
- `SerializationError`: JSON/TOML serialization failed
- `IOError`: File system error
- `BackupError`: Backup operation failed

**Recovery Strategies**:
- Automatic retry with exponential backoff
- Fallback to previous backup
- Manual recovery options
- Data integrity verification

## TESTING
Comprehensive test suite:
- **Backend Consistency**: Verify data integrity across operations
- **Concurrency**: Test thread safety
- **Backup/Restore**: Full backup and restore cycles
- **Error Handling**: Edge cases and failure scenarios
- **Performance**: Benchmark cache and storage operations

**Test Coverage**:
- Unit tests for each backend
- Integration tests for state management
- Property-based tests for cache eviction
- Stress tests for concurrent access

## CONFIGURATION
**Default Settings**:
- Cache size: 100MB
- Backup interval: 24 hours
- Retention: 30 days or 100 backups
- Compression: Level 6 (gzip)
- TTL: 1 hour for cache entries

**Environment Variables**:
```bash
WORKFLOW_TOOLKIT_STORAGE__DATABASE_PATH="./data/workflow.db"
WORKFLOW_TOOLKIT_STORAGE__CACHE_SIZE=104857600  # 100MB
WORKFLOW_TOOLKIT_STORAGE__BACKUP_INTERVAL=86400  # 24h
WORKFLOW_TOOLKIT_STORAGE__RETENTION_DAYS=30
```

## BEST PRACTICES

### Storage
1. **Use Appropriate Backend**: File for simplicity, LanceDB for scale
2. **Atomic Operations**: Ensure data consistency
3. **Regular Backups**: Automate backup schedule
4. **Verify Integrity**: Checksum validation
5. **Monitor Size**: Prevent disk space issues

### Caching
1. **Set Appropriate TTL**: Balance freshness vs performance
2. **Monitor Hit Rate**: Optimize cache size
3. **Use Metrics**: Track cache performance
4. **Eviction Policy**: Choose based on access patterns
5. **Memory Limits**: Prevent memory exhaustion

### Backup
1. **Test Restores**: Regular restore testing
2. **Multiple Copies**: Keep offsite backups
3. **Incremental**: Use incremental for efficiency
4. **Verification**: Verify backup integrity
5. **Retention**: Balance storage cost vs history

## ARCHITECTURE

### Storage Hierarchy
```
StateManager
├── Cache (moka) - Fast access
│   ├── TTL-based expiration
│   ├── LRU eviction
│   └── Metrics tracking
└── Storage Backend - Persistent
    ├── FileStorage (default)
    ├── LanceDbStorage (optional)
    └── SimpleMemoryCache (test)
```

### Backup Flow
```
1. Trigger (interval or manual)
   ↓
2. Create BackupMetadata
   ↓
3. Snapshot Workflow States
   ↓
4. Export Execution Records
   ↓
5. Compress Data (gzip)
   ↓
6. Calculate Checksum
   ↓
7. Write to Backup Directory
   ↓
8. Verify Backup
   ↓
9. Apply Retention Policy
```

### Recovery Flow
```
1. Select Backup
   ↓
2. Verify Checksum
   ↓
3. Decompress Data
   ↓
4. Restore Workflow States
   ↓
5. Restore Execution Records
   ↓
6. Verify Integrity
   ↓
7. Resume Operations
```

## PERFORMANCE OPTIMIZATION

### Cache Optimization
- **Size Tuning**: Adjust based on working set size
- **TTL Tuning**: Balance freshness vs memory
- **Eviction Policy**: LRU for temporal locality
- **Metrics**: Monitor and adjust

### Storage Optimization
- **File Organization**: Directory structure for scalability
- **Batch Operations**: Group related operations
- **Compression**: Enable for large data
- **Indexing**: Use LanceDB for complex queries

### Backup Optimization
- **Incremental**: Only backup changes
- **Compression**: Balance CPU vs storage
- **Parallel**: Multi-threaded backup creation
- **Scheduling**: Off-peak hours

## MONITORING

### Metrics to Track
- **Cache Hit Rate**: Target > 90%
- **Storage Latency**: < 10ms for most operations
- **Backup Duration**: Should complete within window
- **Disk Usage**: Monitor growth rate
- **Error Rate**: Track failure frequency

### Alerts
- Cache hit rate < 80%
- Storage latency > 100ms
- Backup failure
- Disk space < 10%
- Error rate > 1%

## SEE ALSO

- [Root AGENTS.md](../../AGENTS.md) - Project overview
- [Workflow AGENTS.md](../workflow/AGENTS.md) - State usage
- [Performance AGENTS.md](../performance/AGENTS.md) - Caching details
- [Configuration](../../config/default.toml) - Storage settings
 
### 模块级补充细则
- 目标与范围
  - 本模块覆盖持久化、缓存、备份以及状态管理的接口、实现与测试，强调可观测性与稳定性。
- 设计与扩展
  - 新存储后端/缓存实现需提供明确契约、序列化接口及向后兼容性分析。
- 实现规范
  - 导入排序：std -> external -> crate，确保跨 crate 的引用可读性。
  - 命名规范、Rustdoc 注释、错误类型集中化、异步模式等应与全仓库风格保持一致。
- 测试策略
  - 覆盖 FileStorage、LanceDbStorage、SimpleMemoryCache、LocalMemoryCache 的单元测试和集成测试。
  - 回滚、恢复、备份场景的端到端测试。
- 变更与审阅
  - 变更应附带兼容性评估和回归测试方案。
- 文档与审阅
  - 模块级 AGENTS.md 变更需同步。
- 跨模块协作
  - 存储与工作流引擎、审计等其他模块的契约需要清晰化。
- Cursor/Copilot 规则
  - 将 Cursor/Copilot 规则合并到模块级 AGENTS.md 模板中，方便执行。
