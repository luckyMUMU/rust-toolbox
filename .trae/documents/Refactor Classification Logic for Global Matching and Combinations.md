
I will refactor the file classification logic to support keyword combinations (AND logic) and separate the matching phase from the scoring phase, as requested.

### 1. Design Documentation
- **Create `src/plugins/file_management/DESIGN.md`**:
    - Document the new 2-phase classification architecture:
        - **Phase 1: Global Matching**: Use AC Automaton to find *all* occurrences of keywords in the text. This solves the "global unique matching" requirement by ensuring every keyword is detected regardless of which rule it belongs to.
        - **Phase 2: Rule Scoring**: Evaluate rules based on the set of matched keywords. Support both simple keywords (match = score) and combination keywords (all must match = score).
    - Define the updated `ClassificationRule` structure supporting `combinations`.

### 2. Implementation Steps

#### A. Refactor `ClassificationRule`
- **File**: `src/plugins/file_management/classification_tool.rs` (and/or `rule_config.rs`)
- **Action**: 
    - Update `ClassificationRule` struct to add `pub combinations: Option<Vec<Vec<String>>>`.
    - This structure (`Vec<Vec<String>>`) represents "OR list of AND lists", matching the user's example `["51", "酱"]` where both must be present.

#### B. Refactor AC Automaton Logic
- **File**: `src/plugins/file_management/ac_automaton.rs`
- **Action**:
    - Modify `AcAutomaton` (or its usage) to accept a simple list of unique keywords for construction, decoupling it from specific categories/rules during the build phase.
    - Ensure it returns all match positions for the input text.

#### C. Refactor Classification Engine
- **File**: `src/plugins/file_management/classification_tool.rs`
- **Action**:
    - Update `execute` method to implement the 2-phase logic:
        1.  **Collect Keywords**: Extract all simple keywords and all individual words from combinations across all rules. Deduplicate them to build the AC Automaton.
        2.  **Match**: Run AC Automaton on the folder name/text to get a set of *found keywords*.
        3.  **Score**: Iterate through all rules:
            - **Simple Keywords**: +1 score (or weight) for each found keyword.
            - **Combinations**: +1 score (or weight) ONLY if *all* keywords in a combination group are found.
    - This logic mirrors the Python script's behavior and allows multiple rules to match the same keywords without conflict.

#### D. Update Rule Loading
- **File**: `src/plugins/file_management/rule_config.rs`
- **Action**: Ensure the JSON loader can parse the new `combinations` field (or handle mixed types in `keywords` if we want to support the Python format directly, though a separate field is cleaner for Rust). I will implement logic to parse nested arrays in `keywords` from the JSON and move them into `combinations` automatically, preserving backward compatibility with the user's JSON file.

### 3. Verification
- **Action**: Run the `experimental_classify.rs` example again with the original `classfy copy.json`.
- **Expected Outcome**: The classification results should now match or closely resemble the Python script's output, as the logic will now correctly handle the combination rules that were previously failing or being deduplicated out.
