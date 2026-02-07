---
name: "sop-document-maintenance"
description: "Document maintenance workflow for index updates and progressive disclosure. Invoke when documents need synchronization or status updates."
---

# Document Maintenance Workflow

## Input

```markdown
## Task
- Type: [add/update/status/archive]
- Target: [document path]
- Change: [description]

## Impact
- Parent: [path]
- Related: [paths]
```

## Workflow Steps

### Step 1: Content Update

**Purpose**: Update document

**Actions**:
1. Apply changes
2. Update status
3. Mark `[in_progress]` or `[completed]`

### Step 2: Parent Index Update

**Purpose**: Maintain hierarchy

**Actions**:
1. Update parent summary
2. Add/update links
3. Ensure progressive disclosure

### Step 3: Cross-Reference Sync

**Purpose**: Maintain consistency

**Actions**:
1. Update related docs
2. Fix broken links
3. Sync status marks

### Step 4: Validation

**Purpose**: Check quality

**Actions**:
1. Validate links
2. Check structure
3. Verify format

## Output

```markdown
## Document Maintenance Complete

### Summary
- Type: [type]
- Target: [path]
- Status: [old] → [new]

### Updated
| Path | Change | Note |
|------|--------|------|
| [path] | [add/modify] | [note] |

### Validation
- Links: [valid/total]
- Structure: [pass/fail]
- Format: [pass/fail]

### Status
`[completed]`
```

## Constraints

- Parent docs: summary + links only
- Progressive disclosure
- Valid links required
- Status marks: `[in_progress]` / `[completed]`

## Document Levels

| Level | Path | Content |
|-------|------|---------|
| L1 | `docs/` | Concept, navigation |
| L2 | `docs/01_requirements/` | PRD |
| L3 | `docs/02_logical_workflow/` | Architecture |
| L4 | `src/**/design.md` | Implementation |
