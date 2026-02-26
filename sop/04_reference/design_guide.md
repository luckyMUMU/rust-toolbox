---
version: v2.12.0
updated: 2026-02-25
---

# design.md 创建与执行指南

本文档整合 design.md 创建判断规则与目录维度工作策略，为 `sop-implementation-designer` 和 `sop-code-implementation` 提供统一参考。

---

## 1. 概述

design.md 的创建与执行需要综合两个维度：

| 维度 | 判断依据 | 优先级 | 说明 |
|------|----------|--------|------|
| **目录层级** | 目录结构、子目录存在性 | 高 | 决定是否需要独立 design.md |
| **代码复杂度** | 行数、依赖数、功能数 | 低 | 决定 design.md 的详细程度 |

**核心原则**：目录层级优先，复杂度决定粒度。

---

## 2. design.md 创建判断规则

### 2.1 复杂度分级

| 复杂度 | 代码行数 | 功能特点 | 依赖关系 | design.md 要求 |
|--------|----------|----------|----------|----------------|
| **低** | <100行 | 单一功能，纯工具函数 | 无外部依赖 | 极简版（仅接口契约）或合并到父目录 |
| **中** | 100-500行 | 多函数，有业务逻辑 | 有外部依赖 | 简要版（接口契约 + 任务清单） |
| **高** | >500行 | 多模块交互，复杂逻辑 | 多模块依赖 | 完整版（全部章节） |

### 2.2 复杂度评估指标

**必须评估**（至少满足一项即按该级别）：
- 代码行数：统计 `.js/.ts/.py` 等实现文件总行数
- 功能数量：对外暴露的函数/类/接口数量
- 依赖数量：import/require 的外部模块数量

### 2.3 决策流程

```
开始
  ↓
【判断1】是否存在子目录？
  ├─ 是 → 为每个子目录递归执行本流程
  │       当前目录创建 design.md（汇总子目录接口）
  │
  └─ 否 → 【判断2】当前目录代码复杂度？
            ├─ 低（<100行）→ 创建极简 design.md 或合并到父目录
            ├─ 中（100-500行）→ 创建简要 design.md
            └─ 高（>500行）→ 创建完整 design.md
  ↓
结束
```

### 2.4 优先级规则

```
快速路径 > 目录层级 > 代码复杂度
```

- **快速路径**：单文件+<30行+无逻辑变更 → 可省略 design.md
- **目录层级**：存在子目录 → 必须创建 design.md
- **代码复杂度**：无子目录时 → 按复杂度决定 design.md 粒度

---

## 3. 目录层级策略

### 3.1 目录边界定义

```
项目根目录 (depth 0)
  └── src/ (depth 1)
        ├── utils/ (depth 2) ← design.md
        │     ├── string.ts
        │     └── date.ts
        ├── auth/ (depth 2) ← design.md
        │     ├── login/ (depth 3) ← design.md
        │     │     └── index.ts
        │     └── register/ (depth 3) ← design.md
        │           └── index.ts
        └── payment/ (depth 2) ← design.md
              └── processor.ts
```

### 3.2 目录与 design.md 对应关系

| 场景 | 处理方式 | 示例 |
|------|----------|------|
| 叶子目录（无子目录） | 根据复杂度创建对应粒度 design.md | `src/utils/design.md` |
| 中间目录（有子目录） | 必须创建 design.md，汇总子目录接口 | `src/auth/design.md` |
| 单文件目录 | 按文件复杂度判断 | `src/payment/` |

### 3.3 实现 Scope 工作范围（DIR_SCOPE）

```
DIR_SCOPE(dir_with_design_md) = dir/** - {subdir/** | subdir contains design.md}
```

**说明**：
- `sop-code-implementation` 负责当前目录下所有代码文件
- 不跨越子目录的 design.md 边界
- 子目录由独立的实现 Scope 处理

---

## 4. 执行策略

### 4.1 自底向上处理顺序

```
执行顺序 = depth_desc (从深到浅)
并行条件 = same_depth AND no_dependency
等待条件 = parent_dir OR has_dependency
```

**CMD 序列**：
```
LIST_DESIGN_MD(root) -> design_list
SCHEDULE_DIRS(design_list) -> dir_map
RUN_DIR_BATCH(depth_desc)
```

### 4.2 并行执行规则

| 条件 | 说明 |
|------|------|
| **可以并行** | 同深度且无依赖关系的目录；不同子树的目录 |
| **必须串行** | 有依赖关系的目录；父子目录关系 |

**依赖类型**：
1. 显式依赖：design.md 中声明的依赖接口
2. 隐式依赖：代码中的 import/require
3. 父子依赖：目录层级关系

### 4.3 跨模块改动处理

**原则**：只修改 design，不直接修改实现

**处理流程**：
```
REQUEST_CROSS_DIR(src_dir, target_dir, change) -> appended_request
WAIT_DEP(src_dir, target_dir)
```

变更记录位置：目标目录 `design.md` 的"待处理变更"章节

---

## 5. Spec 任务划分规则

### 5.1 划分原则

| 依据 | 说明 | 优先级 |
|------|------|--------|
| **目录边界** | 以 design.md 所在目录为边界 | 高 |
| **深度优先** | 从最深目录开始执行 | 中 |
| **依赖驱动** | 等待依赖目录完成 | 高 |

### 5.2 任务声明格式

```yaml
design_path: src/auth/login/design.md
depth: 3
dependencies:
  - src/auth/design.md
scope: src/auth/login/**
```

### 5.3 动态创建条件

- 跨目录变更：任务需要修改其他目录代码，且目标目录无 design.md
- 复杂度增加：任务复杂度超过当前 design.md 粒度
- 设计先行：执行前确认 design.md 已存在或已创建

---

## 6. 示例场景

### 场景1：低复杂度叶子目录

```
src/auth/login/
  ├── index.ts (30行)
  └── utils.ts (20行)
```

**判断**：无子目录，总行数 50 < 100 → 低复杂度
**处理**：创建极简 design.md（接口契约 + 简要说明）

### 场景2：中复杂度模块

```
src/payment/
  ├── processor.ts (150行)
  ├── validator.ts (100行)
  └── types.ts (50行)
```

**判断**：无子目录，总行数 300 → 中复杂度
**处理**：创建简要 design.md（技术选型 + 接口契约 + 任务清单）

### 场景3：高复杂度模块

```
src/order/
  ├── service.ts (300行)
  ├── repository.ts (250行)
  ├── controller.ts (200行)
  └── types.ts (50行)
```

**判断**：无子目录，总行数 800 > 500 → 高复杂度
**处理**：创建完整 design.md（全部章节）

### 场景4：有子目录的中间节点

```
src/auth/
  ├── design.md ← 必须创建
  ├── login/
  │     ├── design.md
  │     └── index.ts
  └── register/
        ├── design.md
        └── index.ts
```

**判断**：存在子目录 → 必须创建 design.md
**处理**：内容重点为子目录接口汇总、跨子目录依赖声明

---

## 7. 决策检查清单

`sop-implementation-designer` 在创建 design.md 前，按以下顺序检查：

- [ ] **步骤1**：确认是否为快速路径（单文件+<30行）
  - 是 → 可省略 design.md
  - 否 → 继续

- [ ] **步骤2**：确认是否存在子目录
  - 是 → 必须创建 design.md（汇总子目录接口）
  - 否 → 继续

- [ ] **步骤3**：评估代码复杂度
  - 统计代码行数、功能/接口数量、依赖数量

- [ ] **步骤4**：根据复杂度确定 design.md 粒度
  - 低 → 极简版 | 中 → 简要版 | 高 → 完整版

- [ ] **步骤5**：创建 design.md 并声明目录依赖

---

## 8. 相关文档引用

| 文档 | 路径 | 说明 |
|------|------|------|
| 实现设计模板 | implementation_design.md | L3 实现设计模板 |
| Spec 交互式指南 | spec_interactive_guide.md | 交互式提问流程 |

---

## 9. 约束

1. **目录边界**：`sop-code-implementation` 不跨越 design.md 边界修改代码
2. **层级优先**：目录层级判断优先于复杂度判断
3. **快速路径例外**：快速路径可跳过 design.md 创建
4. **依赖声明**：所有 design.md 必须声明目录依赖
5. **渐进式披露**：复杂度越高，设计文档越详细
