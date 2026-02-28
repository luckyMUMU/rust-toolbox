---
version: v3.0.0
updated: 2026-03-01
---

# 深度路径

**适用**：跨文件/新功能/重构/API 变更/架构调整

---

## 1. 来源与依赖准则（全路径通用）

- 仅当产出会影响后续步骤时 → **必须**在交付物中声明来源与依赖（模板：`04_reference/interaction_formats/source_dependency.md`）
- 仅当找不到来源/依赖或存在冲突 → **必须**进入 `[USER_DECISION]` 并落盘决策记录
- 标准：`04_reference/review_standards/source_dependency.standard.md`

---

## 2. 目录维度深度路径（推荐）

### 2.1 核心调用链

```
sop-requirement-analyst
→ sop-architecture-design
→ sop-architecture-reviewer
→ sop-implementation-designer (按目录)
→ sop-code-explorer (LIST_DESIGN_MD → design_list)
→ sop-progress-supervisor (SCHEDULE_DIRS(design_list) → dir_map)
→ sop-code-implementation (按目录并行)
→ sop-code-review
→ sop-document-sync
```

### 2.2 目录并行执行（状态机）

CMD: `LIST_DESIGN_MD(root) -> design_list`（主体：sop-code-explorer）  
CMD: `SCHEDULE_DIRS(design_list) -> dir_map`  
CMD: `RUN_DIR_BATCH(depth_desc)`（同 depth 并行）  
CMD: `WAIT_DEP(dir,deps)` / `COMPLETE_DIR(dir)`  

规则：

- 仅当同深度无依赖 → 允许并行批次
- 仅当父目录所有子目录完成 → 允许父目录进入 `[DIR_WORKING]`
- 仅当发生跨目录依赖 → 必须进入 `[DIR_WAITING_DEP]` 并由 `sop-progress-supervisor` 调度

目录策略 SSOT：`04_reference/design_directory_strategy.md`

---

## 3. 单目录深度路径

```
sop-requirement-analyst
→ sop-implementation-designer
→ sop-code-implementation
→ sop-code-review
→ sop-document-sync
```

---

## 4. 分层验收（叠加）

```
... 深度路径调用链 ...
→ sop-test-design-csv
→ sop-test-implementation
→ sop-code-implementation (RUN_ACCEPTANCE L1-L4)
```

验收命令契约（SSOT：`05_constraints/command_dictionary.md`）：

- CMD: `RUN_ACCEPTANCE(L1) -> [WAITING_FOR_L1_REVIEW] -> REVIEW_ACCEPTANCE(L1)`
- CMD: `RUN_ACCEPTANCE(L2) -> [WAITING_FOR_L2_REVIEW] -> REVIEW_ACCEPTANCE(L2)`
- CMD: `RUN_ACCEPTANCE(L3) -> [WAITING_FOR_L3_REVIEW] -> REVIEW_ACCEPTANCE(L3)`
- CMD: `RUN_ACCEPTANCE(L4) -> [WAITING_FOR_L4_REVIEW] -> REVIEW_ACCEPTANCE(L4)`

门禁 SSOT：`05_constraints/acceptance_criteria.md`

---

## 5. 跨目录依赖处理

### 5.1 原则

- `sop-code-implementation` **只能**修改当前 Scope 内的代码
- 跨目录协作仅允许对目标目录的 `design.md` 做一种变更：**追加“待处理变更”条目**（不得重写既有设计）

### 5.2 处理流程（命令式）

```
发现跨目录依赖
  ↓
仅向目标目录 design.md 追加“待处理变更”
  ↓
进入 [DIR_WAITING_DEP] 并请求 sop-progress-supervisor 调度
  ↓
目标目录完成后唤醒当前目录继续
```

### 5.3 待处理变更标记格式（落盘）

```markdown
## 待处理变更

### 变更 1
- **来源**: <origin-skill> (<origin-path>)
- **类型**: 接口变更/行为变更/依赖变更
- **描述**: ...
- **影响**: ...
- **状态**: [WAITING_FOR_WORKER]
```

---

## 6. 约束

- 必须遵循调用链顺序，不得跳步
- 三错即停适用（SSOT：`03_workflow/three_strike_rule.md`）
- 文档必须同步（由 `sop-document-sync` 负责）
- 实现类 Skill 不跨越 design.md 边界修改代码
