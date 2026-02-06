# Workflow Toolkit 软件约束规范 (SPEC)

> **版本**: v1.0.0  
> **日期**: 2026-02-05

---

## 1. 领域驱动设计规范 (DDD)

### 1.1 分层架构

```
接入层 (Interface) → 应用层 (Application) → 领域层 (Domain) → 基础设施层 (Infrastructure)
```

### 1.2 核心约束

| 类型 | 约束 |
|------|------|
| **实体** | DDD-E001: 必须有唯一标识符；DDD-E002: 相等性基于 id；DDD-E003: 封装业务逻辑 |
| **值对象** | DDD-V001: 不可变；DDD-V002: 相等性基于所有属性；DDD-V003: 方法返回新实例 |
| **聚合根** | DDD-A001: 聚合唯一入口；DDD-A003: 维护一致性；DDD-A004: 包含乐观锁版本 |
| **领域事件** | DDD-DE001: 不可变；DDD-DE002: 包含事件 ID、时间戳、聚合根 ID |
| **仓库** | DDD-R001: 接口在领域层；DDD-R002: 实现在基础设施层；DDD-R004: 包含乐观锁检查 |

### 1.3 依赖规则

```rust
// 允许: 上层依赖下层
src/interfaces/ -> src/application/ -> src/domain/

// 禁止: 下层依赖上层、同层跨模块直接依赖、领域层依赖基础设施
src/domain/ -> X -> src/application/
src/plugins/ -> X -> src/workflow/ (必须通过 domain/port)
src/domain/ -> X -> src/infrastructure/
```

---

## 2. 软件约束规范 (SPEC)

### 2.1 架构约束

- AC-001: 严禁跨层调用，必须通过公开接口
- AC-002: 领域层不得依赖其他层
- AC-003: 基础设施层通过 Port 接口与领域层交互

### 2.2 接口契约

| 接口 | 约束 |
|------|------|
| WorkflowEngine | CE-001: 幂等；CE-002: 并发限制；CE-003: 超时返回 Timeout 状态 |
| Tool | CT-001: 线程安全；CT-002: 执行时间限制；CT-003: 资源使用限制 |

### 2.3 数据模型约束

- CD-001: id 必须符合 `^[a-zA-Z][a-zA-Z0-9_-]*$`
- CD-002: nodes 不能为空
- CD-003: edges 不能形成环 (DAG)
- CD-004: edge.source/target 必须对应存在的 node.id

### 2.4 行为约束

- CP-001: 并行任务必须使用 tokio::spawn
- CP-002: 默认最大并发数: 4
- CP-003: 任一失败应取消其他任务 (可配置)

### 2.5 性能约束

| 操作 | 目标 | 最大容忍 |
|------|------|----------|
| 工作流提交 | < 10ms | 100ms |
| 节点调度 | < 1ms | 10ms |
| 状态查询 | < 5ms | 50ms |

### 2.6 错误处理约束

- CEH-001: 严禁使用 unwrap/expect (除测试外)
- CEH-002: 所有错误必须包含上下文信息
- CEH-003: 用户-facing 错误消息必须是中文

### 2.7 安全约束

- CSI-001: 所有外部输入必须验证
- CSI-002: 字符串输入限制长度 (默认 10KB)
- CSI-003: 文件路径必须规范化
- CSP-001: 插件必须在沙箱中执行
- CSP-002: 插件资源使用受限
- CSP-003: 插件网络访问默认关闭

### 2.8 可观测性约束

- COL-001: 使用 tracing crate 结构化日志
- COL-002: 日志级别: ERROR > WARN > INFO > DEBUG > TRACE
- COL-003: 关键操作记录: workflow_id, node_id, duration

---

## 3. 审查清单

### 代码审查
- [ ] 违反分层架构 (AC-001~003)
- [ ] 违反 DDD 规范 (DDD-E001~A004)
- [ ] 存在 unwrap/expect (CEH-001)
- [ ] 错误消息非中文 (CEH-003)

### 性能审查
- [ ] 使用 tokio::spawn (CP-001)
- [ ] 有背压机制 (CPC-001)

### 安全审查
- [ ] 外部输入验证 (CSI-001)
- [ ] 文件路径规范化 (CSI-003)
- [ ] 插件沙箱化 (CSP-001)
