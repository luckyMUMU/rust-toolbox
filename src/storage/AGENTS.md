# src/storage/ - Storage Layer

## OVERVIEW
Storage backends, state management, backup system, and persistence.

## BACKENDS
**StorageBackend** (trait):
- `save(key, data)` - Persist data
- `load(key)` - Retrieve data
- `delete(key)` - Remove data

**Implementations**:
- **FileStorage**: File-based persistence
- **SimpleMemoryCache**: In-memory cache (moka-based)

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

## BACKUP SYSTEM
**BackupManager**: Automated backup handling
- **Interval**: Configurable (default 24h)
- **Retention**: Configurable days (default 30)
- **Compression**: Gzip support
- **Restoration**: Recovery from backup

**Backup Format**:
- Timestamped snapshots
- Metadata included
- Integrity verification

## USAGE
```rust
use workflow_toolkit::storage::{FileStorage, StateManager, SimpleMemoryCache};

let storage = Arc::new(FileStorage::new("./data/storage")?);
let cache = Arc::new(SimpleMemoryCache::new());
let state_manager = Arc::new(StateManager::new(storage, cache));
```

## PERFORMANCE
- **Moka Cache**: High-performance LRU cache
- **Async I/O**: Non-blocking file operations
- **Batching**: Efficient bulk operations
- **Metrics**: Cache hit/miss tracking

## TESTING
Comprehensive test suite in `tests.rs`:
- Backend consistency tests
- Concurrency tests
- Backup/restore tests
- Error handling tests
