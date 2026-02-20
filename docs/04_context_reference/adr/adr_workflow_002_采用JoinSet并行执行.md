# ADR-Workflow-002: 采用 JoinSet 替代 join_all 进行并行执行

## 状态
- [x] 已接受
- [ ] 已废弃
- [ ] 已替代

## 背景 (Context)

### 问题描述
工作流引擎需要并行执行多个节点，当前使用 `futures::future::join_all` 进行并行执行。但 `join_all` 存在以下问题：
- 无法取消单个任务
- 无法处理任务失败时的其他任务
- 缺乏对任务生命周期的细粒度控制

### 约束条件
- 必须支持任务取消
- 必须支持失败时的快速终止
- 必须支持超时控制
- 不能显著增加内存开销

### 影响范围
- `src/workflow/engine.rs` - 工作流引擎
- `src/workflow/parallel_executor.rs` - 并行执行器
- 所有并行执行的工作流节点

## 决策 (Decision)

### 选择的方案
使用 **tokio::task::JoinSet** 替代 `futures::future::join_all`。

### 决策理由
JoinSet 提供更好的任务管理和取消支持，是 Tokio 官方推荐的并行执行方式。

## 选项对比 (Options Considered)

| 选项 | 优点 | 缺点 | 结论 |
|------|------|------|------|
| **JoinSet** (已选择) | 支持任务取消、AbortHandle、按完成顺序处理 | 需要重构现有代码 | ✅ 选择 |
| join_all | 简单易用、代码量少 | 不支持取消、无法控制单个任务 | ❌ 不选 |
| FuturesUnordered | 支持流式处理 | API 复杂、性能略低 | ❌ 不选 |
| 手动 spawn | 最大灵活性 | 代码复杂、难以维护 | ❌ 不选 |

## 详细分析

### 1. JoinSet 方案

```rust
let mut join_set: JoinSet<Result<T>> = JoinSet::new();

for task in tasks {
    join_set.spawn(async move {
        // 执行任务
    });
}

while let Some(result) = join_set.join_next().await {
    match result {
        Ok(value) => { /* 处理成功 */ }
        Err(e) => { 
            // 可以选择取消剩余任务
            join_set.shutdown().await;
            break;
        }
    }
}
```

**优点**:
- 支持 `abort()` 取消单个任务
- 支持 `shutdown()` 取消所有任务
- 支持 `join_next()` 按完成顺序处理
- 支持 `AbortHandle` 获取任务句柄
- 内置超时支持

**缺点**:
- 需要 `'static` 生命周期
- 需要重构现有代码

### 2. join_all 方案

```rust
let futures: Vec<_> = tasks.iter().map(|t| execute(t)).collect();
let results = join_all(futures).await;
```

**优点**:
- 代码简洁
- 易于理解

**缺点**:
- 无法取消单个任务
- 必须等待所有任务完成
- 无法处理"第一个成功即返回"场景

## 影响 (Consequences)

### 正面影响
- ✅ 支持任务取消，提高资源利用率
- ✅ 支持失败快速终止，减少无效计算
- ✅ 更好的错误处理和恢复能力
- ✅ 支持超时和取消令牌集成

### 负面影响/风险
- ⚠️ 需要重构现有并行执行代码 → 已完成重构
- ⚠️ 测试需要覆盖新的取消逻辑 → 已添加测试

### 技术债务
- 无明显技术债务

## 实施计划

### 已完成
- [x] 创建 `ParallelExecutor` 抽象层
- [x] 实现 `JoinSetExecutor`
- [x] 添加任务取消支持
- [x] 添加超时控制
- [x] 集成到工作流引擎

### 后续优化
- [ ] 添加任务优先级支持
- [ ] 实现任务重试机制

## 相关文档

- **L3 实现设计**: `src/workflow/parallel_executor.rs`
- **相关 ADR**: ADR-003 (DAG 工作流)
- **执行计划**: `.trae/documents/架构优化执行计划.md`

## 决策记录

| 日期 | 决策人 | 动作 | 说明 |
|------|--------|------|------|
| 2026-02-20 | Architecture Team | 创建 | 采用 JoinSet 替代 join_all |
