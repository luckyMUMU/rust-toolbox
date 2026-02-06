# 插件系统 (Plugin System)

## 1. 核心定义 (Stable)

### 1.1 模块职责

插件系统负责管理外部插件的生命周期，支持多种插件类型（Native、Python、Node.js、Docker）。提供统一的插件接口和沙箱执行环境。

### 1.2 模块结构

```
plugins/
├── manager.rs        # 插件管理器
├── runtime.rs        # 运行时管理
├── types.rs          # 插件类型定义
├── integration.rs    # 集成层
├── error.rs          # 错误处理
├── macros.rs         # 宏定义
├── native.rs         # Native 插件
├── python.rs         # Python 插件
├── nodejs.rs         # Node.js 插件
├── docker.rs         # Docker 插件
└── file_management/  # 文件管理插件
    ├── mod.rs
    ├── plugin.rs
    ├── classification/
    ├── batch/
    ├── text/
    └── ui/
```

### 1.3 核心类型

#### 插件 Trait

```rust
/// 插件核心 trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    
    /// 初始化插件
    async fn initialize(&mut self) -> PluginResult<()>;
    
    /// 执行工具
    async fn execute(&self, tool_name: &str, input: Value) -> PluginResult<Value>;
    
    /// 关闭插件
    async fn shutdown(&mut self) -> PluginResult<()>;
}

/// 插件类型
pub enum PluginType {
    Native,     // Rust 动态库（Dynamic Library）
    Python,     // Python 脚本
    NodeJs,     // Node.js 模块
    Wasm,       // WebAssembly（暂时禁用）
    Docker,     // Docker 容器
}
```

#### 插件配置

```rust
/// 插件配置
pub struct PluginConfig {
    pub name: String,
    pub plugin_type: PluginType,
    pub entry_point: String,
    pub resource_limits: ResourceLimits,
    pub security_policy: SecurityPolicy,
    pub metadata: HashMap<String, Value>,
}

/// 资源限制
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_cpu_percent: f64,
    pub max_execution_time_secs: u64,
    pub max_file_descriptors: usize,
}

/// 安全策略
pub struct SecurityPolicy {
    pub allow_network: bool,
    pub allow_file_read: Vec<String>,
    pub allow_file_write: Vec<String>,
    pub allowed_syscalls: Vec<String>,
}
```

### 1.4 插件管理器

```rust
/// 插件管理器
pub struct PluginManager {
    plugins: Arc<DashMap<String, Box<dyn Plugin>>>,
    runtime_manager: Arc<RuntimeManager>,
}

impl PluginManager {
    /// 加载插件
    pub async fn load(&self, config: PluginConfig) -> PluginResult<()>;
    
    /// 卸载插件
    pub async fn unload(&self, name: &str) -> PluginResult<()>;
    
    /// 重新加载插件
    pub async fn reload(&self, name: &str) -> PluginResult<()>;
    
    /// 获取插件
    pub fn get(&self, name: &str) -> Option<Ref<String, Box<dyn Plugin>>>;
    
    /// 列出所有插件
    pub fn list_all(&self) -> Vec<PluginInfo>;
}
```

### 1.5 运行时管理

```rust
/// 运行时管理器
pub struct RuntimeManager {
    python_runtime: Option<Arc<PythonRuntime>>,
    nodejs_runtime: Option<Arc<NodeJsRuntime>>,
    docker_runtime: Option<Arc<DockerRuntime>>,
    pool_config: RuntimePoolConfig,
}

/// 运行时池配置
pub struct RuntimePoolConfig {
    pub min_processes: usize,
    pub max_processes: usize,
    pub idle_timeout_secs: u64,
}

/// 运行时统计
pub struct RuntimeStats {
    pub active_processes: usize,
    pub idle_processes: usize,
    pub total_executions: u64,
    pub failed_executions: u64,
}
```

### 1.6 各类型插件实现

#### Native 插件

```rust
/// Native 插件
pub struct NativePlugin {
    library: Library,
    metadata: PluginMetadata,
}

/// Native 插件构建器
pub struct NativePluginBuilder;
```

#### Python 插件

```rust
/// Python 插件
pub struct PythonPlugin {
    runtime: Arc<PythonRuntime>,
    module_path: PathBuf,
    config: PythonRuntimeConfig,
}

/// Python 运行时配置
pub struct PythonRuntimeConfig {
    pub python_path: String,
    pub virtual_env: Option<PathBuf>,
    pub requirements_file: Option<PathBuf>,
}
```

#### Node.js 插件

```rust
/// Node.js 插件
pub struct NodeJsPlugin {
    runtime: Arc<NodeJsRuntime>,
    package_json: PackageJson,
    config: NodeJsRuntimeConfig,
}

/// Node.js 运行时配置
pub struct NodeJsRuntimeConfig {
    pub node_path: String,
    pub npm_registry: Option<String>,
    pub install_dependencies: bool,
}
```

#### Docker 插件

```rust
/// Docker 插件
pub struct DockerPlugin {
    runtime: Arc<DockerRuntime>,
    image: String,
    config: DockerRuntimeConfig,
}

/// Docker 运行时配置
pub struct DockerRuntimeConfig {
    pub image: String,
    pub mounts: Vec<DockerMount>,
    pub network: DockerNetworkConfig,
    pub resource_limits: DockerResourceLimits,
}
```

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)

#### ADR-P001: 插件隔离机制
- **决策**: 进程级隔离（Process Isolation） + 资源限制
- **理由**: 安全性优先，进程隔离最可靠
- **风险**: 启动开销，使用进程池（Process Pool）优化

#### ADR-P002: 插件类型支持
- **决策**: 优先支持 Native/Python/Node.js/Docker
- **理由**: 覆盖主流场景，WASM 暂时禁用（依赖问题）
- **风险**: WASM 生态发展可能错过机会

#### ADR-P003: 运行时池化
- **决策**: 使用进程池（Process Pool）管理外部运行时
- **理由**: 减少启动开销，提高响应速度
- **风险**: 内存占用增加

### 2.2 任务清单

- [x] Task 0: 插件管理器基础
- [x] Task 1: Native 插件支持
- [x] Task 2: Python 插件支持
- [x] Task 3: Node.js 插件支持
- [x] Task 4: Docker 插件支持
- [ ] Task 5: WASM 插件恢复
- [ ] Task 6: 运行时池优化

### 2.3 接口契约

```rust
/// 插件加载结果
pub struct PluginLoadResult {
    pub plugin_name: String,
    pub success: bool,
    pub load_time_ms: u64,
    pub error: Option<String>,
}

/// 插件执行结果
pub struct PluginExecuteResult {
    pub success: bool,
    pub output: Option<Value>,
    pub execution_time_ms: u64,
    pub resource_usage: ResourceUsage,
}

/// 资源使用统计
pub struct ResourceUsage {
    pub memory_mb: usize,
    pub cpu_percent: f64,
    pub execution_time_ms: u64,
}
```

### 2.4 测试策略

- **单元测试（Unit Test）**: 各类型插件独立测试
- **集成测试（Integration Test）**: 插件加载/执行/卸载完整流程
- **安全测试（Security Test）**: 沙箱（Sandbox）隔离有效性
- **性能测试（Performance Test）**: 进程池性能基准

## 3. 状态记录

- `[进行中]` | 运行时池优化 | 2026-02-06
- `[已完成]` | Docker 插件支持 | 2026-02-01
- `[已完成]` | Node.js 插件支持 | 2026-01-28
- `[已完成]` | Python 插件支持 | 2026-01-25
- `[已完成]` | Native 插件支持 | 2026-01-20

## 4. 插件生命周期

```
1. 配置验证
2. 运行时准备（进程池）
3. 插件加载
4. 初始化调用
5. 工具执行（多次）
6. 关闭调用
7. 资源清理
```

## 5. 子模块

- [文件管理插件](./file_management/design.md) - 文件管理功能插件
