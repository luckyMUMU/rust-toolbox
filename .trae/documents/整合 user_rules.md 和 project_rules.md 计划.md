## 任务概述

1. 将通用工作流内容从 `project_rules.md` 迁移到 `user_rules.md`
2. 在 `project_rules.md` 中添加 DDD (领域驱动设计) 规范
3. 精简 `project_rules.md` 为项目专属内容

---

## 执行计划

### 阶段一：更新 `user_rules.md`

**添加通用工作流内容** (从 project_rules.md 迁移):
- 角色矩阵 (Router/Explorer/Oracle/Librarian/Worker)
- 任务分诊 (快速路径/深度路径)
- 标准作业程序 (SOP)
- 文档规范 (渐进式披露)
- 三错即停机制
- 交互规范

### 阶段二：更新 `project_rules.md`

**1. 添加 DDD 规范章节**:
```
## X. 领域驱动设计规范 (DDD)

### X.1 分层架构
- 接入层 (Interface Layer)
- 应用层 (Application Layer)  
- 领域层 (Domain Layer)
- 基础设施层 (Infrastructure Layer)
- 适配器层 (Adapter Layer)

### X.2 领域建模规范
- 实体 (Entity) 定义规则
- 值对象 (Value Object) 规则
- 聚合根 (Aggregate Root) 规则
- 领域事件 (Domain Event) 规则
- 仓库 (Repository) 接口规范

### X.3 命名规范
- 领域对象命名
- 应用服务命名
- 接口适配器命名
```

**2. 保留项目专属内容**:
- 软件约束规范 (SPEC) - 架构约束、接口契约等
- DDD 规范 (新增)

**3. 移除通用内容** (已迁移到 user_rules.md):
- 角色矩阵、任务分诊、SOP、文档规范、三错即停、交互规范

---

## 文件变更

| 文件 | 变更 |
|------|------|
| `user_rules.md` | 添加通用工作流、角色矩阵、SOP、三错即停等内容 |
| `project_rules.md` | 添加 DDD 规范，精简为项目专属内容 |

---

## 最终结构

### user_rules.md (全局)
- 基础约束、AI 工作流、角色矩阵、任务分诊、SOP、文档规范、三错即停、技术规范、交互规范

### project_rules.md (项目专属)
- 软件约束规范 (SPEC)
- **DDD 领域驱动设计规范 (新增)**
- 引用说明 (指向 user_rules.md 通用规则)