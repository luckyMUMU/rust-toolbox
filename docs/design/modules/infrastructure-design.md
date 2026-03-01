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

---

## 6. 监控指标与告警

### 6.1 核心监控指标

#### 6.1.1 工作流指标

| 指标名称 | 类型 | 描述 | 标签 |
|----------|------|------|------|
| `workflow_executions_total` | Counter | 工作流执行总数 | status, workflow_id |
| `workflow_execution_duration_seconds` | Histogram | 工作流执行时间 | workflow_id |
| `workflow_active_count` | Gauge | 当前活跃工作流数 | - |
| `workflow_node_executions_total` | Counter | 节点执行总数 | status, node_type |
| `workflow_node_duration_seconds` | Histogram | 节点执行时间 | node_type |
| `workflow_queue_depth` | Gauge | 工作流队列深度 | priority |

#### 6.1.2 工具指标

| 指标名称 | 类型 | 描述 | 标签 |
|----------|------|------|------|
| `tool_executions_total` | Counter | 工具执行总数 | status, tool_name, tool_type |
| `tool_execution_duration_seconds` | Histogram | 工具执行时间 | tool_name, tool_type |
| `tool_cache_hits_total` | Counter | 工具缓存命中数 | tool_name |
| `tool_cache_misses_total` | Counter | 工具缓存未命中数 | tool_name |
| `tool_registry_size` | Gauge | 工具注册表大小 | - |

#### 6.1.3 插件指标

| 指标名称 | 类型 | 描述 | 标签 |
|----------|------|------|------|
| `plugin_loads_total` | Counter | 插件加载总数 | status, plugin_name, plugin_type |
| `plugin_load_duration_seconds` | Histogram | 插件加载时间 | plugin_type |
| `plugin_executions_total` | Counter | 插件执行总数 | status, plugin_name |
| `plugin_process_pool_size` | Gauge | 进程池大小 | plugin_type |
| `plugin_process_pool_available` | Gauge | 可用进程数 | plugin_type |
| `plugin_memory_usage_bytes` | Gauge | 插件内存使用 | plugin_name |

#### 6.1.4 存储指标

| 指标名称 | 类型 | 描述 | 标签 |
|----------|------|------|------|
| `storage_operations_total` | Counter | 存储操作总数 | operation, backend, status |
| `storage_operation_duration_seconds` | Histogram | 存储操作时间 | operation, backend |
| `storage_cache_hits_total` | Counter | 缓存命中数 | backend |
| `storage_cache_misses_total` | Counter | 缓存未命中数 | backend |
| `storage_size_bytes` | Gauge | 存储大小 | backend |

#### 6.1.5 系统指标

| 指标名称 | 类型 | 描述 | 标签 |
|----------|------|------|------|
| `system_memory_usage_bytes` | Gauge | 内存使用 | type (heap, stack) |
| `system_cpu_usage_ratio` | Gauge | CPU 使用率 | core |
| `system_gc_pause_seconds` | Histogram | GC 暂停时间 | - |
| `system_goroutines_count` | Gauge | 协程数 | - |
| `system_open_fds` | Gauge | 打开的文件描述符数 | - |

### 6.2 告警规则定义

#### 6.2.1 严重告警（P0）

```yaml
# alerts-p0.yml
groups:
  - name: critical-alerts
    rules:
      - alert: WorkflowEngineDown
        expr: up{job="workflow-engine"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "工作流引擎不可用"
          description: "工作流引擎实例 {{ $labels.instance }} 已停止响应超过 1 分钟"

      - alert: HighErrorRate
        expr: |
          sum(rate(workflow_executions_total{status="failed"}[5m])) 
          / sum(rate(workflow_executions_total[5m])) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "工作流错误率过高"
          description: "工作流失败率超过 10%，当前值: {{ $value | humanizePercentage }}"

      - alert: MemoryExhaustion
        expr: system_memory_usage_bytes{type="heap"} / system_memory_limit_bytes > 0.9
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "内存即将耗尽"
          description: "堆内存使用率超过 90%，当前值: {{ $value | humanizePercentage }}"
```

#### 6.2.2 重要告警（P1）

```yaml
# alerts-p1.yml
groups:
  - name: important-alerts
    rules:
      - alert: HighLatency
        expr: |
          histogram_quantile(0.99, 
            sum(rate(workflow_execution_duration_seconds_bucket[5m])) by (le)
          ) > 30
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "工作流执行延迟过高"
          description: "P99 延迟超过 30 秒，当前值: {{ $value | humanizeDuration }}"

      - alert: PluginLoadFailure
        expr: rate(plugin_loads_total{status="failed"}[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "插件加载失败率过高"
          description: "插件加载失败率: {{ $value }}/s"

      - alert: CacheHitRateLow
        expr: |
          sum(rate(storage_cache_hits_total[5m])) 
          / (sum(rate(storage_cache_hits_total[5m])) + sum(rate(storage_cache_misses_total[5m]))) < 0.5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "缓存命中率过低"
          description: "缓存命中率低于 50%，当前值: {{ $value | humanizePercentage }}"

      - alert: ProcessPoolExhausted
        expr: plugin_process_pool_available / plugin_process_pool_size < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "进程池即将耗尽"
          description: "可用进程数低于 10%"
```

#### 6.2.3 一般告警（P2）

```yaml
# alerts-p2.yml
groups:
  - name: general-alerts
    rules:
      - alert: HighConcurrency
        expr: workflow_active_count > 100
        for: 10m
        labels:
          severity: info
        annotations:
          summary: "并发工作流数量较高"
          description: "当前活跃工作流数: {{ $value }}"

      - alert: SlowStorageOperations
        expr: |
          histogram_quantile(0.95, 
            sum(rate(storage_operation_duration_seconds_bucket[5m])) by (le, operation)
          ) > 1
        for: 10m
        labels:
          severity: info
        annotations:
          summary: "存储操作延迟较高"
          description: "{{ $labels.operation }} 操作 P95 延迟超过 1 秒"

      - alert: ToolCacheMissRateHigh
        expr: |
          sum(rate(tool_cache_misses_total[5m])) 
          / sum(rate(tool_executions_total[5m])) > 0.3
        for: 15m
        labels:
          severity: info
        annotations:
          summary: "工具缓存未命中率较高"
          description: "工具缓存未命中率: {{ $value | humanizePercentage }}"
```

### 6.3 告警阈值配置

| 指标 | 正常范围 | 警告阈值 | 严重阈值 | 处理建议 |
|------|----------|----------|----------|----------|
| 错误率 | < 1% | 1-5% | > 5% | 检查日志、排查错误 |
| P99 延迟 | < 10s | 10-30s | > 30s | 性能分析、扩容 |
| 内存使用 | < 70% | 70-85% | > 85% | 内存分析、扩容 |
| CPU 使用 | < 60% | 60-80% | > 80% | 扩容、优化 |
| 缓存命中率 | > 80% | 50-80% | < 50% | 调整缓存策略 |
| 队列深度 | < 100 | 100-500 | > 500 | 扩容、限流 |
| 进程池可用 | > 30% | 10-30% | < 10% | 扩大进程池 |

### 6.4 监控仪表盘配置

```json
{
  "dashboard": {
    "title": "Workflow Toolkit 监控仪表盘",
    "panels": [
      {
        "title": "工作流执行概览",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(workflow_executions_total[5m]))",
            "legendFormat": "执行速率"
          },
          {
            "expr": "sum(rate(workflow_executions_total{status=\"failed\"}[5m]))",
            "legendFormat": "失败速率"
          }
        ]
      },
      {
        "title": "执行延迟分布",
        "type": "heatmap",
        "targets": [
          {
            "expr": "sum(rate(workflow_execution_duration_seconds_bucket[5m])) by (le)",
            "format": "heatmap"
          }
        ]
      },
      {
        "title": "资源使用",
        "type": "gauge",
        "targets": [
          {
            "expr": "system_memory_usage_bytes{type=\"heap\"} / system_memory_limit_bytes * 100",
            "legendFormat": "内存使用 %"
          },
          {
            "expr": "avg(system_cpu_usage_ratio) * 100",
            "legendFormat": "CPU 使用 %"
          }
        ]
      },
      {
        "title": "插件状态",
        "type": "table",
        "targets": [
          {
            "expr": "plugin_process_pool_size",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

### 6.5 日志规范

#### 结构化日志格式

```json
{
  "timestamp": "2026-02-27T10:30:00.000Z",
  "level": "INFO",
  "target": "workflow_engine",
  "message": "Workflow execution started",
  "trace_id": "abc123",
  "span_id": "def456",
  "fields": {
    "workflow_id": "wf-001",
    "execution_id": "exec-001",
    "node_count": 10
  }
}
```

#### 日志级别使用规范

| 级别 | 使用场景 | 示例 |
|------|----------|------|
| ERROR | 需要立即处理的错误 | 工作流执行失败、插件崩溃 |
| WARN | 需要关注但不需要立即处理 | 重试成功、性能下降 |
| INFO | 重要的业务事件 | 工作流启动/完成、插件加载 |
| DEBUG | 调试信息 | 节点执行详情、参数值 |
| TRACE | 详细追踪 | 函数调用栈、变量状态 |

### 6.6 监控数据保留策略

| 数据类型 | 高精度保留 | 低精度保留 | 说明 |
|----------|------------|------------|------|
| 原始指标 | 7 天 | - | 秒级数据 |
| 聚合指标 | 30 天 | 1 年 | 分钟/小时级聚合 |
| 日志数据 | 7 天 | 30 天 | 压缩存储 |
| 追踪数据 | 3 天 | 7 天 | 采样存储 |
| 告警历史 | 30 天 | 1 年 | 用于趋势分析 |
