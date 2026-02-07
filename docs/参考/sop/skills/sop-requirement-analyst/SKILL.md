---
name: "sop-requirement-analysis"
description: "Requirement analysis workflow to elicit needs and generate PRD. Invoke when starting a new feature or when requirements need clarification."
---

# Requirement Analysis Workflow

## Input

```markdown
## User Request
[Original description]

## Context
- Background: [info]
- Constraints: [constraints]
- Related: [docs]
```

## Workflow Steps

### Step 1: Dialogue Elicitation

**Purpose**: Clarify user needs

**Rounds**:
1. **Core**: Understand problem and goal
2. **Clarify**: Resolve ambiguities
3. **Explore**: Discover hidden needs
4. **Confirm**: Validate understanding

### Step 2: PRD Generation

**Purpose**: Document requirements

**Sections**:
1. Overview (background, goals, metrics)
2. Business analysis (value, process, rules)
3. User analysis (persona, scenarios, stories)
4. Functional requirements (Must/Should/Could/Won't)
5. Non-functional requirements (performance, security)

**Output**: `docs/01_requirements/[feature]_prd.md`

### Step 3: Multi-Dimension Analysis

| Dimension | Check | Output |
|-----------|-------|--------|
| Business | Goals, value, process | Conclusion + risk |
| User | Persona, scenario, pain | Conclusion + risk |
| Function | Scope, priority, dependency | Conclusion + risk |
| Tech | Feasibility, constraint, integration | Conclusion + risk |
| Risk | Uncertainty, mitigation | Conclusion + risk |
| Acceptance | Criteria, metrics | Conclusion + risk |

### Step 4: User Confirmation

**Purpose**: Validate PRD

**Checklist**:
- [ ] Requirements correct?
- [ ] Scope accurate?
- [ ] Priority reasonable?
- [ ] Any missing?

**Stop Point**: `[WAITING_FOR_REQUIREMENTS]`

## Output

```markdown
## Requirement Analysis Complete

### PRD
- Location: `docs/01_requirements/[feature]_prd.md`
- Link: [link]

### Analysis Summary
| Dimension | Conclusion | Risk |
|-----------|------------|------|
| Business | [conclusion] | [H/M/L] |
| User | [conclusion] | [H/M/L] |
| Function | [conclusion] | [H/M/L] |
| Tech | [conclusion] | [H/M/L] |
| Risk | [conclusion] | [H/M/L] |
| Acceptance | [conclusion] | [H/M/L] |

### User Confirmation
- [ ] Requirements correct
- [ ] Scope accurate
- [ ] Priority reasonable
- [ ] No missing items

### Next
After confirmation → Architecture Design
```

## Constraints

- Must have multi-round dialogue
- Must cover 6 dimensions
- Must get user confirmation
- Must define clear boundaries
