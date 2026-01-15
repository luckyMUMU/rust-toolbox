# Design: Two-Phase Classification with AC Automaton and Combinations

## Context
The current classification system uses an Aho-Corasick (AC) automaton where each keyword is directly tied to a specific rule/category. This causes issues when the same keyword appears in multiple rules (conflict/deduplication required) and limits the ability to support complex logic like "Match A AND B".

## Goals
- Support **Keyword Combinations**: Rules can define `["A", "B"]` meaning both A and B must be present to score.
- Support **Global Matching**: The AC automaton should find all occurrences of keywords efficiently, regardless of which rule they belong to.
- **Decoupling**: Separate the "Finding" phase (AC Automaton) from the "Scoring" phase (Rule Logic).

## Architecture

### 1. Data Structures

#### `ClassificationRule`
Updated to support combinations.
```rust
pub struct ClassificationRule {
    pub category: String,
    pub keywords: Vec<String>, // Simple: Match any -> Score
    pub combinations: Option<Vec<Vec<String>>>, // Complex: Match ALL in sub-list -> Score
    pub score_weight: f64,
    // ... other fields
}
```

### 2. Process Flow

#### Phase 1: Global Keyword Extraction (AC Automaton)
1.  **Preparation**:
    - Iterate all `ClassificationRules`.
    - Collect all strings from `keywords` and `combinations`.
    - Deduplicate strings to create a set of `UniqueKeywords`.
2.  **Build Automaton**:
    - Insert each `UniqueKeyword` into the AC Automaton.
    - *Note*: The AC Automaton acts purely as a fast multi-pattern searcher. It does not know about categories or scores at this stage.
3.  **Execution**:
    - Run the AC Automaton on the target text (Folder Name).
    - Result: A set of `MatchedKeywords` (e.g., `HashSet<String>`).

#### Phase 2: Rule Scoring
1.  **Evaluation**:
    - Iterate through each `ClassificationRule`.
    - **Simple Score**: Count how many of `rule.keywords` are in `MatchedKeywords`.
        - `score += match_count * rule.score_weight`
    - **Combination Score**: Check each combination group (e.g., `["A", "B"]`).
        - If `MatchedKeywords` contains "A" **AND** "B":
            - `score += rule.score_weight` (or a specific multiplier)
2.  **Ranking**:
    - Sort rules by final score.
    - Apply `ambiguity_threshold` to determine if the result is confident or ambiguous.

## Advantages
- **No Duplication Errors**: Same keyword can exist in 100 rules; AC only sees it once.
- **Complex Logic**: Easy to implement AND/OR/NOT logic in Phase 2 once we have the set of matched tokens.
- **Performance**: AC Automaton is still used for the expensive part (string matching), while Phase 2 is fast (hash lookups).

## Migration
- Existing JSON rules with simple `keywords` continue to work (treated as Phase 2 Simple Score).
- New JSON structure can use `combinations` for stricter logic.
