I will update the following documentation files to reflect the recent refactoring of the File Management Tools, specifically the Interactive Classification Workflow and the Human Decision Tool enhancements.

**Proposed Updates:**

1.  **`docs/plugins/file_management/FILE_MANAGEMENT_API_REFERENCE.md`**:
    *   **Tool Registration Mapping**: Add new granular tools (`rule-loader`, `rule-preprocessor`, `ac-builder`, `directory-scanner`, `folder-preprocessor`, `parallel-matcher`, `score-calculator`, `ambiguity-detector`, `result-merger`, `experimental-checker`, `report-generator`).
    *   **Human Decision Tool**: Update schema to include `items` (for batch decisions) and `experimental_mode`. Update return schema to include `folder_path` and `selected_category`.
    *   **New Tools API**: Add a new section "Granular Classification Tools API" briefly describing the inputs/outputs of the new tools.

2.  **`docs/plugins/file_management/FILE_MANAGEMENT_HUMAN_DECISION_GUIDE.md`**:
    *   **Batch Decisions**: Explicitly mention the `items` parameter for handling multiple decisions in a single tool call.
    *   **Examples**: Add a YAML example showing how to configure `HumanDecisionTool` with `items`.

3.  **`docs/plugins/file_management/FILE_MANAGEMENT_TOOLS_GUIDE.md`**:
    *   **Tool Reference**: Update the "Folder Classifier" section to explain it is now composed of granular tools, or list the granular tools if they are intended for direct use. I will focus on high-level usage but mention the granular nature.
    *   **Human Decision**: Update to reflect batch capabilities.

4.  **`docs/plugins/file_management/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md`**:
    *   (Already updated steps) Review to ensure parameter descriptions (like `classification_rules` supporting legacy JSON) are clear.

5.  **`docs/plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md`**:
    *   Verify links and consistency.

I will proceed by updating `FILE_MANAGEMENT_API_REFERENCE.md` first, as it defines the technical contract.