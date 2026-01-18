I will implement the requested 13-step granular workflow by creating a new module with specialized tools that extend the existing AC automaton and text processing capabilities.

### Plan: Granular Classification Workflow Implementation

1.  **Create Granular Tools Module** (`src/plugins/file_management/classification_flow.rs`)
    *   Implement strict single-function `ToolNode`s for each step:
        *   `rule-loader`: Validates and loads rules.
        *   `rule-preprocessor`: Extends `TextProcessor` logic to prepare patterns (pinyin, lowercase).
        *   `ac-automaton-builder`: Prepares optimized pattern configuration for the automaton.
        *   `directory-scanner`: Scans source directories.
        *   `folder-name-preprocessor`: Normalizes folder names.
        *   `parallel-matcher`: Reuses `AhoCorasickMatcher` to perform parallel matching on the processed list.
        *   `score-calculator`: Implements the specific scoring logic (Simple/Combined/Weighted).
        *   `ambiguity-detector`: Logic to identify ambiguous classifications.
        *   `result-merger`: Merges automatic and potential manual results.
        *   `experimental-check`: Logic tool for safety validation.
        *   `report-generator`: Compiles execution statistics.

2.  **Register Tools** (`src/plugins/file_management/registry.rs`)
    *   Update the registry to expose these new tools alongside the existing ones.

3.  **Update Workflow Template** (`examples/templates/interactive-classification-workflow.yaml`)
    *   Rewrite the workflow nodes to strictly follow the 13 steps.
    *   Map data flow between these fine-grained nodes (e.g., passing `processed_patterns` from Step 2 to Step 3).

4.  **Integration**
    *   Expose the new module in `src/plugins/file_management/mod.rs`.

This approach ensures strict adherence to the "single function per node" requirement while reusing the robust `ac_automaton` and `text_processor` logic already present in the codebase.
