---
version: v2.12.0
updated: 2026-02-25
---

# 分层验收标准规范

---

## 概述

本文档定义 SOP 流程中的**分层验收标准**，明确 L1-L4 各层级的验收要求、测试充分性标准和审查流程。

**核心原则**:
- **sop-test-design-csv 设计**: 基于设计文档设计分层验收测试
- **sop-test-implementation 实现**: 实现验收测试代码
- **sop-code-implementation 运行**: 仅运行测试，不创建测试资产
- **先低后高**: 必须先通过低层级，才能进行高层级
- **逐层审查**: 每层验收通过后必须审查

---

## 验收 Skill 分工

| Skill | 职责 | 禁止 |
|------|------|------|
| **sop-test-design-csv** | 设计分层验收测试（L1-L4） | 实现测试代码、运行测试 |
| **sop-test-implementation** | 实现验收测试代码 | 修改 CSV、修改验收标准 |
| **sop-code-implementation** | 运行验收测试 | 创建/修改测试资产、修改验收标准 |

---

## 测试设计载体（主从约定）

- **默认/推荐载体**：**CSV**。与 Skill 矩阵、sop-test-design-csv 一致；建议位置 `docs/03_technical_spec/test_cases/*.csv`，测试代码落地 `tests/acceptance/l1-l4/`（参见 04_reference/document_directory_mapping.md）。
- **替代形态**：非 TDD 或项目自选时，可采用 `.md` 等形态（如下文 L1–L4 中的 `*_test_design.md`）；在实现设计中明确载体与路径并保持可复制执行即可。

## TDD 路径的测试资产约定

当启用 TDD 深度路径（参见 `skills/sop-tdd-workflow/SKILL.md`）时：
- **sop-test-design-csv 的测试设计载体**：以 **CSV 为主**，建议位置 `docs/03_technical_spec/test_cases/*.csv`（参见 04_reference/document_directory_mapping.md）
- **sop-test-implementation 的测试代码落地**：将 CSV 用例实现为分层验收测试，建议目录 `tests/acceptance/l1-l4/`
- **sop-code-implementation 的边界不变**：仅运行测试，不创建/修改测试资产

当未启用 TDD 路径时：仍建议优先使用 CSV；如项目已有其他测试设计载体，可在实现设计中明确并保持可复制执行。

---

## 项目适配与 CI 门禁（推荐）

### 命令适配原则

SOP 不绑定具体语言/框架。每个项目应当明确以下“可执行命令契约”，供 `sop-code-implementation` 在运行验收时直接使用：

| 目标 | 建议提供的命令别名 |
|------|--------------------|
| L1 验收 | `test:l1` |
| L2 验收 | `test:l2` |
| L3 验收 | `test:l3` |
| L4 验收 | `test:l4` |
| 代码规范 | `lint` |
| 类型检查（如适用） | `typecheck` |

当项目无法提供统一别名时，文档或实现设计必须明确写出对应的实际命令（并保持可复制执行）。

### CI 门禁建议

| 场景 | 必须通过 | 建议通过 |
|------|----------|----------|
| PR/合并请求 | `lint`、`typecheck`（如适用）、L1 | L2（受影响模块） |
| 合并到主干 | `lint`、`typecheck`（如适用）、L1、L2 | L3 |
| 夜间/发布前 | 全量 L1-L4 | 安全扫描、依赖审计 |

---

## 分层验收体系

### 验收层级概览

| 层级 | 验收对象 | 测试类型 | 测试设计 Skill | 测试实现 Skill | 运行 Skill | 审查 Skill |
|------|----------|----------|--------|--------|--------|--------|
| **L1** | 单元/函数 | 单元测试 | sop-test-design-csv | sop-test-implementation | sop-code-implementation | sop-code-review |
| **L2** | 模块 | 集成测试 | sop-test-design-csv | sop-test-implementation | sop-code-implementation | sop-code-review |
| **L3** | 功能 | 验收测试 | sop-test-design-csv | sop-test-implementation | sop-code-implementation | sop-code-review |
| **L4** | 系统 | E2E测试 | sop-test-design-csv | sop-test-implementation | sop-code-implementation | sop-code-review |

### 覆盖率阈值建议

| 层级 | 建议覆盖率 | 关注点 | 说明 |
|------|-----------|--------|------|
| **L1** | >= 80% | 代码行覆盖 | 单元测试，关注函数内部逻辑 |
| **L2** | >= 70% | 接口覆盖 | 集成测试，关注模块间接口调用 |
| **L3** | >= 60% | 场景覆盖 | 验收测试，关注用户场景完整性 |
| **L4** | 关键路径 100% | 流程覆盖 | E2E测试，关注核心业务流程 |

**覆盖率计算规则**：
- L1：代码行覆盖率（Line Coverage）
- L2：接口覆盖率（API Coverage）
- L3：场景覆盖率（Scenario Coverage）
- L4：关键路径覆盖率（Critical Path Coverage）

**不满足阈值的处理**：
- 输出警告，必须补充测试
- 若无法补充，需在 design.md 中说明原因并经用户确认

### 验收顺序

```
L1 (单元测试)
  ↓ 通过
L1 审查 (sop-code-review)
  ↓ 通过
L2 (模块集成测试)
  ↓ 通过
L2 审查 (sop-code-review)
  ↓ 通过
L3 (功能验收测试)
  ↓ 通过
L3 审查 (sop-code-review)
  ↓ 通过
L4 (系统E2E测试)
  ↓ 通过
L4 审查 (sop-code-review)
  ↓ 通过
验收完成
```

**原则**: 任何一层失败或审查不通过，必须修复后重新从该层开始。

---

## L1 - 单元/函数级别验收

### 验收范围
- 单个函数/方法
- 单个类
- 独立工具函数

### 测试设计（sop-test-design-csv）

**设计输出**: `tests/acceptance/l1/[module]_l1_test_design.md`

**设计内容**（最小字段）:
- 目标（函数/位置）
- 场景（正向/边界/异常）
- 标准（覆盖率/通过率）

### 测试实现（sop-test-implementation）

**实现位置**: `tests/acceptance/l1/test_[function].py`

**实现要求**:
- 基于 `sop-test-design-csv` 的设计实现
- 使用标准测试框架（pytest/jest等）
- 包含所有设计场景

### 验收标准（Worker检查）

| 检查项 | 标准 | 不充分时 |
|--------|------|----------|
| 测试存在性 | L1测试文件存在 | 中断，询问用户 |
| 覆盖率 | >= 80% | 中断，询问用户 |
| 通过率 | 100% | 修复后重试 |
| 代码质量 | 无lint/type错误 | 修复后重试 |

### 验收命令

```bash
# Python
pytest tests/acceptance/l1/ -v --cov=src --cov-report=term-missing

# JavaScript
npm run test:l1 -- --coverage

# Go
go test ./tests/acceptance/l1/ -v -cover
```

### 审查检查点（sop-code-review）

- [ ] 接口实现符合 design.md 定义
- [ ] 异常处理完整
- [ ] 日志记录规范
- [ ] 单元测试覆盖所有分支

---

## L2 - 模块级别验收

### 验收范围
- 模块内部组件集成
- 模块对外接口
- 模块间依赖（同层）

### 测试设计（sop-test-design-csv）

**设计输出**: `tests/acceptance/l2/[module]_l2_test_design.md`

**设计内容**（最小字段）:
- 目标（模块/接口）
- 集成场景（组件集成/接口调用）
- 依赖验证（mock 策略）

### 测试实现（sop-test-implementation）

**实现位置**: `tests/acceptance/l2/test_[module]_integration.py`

### 验收标准

| 检查项 | 标准 | 不充分时 |
|--------|------|----------|
| 测试存在性 | L2测试文件存在 | 中断，询问用户 |
| 模块覆盖 | 所有模块接口有测试 | 中断，询问用户 |
| 集成度 | 验证模块内组件协作 | 中断，询问用户 |
| 通过率 | 100% | 修复后重试 |

### 验收命令

```bash
# Python
pytest tests/acceptance/l2/ -v

# JavaScript
npm run test:l2

# Go
go test ./tests/acceptance/l2/ -v
```

### 审查检查点（sop-code-review）

- [ ] 模块设计符合 design.md
- [ ] 模块间依赖正确
- [ ] 模块边界清晰
- [ ] 接口契约满足

---

## L3 - 功能级别验收

### 验收范围
- 完整功能流程
- 用户场景
- 业务规则验证

### 测试设计（sop-test-design-csv）

**设计输出**: `tests/acceptance/l3/[feature]_l3_test_design.md`

**设计内容**（最小字段）:
- 目标（feature + FRD link）
- 场景（主/替代/异常）
- 业务规则（规则清单）

### 测试实现（sop-test-implementation）

**实现位置**: `tests/acceptance/l3/test_[feature].py`

### 验收标准

| 检查项 | 标准 | 不充分时 |
|--------|------|----------|
| 测试存在性 | L3测试文件存在 | 中断，询问用户 |
| 场景覆盖 | 覆盖FRD所有场景 | 中断，询问用户 |
| 业务规则 | 验证所有业务规则 | 中断，询问用户 |
| 通过率 | 100% | 修复后重试 |

### 验收命令

```bash
# Python
pytest tests/acceptance/l3/ -v

# JavaScript
npm run test:l3

# Go
go test ./tests/acceptance/l3/ -v
```

### 审查检查点（sop-code-review）

- [ ] 功能实现符合 design.md
- [ ] 符合 FRD 需求
- [ ] 用户场景完整覆盖
- [ ] 业务规则正确实现

---

## L4 - 系统级别验收

### 验收范围
- 端到端流程
- 系统性能
- 架构约束验证

### 测试设计（sop-test-design-csv）

**设计输出**: `tests/acceptance/l4/system_l4_test_design.md`

**设计内容**（最小字段）:
- 目标（系统流程/架构约束）
- 场景（E2E/性能/可靠性）
- 指标/约束（阈值/校验项）

### 测试实现（sop-test-implementation）

**实现位置**: `tests/acceptance/l4/test_system_e2e.py`

### 验收标准

| 检查项 | 标准 | 不充分时 |
|--------|------|----------|
| 测试存在性 | L4测试文件存在 | 中断，询问用户 |
| E2E覆盖 | 覆盖核心业务流程 | 中断，询问用户 |
| 性能达标 | 满足性能指标 | 中断，询问用户 |
| 架构约束 | 满足架构约束 | 中断，询问用户 |
| 通过率 | 100% | 修复后重试 |

### 验收命令

```bash
# Python
pytest tests/acceptance/l4/ -v

# JavaScript
npm run test:l4

# Go
go test ./tests/acceptance/l4/ -v

# 性能测试
k6 run performance-tests.js
```

### 审查检查点（sop-code-review）

- [ ] 符合架构设计文档
- [ ] 符合 design.md 整体设计
- [ ] 系统级约束满足
- [ ] 性能指标达标
- [ ] 可扩展性满足

---

## 测试充分性检查

### sop-code-implementation 运行前检查清单

`sop-code-implementation` 在运行每层验收测试前，必须检查：

检查项（每层一致）：测试文件存在 / 测试设计存在 / 测试代码存在 / 覆盖关键场景与指标。
不充分：中断并标记 `[WAITING_FOR_TEST_CREATION]`（等待用户决策）

---

## 停止点定义

| 停止点 | 触发时机 | 等待内容 | 处理 Skill |
|--------|----------|----------|----------|
| `[WAITING_FOR_TEST_DESIGN]` | sop-test-design-csv 完成测试设计 | 用户确认测试设计充分 | sop-test-design-csv |
| `[WAITING_FOR_TEST_IMPLEMENTATION]` | sop-test-implementation 完成测试实现 | sop-code-review 审查测试代码充分性与合规性 | sop-code-review |
| `[WAITING_FOR_L1_REVIEW]` | L1测试通过后 | sop-code-review 审查 | sop-code-review |
| `[WAITING_FOR_L2_REVIEW]` | L2测试通过后 | sop-code-review 审查 | sop-code-review |
| `[WAITING_FOR_L3_REVIEW]` | L3测试通过后 | sop-code-review 审查 | sop-code-review |
| `[WAITING_FOR_L4_REVIEW]` | L4测试通过后 | sop-code-review 审查 | sop-code-review |
| `[WAITING_FOR_TEST_CREATION]` | 测试不充分时 | 用户决策（补充测试/继续/暂停） | sop-code-implementation → 用户 |

---

## 失败处理流程

### 测试失败

```
Worker运行测试
  ↓
测试失败
  ↓
Worker修复代码
  ↓
重新运行该层测试
  ↓
通过 → 进入审查
```

### 审查不通过

```
审查不通过
  ↓
返回对应设计阶段
  ↓
修复设计/代码
  ↓
从该层重新开始验收
```

### 测试不充分（sop-code-implementation 发现）

```
`sop-code-implementation` 检查测试充分性
  ↓
测试不充分
  ↓
标记 [WAITING_FOR_TEST_CREATION]
  ↓
停止工作
  ↓
等待用户决策
  ├─ 补充测试 → sop-test-design-csv 设计 → sop-test-implementation 实现
  ├─ 继续（接受风险）→ 继续运行
  └─ 暂停 → 暂停任务
```

---

## 验收文档结构

### 目录结构

```
tests/
├── acceptance/
│   ├── l1/                          # L1 单元测试
│   │   ├── [module]_l1_test_design.md   # sop-test-design-csv（非CSV形态时的替代载体）
│   │   └── test_[function].py           # sop-test-implementation
│   ├── l2/                          # L2 模块集成测试
│   │   ├── [module]_l2_test_design.md
│   │   └── test_[module]_integration.py
│   ├── l3/                          # L3 功能验收测试
│   │   ├── [feature]_l3_test_design.md
│   │   └── test_[feature].py
│   └── l4/                          # L4 系统E2E测试
│       ├── system_l4_test_design.md
│       └── test_system_e2e.py
└── ...
```

### 验收报告

每层验收完成后，`sop-code-implementation` 生成验收报告：

模板：04_reference/interaction_formats/worker_execution_result.md

---

## 相关文档

- [禁止项矩阵](constraint_matrix.md) - 黑白名单约束
- [design.md模板](04_reference/document_templates/implementation_design.md) - 实现设计模板
- [工作流规范](03_workflow/deep_path.md) - 深度路径流程

---

**注意**: 分层验收是质量保证的关键环节。所有 Skill 必须严格遵守，确保每层验收通过后再进入下一层。
