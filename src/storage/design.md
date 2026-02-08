# 存储系统 (Storage System)

## 1. 核心定义 (Stable)

### 1.1 模块职责

存储系统负责工作流状态、执行记录和系统配置的持久化。支持多种存储后端，提供统一的存储接口。

### 1.2 模块结构

```
storage/
├── backends.rs     # 存储后端实现
├── state_manager.rs # 状态管理器
├── backup.rs       # 备份恢复
└── tests.rs        # 测试
```

### 1.3 核心组件

#### 存储后端

```rust
/// 存储后端（Storage Backend）trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 存储数据
    async fn store(&self, key: &str, value: Vec<u8>) -> Result<()>;
    
    /// 检索数据
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// 删除数据
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// 列出键
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    
    /// 检查存在
    async fn exists(&self, key: &str) -> Result<bool>;
}

/// 文件存储
pub struct FileStorage {
    base_path: PathBuf,
}

/// 内存存储
pub struct SimpleMemoryCache {
    data: Arc<DashMap<String, StorageRecord>>,
}

/// Redis 存储（可选）
pub struct RedisStorage;

/// LanceDB 存储（可选，feature = "lancedb"）
pub struct LanceDbStorage;
```

#### 状态管理器

```rust
/// 状态管理器
pub struct StateManager {
    storage: Arc<dyn StorageBackend>,
    cache: Arc<dyn CacheBackend>,
}

impl StateManager {
    /// 保存工作流状态
    pub async fn save_workflow_state(&self, state: &WorkflowState) -> Result<()>;
    
    /// 加载工作流状态
    pub async fn load_workflow_state(&self, workflow_id: &Uuid) -> Result<Option<WorkflowState>>;
    
    /// 保存执行记录
    pub async fn save_execution(&self, execution: &ExecutionRecord) -> Result<()>;
    
    /// 加载执行记录
    pub async fn load_execution(&self, execution_id: &str) -> Result<Option<ExecutionRecord>>;
    
    /// 获取执行统计
    pub async fn get_execution_statistics(&self) -> Result<ExecutionStatistics>;
}

/// 执行统计
pub struct ExecutionStatistics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: u64,
}
```

#### 备份恢复

```rust
/// 备份管理器
pub struct BackupManager {
    storage: Arc<dyn StorageBackend>,
    config: BackupConfig,
}

impl BackupManager {
    /// 创建备份
    pub async fn create_backup(&self, backup_type: BackupType) -> Result<BackupMetadata>;
    
    /// 恢复备份
    pub async fn restore(&self, backup_id: &str) -> Result<RestoreResult>;
    
    /// 列出备份
    pub async fn list_backups(&self) -> Result<Vec<BackupMetadata>>;
    
    /// 验证备份
    pub async fn verify_backup(&self, backup_id: &str) -> Result<BackupVerification>;
}

/// 备份类型
pub enum BackupType {
    Full,       // 全量备份
    Incremental, // 增量备份
    Differential, // 差异备份
}

/// 备份元数据
pub struct BackupMetadata {
    pub id: String,
    pub backup_type: BackupType,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
}
```

### 1.4 存储记录

```rust
/// 存储记录
pub struct StorageRecord {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ttl: Option<Duration>,
}

/// 保留策略
pub struct RetentionPolicy {
    pub max_age: Option<Duration>,
    pub max_records: Option<usize>,
    pub backup_before_delete: bool,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-S001: 存储后端抽象
- **决策**: 使用 trait 抽象存储后端
- **理由**: 支持多种后端，便于测试和扩展
- **风险**: 接口设计需要平衡通用性和性能

#### ADR-S002: 缓存策略
- **决策**: 两级缓存（内存 + 持久化）
- **理由**: 读多写少场景性能优化
- **风险**: 一致性（Consistency）问题

#### ADR-S003: 备份策略
- **决策**: 支持全量/增量/差异备份
- **理由**: 平衡备份时间和存储空间
- **风险**: 恢复复杂度增加

### 2.2 任务清单

- [x] Task 0: 基础存储接口
- [x] Task 1: 文件存储实现
- [x] Task 2: 内存缓存实现
- [x] Task 3: Redis 后端
- [x] Task 4: 备份系统完善
- [ ] Task 5: 数据迁移工具

### 2.3 接口契约

```rust
/// 存储配置
pub struct StorageConfig {
    pub backend_type: StorageBackendType,
    pub file_path: Option<PathBuf>,
    pub redis_url: Option<String>,
    pub cache_enabled: bool,
    pub cache_size: usize,
}

pub enum StorageBackendType {
    File,
    Memory,
    Redis,
    #[cfg(feature = "lancedb")]
    LanceDb,
}

/// 存储结果
pub struct StoreResult {
    pub key: String,
    pub bytes_written: usize,
    pub duration_ms: u64,
}
```

### 2.4 测试策略

- **单元测试（Unit Test）**: 各后端独立测试
- **集成测试（Integration Test）**: 状态管理器完整流程
- **性能测试（Performance Test）**: 读写性能基准
- **故障测试（Fault Tolerance Test）**: 备份恢复测试

## 3. 状态记录

- `[已完成]` | 备份系统完善 | 2026-02-07
- `[已完成]` | 文件存储 | 2026-01-28
- `[已完成]` | 内存缓存 | 2026-01-25
- `[已完成]` | Redis 后端 | 2026-02-07

## 4. 数据模型

```
存储键设计：
- workflow:{workflow_id} -> WorkflowDefinition
- execution:{execution_id} -> ExecutionRecord
- state:{workflow_id} -> WorkflowState
- checkpoint:{execution_id}:{sequence} -> Checkpoint
- backup:{backup_id} -> BackupData
```
