I will update the documentation to reflect the recent refactoring of the Interactive Classification Workflow into 13 granular steps.

**Proposed Updates:**

1.  **File**: `docs/plugins/file_management/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md`
    *   **Section**: "Interactive Classification Workflow" -> "Workflow Steps"
    *   **Action**: Replace the outdated 9-step list with the new 13-step process description (Load Rules, Preprocess Rules, Build Automaton, Scan, Match, Score, etc.).
    *   **Reason**: The workflow structure has been fundamentally changed to support granular execution and legacy rule compatibility, and the documentation needs to match the actual implementation.

2.  **File**: `docs/plugins/file_management/FILE_MANAGEMENT_TOOLS_GUIDE.md` (Optional Review)
    *   **Action**: Review for any direct contradictions, though the primary detailed steps are in the templates guide.

I will primarily focus on `FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md` as it is the source of truth for the workflow structure.