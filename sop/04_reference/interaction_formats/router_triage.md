---
version: v2.4.0
updated: 2026-02-22
---

# Workflow Orchestrator 分诊输出格式

```markdown
## 任务分诊结果

### 路径
- Path: [fast/deep/deep+tdd]

### 需求层级
- L: [L1/L2/L3]

### Skill 调用链
| Stage | Skill | Output |
|---|---|---|
| 1 | sop-requirement-analyst | PRD/MRD/FRD |
| 2 | sop-architecture-design | L2(.md) + ADR |
| 3 | sop-architecture-reviewer | design review report |
| 4 | sop-implementation-designer | design.md |
| 5 | (optional) sop-test-design-csv | test cases CSV |
| 6 | sop-code-implementation | code + diff |
| 6 | (optional) sop-test-implementation | test code |
| 7 | sop-code-review | review report |
| 8 | sop-document-sync | indexes/changelog sync |
| * | (optional) sop-progress-supervisor | scheduling / fuse |

### 文档位置
参见 04_reference/document_directory_mapping.md

### 下一步
CMD REQ_ANALYZE(input)
```

