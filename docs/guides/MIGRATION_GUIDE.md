# Python to Rust Migration Guide

> **Complete guide for migrating Python scripts to rust-tool-v2**  
> *Last Updated: 2026-01-14*

---

## 🎯 Migration Overview

### Why Migrate?
| Metric | Python | Rust | Improvement |
|--------|--------|------|-------------|
| **Startup Time** | 2.1s | 0.1s | **21x faster** |
| **Classification** | 12.3s | 2.5s | **5x faster** |
| **File Operations** | 8.7s | 2.1s | **4x faster** |
| **Memory Usage** | 180MB | 45MB | **4x less** |
| **Binary Size** | 2.5MB + Python | 8MB single | **Self-contained** |

### Migration Complexity
- **Simple scripts**: 1-2 hours
- **Medium complexity**: 4-8 hours
- **Complex systems**: 1-3 days

---

## 📋 Pre-Migration Checklist

### 1. Analyze Your Python Script
```bash
# Check script size and complexity
wc -l your_script.py
grep -c "def " your_script.py
grep -c "import " your_script.py

# Check dependencies
pip freeze | grep -f <(grep "import " your_script.py | awk '{print $2}' | sed 's/from //;s/;//')

# Profile performance
python -m cProfile -o profile.stats your_script.py
python -m pstats profile.stats
```

### 2. Identify Required Features
```bash
# Check for common patterns
grep -E "(threading|multiprocessing|asyncio)" your_script.py  # Concurrency
grep -E "(json|yaml|toml)" your_script.py                    # Serialization
grep -E "(re|glob|fnmatch)" your_script.py                   # Pattern matching
grep -E "(os\.|shutil|pathlib)" your_script.py               # File operations
grep -E "(socket|http|requests)" your_script.py              # Networking
```

### 3. Verify Rust Equivalents
```bash
# Check available tools
cargo run -- file-classifier --help
cargo run -- file-mover --help
cargo run -- folder-merger --help
cargo run -- batch-processor --help

# Check if custom logic needed
# Review src/tools/ for existing implementations
```

---

## 🔧 Migration Strategies

### Strategy 1: Direct Replacement (Simple)

**Python:**
```python
import os
import shutil
from pathlib import Path

def classify_files(source, dest):
    for file in Path(source).glob("*.txt"):
        dest_path = Path(dest) / file.name
        shutil.move(str(file), str(dest_path))
```

**Rust Equivalent:**
```bash
cargo run -- file-classifier \
  --source ./source \
  --dest ./dest \
  --pattern "*.txt"
```

**Migration Steps:**
1. Identify file patterns
2. Map to `file-classifier` options
3. Create command or workflow file
4. Test with `--dry-run`

---

### Strategy 2: Workflow Composition (Medium)

**Python:**
```python
import os
import shutil
import json

def process_documents(source, dest):
    # Step 1: Classify by type
    for root, dirs, files in os.walk(source):
        for file in files:
            if file.endswith(('.pdf', '.docx')):
                shutil.move(os.path.join(root, file), 
                          os.path.join(dest, 'documents', file))
            elif file.endswith(('.jpg', '.png')):
                shutil.move(os.path.join(root, file), 
                          os.path.join(dest, 'images', file))
    
    # Step 2: Merge duplicates
    # ... merge logic ...
    
    # Step 3: Create index
    index = {}
    for root, dirs, files in os.walk(dest):
        for file in files:
            path = os.path.join(root, file)
            index[file] = {
                'size': os.path.getsize(path),
                'modified': os.path.getmtime(path)
            }
    
    with open(os.path.join(dest, 'index.json'), 'w') as f:
        json.dump(index, f, indent=2)
```

**Rust Workflow:**
```yaml
# process_documents.yaml
name: document-processor
steps:
  - name: classify-docs
    tool: file-classifier
    params:
      source: ./source
      dest: ./dest/documents
      pattern: "*.pdf,*.docx"
      
  - name: classify-images
    tool: file-classifier
    params:
      source: ./source
      dest: ./dest/images
      pattern: "*.jpg,*.png"
      
  - name: merge-duplicates
    tool: folder-merger
    params:
      source: ./dest/documents
      dest: ./dest/documents
      strategy: keep-both
      duplicate-check: true
      
  - name: create-index
    tool: custom-script
    params:
      script: create_index.py
      source: ./dest
      output: ./dest/index.json
```

**Migration Steps:**
1. Break Python script into logical steps
2. Map each step to Rust tool
3. Create workflow YAML
4. Add custom scripts for missing logic
5. Test incrementally

---

### Strategy 3: Custom Tool Development (Complex)

**When to use:**
- Logic not covered by existing tools
- Need custom business rules
- Integration with external systems
- Performance-critical operations

**Python:**
```python
import asyncio
import aiohttp
import json
from pathlib import Path

async def fetch_and_process(url, output_dir):
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            data = await response.json()
            
            # Custom processing
            processed = {
                'id': data['id'],
                'name': data['name'].upper(),
                'timestamp': datetime.now().isoformat()
            }
            
            # Save
            output_path = Path(output_dir) / f"{data['id']}.json"
            with open(output_path, 'w') as f:
                json.dump(processed, f, indent=2)
            
            return output_path

async def main(urls):
    tasks = [fetch_and_process(url, './output') for url in urls]
    return await asyncio.gather(*tasks)
```

**Rust Implementation:**
```rust
// src/tools/custom_fetcher.rs
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

pub struct FetcherTool;

#[async_trait]
impl Tool for FetcherTool {
    async fn execute(&self, params: Value, _context: Context) -> Result<Value> {
        let url = params["url"]
            .as_str()
            .ok_or(WorkflowError::tool("Missing URL"))?;
        
        let output_dir = params["output_dir"]
            .as_str()
            .unwrap_or("./output");
        
        // Fetch
        let response = reqwest::get(url).await?;
        let data: Value = response.json().await?;
        
        // Process
        let processed = serde_json::json!({
            "id": data["id"],
            "name": data["name"].as_str().unwrap().to_uppercase(),
            "timestamp": chrono::Local::now().to_rfc3339()
        });
        
        // Save
        let output_path = PathBuf::from(output_dir)
            .join(format!("{}.json", data["id"]));
        
        fs::write(
            output_path,
            serde_json::to_string_pretty(&processed)?
        ).await?;
        
        Ok(serde_json::json!({"status": "success"}))
    }
}
```

**Workflow Usage:**
```yaml
name: custom-fetcher-workflow
steps:
  - name: fetch-data
    tool: custom-fetcher
    params:
      url: "https://api.example.com/data"
      output_dir: "./output"
```

---

## 📊 Feature Mapping Guide

### File Operations

| Python | Rust Tool | Command/Config |
|--------|-----------|----------------|
| `os.listdir()` | `file-classifier` | `--source <dir>` |
| `shutil.move()` | `file-mover` | `--source <file> --dest <dir>` |
| `shutil.copy()` | `file-mover` | `--source <file> --dest <dir> --preserve` |
| `os.walk()` | `file-classifier` | `--recursive` |
| `glob.glob()` | `file-classifier` | `--pattern "*.ext"` |
| `os.remove()` | `file-mover` | Move to trash or use `--delete` |
| `shutil.rmtree()` | `folder-merger` | Merge with empty or use system commands |

### Pattern Matching

| Python | Rust Tool | Command/Config |
|--------|-----------|----------------|
| `fnmatch()` | `file-classifier` | `--pattern "*.txt"` |
| `re.match()` | `file-classifier` | Use regex patterns |
| `pathlib.Path.glob()` | `file-classifier` | `--pattern "*.ext"` |

### Data Processing

| Python | Rust Tool | Command/Config |
|--------|-----------|----------------|
| `json.load()` | `batch-processor` | Use custom script |
| `yaml.safe_load()` | `batch-processor` | Use custom script |
| `csv.reader()` | `batch-processor` | Use custom script |
| `pandas` | Custom tool | Develop custom tool |

### Concurrency

| Python | Rust Tool | Command/Config |
|--------|-----------|----------------|
| `threading` | `batch-processor` | `--parallel <N>` |
| `multiprocessing` | `batch-processor` | `--parallel <N>` |
| `asyncio` | Custom tool | Async Rust implementation |

### Networking

| Python | Rust Tool | Command/Config |
|--------|-----------|----------------|
| `requests.get()` | Custom tool | `reqwest` crate |
| `aiohttp` | Custom tool | `tokio` + `reqwest` |
| `socket` | Custom tool | `tokio::net` |

---

## 🔨 Step-by-Step Migration Process

### Step 1: Analyze Current Script
```bash
# Create analysis report
python analyze_script.py your_script.py > analysis.txt

# Check dependencies
pip show $(pip freeze | cut -d'=' -f1) > dependencies.txt

# Profile performance
python -m cProfile -o profile.stats your_script.py
```

### Step 2: Map to Rust Tools
```bash
# List available tools
cargo run -- --help

# Check tool capabilities
cargo run -- file-classifier --help
cargo run -- file-mover --help
cargo run -- folder-merger --help
cargo run -- batch-processor --help

# Identify gaps
# If tools don't cover your needs → Plan custom tool
```

### Step 3: Create Workflow or Command
```bash
# For simple cases: Create command
cat > migrate_command.sh << 'EOF'
#!/bin/bash
cargo run -- file-classifier \
  --source ./input \
  --dest ./output \
  --pattern "*.pdf,*.docx" \
  --recursive \
  --threads 4
EOF

# For complex cases: Create workflow
cat > workflow.yaml << 'EOF'
name: migration-workflow
steps:
  - name: step1
    tool: file-classifier
    params:
      source: ./input
      dest: ./temp
      pattern: "*.pdf,*.docx"
  - name: step2
    tool: folder-merger
    params:
      source: ./temp
      dest: ./output
      strategy: keep-both
EOF
```

### Step 4: Test Incrementally
```bash
# Test with dry-run
cargo run -- file-classifier --source ./test --dest ./output --dry-run

# Test small batch
cargo run -- batch-processor --source ./small --dest ./output --batch-size 10

# Verify results
diff -r expected/ output/
```

### Step 5: Handle Custom Logic
```bash
# If Python logic is complex:
# 1. Check if Rust tool can be extended
# 2. Create custom tool in src/tools/
# 3. Register in tool registry
# 4. Use in workflow

# Example: Create custom tool
cat > src/tools/custom_processor.rs << 'EOF'
// Your custom implementation
EOF
```

### Step 6: Performance Testing
```bash
# Compare performance
time python your_script.py
time cargo run --release -- workflow execute workflow.yaml

# Profile if needed
cargo build --release
/usr/bin/time -v ./target/release/rust-tool-v2 workflow execute workflow.yaml
```

### Step 7: Deployment
```bash
# Build release
cargo build --release

# Create deployment package
tar -czf rust-tool-v2.tar.gz target/release/rust-tool-v2 examples/ docs/

# Deploy
# Copy binary to target system
# Test on target system
```

---

## 📝 Common Python Patterns → Rust Solutions

### Pattern 1: Simple File Copy
**Python:**
```python
import shutil
shutil.copytree("source", "dest")
```

**Rust:**
```bash
# Option 1: Use file-mover with preserve
cargo run -- file-mover --source source --dest dest --preserve

# Option 2: Use folder-merger
cargo run -- folder-merger --source source --dest dest --strategy replace
```

### Pattern 2: Conditional Processing
**Python:**
```python
for file in os.listdir("."):
    if file.endswith(".txt") and os.path.getsize(file) > 1024:
        process(file)
```

**Rust:**
```bash
# Use file-classifier with pattern
cargo run -- file-classifier --source . --dest ./processed --pattern "*.txt"

# For size filtering: Use custom tool or script
# Create workflow with custom step
```

### Pattern 3: Batch Processing with Progress
**Python:**
```python
from tqdm import tqdm
for item in tqdm(items):
    process(item)
```

**Rust:**
```bash
cargo run -- batch-processor --source ./data --dest ./output --progress
```

### Pattern 4: Error Handling and Logging
**Python:**
```python
import logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

try:
    process()
except Exception as e:
    logger.error(f"Error: {e}")
    raise
```

**Rust:**
```bash
# Enable logging
RUST_LOG=info cargo run -- workflow execute workflow.yaml

# Or in code:
# Use tracing macros: info!(), error!(), debug!()
```

### Pattern 5: Configuration Files
**Python:**
```python
import json
with open("config.json") as f:
    config = json.load(f)
```

**Rust:**
```bash
# Use config file
cargo run -- config show

# Or environment variable
export RUST_TOOL_V2_CONFIG=/path/to/config.toml
cargo run -- workflow execute workflow.yaml
```

---

## 🎯 Migration Examples

### Example 1: Folder Classifier (65KB Python → Rust)

**Original Python (folder_classifier_v5_improved2.py):**
```python
# Features: AC Automaton, Chinese pinyin, multi-threading
# ~65KB, 2.1s startup, 12.3s classification
```

**Rust Equivalent:**
```bash
# Single command
cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt,*.jpg,*.png" \
  --recursive \
  --threads 8 \
  --verbose

# Or workflow
cat > classify.yaml << 'EOF'
name: classify-files
steps:
  - name: classify
    tool: file-classifier
    params:
      source: ./downloads
      dest: ./organized
      pattern: "*.pdf,*.docx,*.txt,*.jpg,*.png"
      recursive: true
      threads: 8
EOF

cargo run -- workflow execute classify.yaml
```

**Performance Gain:**
- Startup: 21x faster
- Classification: 5x faster
- Memory: 4x less

### Example 2: Folder Merger (18KB Python → Rust)

**Original Python (mergeClassifierSimple.py):**
```python
# Features: Merging, duplicate handling, conflict resolution
# ~18KB, 8.7s file operations
```

**Rust Equivalent:**
```bash
# Interactive merge
cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --interactive

# Or batch mode
cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --backup
```

**Performance Gain:**
- File operations: 4x faster
- Memory: 4x less
- Binary: Self-contained

---

## 🛠️ Custom Tool Development

### When to Create Custom Tool
- Logic not covered by existing tools
- Need specific business rules
- Integration with external APIs
- Performance-critical custom algorithms

### Development Steps

#### 1. Create Tool File
```bash
# In src/tools/
cat > src/tools/custom_processor.rs << 'EOF'
use async_trait::async_trait;
use serde_json::Value;
use crate::error::{Result, WorkflowError};
use crate::tools::Tool;

pub struct CustomProcessor;

#[async_trait]
impl Tool for CustomProcessor {
    async fn execute(&self, params: Value, context: Context) -> Result<Value> {
        // Your logic here
        let input = params["input"]
            .as_str()
            .ok_or(WorkflowError::tool("Missing input"))?;
        
        // Process
        let result = process_input(input);
        
        Ok(serde_json::json!({
            "status": "success",
            "result": result
        }))
    }
}

fn process_input(input: &str) -> String {
    // Custom processing logic
    input.to_uppercase()
}
EOF
```

#### 2. Register Tool
```bash
# In src/tools/registry.rs
pub fn register_custom_tools(registry: &mut ToolRegistry) -> Result<()> {
    registry.register(
        "custom-processor",
        Box::new(CustomProcessor),
        "Custom processing tool"
    )?;
    Ok(())
}
```

#### 3. Use in Workflow
```yaml
name: custom-workflow
steps:
  - name: process
    tool: custom-processor
    params:
      input: "hello world"
```

---

## 📊 Performance Comparison Tool

### Benchmark Script
```bash
cat > benchmark.sh << 'EOF'
#!/bin/bash
echo "Performance Comparison"
echo "======================"

echo -e "\n1. Python Script:"
time python your_script.py

echo -e "\n2. Rust Equivalent:"
time cargo run --release -- workflow execute workflow.yaml

echo -e "\n3. Memory Usage (Linux/macOS):"
/usr/bin/time -v python your_script.py 2>&1 | grep "Maximum resident"
/usr/bin/time -v cargo run --release -- workflow execute workflow.yaml 2>&1 | grep "Maximum resident"
EOF

chmod +x benchmark.sh
./benchmark.sh
```

---

## ✅ Migration Verification

### Checklist
- [ ] All Python features mapped to Rust tools
- [ ] Workflow or commands created
- [ ] Tested with `--dry-run`
- [ ] Tested with small dataset
- [ ] Performance benchmarked
- [ ] Error handling verified
- [ ] Logging configured
- [ ] Documentation updated
- [ ] Deployment tested

### Verification Commands
```bash
# 1. Compilation
cargo check
cargo build --release

# 2. Basic functionality
cargo run -- workflow execute test.yaml

# 3. Performance test
time cargo run --release -- workflow execute large.yaml

# 4. Error handling
# Test with invalid inputs
cargo run -- workflow execute invalid.yaml  # Should fail gracefully

# 5. Memory usage
/usr/bin/time -v cargo run --release -- workflow execute test.yaml
```

---

## 🚀 Quick Start Templates

### Template 1: File Organization
```bash
# Python equivalent: ~20 lines
# Rust: 1 command

cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt,*.jpg,*.png" \
  --recursive \
  --threads 8
```

### Template 2: Folder Sync
```bash
# Python equivalent: ~30 lines
# Rust: 1 command

cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --backup
```

### Template 3: Batch Processing
```bash
# Python equivalent: ~50 lines with threading
# Rust: 1 command

cargo run -- batch-processor \
  --source ./data \
  --dest ./processed \
  --batch-size 100 \
  --parallel 8 \
  --progress
```

### Template 4: Complex Workflow
```yaml
# Python equivalent: ~100+ lines
# Rust: Workflow YAML

name: complex-workflow
steps:
  - name: classify
    tool: file-classifier
    params:
      source: ./input
      dest: ./temp
      pattern: "*.pdf,*.docx"
      
  - name: merge
    tool: folder-merger
    params:
      source: ./temp
      dest: ./output
      strategy: keep-both
      
  - name: index
    tool: custom-indexer
    params:
      source: ./output
      output: ./index.json
```

---

## 📚 Migration Resources

### Documentation
- **[USER_GUIDE.md](USER_GUIDE.md)** - Usage examples
- **[API_INDEX.md](API_INDEX.md)** - Command reference
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - Custom tools
- **[FILE_MANAGEMENT_TOOLS_GUIDE.md](FILE_MANAGEMENT_TOOLS_GUIDE.md)** - File operations

### Tools
- **[INDEX.md](INDEX.md)** - Complete navigation
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues

### External
- Rust Book: https://doc.rust-lang.org/book/
- Tokio: https://tokio.rs/
- Serde: https://serde.rs/

---

## 🎯 Migration Success Criteria

### Performance
- [ ] 5x+ faster execution
- [ ] 4x+ less memory
- [ ] <0.5s startup time

### Functionality
- [ ] All Python features supported
- [ ] Error handling equivalent
- [ ] Logging equivalent
- [ ] Configuration equivalent

### Quality
- [ ] No compilation warnings
- [ ] All tests pass
- [ ] Code formatted
- [ ] Documentation complete

### Deployment
- [ ] Single binary deployment
- [ ] No external dependencies
- [ ] Cross-platform compatible
- [ ] Self-contained

---

## 📞 Getting Help

### Migration Issues
1. **Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)** for common issues
2. **Review [API_INDEX.md](API_INDEX.md)** for command options
3. **Consult [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** for custom tools
4. **Use [INDEX.md](INDEX.md)** to find related docs

### Custom Development
1. **Read [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** for code style
2. **Check [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md)** for extensions
3. **Review existing tools** in `src/tools/`

---

**← Back to [INDEX.md](INDEX.md)** | **Top** ↑