---
name: "sop-code-implementation"
description: "Code implementation workflow for physical coding. Invoke when implementation design is approved and ready for coding."
---

# Code Implementation Workflow

## Input

```markdown
## Implementation Design
[Design document link]

## Context
- Tech stack: [stack]
- Test command: [command]
- Lint command: [command]
```

## Workflow Steps

### Step 1: Checkpoint

**Purpose**: Create rollback point

**Actions**:
1. Mark `[in_progress]`
2. Note current state
3. Prepare rollback plan

### Step 2: Code Development

**Purpose**: Implement by design

**Actions**:
1. Follow design doc strictly
2. No design changes
3. Add code comments

### Step 3: Testing

**Purpose**: Verify correctness

**Actions**:
1. Run unit tests
2. Run integration tests
3. Fix failures

### Step 4: Quality Check

**Purpose**: Ensure code quality

**Actions**:
1. Run linter
2. Run type checker
3. Fix issues

### Step 5: Diff Review

**Purpose**: Present changes

**Actions**:
1. Generate diff
2. Show summary
3. Wait for approval

## Output

```markdown
## Implementation Complete

### Changes
- Files: [N]
- Lines: [+N/-M]

### Tests
- Unit: [pass/fail]
- Integration: [pass/fail]

### Quality
- Lint: [pass/fail]
- Type check: [pass/fail]

### Diff
[diff]

### Status
Waiting for review
```

## Constraints

- Follow design strictly
- No design changes
- Must pass tests
- Must pass quality checks

## 3-Strike Rule

| Strike | Condition | Action |
|--------|-----------|--------|
| 1 | Test/quality fail | Auto-fix |
| 2 | Fail again | Audit + redesign |
| 3 | Fail again | **Break**, user decision |
