# 基础设施层 (Infrastructure Layer)

## 1. 核心定义 (Stable)

### 1.1 层职责

基础设施层提供技术实现，包括数据持久化、外部服务调用、缓存、配置管理等。该层实现领域层定义的端口（Port），依赖领域层。

### 1.2 模块结构

```
infrastructure/
├── persistence/    # 数据持久化
│   ├── repository/    # 仓储实现
│   └── storage/       # 存储后端
├── plugin/         # 插件运行时
├── cache/          # 缓存实现
├── external/       # 外部服务
└── config/         # 配置管理
```

### 1.3 持久化实现

#### 仓储实现

```rust
/// 工作流仓储实现
pub struct WorkflowRepositoryImpl {
    storage: Arc<dyn StorageBackend>,
}

#[async_trait]
impl WorkflowRepository for WorkflowRepositoryImpl {
    async fn save(&self, workflow: &WorkflowDefinition) -> Result<()> {
        let data = serde_json::to_vec(workflow)?;
        self.storage.store(&workflow.id.to_string(), data).await
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<WorkflowDefinition>> {
        match self.storage.retrieve(&id.to_string()).await? {
            Some(data) => {
                let workflow = serde_json::from_slice(&data)?;
                Ok(Some(workflow))
            }
            None => Ok(None),
        }
    }
    // ...
}
```

#### 存储后端

```rust
/// 存储后端 trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
}

/// 文件存储实现
pub struct FileStorage {
    base_path: PathBuf,
}

/// 内存存储实现
pub struct MemoryStorage {
    data: Arc<DashMap<String, Vec<u8>>>,
}
```

### 1.4 插件运行时

```rust
/// 运行时管理器
pub struct RuntimeManager {
    python_runtime: Option<PythonRuntime>,
    nodejs_runtime: Option<NodeJsRuntime>,
    docker_runtime: Option<DockerRuntime>,
}

/// 插件加载器
pub struct PluginLoader {
    runtime_manager: Arc<RuntimeManager>,
    security_policy: SecurityPolicy,
}
```

### 1.5 缓存实现

```rust
/// 缓存后端
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

/// 内存缓存
pub struct LocalMemoryCache {
    data: Arc<DashMap<String, CacheEntry>>,
}
```

### 1.6 配置管理

```rust
/// 配置管理器
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    pub fn load(path: &Path) -> Result<Self>;
    pub fn get(&self) -> Config;
    pub async fn reload(&self) -> Result<()>;
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-I001: 存储后端抽象
- **决策**: 使用 trait 抽象存储后端，支持多种实现
- **理由**: 便于测试（Mock）和扩展（Redis、S3 等）
- **风险**: 性能开销，需要仔细设计接口

#### ADR-I002: 插件沙箱机制
- **决策**: 使用进程隔离 + 资源限制实现沙箱
- **理由**: 安全性优先，进程级隔离最可靠
- **风险**: 启动开销，需要进程池优化

#### ADR-I003: 缓存策略
- **决策**: 多级缓存（内存 + 可选 Redis）
- **理由**: 平衡性能和一致性
- **风险**: 缓存一致性问题

### 2.2 任务清单

- [x] Task 0: 基础存储实现
- [x] Task 1: 文件存储后端
- [x] Task 2: Redis 存储后端
- [x] Task 3: 插件运行时完善
- [ ] Task 4: 缓存策略优化

### 2.3 接口契约

```rust
/// 基础设施配置
pub struct InfrastructureConfig {
    pub storage: StorageConfig,
    pub cache: CacheConfig,
    pub plugin: PluginRuntimeConfig,
}

/// 存储配置
pub struct StorageConfig {
    pub backend_type: StorageBackendType,
    pub file_path: Option<PathBuf>,
    pub redis_url: Option<String>,
}

pub enum StorageBackendType {
    File,
    Memory,
    Redis,
}
```

### 2.4 测试策略

- **集成测试**: 真实存储后端测试
- **契约测试**: 验证端口实现符合契约
- **性能测试**: 存储和缓存性能基准

## 3. 状态记录

- `[已完成]` | Redis 后端实现 | 2026-02-07
- `[已完成]` | 文件存储实现 | 2026-01-28
- `[已完成]` | 内存缓存实现 | 2026-01-26
- `[已完成]` | 插件运行时完善 | 2026-02-07

## 4. 依赖关系

```
infrastructure/
├── 依赖: domain (领域层)
│   ├── port::WorkflowRepository
│   ├── port::ExecutionRepository
│   └── port::ToolRegistry
├── 依赖: workflow (工作流引擎)
│   ├── WorkflowEngine trait 实现
│   ├── Component 系统
│   └── CheckpointManager
└── 被依赖: 无（最底层）
    └── 通过 DI 容器注入到上层
```

### 4.1 子模块索引

- [工作流引擎](../workflow/design.md) - 工作流执行引擎实现

## 5. 外部依赖

| 依赖 | 用途 | 版本 |
|-----|------|------|
| tokio | 异步运行时 | 1.x |
| serde | 序列化 | 1.x |
| dashmap | 并发 HashMap | 5.x |
| chrono | 时间处理 | 0.4.x |
| tracing | 日志追踪 | 0.1.x |
