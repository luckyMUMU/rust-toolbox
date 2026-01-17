# Workflow Toolkit - 开发指南

## 快速开始

### 环境要求

```bash
# Rust 1.70+ (2021 Edition)
rustc --version  # 应 >= 1.70.0

# Cargo
cargo --version

# 可选：用于插件开发
python3 --version  # >= 3.8
node --version     # >= 16
docker --version   # 任意版本
```

### 安装和构建

```bash
# 1. 克隆项目
git clone <repository>
cd rust-tool-v2

# 2. 快速类型检查
cargo check

# 3. 调试构建
cargo build

# 4. 生产构建
cargo build --release

# 5. 带LanceDB支持
cargo build --all-features
```

### 运行测试

```bash
# 所有测试
cargo test

# 特定模块测试
cargo test workflow::validator::tests
cargo test tools::version::tests
cargo test storage::tests

# 带输出
cargo test -- --nocapture

# 集成测试
cargo test --test integration_tests

# 性能基准
cargo test --release performance
```

## 开发工作流

### 1. 代码风格

#### 导入顺序 (严格遵循)
```rust
// 1. 标准库
use std::sync::Arc;
use std::time::Duration;

// 2. 外部依赖 (字母顺序)
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Semaphore};

// 3. 内部模块
use crate::core::{ExecutionContext, WorkflowId};
use crate::error::{Result, WorkflowError};
use crate::tools::ToolNode;
```

#### 错误处理
```rust
// ✅ 正确: 使用thiserror构造函数
pub fn do_something() -> Result<()> {
    let value = operation().map_err(|e| {
        WorkflowError::workflow_execution(&format!("Failed: {}", e))
    })?;
    Ok(())
}

// ✅ 正确: 常见错误构造器
return Err(WorkflowError::tool("Invalid parameters"));
return Err(WorkflowError::plugin("Loading failed"));
return Err(WorkflowError::storage("Connection lost"));

// ❌ 禁止: 类型转换
let x: u32 = value as any;           // 禁止
#[ts-ignore]                         // 禁止
let x = value as u32;                // 应使用 try_into()

// ❌ 禁止: 空错误处理
catch(e) {}                          // 禁止

// ❌ 禁止: unwrap()
let value = some_result.unwrap();    // 禁止 - 使用 ?
```

#### 异步模式
```rust
// ✅ 正确: 异步trait
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn execute(&self, def: WorkflowDefinition) -> Result<WorkflowExecution>;
}

// ✅ 正确: 并发集合
use dashmap::DashMap;              // 读密集型并发访问
use tokio::sync::RwLock;           // 可变共享状态

// ✅ 正确: 信号量控制并发
let semaphore = Arc::new(Semaphore::new(4));
let permit = semaphore.acquire().await?;
```

#### 测试模式
```rust
// ✅ 正确: 单元测试在同文件
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        // 测试逻辑
    }
    
    #[tokio::test]
    async fn test_async_something() {
        // 异步测试逻辑
    }
}

// ✅ 正确: 使用tempfile隔离
#[tokio::test]
async fn test_with_temp_dir() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    // 使用 temp_dir.path()
}
```

### 2. 项目结构理解

```
rust-tool-v2/
├── src/                          # 核心库
│   ├── lib.rs                    # 模块声明
│   ├── core.rs                   # 共享类型
│   ├── error.rs                  # 错误处理
│   ├── config.rs                 # 配置管理
│   │
│   ├── workflow/                 # DAG工作流引擎
│   │   ├── engine.rs             # 执行引擎
│   │   ├── scheduler.rs          # DAG调度器
│   │   ├── validator.rs          # 验证器
│   │   ├── execution_manager.rs  # 执行管理
│   │   ├── audit.rs              # 审计日志
│   │   └── result_cache.rs       # 结果缓存
│   │
│   ├── tools/                    # 工具系统
│   │   ├── registry.rs           # 工具注册表
│   │   ├── node.rs               # 工具节点
│   │   ├── templates.rs          # 参数模板
│   │   ├── versioning.rs         # 版本管理
│   │   └── dependency.rs         # 依赖解析
│   │
│   ├── plugins/                  # 插件系统
│   │   ├── manager.rs            # 插件管理器
│   │   ├── native.rs             # 原生插件
│   │   ├── python.rs             # Python插件
│   │   ├── nodejs.rs             # Node.js插件
│   │   ├── docker.rs             # Docker插件
│   │   └── file_management/      # 文件管理插件
│   │       ├── classifier.rs     # AI分类
│   │       ├── batch_processor.rs # 批处理
│   │       └── text_processor.rs # 文本处理
│   │
│   ├── interfaces/               # 用户接口
│   │   ├── cli/                  # CLI (clap)
│   │   │   ├── app.rs            # 主应用
│   │   │   ├── commands.rs       # 命令定义
│   │   │   └── output.rs         # 输出格式化
│   │   └── tui/                  # TUI (ratatui)
│   │       ├── app.rs            # TUI应用
│   │       ├── event.rs          # 事件处理
│   │       ├── widgets/          # 组件系统
│   │       ├── theme.rs          # 主题系统
│   │       └── performance.rs    # 性能优化
│   │
│   ├── storage/                  # 持久化层
│   │   ├── state_manager.rs      # 状态管理
│   │   ├── backends.rs           # 存储后端
│   │   ├── cache.rs              # 缓存系统
│   │   └── backup.rs             # 备份恢复
│   │
│   └── performance/              # 性能优化
│       ├── cache.rs              # 缓存管理
│       ├── metrics.rs            # 指标收集
│       ├── profiler.rs           # 性能分析
│       └── concurrency.rs        # 并发控制
│
├── tests/                        # 测试套件
│   ├── integration_tests.rs      # 端到端测试
│   ├── tui_standalone_unit_tests.rs # TUI单元测试
│   ├── file_management_integration_tests.rs # 文件管理测试
│   └── template_property_tests.rs # 属性测试
│
├── examples/                     # 示例
│   ├── comprehensive_workflow_example.rs
│   ├── python_plugin_example.rs
│   ├── file_management_example.rs
│   └── templates/                # 工作流模板
│
├── config/                       # 默认配置
│   └── default.toml
│
├── docs/                         # 文档
│   └── AGENTS.md                 # 模块文档
│
├── openspec/                     # 规范驱动开发
│   ├── AGENTS.md                 # OpenSpec说明
│   └── specs/                    # 能力规范
│
└── Cargo.toml                    # 项目配置
```

### 3. 关键组件交互

#### 工作流执行流程
```
1. 用户输入 (CLI/TUI/MCP)
   ↓
2. 配置加载 (ConfigManager)
   ↓
3. 工作流定义 (WorkflowDefinition)
   ↓
4. 验证器 (WorkflowValidator)
   - 检查DAG结构
   - 验证工具存在
   - 检查循环依赖
   ↓
5. 调度器 (DagScheduler)
   - 拓扑排序
   - 确定执行顺序
   - 识别并行节点
   ↓
6. 执行引擎 (DefaultWorkflowEngine)
   - 信号量控制并发
   - 节点执行
   - 错误处理
   - 检查点保存
   ↓
7. 状态管理 (StateManager)
   - 持久化执行状态
   - 缓存结果
   ↓
8. 审计日志 (AuditLogger)
   - 记录所有操作
   - 错误详情
   ↓
9. 输出结果 (CLI/TUI/MCP)
```

#### 插件加载流程
```
1. 配置文件/自动发现
   ↓
2. PluginManager.load_plugin()
   ↓
3. RuntimeManager.create_runtime()
   ↓
4. 根据类型加载:
   - Native: libloading加载.so/.dll
   - Python: 启动Python进程
   - Node.js: 启动Node进程
   - Docker: 创建容器
   - WASM: 加载wasm模块
   ↓
5. 验证符号/接口
   ↓
6. 注册工具到ToolRegistry
   ↓
7. 工作流可使用插件工具
```

### 4. 开发任务示例

#### 添加新工具节点
```rust
// 1. 在 src/tools/node.rs 定义工具
pub struct MyTool {
    info: ToolInfo,
    executor: Arc<dyn ToolExecutor>,
}

#[async_trait]
impl ToolNode for MyTool {
    fn name(&self) -> &str { &self.info.name }
    fn version(&self) -> &str { &self.info.version }
    
    fn validate_parameters(&self, params: &Value) -> Result<()> {
        // 验证逻辑
        Ok(())
    }
    
    async fn execute(
        &self,
        params: Value,
        context: ExecutionContext,
    ) -> Result<Value> {
        // 执行逻辑
        Ok(result)
    }
    
    fn get_info(&self) -> &ToolInfo { &self.info }
}

// 2. 在 src/tools/registry.rs 注册
let tool = MyTool::builder()
    .name("my_tool")
    .version("1.0.0")
    .executor(executor)
    .build()?;

registry.register_tool(Arc::new(tool))?;

// 3. 在工作流中使用
// workflow.yaml
nodes:
  - id: my_step
    tool: my_tool
    params:
      input: "data"
```

#### 添加新插件类型
```rust
// 1. 在 src/plugins/types.rs 定义插件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Native,
    Python,
    NodeJs,
    Docker,
    WASM,
    Custom(String),  // 新增自定义类型
}

// 2. 在 src/plugins/manager.rs 实现加载逻辑
impl PluginManager {
    pub async fn load_plugin(
        &mut self,
        plugin: Box<dyn Plugin>,
        config: PluginConfig,
    ) -> Result<()> {
        match config.plugin_type {
            PluginType::Custom(ref plugin_type) => {
                // 自定义加载逻辑
            }
            // ... 其他类型
        }
    }
}

// 3. 在 src/plugins/mod.rs 导出
pub mod custom;
```

#### 添加新接口
```rust
// 1. 在 src/interfaces/ 创建新接口模块
// src/interfaces/web.rs
pub struct WebInterface {
    app: WebApp,
    server: HttpServer,
}

#[async_trait]
pub trait WebInterfaceTrait {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

// 2. 实现接口
impl WebInterface {
    pub fn new(config: WebConfig, components: Components) -> Self {
        // 初始化
    }
}

// 3. 在 src/interfaces/mod.rs 导出
pub mod web;

// 4. 在 src/main.rs 添加命令
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令
    Web {
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}
```

### 5. 调试和优化

#### 启用详细日志
```bash
RUST_LOG=debug cargo run -- workflow execute example.yaml
RUST_LOG=workflow_toolkit=debug cargo test
```

#### 性能分析
```rust
// 使用内置性能分析器
use workflow_toolkit::performance::Profiler;

let profiler = Profiler::new();
profiler.start_session("workflow_execution");

// 执行工作流...

let report = profiler.generate_report();
println!("{:?}", report.hotspots);
```

#### 内存监控
```rust
use workflow_toolkit::performance::MemoryManager;

let memory_manager = MemoryManager::new();
let snapshot = memory_manager.get_snapshot();
println!("Peak usage: {} bytes", snapshot.peak_bytes);
```

### 6. 常见问题解决

#### 编译错误
```bash
# 问题: 类型不匹配
# 解决: 使用 try_into() 而非 as
let count: usize = value.try_into()?;

# 问题: 生命周期错误
# 解决: 使用 Arc 共享所有权
let shared = Arc::new(data);

# 问题: 异步trait问题
# 解决: 添加 #[async_trait] 属性
#[async_trait]
pub trait MyTrait { ... }
```

#### 运行时错误
```bash
# 问题: 工具未找到
# 解决: 检查工具注册
cargo run -- tool list

# 问题: 工作流验证失败
# 解决: 检查DAG结构
cargo run -- workflow create --validate-only file.yaml

# 问题: 插件加载失败
# 解决: 检查配置和权限
RUST_LOG=debug cargo run -- plugin list
```

#### 测试失败
```bash
# 问题: 并发测试不稳定
# 解决: 使用单线程
cargo test -- --test-threads=1

# 问题: 临时文件冲突
# 解决: 使用 tempfile::TempDir
let temp_dir = tempfile::TempDir::new()?;
```

### 7. 最佳实践

#### 1. 始终使用Result类型
```rust
// ✅ 好
fn process() -> Result<Value> {
    let data = load_data()?;
    Ok(transform(data))
}

// ❌ 坏
fn process() -> Value {
    let data = load_data().unwrap();
    transform(data)
}
```

#### 2. 避免克隆不必要的数据
```rust
// ✅ 好
let registry_clone = Arc::clone(&registry);

// ❌ 坏
let registry_clone = registry.clone();
```

#### 3. 使用并发集合
```rust
// ✅ 好 (读密集)
use dashmap::DashMap;
let map = DashMap::new();

// ✅ 好 (写密集)
use tokio::sync::RwLock;
let state = RwLock::new(State::new());
```

#### 4. 添加测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // 单元测试
    }
    
    #[tokio::test]
    async fn test_async_feature() {
        // 异步测试
    }
    
    proptest! {
        #[test]
        fn test_property(input: Vec<String>) {
            // 属性测试
        }
    }
}
```

#### 5. 文档化代码
```rust
/// 执行工作流定义
///
/// # Arguments
/// * `definition` - 工作流定义
///
/// # Returns
/// 执行结果或错误
///
/// # Examples
/// ```
/// let result = engine.execute_workflow(definition).await?;
/// ```
pub async fn execute_workflow(
    &self,
    definition: WorkflowDefinition,
) -> Result<WorkflowExecution> {
    // 实现
}
```

### 8. 性能优化检查清单

- [ ] 使用 `cargo check` 快速验证
- [ ] 运行 `cargo clippy -- -D warnings`
- [ ] 所有测试通过
- [ ] 无unwrap()在生产代码
- [ ] 使用Arc::clone()而非clone()
- [ ] 使用并发集合(DashMap/RwLock)
- [ ] 异步I/O操作
- [ ] 批量操作减少I/O
- [ ] 缓存热点数据
- [ ] 避免不必要的内存分配

### 9. 提交前检查

```bash
# 1. 代码格式化
cargo fmt

# 2. Lint检查
cargo clippy -- -D warnings

# 3. 运行测试
cargo test

# 4. 类型检查
cargo check

# 5. 构建验证
cargo build --release

# 6. 文档生成
cargo doc --open
```

### 10. 学习资源

#### 核心文档
- **AGENTS.md**: 各模块的开发指南
- **DESIGN.md**: 架构设计文档
- **README.md**: 项目概述

#### 设计文档
- `src/workflow/DESIGN.md` - 工作流引擎设计
- `src/plugins/DESIGN.md` - 插件系统设计
- `src/tools/DESIGN.md` - 工具系统设计
- `src/storage/DESIGN.md` - 存储层设计
- `src/interfaces/cli/DESIGN.md` - CLI设计

#### 外部资源
- [Rust异步编程](https://rust-lang.github.io/async-book/)
- [Tokio文档](https://tokio.rs/)
- [Clap文档](https://clap.rs/)
- [Ratatui文档](https://ratatui.rs/)
- [Petgraph文档](https://docs.rs/petgraph)

---

## 总结

本指南提供了完整的开发流程，从环境搭建到代码编写，从测试到优化。遵循这些实践将帮助你：

1. **快速上手**: 理解项目结构和核心概念
2. **编写高质量代码**: 遵循最佳实践和代码规范
3. **有效调试**: 使用工具和日志定位问题
4. **性能优化**: 识别和解决性能瓶颈
5. **团队协作**: 统一的代码风格和开发流程

记住：**始终阅读AGENTS.md文件**，它们包含每个模块的具体开发指导！
