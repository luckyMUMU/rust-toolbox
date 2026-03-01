# 架构原则 (Architecture Principles)

> **版本**: v1.0.0  
> **创建日期**: 2026-03-01  
> **最后更新**: 2026-03-01  
> **状态**: Active  
> **级别**: P0 级（不可违背）

---

## 1. 概述

本文档定义了 Workflow Toolkit 项目的架构原则，这些是**不可违背**的设计指导原则。所有架构决策和代码实现必须遵循这些原则。

---

## 2. 核心架构原则

### 2.1 领域驱动设计 (DDD)

**原则**: 采用 DDD 分层架构，实现业务逻辑与技术实现的分离

**分层结构**:
```
┌─────────────────────────────────────┐
│   接口层 (Interfaces)                │
│   CLI | TUI | MCP Server            │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   应用层 (Application)               │
│   UseCase | Service | Workflow      │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   领域层 (Domain)                    │
│   Entity | ValueObject | Port       │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│   基础设施层 (Infrastructure)        │
│   Repository 实现 | Plugin | Cache  │
└─────────────────────────────────────┘
```

**依赖规则**:
- 接口层 → 应用层 → 领域层 ← 基础设施层
- 外层依赖内层，内层不依赖外层
- 领域层是核心，不依赖任何外层

**验证方式**:
- 架构审查
- 依赖分析工具

### 2.2 单一职责原则 (SRP)

**原则**: 一个模块/类/方法只对一种变化原因负责

**示例**:
```rust
// ❌ 违反 SRP：一个结构体负责太多职责
pub struct WorkflowExecutor {
    // 执行逻辑
    // 缓存逻辑
    // 日志逻辑
    // 重试逻辑
}

// ✅ 遵循 SRP：职责分离
pub struct WorkflowExecutor {
    cache: Arc<CacheMiddleware>,
    logger: Arc<LoggingMiddleware>,
    retry: Arc<RetryMiddleware>,
}
```

### 2.3 开闭原则 (OCP)

**原则**: 对扩展开放，对修改关闭

**示例**:
```rust
// ✅ 遵循 OCP：通过 trait 扩展新插件类型
pub trait Plugin: Send + Sync {
    fn initialize(&mut self) -> Result<()>;
    fn execute(&self, input: Value) -> Result<Value>;
}

// 新增插件类型无需修改现有代码
pub struct PythonPlugin { /* ... */ }
pub struct NodeJsPlugin { /* ... */ }
```

### 2.4 依赖倒置原则 (DIP)

**原则**: 高层模块不依赖低层模块，二者都依赖抽象

**示例**:
```rust
// ✅ 遵循 DIP
// 领域层定义抽象接口
pub trait ToolRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Tool>>;
}

// 基础设施层实现接口
pub struct InMemoryToolRepository {
    // ...
}

impl ToolRepository for InMemoryToolRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Tool>> {
        // ...
    }
}
```

---

## 3. 模块设计原则

### 3.1 高内聚低耦合

**原则**: 模块内部高度相关，模块之间依赖最小化

**衡量标准**:
- 模块内方法操作相同数据
- 模块间通过接口通信
- 避免模块间循环依赖

### 3.2 接口隔离原则 (ISP)

**原则**: 使用多个专门的接口，不使用单一的总接口

**示例**:
```rust
// ❌ 违反 ISP：胖接口
pub trait Worker {
    fn execute(&self) -> Result<()>;
    fn cache(&self) -> Result<()>;
    fn log(&self) -> Result<()>;
    fn retry(&self) -> Result<()>;
}

// ✅ 遵循 ISP：接口分离
pub trait Executable {
    fn execute(&self) -> Result<()>;
}

pub trait Cacheable {
    fn cache(&self) -> Result<()>;
}

pub trait Loggable {
    fn log(&self) -> Result<()>;
}
```

### 3.3 最少知识原则 (Law of Demeter)

**原则**: 只与直接朋友通信，避免链式访问

**违规示例**:
```rust
// ❌ 违反 LoD：链式访问
let config = workflow.executor.context.config.timeout;
```

**正确做法**:
```rust
// ✅ 遵循 LoD：委托方法
impl Workflow {
    pub fn timeout(&self) -> Duration {
        self.executor.context.config.timeout
    }
}

let timeout = workflow.timeout();
```

---

## 4. 并发设计原则

### 4.1 共享状态最小化

**原则**: 优先使用消息传递，避免共享可变状态

**示例**:
```rust
// ✅ 推荐：使用通道传递消息
let (tx, rx) = mpsc::channel();
tx.send(message).await?;
let result = rx.recv().await?;

// ❌ 避免：共享可变状态
let shared_state = Arc<Mutex<State>>();
```

### 4.2 异步边界清晰

**原则**: 明确标识异步函数，避免同步 - 异步混用

**示例**:
```rust
// ✅ 清晰标识
pub async fn execute_workflow(&self) -> Result<()> {
    // 异步实现
}

pub fn execute_workflow_sync(&self) -> Result<()> {
    // 同步包装
    tokio::runtime::Handle::current().block_on(self.execute_workflow())
}
```

---

## 5. 错误处理原则

### 5.1 错误类型明确

**原则**: 使用明确的错误类型，避免泛型错误

**示例**:
```rust
// ✅ 推荐：明确的错误类型
pub enum WorkflowError {
    NotFound { workflow_id: String },
    Validation { message: String },
    Execution { node_id: String, reason: String },
}

// ❌ 避免：泛型错误
pub type Result<T> = std::result::Result<T, String>;
```

### 5.2 错误传播清晰

**原则**: 使用 `?` 操作符传播错误，避免隐藏错误

**示例**:
```rust
// ✅ 推荐：清晰传播
pub fn process(&self) -> Result<()> {
    let data = self.load_data()?;
    self.validate(&data)?;
    self.save(data)?;
    Ok(())
}
```

---

## 6. 测试设计原则

### 6.1 测试金字塔

**原则**: 遵循测试金字塔（单元测试 > 集成测试 > E2E 测试）

**比例**:
- 单元测试：70%
- 集成测试：20%
- E2E 测试：10%

### 6.2 测试独立性

**原则**: 测试用例必须独立，不依赖其他测试

**示例**:
```rust
// ✅ 推荐：独立测试
#[test]
fn test_create_user() {
    let user = create_test_user();
    // ...
}

#[test]
fn test_delete_user() {
    let user = create_test_user(); // 自己创建数据
    // ...
}
```

---

## 7. 文档设计原则

### 7.1 文档分层

**原则**: 遵循四级文档架构

```
L1: 核心概念层 (Concept)     - 价值与痛点
L2: 逻辑流转层 (Workflow)    - 伪代码描述流程
L3: 技术规格层 (Spec)        - 接口契约与数据模型
L4: 决策参考层 (Decision)    - 架构决策记录
```

### 7.2 代码即文档

**原则**: 代码本身应该是自文档化的

**示例**:
```rust
// ✅ 推荐：自文档化代码
pub struct WorkflowExecutor {
    pub max_concurrent_workflows: usize,
    pub default_timeout: Duration,
}

// ❌ 避免：需要注释解释
pub struct Config {
    pub a: usize,  // 最大并发工作流数
    pub b: u64,    // 默认超时（毫秒）
}
```

---

## 8. 性能设计原则

### 8.1 零成本抽象

**原则**: 使用 Rust 的零成本抽象，避免不必要的运行时开销

**示例**:
```rust
// ✅ 推荐：零成本抽象
pub trait Component: Send + Sync {
    fn execute(&self, ctx: &mut Context) -> Result<()>;
}

// ❌ 避免：动态分发开销
pub struct ComponentBox {
    inner: Box<dyn std::any::Any>,
}
```

### 8.2 延迟优化

**原则**: 优化关键路径延迟，非关键路径可接受适度延迟

**关键路径**:
- 工作流提交
- 节点调度
- 状态查询

**优化目标**:
- 工作流提交延迟 < 10ms
- 节点调度延迟 < 1ms
- 状态查询延迟 < 5ms

---

## 9. 安全设计原则

### 9.1 默认安全

**原则**: 默认配置应该是安全的，不安全选项需要显式启用

**示例**:
```rust
// ✅ 推荐：默认安全
pub struct PluginConfig {
    pub sandbox_enabled: bool = true,  // 默认启用沙箱
    pub network_access: bool = false,  // 默认禁止网络
}
```

### 9.2 深度防御

**原则**: 多层防御，不依赖单一安全措施

**示例**:
```rust
// ✅ 推荐：多层验证
pub fn validate_path(path: &str) -> Result<PathBuf> {
    // 第一层：格式验证
    if !path.starts_with('/') {
        return Err(Error::InvalidPath);
    }
    
    // 第二层：长度验证
    if path.len() > 1024 {
        return Err(Error::PathTooLong);
    }
    
    // 第三层：路径遍历检查
    if path.contains("..") {
        return Err(Error::PathTraversal);
    }
    
    Ok(PathBuf::from(path))
}
```

---

## 10. 变更历史

| 版本 | 日期 | 变更人 | 变更描述 |
|------|------|--------|----------|
| v1.0.0 | 2026-03-01 | Workflow Toolkit Team | 初始版本 |

---

*本文档是 P0 级架构原则，所有架构决策和代码实现必须遵循这些原则。*
