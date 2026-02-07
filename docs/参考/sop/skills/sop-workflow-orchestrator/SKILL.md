---
name: "sop-task-triage"
description: "Task triage workflow to determine fast path or deep path. Invoke when receiving a new task request to analyze complexity and assign workflow."
---

# Task Triage Workflow

## Input

```markdown
## Task Request
[User request]

## Context
- Project type: [new/feature/refactor]
- Related files: [list]
- Urgency: [high/medium/low]
- Constraints: [constraints]
```

## Workflow Steps

### Step 1: Analyze Task Complexity

Check conditions:
| Condition | Fast Path | Deep Path |
|-----------|-----------|-----------|
| Single file | ✅ Yes | ❌ No |
| Lines < 30 | ✅ Yes | ❌ No |
| No logic change | ✅ Yes | ❌ No |
| Cross-file | ❌ No | ✅ Yes |
| New feature | ❌ No | ✅ Yes |
| Refactor | ❌ No | ✅ Yes |
| API change | ❌ No | ✅ Yes |

### Step 2: Select Path

**Fast Path** (all conditions met):
- Single file + <30 lines + no logic change

**Deep Path** (any condition met):
- Cross-file / new feature / refactor / API change / architecture

### Step 3: Assign Roles

**Fast Path Flow**:
```
Explorer → Worker → Librarian
```

**Deep Path Flow**:
```
New project: Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
Feature:      Analyst → Oracle → Worker → Librarian
```

## Output

```markdown
## Triage Result

### Path Selection
- [ ] Fast Path
- [x] Deep Path

### Reason
[Why this path]

### Role Assignment
| Stage | Role | Task |
|-------|------|------|
| 1 | [Role] | [Task] |

### Next
@[Role]: [Specific task]
```

## Constraints

- Must accurately judge complexity
- Must consider dependencies
- Must provide clear next steps
