---
name: "design-creator"
description: "逐层创建 design.md 设计文档。Invoke when starting a new feature, refactoring, or when design documentation is missing. Creates hierarchical design documents following the progressive disclosure principle."
---

# Design Creator Skill

逐层创建 design.md 设计文档，遵循渐进式披露原则。

## 使用场景

- 开始新功能开发时
- 进行代码重构时
- 发现设计文档缺失时
- 需要建立模块级设计文档时

## 工作流程

### Step 1: 确定文档层级

```
根目录 design.md (项目级)
    ↓
模块级 design.md (src/module/design.md)
    ↓
子模块级 design.md (src/module/sub/design.md)
```

### Step 2: 创建文档

使用模板：

```markdown
# 模块：[名称]

## 1. 核心定义 (Stable)

### 1.1 领域模型
[实体、值对象定义]

### 1.2 接口定义
[公开接口 Trait/Struct]

### 1.3 依赖关系
[模块依赖图]

## 2. 待实现方案 (In Progress) 🟢

### 2.1 决策记录 (ADR)
- **决策**: [技术选型决策]
- **理由**: [选择理由]
- **风险**: [潜在风险]

### 2.2 任务清单
- [ ] Task 0: 渐进式重构 (修复相关旧逻辑/格式)
- [ ] Task 1: [具体任务1]
- [ ] Task 2: [具体任务2]

### 2.3 接口契约
```rust
// 伪代码定义
pub trait NewFeature {
    fn method(&self) -> Result<()>;
}
```

### 2.4 测试策略
[单元测试/集成测试计划]

## 3. 状态记录

- `[进行中]` | [描述] | [日期]
- `[已完成]` | [历史清单]
```

### Step 3: 更新父级索引

向上递归更新父级 design.md，添加链接：

```markdown
## 子模块
- [子模块A](./submoduleA/design.md)
- [子模块B](./submoduleB/design.md)
```

## 输入格式

```markdown
## 创建目标
[模块路径，如 src/workflow/executor]

## 设计内容
- 领域模型: [描述]
- 接口定义: [描述]
- 任务清单: [列表]

## 父级文档
[父级 design.md 路径]
```

## 输出格式

```markdown
## 创建完成

- 文件: [design.md 路径]
- 层级: [根/模块/子模块]
- 父级索引: [已更新/无需更新]
- 状态: `[进行中]`

## 下一步
@Router: 设计文档已创建，可进入实现阶段
```

## 示例

### 创建模块级设计文档

**输入**:
```markdown
## 创建目标
src/workflow/executor

## 设计内容
- 领域模型: RetryPolicy, RetryExecutor
- 接口定义: RetryExecutor Trait
- 任务清单: 实现重试策略、集成到工作流引擎

## 父级文档
src/workflow/design.md
```

**输出**:
```markdown
## 创建完成

- 文件: src/workflow/executor/design.md
- 层级: 子模块级
- 父级索引: src/workflow/design.md 已更新
- 状态: `[进行中]`

## 下一步
@Router: 设计文档已创建，可进入实现阶段
```

## 约束

- 父目录只保留摘要和链接，严禁展开细节
- 必须使用 `[进行中]` / `[已完成]` 状态标记
- 必须包含 ADR (架构决策记录)
- 必须定义接口契约 (伪代码)
