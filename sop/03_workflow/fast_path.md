---
version: v2.12.0
updated: 2026-02-25
---

# 快速路径

**适用**: 单文件+<30行+无逻辑变更

---

## 流程

```
sop-code-explorer → sop-code-implementation → sop-code-review → sop-document-sync
```

| 阶段 | 输入 | 输出 | 停止点 |
|------|------|------|--------|
| sop-code-explorer | 目标文件 | 审计报告 | `[USER_DECISION]` |
| sop-code-implementation | 审计报告 | 代码修改 | `[WAITING_FOR_CODE_REVIEW]` / `[DIR_WAITING_DEP]` |
| sop-code-review | Diff+设计依据 | 审查报告 | `[USER_DECISION]` |
| sop-document-sync | 代码修改 | 文档更新 | `[USER_DECISION]` |

---

## 步骤

### 1. sop-code-explorer 分析
CMD: `AUDIT(file) -> audit_report`（模板：04_reference/interaction_formats/code_audit_report.md）

### 2. sop-code-implementation 修改
CMD: `IMPLEMENT(dir, design|audit) -> [WAITING_FOR_CODE_REVIEW]`（模板：04_reference/interaction_formats/worker_execution_result.md）

### 3. sop-code-review 审查
CMD: `CODE_REVIEW(diff, design_refs) -> review_report`（模板：04_reference/interaction_formats/code_review.md）

### 4. sop-document-sync 更新
CMD: `DOC_SYNC(scope) -> [已完成]`

---

## 约束

- 单文件
- <30行
- 无逻辑变更
- 测试通过

---

## 变更分类与升级红线

CMD: `FAST_PATH_CHECK(change) -> allow|upgrade`

### 量化判断标准

#### 允许条件（全满足）

| 条件 | 量化标准 | 检测方法 |
|------|----------|----------|
| 单文件 | `affected_files_count == 1` | Git diff 统计 |
| 行数限制 | `delta_lines < 30`（新增+删除） | Git diff 统计 |
| 无接口变更 | `interface_signature_changed == false` | AST 分析 |
| 无控制流变更 | `control_flow_nodes_changed == false` | AST 分析 |
| 无数据模型变更 | `data_model_fields_changed == false` | Schema/Model 文件检测 |
| 无安全边界变更 | `authz_check_changed == false` | 关键函数调用检测 |
| 无并发变更 | `concurrency_pattern_changed == false` | 锁/信号量/async 检测 |
| 无依赖行为变更 | `dependency_call_changed == false` | Import/调用分析 |
| 测试通过 | `tests_passed == true` | 执行测试命令 |

#### 升级条件（任一满足）

| 条件 | 量化标准 | 检测方法 |
|------|----------|----------|
| 跨文件/目录 | `affected_files_count > 1` | Git diff 统计 |
| 接口变更 | `interface_signature_changed == true` | AST 分析 |
| 控制流变更 | `control_flow_nodes_changed == true` | AST 分析 |
| 数据模型变更 | `data_model_fields_changed == true` | Schema/Model 文件检测 |
| 安全边界变更 | `authz_check_changed == true` | 关键函数调用检测 |
| 并发变更 | `concurrency_pattern_changed == true` | 锁/信号量/async 检测 |
| 依赖行为变更 | `dependency_call_changed == true` | Import/调用分析 |
| 不确定 | `cannot_prove_no_behavior_change == true` | 人工判断 |

### AST 变化检测说明

以下 AST 节点变更视为"逻辑变更"：

| 节点类型 | 示例 |
|----------|------|
| 函数签名 | 参数类型/数量/返回值变化 |
| 条件分支 | if/switch/case 变化 |
| 循环结构 | for/while/递归变化 |
| 异常处理 | try/catch/throw 变化 |
| 类继承 | extends/implements 变化 |
| 方法调用 | 关键方法调用变化 |

参见：05_constraints/command_dictionary.md
