---
name: "sop-design-placement"
description: "指导AI Agent正确放置设计文档和创建design.md。Invoke when creating design documents or design.md files, to determine the correct placement based on module division and complexity."
---

# 设计文档放置指南

指导 AI Agent 正确放置设计文档，确保文档结构清晰、权限正确。

---

## 目录结构规范

### 文档放置位置

| 目录 | 用途 | 变更权限 | 示例 |
|------|------|----------|------|
| `/docs` | 项目设计文档根目录 | 项目开发过程中动态创建 | `docs/README.md` |
| `/docs/01_requirements/` | PRD 需求文档 | Analyst 创建 | `docs/01_requirements/feature-x.md` |
| `/docs/02_logical_workflow/` | 架构设计文档 | Prometheus 创建 | `docs/02_logical_workflow/core.pseudo` |
| `/docs/参考/` | SOP 参考文档、模板 | **非指定不变更** | `docs/参考/sop/...` |
| `src/**/` | 源代码目录 | Worker 修改 | `src/module/design.md` |
| `docs/**/` | 模块设计文档 | Oracle 创建 | `docs/module/design.md` |

### 重要约束

⚠️ **`/docs/参考/` 非指定不变更**
- 该目录包含 SOP 标准文档
- 仅 Librarian 角色维护
- 其他角色**禁止**修改此目录

---

## design.md 创建规则

### 1. 基于模块划分

**模块定义**:
- 功能边界清晰的独立单元
- 可独立开发、测试、部署
- 有明确的输入/输出接口

**放置位置**:
```
模块根目录/
├── design.md          # 模块设计文档
├── src/               # 源代码
├── tests/             # 测试代码
└── README.md          # 模块说明
```

**路径选择**:
| 模块位置 | design.md 位置 |
|----------|----------------|
| `src/module/` | `src/module/design.md` |
| `packages/package/` | `packages/package/design.md` |
| 顶层模块 | `docs/module/design.md` |

---

### 2. 基于复杂度判断

**复杂度评估**:

| 复杂度 | 代码行数 | 功能特点 | design.md 要求 |
|--------|----------|----------|----------------|
| **低** | <100行 | 单一功能，无外部依赖 | 可省略，代码注释说明 |
| **中** | 100-500行 | 多函数，有外部依赖 | 创建简要 design.md，含接口契约 |
| **高** | >500行 | 多模块交互，复杂逻辑 | 创建完整 design.md，含详细设计 |

**决策流程**:
```
评估模块复杂度
    │
    ├─ 低 → 代码注释说明，不创建独立 design.md
    │
    ├─ 中 → 创建简要 design.md（接口契约 + 任务清单）
    │
    └─ 高 → 创建完整 design.md（完整设计 + 详细契约）
```

---

### 3. 接口契约规范

每个 design.md 必须包含接口契约章节：

```markdown
## 接口契约

### 输入
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| [param1] | [type] | 是 | [description] |
| [param2] | [type] | 否 | [description] |

### 输出
| 返回值 | 类型 | 说明 |
|--------|------|------|
| [return1] | [type] | [description] |
| [error] | [Error] | [error description] |

### 依赖接口
| 模块 | 接口 | 用途 |
|------|------|------|
| [module] | [interface] | [purpose] |

### 被依赖接口
| 模块 | 接口 | 用途 |
|------|------|------|
| [module] | [interface] | [purpose] |
```

---

## 工作流程

### Step 1: 确定文档类型

| 文档类型 | 创建者 | 放置位置 |
|----------|--------|----------|
| PRD | Analyst | `docs/01_requirements/*.md` |
| 架构设计 | Prometheus | `docs/02_logical_workflow/*.pseudo` |
| 实现设计 | Oracle | `src/**/design.md` 或 `docs/**/design.md` |

### Step 2: 评估模块复杂度

**检查项**:
- [ ] 估算代码行数
- [ ] 识别外部依赖数量
- [ ] 评估功能复杂度
- [ ] 判断是否需要独立模块

### Step 3: 确定放置位置

**决策树**:
```
创建 design.md？
    │
    ├─ 是 → 模块位置？
    │       │
    │       ├─ src/ 下 → `src/module/design.md`
    │       │
    │       └─ 顶层模块 → `docs/module/design.md`
    │
    └─ 否 → 代码注释说明
```

### Step 4: 创建接口契约

**必须包含**:
- 输入参数定义
- 输出返回值定义
- 依赖接口列表
- 被依赖接口列表

---

## 输入格式

```markdown
## 模块信息
- **模块名称**: [name]
- **模块路径**: [path]
- **功能描述**: [description]

## 复杂度评估
- [ ] 低（<100行）
- [ ] 中（100-500行）
- [ ] 高（>500行）

## 依赖分析
- 依赖模块: [list]
- 被依赖模块: [list]
```

---

## 输出格式

### 场景 1: 省略 design.md
```markdown
## 决策结果
- **创建 design.md**: 否
- **原因**: 复杂度低，代码注释即可
- **建议**: 在代码文件中添加详细注释
```

### 场景 2: 创建简要 design.md
```markdown
## 决策结果
- **创建 design.md**: 是
- **位置**: `src/module/design.md`
- **复杂度**: 中
- **内容**: 接口契约 + 简要任务清单
```

### 场景 3: 创建完整 design.md
```markdown
## 决策结果
- **创建 design.md**: 是
- **位置**: `docs/module/design.md`
- **复杂度**: 高
- **内容**: 完整设计 + 详细接口契约 + 任务清单 + 测试策略
```

---

## 示例

### 示例 1: 简单工具函数

**模块**: `src/utils/formatDate.js`
**代码行数**: 30行
**功能**: 日期格式化

**决策**:
```markdown
## 决策结果
- **创建 design.md**: 否
- **原因**: 复杂度低（30行，单一功能）
- **建议**: 在代码中添加 JSDoc 注释
```

---

### 示例 2: API 客户端模块

**模块**: `src/api/client/`
**代码行数**: 约300行
**功能**: HTTP 请求封装、错误处理、重试机制

**决策**:
```markdown
## 决策结果
- **创建 design.md**: 是
- **位置**: `src/api/client/design.md`
- **复杂度**: 中
- **内容要求**:
  - 接口契约（输入/输出/依赖）
  - 简要任务清单
  - 错误处理策略
```

**生成的接口契约**:
```markdown
## 接口契约

### 输入
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| url | string | 是 | 请求地址 |
| method | string | 否 | HTTP方法，默认GET |
| data | object | 否 | 请求数据 |

### 输出
| 返回值 | 类型 | 说明 |
|--------|------|------|
| response | Promise<Response> | 响应对象 |
| error | ApiError | 错误对象 |

### 依赖接口
| 模块 | 接口 | 用途 |
|------|------|------|
| config | getApiConfig | 获取API配置 |
```

---

### 示例 3: 核心业务模块

**模块**: `src/core/workflow/`
**代码行数**: 约1000行
**功能**: 工作流引擎、状态管理、事件驱动

**决策**:
```markdown
## 决策结果
- **创建 design.md**: 是
- **位置**: `docs/core/workflow/design.md`
- **复杂度**: 高
- **内容要求**:
  - 完整架构设计
  - 详细接口契约
  - 任务清单（分解为子任务）
  - 测试策略
  - 性能考虑
```

---

## 约束

1. **禁止修改 `/docs/参考/`** - 仅 Librarian 可维护
2. **基于模块划分** - 每个独立模块创建独立 design.md
3. **基于复杂度判断** - 低复杂度可省略，中高复杂度必须创建
4. **必须包含接口契约** - 输入/输出/依赖必须明确定义
5. **渐进式披露** - 复杂度越高，设计文档越详细

---

## 快速参考

| 场景 | 决策 | 位置 | 内容 |
|------|------|------|------|
| 简单工具函数（<100行） | 不创建 | - | 代码注释 |
| 中等模块（100-500行） | 创建 | `src/module/design.md` | 接口契约 + 任务清单 |
| 复杂模块（>500行） | 创建 | `docs/module/design.md` | 完整设计 + 详细契约 |

---

👉 [返回 L4: 参考文档](../../04_reference/index.md)
