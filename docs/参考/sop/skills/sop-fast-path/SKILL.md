---
name: "sop-fast-path"
description: "Fast path workflow for single-file, small changes. Invoke when task is triaged as fast path (single file, <30 lines, no logic change)."
---

# Fast Path Workflow

## Input

```markdown
## Task
[Change description]

## Target
- File: [path]
- Lines: [N]
- Type: [fix/doc/config]
```

## Workflow Steps

### Step 1: Code Audit

**Purpose**: Quick impact assessment

**Actions**:
1. Read target file
2. Identify dependencies
3. Check for risks

**Output**: Brief audit report

### Step 2: Code Modification

**Purpose**: Implement the change

**Actions**:
1. Create checkpoint
2. Apply changes
3. Run tests
4. Quality checks

**Stop Point**: Show diff for review

### Step 3: Document Sync

**Purpose**: Update related docs

**Actions**:
1. Update code comments
2. Update related docs
3. Mark `[completed]`

## Output

```markdown
## Fast Path Complete

### Changes
- File: [path]
- Lines: [+N/-M]

### Tests
- [x] Passed

### Quality
- [x] Lint passed
- [x] Type check passed

### Status
`[completed]`
```

## Constraints

- Single file only
- <30 lines changed
- No logic changes
- Must pass tests
