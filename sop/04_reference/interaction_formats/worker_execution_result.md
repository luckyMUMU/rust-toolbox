---
version: v2.12.0
updated: 2026-02-25
---

# 实现执行结果格式（Implementation Result）

```markdown
## 执行结果

### Directory
- path: [dir]
- status: [DIR_WORKING/DIR_WAITING_DEP/DIR_COMPLETED]
- deps: [..]

### Changes
| file | summary |
|---|---|
| [path] | [..] |

### Checks
| check | status | notes |
|---|---|---|
| compile | [passed/failed/na] | [..] |
| test:l1 | [passed/failed/na] | [..] |
| test:l2 | [passed/failed/na] | [..] |
| test:l3 | [passed/failed/na] | [..] |
| test:l4 | [passed/failed/na] | [..] |
| lint | [passed/failed/na] | [..] |
| typecheck | [passed/failed/na] | [..] |

### Diff
- [link or summary]

### Next
- CMD: [..]
```

