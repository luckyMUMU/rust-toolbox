# examples/templates/ - Workflow Templates & Patterns

## OVERVIEW
Production-ready workflow templates with comprehensive documentation and interactive examples.

## WORKFLOW TEMPLATES

### Interactive Classification
**File**: `interactive-classification-workflow.yaml`  
**Purpose**: Folder classification with human decision support  
**Key Features**:
- AI-powered classification with confidence scoring
- Interactive prompts for human review
- Experimental mode for advanced heuristics
- Audit trail for all decisions

**Usage**: `cargo run --example interactive-classification-example`

### Interactive Batch Processing
**File**: `interactive-batch-processing-workflow.yaml`  
**Purpose**: Bulk file operations with progress tracking  
**Key Features**:
- Real-time progress monitoring
- Per-file error isolation
- Batch approval workflow
- Performance metrics collection

**Usage**: `cargo run --example interactive-batch-processing-example`

### Interactive Merge
**File**: `interactive-merge-workflow.yaml`  
**Purpose**: Folder merging with multiple strategies  
**Key Features**:
- Multiple merge strategies (overwrite, skip, rename)
- Conflict resolution
- Human validation before commit
- Rollback capability

**Usage**: `cargo run --example interactive-merge-example`

## CONFIGURATION TEMPLATES

### Environment Config Examples
**File**: `environment-config-examples.yaml`  
Demonstrates hierarchical configuration with environment variables.

### Common Use Cases
**File**: `common-use-cases.yaml`  
Real-world workflow patterns and best practices.

## DOCUMENTATION

### Configuration Guide
**File**: `CONFIGURATION_GUIDE.md`  
Complete configuration reference with examples.

### Human Decision Best Practices
**File**: `HUMAN_DECISION_BEST_PRACTICES.md`  
Guidelines for interactive workflows and human-in-the-loop patterns.

### Parameter Documentation
**File**: `PARAMETER_DOCUMENTATION.md`  
Template parameter syntax and expansion examples.

### Usage Examples
**File**: `USAGE_EXAMPLES.md`  
Step-by-step tutorials for common scenarios.

### README
**File**: `README.md`  
Overview of all templates and how to use them.

## RULES & RULE FILES

### Classification Rules
- `classification-rules-example.json`: Basic rules
- `classification-rules-comprehensive.json`: Advanced rules
- `classification-rules-chinese.json`: Chinese-specific rules

## EXAMPLE IMPLEMENTATIONS

Each workflow template has a corresponding Rust example:
- `interactive-classification-example.rs`
- `interactive-batch-processing-example.rs`
- `interactive-merge-example.rs`
- `real-world-scenario-example.rs`

These demonstrate:
- Programmatic workflow creation
- Custom tool integration
- Error handling patterns
- State management

## BEST PRACTICES

1. **Start with templates**: Copy and modify for your use case
2. **Test interactively**: Use the example files to test workflows
3. **Customize rules**: Adapt classification rules to your domain
4. **Document parameters**: Use parameter-documentation.md as reference
5. **Follow patterns**: Use common-use-cases.yaml for inspiration

## QUICK START

```bash
# Browse templates
ls examples/templates/

# Run interactive example
cargo run --example interactive-classification-example

# Execute workflow from template
cargo run -- workflow execute examples/templates/interactive-classification-workflow.yaml
```
