# Documentation Standards

> **Complete guide for writing and maintaining documentation**  
> *Last Updated: 2026-01-14*

---

## 📋 Overview

### Purpose
This document defines the standards for all documentation in the rust-tool-v2 project. Following these standards ensures:
- Consistency across all documentation
- Clear communication for all audiences
- Easy maintenance and updates
- Professional quality

### Audience
- **Technical Writers**: Creating new documentation
- **Developers**: Updating API docs
- **Maintainers**: Reviewing contributions
- **Users**: Reading guides

---

## 🎯 Documentation Types

### 1. User Guides
**Purpose**: Help users accomplish tasks  
**Audience**: End users, operators  
**Tone**: Friendly, direct, task-oriented  
**Examples**: USER_GUIDE.md, TUTORIAL.md

### 2. Developer Guides
**Purpose**: Explain architecture and development practices  
**Audience**: Contributors, maintainers  
**Tone**: Technical, precise, comprehensive  
**Examples**: DEVELOPMENT_GUIDE.md, PLUGIN_DEVELOPMENT.md

### 3. API Reference
**Purpose**: Document commands, functions, and interfaces  
**Audience**: Developers, power users  
**Tone**: Formal, structured, complete  
**Examples**: API_REFERENCE.md, API_INDEX.md

### 4. Tutorials
**Purpose**: Step-by-step learning  
**Audience**: New users  
**Tone**: Encouraging, progressive, hands-on  
**Examples**: TUTORIAL.md

### 5. Troubleshooting
**Purpose**: Solve common problems  
**Audience**: All users  
**Tone**: Empathetic, solution-focused  
**Examples**: TROUBLESHOOTING.md

### 6. Migration Guides
**Purpose**: Help users migrate from alternatives  
**Audience**: Users of other tools  
**Tone**: Comparative, encouraging, detailed  
**Examples**: MIGRATION_GUIDE.md

---

## 📝 Writing Standards

### Language & Tone

#### 1. Use Clear, Simple Language
✅ **Good:**
```markdown
Run `cargo run -- workflow execute workflow.yaml` to execute a workflow.
```

❌ **Avoid:**
```markdown
The execution of a workflow can be initiated by invoking the command 
`cargo run -- workflow execute workflow.yaml` which will commence the 
processing of the specified workflow definition file.
```

#### 2. Be Direct and Active
✅ **Good:**
```markdown
Create a workflow file.
Run the command.
Check the output.
```

❌ **Avoid:**
```markdown
A workflow file should be created.
The command should be run.
The output should be checked.
```

#### 3. Use Consistent Terminology
| Concept | Use | Avoid |
|---------|-----|-------|
| Execute workflow | `execute`, `run` | `invoke`, `trigger`, `start` |
| Command | `command`, `CLI` | `tool`, `utility`, `program` |
| Workflow file | `workflow file`, `YAML file` | `script`, `config`, `definition` |
| Parameter | `parameter`, `option` | `argument`, `flag` (unless specific) |

### Structure & Organization

#### 1. Hierarchical Headings
```markdown
# Title (H1) - Document name
## Section (H2) - Major topics
### Subsection (H3) - Specific items
#### Detail (H4) - Fine details
```

**Rules:**
- Only one H1 per document
- H2 for main sections
- H3-H4 for subsections
- Never skip levels (H1 → H3)

#### 2. Logical Flow
```
1. Introduction (what & why)
2. Prerequisites
3. Quick start / Basic usage
4. Detailed explanation
5. Advanced topics
6. Examples
7. Troubleshooting
8. Related resources
```

#### 3. Consistent Formatting

**Code Blocks:**
```markdown
```bash
# Use language identifier
cargo run -- --help
```

**Inline Code:**
```markdown
Use `--verbose` flag for detailed output.
```

**Lists:**
```markdown
- Use hyphens for unordered lists
- Keep items parallel in structure
- Use numbers for ordered lists

1. First step
2. Second step
3. Third step
```

**Tables:**
```markdown
| Option | Description | Default |
|--------|-------------|---------|
| `--verbose` | Show detailed output | `false` |
```

### Markdown Conventions

#### 1. Linking
```markdown
# Relative links (preferred)
[USER_GUIDE.md](USER_GUIDE.md)

# With anchor
[Quick Start](#quick-start)

# External links
[Rust Documentation](https://doc.rust-lang.org/)
```

**Rules:**
- Use relative links within docs/
- Use anchors for internal navigation
- Always use descriptive text
- Never use "click here"

#### 2. Code Examples
```markdown
**Good:**
```bash
# Comment explaining what this does
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

**Better:**
```bash
# Execute a simple workflow
cargo run -- workflow execute workflows/basic/hello-world.yaml

# Expected output:
# Workflow completed successfully
```

**Best:**
```bash
# Execute a simple workflow
cargo run -- workflow execute workflows/basic/hello-world.yaml

# Expected output:
# Workflow completed successfully

# Common errors:
# - File not found: Check path to workflow file
# - Syntax error: Validate YAML structure
```
```

#### 3. Warnings & Notes
```markdown
> **⚠️ Warning:** This operation cannot be undone. Always backup first.

> **ℹ️ Note:** This feature requires the `lancedb` feature flag.

> **💡 Tip:** Use `--dry-run` to preview changes before executing.
```

#### 4. Emojis (Optional)
Use sparingly for visual hierarchy:
- ✅ Success/Complete
- ❌ Error/Avoid
- ⚠️ Warning
- ℹ️ Information
- 💡 Tip
- 🚀 Quick start
- 🔧 Technical
- 📚 Reference
- 🎯 Goals

---

## 📄 Document Templates

### Template 1: User Guide

```markdown
# [Feature Name] Guide

> **Purpose**: [What this feature does]  
> **Audience**: [Who should use this]  
> **Prerequisites**: [What they need to know]

---

## 🚀 Quick Start

```bash
# Simplest possible example
cargo run -- [command] --help
```

## 📖 Basic Usage

### [Common Use Case 1]
```bash
# Example with explanation
cargo run -- [command] [options]
```

### [Common Use Case 2]
```bash
# Another example
cargo run -- [command] [options]
```

## ⚙️ Options Reference

| Option | Description | Default | Required |
|--------|-------------|---------|----------|
| `--source` | Source directory | None | Yes |
| `--dest` | Destination directory | None | Yes |

## 🎯 Examples

### Example 1: [Scenario]
```bash
# Command
cargo run -- [command] [options]

# Explanation
This will [what happens].
```

### Example 2: [Scenario]
```bash
# Command
cargo run -- [command] [options]

# Explanation
This will [what happens].
```

## 🔧 Troubleshooting

### Problem: [Common issue]
**Solution:** [Solution]

### Problem: [Common issue]
**Solution:** [Solution]

## 📚 Related
- [INDEX.md](INDEX.md) - Complete documentation index
- [API_INDEX.md](API_INDEX.md) - Command reference
```

### Template 2: API Reference

```markdown
# [Component] API Reference

> **Last Updated**: [Date]  
> **Version**: [Version]

---

## Overview

[Brief description of component]

## Types

### [Type Name]
```rust
pub struct TypeName {
    pub field: Type,
}
```

**Fields:**
- `field`: Description

## Functions

### `function_name()`
```rust
pub fn function_name(param: Type) -> Result<Output>
```

**Parameters:**
- `param`: Description

**Returns:**
- `Result<Output>`: Description

**Example:**
```rust
let result = function_name(value)?;
```

## Traits

### `TraitName`
```rust
#[async_trait]
pub trait TraitName: Send + Sync {
    async fn method(&self, param: Type) -> Result<Output>;
}
```

**Methods:**
- `method()`: Description

## Enums

### `EnumName`
```rust
pub enum EnumName {
    Variant1,
    Variant2(Type),
}
```

**Variants:**
- `Variant1`: Description
- `Variant2(Type)`: Description

---

**← Back to [INDEX.md](INDEX.md)**
```

### Template 3: Troubleshooting

```markdown
# Troubleshooting [Feature]

> **Common issues and solutions**

---

## 🔍 Quick Fixes

Try these first:
1. [Fix 1]
2. [Fix 2]
3. [Fix 3]

## 🚨 Common Issues

### Issue: [Problem description]
**Symptoms:** [What you see]

**Causes:**
- [Cause 1]
- [Cause 2]

**Solutions:**
```bash
# Solution 1
command to fix

# Solution 2
alternative command
```

### Issue: [Problem description]
**Symptoms:** [What you see]

**Causes:**
- [Cause 1]

**Solutions:**
```bash
command to fix
```

## 🔧 Advanced Debugging

### Enable Logging
```bash
RUST_LOG=debug cargo run -- [command]
```

### Check System
```bash
# Check Rust version
rustc --version

# Check disk space
df -h

# Check permissions
ls -la /path/to/directory
```

## 📞 Get More Help

- [INDEX.md](INDEX.md) - Find related docs
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Debugging techniques
- [README.md](../README.md) - Project overview

---

**← Back to [INDEX.md](INDEX.md)**
```

---

## 🔍 Quality Checklist

### Before Publishing
- [ ] **Spelling**: Run spell check
- [ ] **Grammar**: Check for errors
- [ ] **Links**: All links work
- [ ] **Code**: All examples run
- [ ] **Consistency**: Follows standards
- [ ] **Completeness**: Covers all cases
- [ ] **Clarity**: Easy to understand
- [ ] **Tone**: Appropriate for audience

### Technical Review
- [ ] **Accuracy**: Information is correct
- [ ] **Current**: Up to date with code
- [ ] **Examples**: All examples work
- [ ] **Commands**: Commands are valid
- [ ] **Options**: All options documented
- [ ] **Errors**: Common errors covered

### User Experience
- [ ] **Navigation**: Easy to find information
- [ ] **Readability**: Clear structure
- [ ] **Scannable**: Good use of headings/lists
- [ ] **Actionable**: Clear next steps
- [ ] **Complete**: No missing information

---

## 📊 Documentation Structure

### Project Layout
```
docs/
├── INDEX.md                    # Master index
├── USER_GUIDE.md              # User manual
├── DEVELOPMENT_GUIDE.md       # Dev practices
├── API_INDEX.md               # Command reference
├── TROUBLESHOOTING.md         # Problem solving
├── MIGRATION_GUIDE.md         # Python → Rust
├── DOCUMENTATION_STANDARDS.md # This file
├── CHEATSHEET.md              # Quick reference
├── TUTORIAL.md                # Step-by-step
├── PROJECT_OVERVIEW.md        # Architecture
├── PLUGIN_DEVELOPMENT.md      # Extensions
├── API_REFERENCE.md           # Technical specs
├── API_USAGE_GUIDE.md         # Usage examples
└── FILE_MANAGEMENT/           # File mgmt docs
    ├── TOOLS_GUIDE.md
    ├── API_REFERENCE.md
    ├── HUMAN_DECISION_GUIDE.md
    ├── TOOLS_INDEX.md
    └── WORKFLOW_TEMPLATES.md
```

### Cross-References
```markdown
# In any document, link to:
- [INDEX.md](INDEX.md) - Find any document
- [USER_GUIDE.md](USER_GUIDE.md) - Usage examples
- [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md) - Dev practices
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Solutions
- [API_INDEX.md](API_INDEX.md) - Commands
```

---

## 🎯 Writing Process

### Step 1: Plan
```markdown
1. Identify audience
2. Define purpose
3. Outline structure
4. Gather examples
5. Check existing docs
```

### Step 2: Draft
```markdown
1. Write H1 and intro
2. Create main sections (H2)
3. Add subsections (H3-H4)
4. Write examples
5. Add cross-references
```

### Step 3: Review
```markdown
1. Check quality checklist
2. Verify all examples work
3. Test all commands
4. Check all links
5. Get peer review
```

### Step 4: Publish
```markdown
1. Update INDEX.md
2. Update README.md if needed
3. Check for broken links
4. Verify navigation
5. Announce updates
```

---

## 🔄 Maintenance Standards

### Update Schedule
- **Weekly**: Check for outdated information
- **Monthly**: Review all examples
- **Quarterly**: Full documentation audit

### Update Triggers
- New feature added
- API changes
- Command changes
- User feedback
- Bug fixes affecting usage

### Version Tracking
```markdown
> **Last Updated**: 2026-01-14  
> **Version**: 1.0.0  
> **Changes**: [Brief description]
```

---

## 📝 Style Guide

### Voice & Tone

#### Active Voice (Preferred)
```markdown
✅ Run the command to execute the workflow.
❌ The command is run to execute the workflow.
```

#### Second Person (You)
```markdown
✅ You can run the command with --verbose.
❌ Users can run the command with --verbose.
```

#### Positive Language
```markdown
✅ Use --verbose for detailed output.
❌ Don't use --verbose unless you need details.
```

### Formatting Rules

#### Code Blocks
- Always specify language
- Keep lines under 80 characters
- Add comments for clarity
- Show expected output when helpful

#### Inline Code
- Use for commands, options, file paths
- Use for code snippets
- Use for error messages

#### Lists
- Use parallel structure
- Keep items concise
- Use consistent punctuation

#### Tables
- Left-align text
- Use clear headers
- Keep concise

### Terminology Consistency

| Term | Use For | Example |
|------|---------|---------|
| Execute | Running workflows | `execute a workflow` |
| Run | Running commands | `run the command` |
| Parameter | Command options | `--source parameter` |
| Flag | Boolean options | `--verbose flag` |
| Option | Any command-line choice | `available options` |
| Workflow | Workflow files | `create a workflow` |
| Tool | Individual tools | `file-classifier tool` |
| Command | CLI commands | `cargo run -- command` |

---

## 📚 Examples from Existing Docs

### Good Example: USER_GUIDE.md
```markdown
## 🚀 Quick Start

### Execute Your First Workflow
```bash
cargo run -- workflow execute workflows/basic/hello-world.yaml
```

This will execute a simple workflow that prints "Hello World".
```

### Good Example: DEVELOPMENT_GUIDE.md
```markdown
## Code Style

### Import Order
```rust
// 1. Standard library
use std::sync::Arc;

// 2. External crates (alphabetical)
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// 3. Internal modules
use crate::core::WorkflowId;
```
```

### Good Example: TROUBLESHOOTING.md
```markdown
### Cargo Build Fails

**Symptoms:**
```
error: could not compile `rust-tool-v2`
```

**Solutions:**
```bash
# 1. Check Rust version
rustc --version

# 2. Update Rust
rustup update
```
```

---

## 🎯 Common Mistakes to Avoid

### ❌ Don't Do This
```markdown
# Too vague
## Options
Use --verbose for more info.

# No examples
Run the command.

# Broken links
[Click here](some-file.md)

# Inconsistent terminology
Use "run" in one place, "execute" in another

# Missing prerequisites
No mention of required Rust version

# No error handling
Assumes everything works
```

### ✅ Do This Instead
```markdown
## Options Reference

| Option | Description | Default |
|--------|-------------|---------|
| `--verbose` | Show detailed execution logs | `false` |

**Example:**
```bash
cargo run -- workflow execute workflow.yaml --verbose
```

**Expected Output:**
```
[INFO] Loading workflow...
[DEBUG] Parsing YAML...
[INFO] Executing step 1...
```

**Common Errors:**
- File not found: Check path
- Syntax error: Validate YAML
```

---

## 📖 Reference Links

### Internal Documentation
- **[INDEX.md](INDEX.md)** - Complete navigation
- **[README.md](../README.md)** - Project overview
- **[USER_GUIDE.md](USER_GUIDE.md)** - Usage manual
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - Dev guide
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Solutions

### External Resources
- **Rust Documentation**: https://doc.rust-lang.org/
- **Markdown Guide**: https://www.markdownguide.org/
- **Technical Writing**: https://developers.google.com/tech-writing

---

## ✅ Quick Reference

### Essential Commands
```bash
# Check all docs for broken links
grep -r "\[.*\](.*\.md)" docs/ | grep -v "http"

# Find all markdown files
find docs/ -name "*.md"

# Count lines in docs
wc -l docs/*.md
```

### Common Patterns
```markdown
# Link to another doc
[Link text](FILENAME.md)

# Link with anchor
[Link text](FILENAME.md#section-name)

# Code block
\`\`\`bash
command
\`\`\`

# Warning
> **⚠️ Warning:** Important information

# Note
> **ℹ️ Note:** Helpful information

# Tip
> **💡 Tip:** Pro tip
```

---

## 🔄 Updating This Document

### When to Update
- New documentation type added
- New tool or feature
- User feedback on clarity
- Pattern changes in existing docs

### How to Update
1. Add new section or pattern
2. Provide clear examples
3. Update version/date
4. Announce to team
5. Update INDEX.md

---

**← Back to [INDEX.md](INDEX.md)** | **Top** ↑