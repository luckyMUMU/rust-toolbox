---
name: "sop-progress-monitoring"
description: "Progress monitoring workflow for tracking execution and triggering circuit breakers. Invoke when monitoring task execution or detecting anomalies."
---

# Progress Monitoring Workflow

## Input

```markdown
## Monitor Task
[ID/name]

## Current State
- Stage: [current]
- Role: [current]
- Start: [time]
- Elapsed: [duration]

## Latest Feedback
[Role feedback]

## Failures
- Count: [0/1/2/3]
- Reason: [reason]
```

## Workflow Steps

### Step 1: State Collection

**Purpose**: Gather current status

**Actions**:
1. Read task state
2. Check stage progress
3. Note any blockers

### Step 2: Deviation Detection

**Purpose**: Identify issues

**Actions**:
1. Compare to plan
2. Check for delays
3. Identify risks

### Step 3: Risk Assessment

**Purpose**: Evaluate severity

**Severity**:
- 🔴 Critical: Blocked, needs immediate action
- 🟡 Warning: Delayed, needs attention
- 🟢 Normal: On track

### Step 4: Decision

**Purpose**: Determine next action

**Options**:
- Continue: Normal progress
- Alert: Warning, notify stakeholders
- Break: Critical, trigger circuit breaker

## Output

### Progress Update
```markdown
## Progress Update

### State
- Task: [name]
- Stage: [stage]
- Status: [in_progress/done/blocked]

### Progress
| Stage | Status | Owner |
|-------|--------|-------|
| [name] | [status] | [role] |

### Risks
- 🟡 [warning]
- 🔴 [critical]

### Next
@[role]: [task]
```

### Circuit Breaker
```markdown
## Circuit Breaker Triggered

### Reason
- Type: [3 strikes/deadlock/high risk]
- Detail: [description]

### State
- Failures: [count]/3
- Roles: [list]
- Blocked: [duration]

### Options
- A: [desc]
- B: [desc]

**User decision required**
```

## Constraints

- Read all: code, docs, status
- Write status only
- Trigger `[FUSION_TRIGGERED]` when needed
- No implementation

## 3-Strike Rule

| Strike | Condition | Action |
|--------|-----------|--------|
| 1 | Implementation fails | Log, allow retry |
| 2 | Fails again | Audit + redesign |
| 3 | Fails again | **Break**, `[FUSION_TRIGGERED]` |
