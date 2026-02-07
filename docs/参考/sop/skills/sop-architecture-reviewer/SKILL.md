---
name: "sop-architecture-review"
description: "Architecture review workflow with multi-round iteration. Invoke when architecture design is complete and needs quality review."
---

# Architecture Review Workflow

## Input

```markdown
## Review Target
[Architecture document link]

## Round
[N]

## Previous Response
[Designer reply summary]
```

## Workflow Steps

### Step 1: Dimension Review

**Purpose**: Check 6 dimensions

| Dimension | Check |
|-----------|-------|
| Completeness | All requirements covered? |
| Consistency | Terms and logic aligned? |
| Feasibility | Technically achievable? |
| Performance | Meets requirements? |
| Security | Any vulnerabilities? |
| Scalability | Easy to extend? |

### Step 2: Issue Identification

**Purpose**: Find problems

**Severity**:
- 🔴 Critical: Must fix
- 🟡 Warning: Should fix
- 🟢 Suggestion: Nice to have

### Step 3: Iteration

**Purpose**: Resolve issues

**Max**: 3 rounds

**Flow**:
```
Round 1: Identify issues → Designer fixes
Round 2: Verify fixes → New issues?
Round 3: Final check → Pass or deadlock
```

## Output

### Continue Review
```markdown
## Review Round [N]

### Issues
| Severity | Location | Description | Fix |
|----------|----------|-------------|-----|
| 🔴 | [loc] | [desc] | [fix] |

### Next
@Designer: Fix issues above
```

### Pass
```markdown
## Review Passed ✅

### Stats
- Rounds: [N]
- Issues fixed: [N]

### Next
@Implementer: Start implementation design
```

### Deadlock
```markdown
## Review Deadlock

### Dispute
[Topic]: [conflict]

### Options
- A: [desc]
- B: [desc]

**User decision required**
```

## Constraints

- 6 dimensions
- Max 3 rounds
- Constructive feedback
- Clear severity levels
