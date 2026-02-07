---
name: "sop-implementation-design"
description: "Implementation design workflow for creating detailed technical designs. Invoke when architecture is approved and ready for implementation design."
---

# Implementation Design Workflow

## Input

```markdown
## Architecture
[Architecture document link]

## Context
- Tech stack: [stack]
- Constraints: [constraints]
- Existing code: [files]
```

## Workflow Steps

### Step 1: Tech Mapping

**Purpose**: Map architecture to tech stack

**Actions**:
1. Identify tech options
2. Compare alternatives
3. Select best fit

### Step 2: Module Design

**Purpose**: Design project-specific modules

**Actions**:
1. Map to file structure
2. Define module boundaries
3. Assign responsibilities

### Step 3: Interface Contract

**Purpose**: Define detailed interfaces

**Input**:
| Param | Type | Required | Desc |
|-------|------|----------|------|
| [name] | [type] | [Y/N] | [desc] |

**Output**:
| Return | Type | Desc |
|--------|------|------|
| [name] | [type] | [desc] |

**Dependencies**:
| Module | Interface | Purpose |
|--------|-----------|---------|
| [name] | [iface] | [purpose] |

### Step 4: Task Decomposition

**Purpose**: Create executable tasks

**Actions**:
1. Break down work
2. Define dependencies
3. Estimate effort

### Step 5: Test Strategy

**Purpose**: Define testing approach

**Actions**:
1. Unit test scope
2. Integration test scope
3. Coverage targets

## Output

```markdown
## Implementation Design Complete

### Document
- Location: `src/[module]/design.md`
- Link: [link]

### Tech Choices
| Component | Choice | Rationale |
|-----------|--------|-----------|
| [name] | [choice] | [reason] |

### Interface Contract
[Input/Output/Dependencies tables]

### Task List
- [ ] [task 1]
- [ ] [task 2]

### Test Strategy
- Unit: [scope]
- Integration: [scope]

### Stop Point
`[WAITING_FOR_DESIGN]`
```

## Constraints

- Project-specific
- Traceable to architecture
- Clear interfaces
- Actionable tasks

## design.md Rules

**Module-based**:
- Create in module root
- One per independent module

**Complexity-based**:
| Complexity | Lines | Action |
|------------|-------|--------|
| Low | <100 | Skip |
| Medium | 100-500 | Brief |
| High | >500 | Full |

**Required**: Interface contract
