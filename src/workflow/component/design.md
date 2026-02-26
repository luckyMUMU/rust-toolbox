# Component 模块设计文档

> 模块路径: `src/workflow/component`
> 设计原则: LiteFlow "一切皆组件" - 所有逻辑都是组件

---

## 1. 核心定义 (Stable)

### 1.1 模块职责

Component 模块是工作流系统的核心抽象层，负责：

- 定义统一的组件接口 (`Component` trait)
- 提供组件类型枚举 (`ComponentType`)
- 管理组件执行状态 (`ComponentStatus`)
- 封装组件执行输出 (`ComponentOutput`)
- 实现组件注册与发现 (`ComponentRegistry`)

### 1.2 模块结构

```
src/workflow/component/
├── mod.rs          # 核心定义与 trait
├── tool.rs         # Tool 组件实现
├── parallel.rs     # Parallel 组件实现
└── registry.rs     # 组件注册中心
```

### 1.3 核心类型

#### ComponentType 枚举

| 变体 | 用途 | 说明 |
|------|------|------|
| `Tool` | 工具执行 | 执行已注册的工具 |
| `Condition` | 条件判断 | 表达式求值，用于分支 |
| `Loop` | 循环迭代 | 集合遍历或条件循环 |
| `Parallel` | 并行执行 | 多节点并发执行 |
| `Switch` | 多路分支 | 基于表达式值的多分支选择 |
| `Checkpoint` | 检查点 | 保存执行状态用于恢复 |

#### ComponentStatus 枚举

| 变体 | 含义 |
|------|------|
| `Success` | 执行成功 |
| `Failure(String)` | 执行失败，包含错误信息 |
| `Skip` | 跳过执行（如条件为 false） |
| `Break` | 跳出循环 |
| `Continue` | 继续下一次循环迭代 |

#### ComponentOutput 结构

```rust
pub struct ComponentOutput {
    pub status: ComponentStatus,      // 执行状态
    pub next_nodes: Vec<String>,      // 动态确定的下一节点
    pub result: Option<Value>,        // 执行结果值
    pub metadata: HashMap<String, Value>,  // 附加元数据
}
```

**便捷构造方法**:
- `success()` - 成功输出
- `success_with_result(Value)` - 带结果的成功输出
- `success_with_next(Vec<String>)` - 指定下一节点的成功输出
- `failure(impl Into<String>)` - 失败输出
- `skip()` - 跳过输出

#### Component Trait

```rust
#[async_trait]
pub trait Component: Send + Sync {
    fn id(&self) -> &str;                                    // 唯一标识
    fn component_type(&self) -> ComponentType;               // 组件类型
    async fn execute(&self, context: &mut DataContext, 
                     execution_ctx: &ExecutionContext) -> Result<ComponentOutput>;  // 执行逻辑
    fn validate(&self) -> Result<()>;                        // 配置验证
    fn cacheable(&self) -> bool;                             // 是否可缓存
    fn description(&self) -> Option<&str>;                   // 描述信息
}
```

### 1.4 设计原则

1. **单一职责**: 每个组件只处理自己的执行逻辑
2. **开闭原则**: 通过 trait 扩展新组件类型，无需修改核心代码
3. **依赖倒置**: 工作流引擎依赖抽象 trait，而非具体实现
4. **线程安全**: `Component: Send + Sync` 确保异步安全

---

## 2. 待实现方案 (In Progress)

### 2.1 决策记录

| 日期 | 决策 | 理由 | 状态 |
|------|------|------|------|
| - | 使用 async_trait 支持异步执行 | 工作流需要 I/O 操作 | 已采纳 |
| - | ComponentOutput 包含动态 next_nodes | 支持运行时决定分支 | 已采纳 |
| - | 默认 cacheable 返回 false | 安全优先，避免缓存问题 | 已采纳 |

### 2.2 任务清单

#### 待实现组件

- [ ] `ConditionComponent` - 条件判断组件
- [ ] `LoopComponent` - 循环组件
- [ ] `SwitchComponent` - 多路分支组件
- [ ] `CheckpointComponent` - 检查点组件

#### 功能增强

- [ ] 组件执行超时控制
- [ ] 组件执行重试机制
- [ ] 组件执行指标收集 (metrics)
- [ ] 组件缓存策略实现

#### 测试覆盖

- [ ] ComponentOutput 单元测试
- [ ] ComponentStatus 状态转换测试
- [ ] 各组件类型的集成测试

---

## 3. 状态记录

### 3.1 变更历史

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-02-26 | v1.0 | 初始设计文档，基于 mod.rs 实际代码 |

### 3.2 当前状态

- **稳定性**: Stable (核心定义)
- **完成度**: 
  - 核心类型: 100%
  - Tool 组件: 已实现
  - Parallel 组件: 已实现
  - Registry: 已实现
  - 其他组件: 待实现

### 3.3 依赖关系

```
Component trait
    ├── crate::core::ExecutionContext (执行上下文)
    ├── crate::error::Result (错误处理)
    ├── crate::workflow::context::DataContext (数据上下文)
    └── serde_json::Value (JSON 值类型)
```

---

## 4. 参考资料

- LiteFlow 设计理念: "一切皆组件"
- 项目 SOP: [AGENT_SOP.md](../../../sop/AGENT_SOP.md)
