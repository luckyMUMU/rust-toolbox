---
name: "sop-code-audit"
description: "Code audit workflow for impact assessment and risk identification. Invoke before implementation to understand existing code and assess change impact."
---

# Code Audit Workflow

## Input

```markdown
## Audit Target
- Files: [paths]
- Scope: [change description]

## Context
- Project type: [new/existing]
- Related modules: [list]
- Constraints: [constraints]
```

## Workflow Steps

### Step 1: Code Reading

**Purpose**: Understand current implementation

**Actions**:
1. Read target files
2. Identify key logic
3. Note dependencies

### Step 2: Dependency Analysis

**Purpose**: Map relationships

**Actions**:
1. Identify imports
2. Map call chains
3. Find coupling points

### Step 3: Impact Assessment

**Purpose**: Evaluate change scope

**Actions**:
1. Identify affected modules
2. Assess impact level
3. Estimate effort

### Step 4: Risk Identification

**Purpose**: Find potential issues

**Severity**:
- 🔴 Critical: Breaking changes
- 🟡 Warning: High risk
- 🟢 Suggestion: Improvements

## Output

```markdown
## Code Audit Report

### Target
- Files: [paths]
- Lines: [count]

### Impact Analysis
| Module | Level | Description |
|--------|-------|-------------|
| [name] | [H/M/L] | [desc] |

### Dependencies
```
[Module A] → [Module B] → [Module C]
```

### Risks
- 🔴 [Critical]: [desc]
- 🟡 [Warning]: [desc]
- 🟢 [Suggestion]: [desc]

### Recommendations
1. [rec]
2. [rec]
```

## Constraints

- Read-only
- No modifications
- Objective analysis
- Clear risk levels
