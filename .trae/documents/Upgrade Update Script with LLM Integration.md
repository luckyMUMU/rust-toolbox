I will upgrade the `scripts/update_agents_md.py` script to integrate LLM capabilities for generating intelligent summaries of directories and modules.

## Plan: Upgrade Agent Map Script with LLM Integration

### 1. Script Modification (`scripts/update_agents_md.py`)
I will refactor the existing script to include an **LLM Client** that communicates with OpenAI-compatible APIs (supporting OpenAI, DeepSeek, local LLMs via Ollama/LM Studio, etc.).

#### Key Features:
- **Environment Configuration**: Read `OPENAI_API_KEY`, `OPENAI_BASE_URL` (optional), and `LLM_MODEL` (optional) from environment variables.
- **Intelligent Summarization**:
  - **For directories WITH `AGENT.md`**: Read the `AGENT.md` content and use LLM to generate a high-quality one-sentence summary for the parent map.
  - **For directories WITHOUT `AGENT.md`**: Read the file list and the first 50 lines of key source files (`.rs`, `.py`, `.md`). Send this context to the LLM to generate a descriptive summary of the directory's functionality.
- **Fallback Mechanism**: If no API key is detected or the API call fails, fallback to the existing rule-based summary (extracting OVERVIEW or counting files) to ensure the script always works.
- **Dependency-Free**: Use Python's standard `urllib` library to avoid needing `pip install requests`.

### 2. Execution Flow
1.  **Scan**: Recursively walk the directory tree.
2.  **Context Gathering**:
    -   If `AGENT.md` exists -> Read content.
    -   If not -> Read filenames + snippets of source code.
3.  **LLM Call**:
    -   Construct a prompt: "Analyze this context and provide a 1-sentence technical summary of this module/directory."
    -   Send to API.
4.  **Update**: Inject the generated summaries and the current timestamp into the parent `AGENT.md`.

### 3. Usage
You will be able to run the script like this:
```powershell
$env:OPENAI_API_KEY="sk-..."
python scripts/update_agents_md.py
```

### Verification
- I will verify the script runs without errors in the current environment (triggering the fallback since no key is set).
- I will review the code to ensure it correctly handles API errors and timeouts.
