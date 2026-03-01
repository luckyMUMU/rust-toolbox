# Context 模块设计文档

## 1. 核心定义 (Stable)

### 1.1 模块职责

Context 模块为工作流执行提供数据传递机制，灵感来源于 LiteFlow 的 Slot 概念。核心职责：

- **类型安全的数据存储**：通过 `SlotValue` 包装器实现类型信息保留
- **作用域管理**：支持嵌套执行作用域（循环、条件分支）
- **线程安全**：全局槽位使用 `DashMap` 实现并发安全
- **节点隔离**：节点本地槽位私有化，避免数据污染

### 1.2 模块结构

```
src/workflow/context/
├── mod.rs      # DataContext 主结构及操作
└── slot.rs     # SlotValue 类型安全包装器
```

### 1.3 核心类型

#### DataContext

```rust
/// 数据上下文
/// 
/// 版本: v1.0 | 最后更新: 2026-02-26
/// 
/// 为组件提供数据传递机制，支持全局共享槽位和节点本地槽位。
/// 灵感来源于 LiteFlow 的 Slot 概念。
#[derive(Debug, Clone)]
pub struct DataContext {
    /// 全局共享槽位（线程安全，使用 DashMap 实现并发安全）
    global_slots: Arc<DashMap<String, SlotValue>>,
    
    /// 节点本地槽位（私有 HashMap，节点隔离）
    node_slots: HashMap<String, SlotValue>,
    
    /// 父节点链（用于嵌套作用域追踪，如循环/条件分支）
    parent_chain: Vec<String>,
    
    /// 当前节点 ID（如果处于节点执行上下文中）
    current_node: Option<String>,
}
```

**设计要点**：
- `global_slots` 使用 `Arc<DashMap>` 实现线程安全的全局共享，支持高并发读写
- `node_slots` 使用 `HashMap` 实现节点私有的本地存储，避免数据污染
- `parent_chain` 追踪嵌套作用域深度，用于调试和状态追踪
- `current_node` 标识当前执行的节点，便于上下文感知

#### SlotValue

```rust
pub struct SlotValue {
    value: Value,           // JSON 值
    type_name: String,      // 原始类型名（调试用）
}
```

**设计要点**：
- 内部使用 `serde_json::Value` 存储任意类型
- 保留原始类型名用于调试和错误提示
- 提供便捷的类型判断和提取方法

### 1.4 API 概览

| 分类 | 方法 | 说明 |
|------|------|------|
| 全局槽位 | `set_global` / `get_global` | 类型安全的全局存储 |
| 全局槽位 | `set_global_raw` / `get_global_raw` | 原始 JSON 值操作 |
| 全局槽位 | `has_global` / `remove_global` | 存在性检查与删除 |
| 节点槽位 | `set_node` / `get_node` | 节点本地存储 |
| 节点槽位 | `get_node_or_default` | 带默认值的获取 |
| 作用域 | `enter_scope` | 创建子作用域 |
| 作用域 | `scope_depth` | 获取当前深度 |
| 便捷方法 | `store_node_output` | 存储节点输出 |
| 便捷方法 | `get_node_output` | 获取节点输出 |
| 序列化 | `export_global_slots` / `import_global_slots` | 检查点支持 |

### 1.5 作用域机制

```
Workflow (depth=0)
├── global_slots: 共享
│
└── Loop Node (depth=1)
    ├── global_slots: 同一 Arc 引用（共享）
    ├── node_slots: 独立 HashMap
    └── parent_chain: ["loop_1"]
        │
        └── Nested Condition (depth=2)
            ├── global_slots: 共享
            ├── node_slots: 独立
            └── parent_chain: ["loop_1", "cond_1"]
```

---

## 2. 待实现方案 (In Progress)

### 2.1 决策记录

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 全局存储并发安全 | `DashMap` | 比 `RwLock<HashMap>` 更细粒度的锁，适合高并发读场景 |
| 值类型 | `serde_json::Value` | 通用性强，支持模板引擎直接使用 |
| 作用域实现 | 复制 parent_chain | 简单高效，避免复杂的生命周期管理 |

### 2.2 任务清单

- [ ] **性能优化**：评估 `DashMap` 在高并发场景下的内存占用
- [ ] **错误增强**：为 `SlotValue` 类型转换错误添加更详细的上下文信息
- [ ] **API 扩展**：考虑添加 `get_global_or` 方法支持默认值
- [ ] **文档完善**：添加更多使用示例到模块文档

### 2.3 待讨论

1. 是否需要支持槽位过期/清理机制？
2. 是否需要支持槽位变更监听（Observer 模式）？
3. `get_path` 方法是否应该支持更复杂的 JSONPath 语法？

---

## 3. 状态记录

### 3.1 实现状态

| 组件 | 状态 | 说明 |
|------|------|------|
| `SlotValue` | ✅ 完成 | 类型安全包装，支持路径访问 |
| `DataContext` 核心操作 | ✅ 完成 | 全局/节点槽位 CRUD |
| 作用域管理 | ✅ 完成 | `enter_scope` 实现 |
| 节点输出存储 | ✅ 完成 | 双重存储策略 |
| 序列化支持 | ✅ 完成 | 检查点导入导出 |
| 单元测试 | ✅ 完成 | 覆盖核心场景 |

### 3.2 变更历史

| 日期 | 变更内容 |
|------|----------|
| - | 初始实现：DataContext + SlotValue |
| - | 添加 `get_path` 支持嵌套访问 |
| - | 添加 `all_variables` 方法用于调试 |
| - | 添加 `get_node_or_default` 便捷方法 |

### 3.3 依赖关系

```
context
├── dashmap (外部依赖)
├── serde / serde_json (外部依赖)
└── crate::error (内部依赖)
```
