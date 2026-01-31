# Python到Rust迁移指南

> **将Python脚本迁移到rust-tool-v2的完整指南**  
> *最后更新：2026-01-14*

---

## 🎯 迁移概述

### 为什么要迁移？
| 指标 | Python | Rust | 提升 |
|--------|--------|------|-------------|
| **启动时间** | 2.1秒 | 0.1秒 | **21倍更快** |
| **分类速度** | 12.3秒 | 2.5秒 | **5倍更快** |
| **文件操作** | 8.7秒 | 2.1秒 | **4倍更快** |
| **内存使用** | 180MB | 45MB | **减少4倍** |
| **二进制大小** | 2.5MB + Python | 8MB单一文件 | **自包含** |

### 迁移复杂度
- **简单脚本**：1-2小时
- **中等复杂度**：4-8小时
- **复杂系统**：1-3天

---

## 📋 迁移前检查清单

### 1. 分析你的Python脚本
```bash
# 检查脚本大小和复杂度
wc -l your_script.py
grep -c "def " your_script.py
grep -c "import " your_script.py

# 检查依赖
pip freeze | grep -f <(grep "import " your_script.py | awk '{print $2}' | sed 's/from //;s/;//')

# 性能分析
python -m cProfile -o profile.stats your_script.py
python -m pstats profile.stats
```

### 2. 识别所需功能
```bash
# 检查常见模式
grep -E "(threading|multiprocessing|asyncio)" your_script.py  # 并发
grep -E "(json|yaml|toml)" your_script.py                    # 序列化
grep -E "(re|glob|fnmatch)" your_script.py                   # 模式匹配
grep -E "(os\.|shutil|pathlib)" your_script.py               # 文件操作
grep -E "(socket|http|requests)" your_script.py              # 网络
```

### 3. 验证Rust等效工具
```bash
# 检查可用工具
cargo run -- file-classifier --help
cargo run -- file-mover --help
cargo run -- folder-merger --help
cargo run -- batch-processor --help

# 检查是否需要自定义逻辑
# 查看 src/tools/ 了解现有实现
```

---

## 🔧 迁移策略

### 策略1：直接替换（简单）

**Python：**
```python
import os
import shutil
from pathlib import Path

def classify_files(source, dest):
    for file in Path(source).glob("*.txt"):
        dest_path = Path(dest) / file.name
        shutil.move(str(file), str(dest_path))
```

**Rust等效：**
```bash
cargo run -- file-classifier \
  --source ./source \
  --dest ./dest \
  --pattern "*.txt"
```

**迁移步骤：**
1. 识别文件模式
2. 映射到 `file-classifier` 选项
3. 创建命令或工作流文件
4. 使用 `--dry-run` 测试

---

### 策略2：工作流组合（中等）

**Python：**
```python
import os
import shutil
import json

def process_documents(source, dest):
    # 步骤1：按类型分类
    for root, dirs, files in os.walk(source):
        for file in files:
            if file.endswith(('.pdf', '.docx')):
                shutil.move(os.path.join(root, file), 
                          os.path.join(dest, 'documents', file))
            elif file.endswith(('.jpg', '.png')):
                shutil.move(os.path.join(root, file), 
                          os.path.join(dest, 'images', file))
    
    # 步骤2：合并重复项
    # ... 合并逻辑 ...
    
    # 步骤3：创建索引
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

**Rust工作流：**
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

**迁移步骤：**
1. 将Python脚本分解为逻辑步骤
2. 将每个步骤映射到Rust工具
3. 创建工作流YAML
4. 为缺失的逻辑添加自定义脚本
5. 增量测试

---

### 策略3：自定义工具开发（复杂）

**何时使用：**
- 现有工具未涵盖的逻辑
- 需要自定义业务规则
- 与外部系统集成
- 性能关键的操作

**Python：**
```python
import asyncio
import aiohttp
import json
from pathlib import Path

async def fetch_and_process(url, output_dir):
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            data = await response.json()
            
            # 自定义处理
            processed = {
                'id': data['id'],
                'name': data['name'].upper(),
                'timestamp': datetime.now().isoformat()
            }
            
            # 保存
            output_path = Path(output_dir) / f"{data['id']}.json"
            with open(output_path, 'w') as f:
                json.dump(processed, f, indent=2)
            
            return output_path

async def main(urls):
    tasks = [fetch_and_process(url, './output') for url in urls]
    return await asyncio.gather(*tasks)
```

**Rust实现：**
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
        
        // 获取
        let response = reqwest::get(url).await?;
        let data: Value = response.json().await?;
        
        // 处理
        let processed = serde_json::json!({
            "id": data["id"],
            "name": data["name"].as_str().unwrap().to_uppercase(),
            "timestamp": chrono::Local::now().to_rfc3339()
        });
        
        // 保存
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

**工作流使用：**
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

## 📊 功能映射指南

### 文件操作

| Python | Rust工具 | 命令/配置 |
|--------|-----------|----------------|
| `os.listdir()` | `file-classifier` | `--source <dir>` |
| `shutil.move()` | `file-mover` | `--source <file> --dest <dir>` |
| `shutil.copy()` | `file-mover` | `--source <file> --dest <dir> --preserve` |
| `os.walk()` | `file-classifier` | `--recursive` |
| `glob.glob()` | `file-classifier` | `--pattern "*.ext"` |
| `os.remove()` | `file-mover` | 移动到回收站或使用 `--delete` |
| `shutil.rmtree()` | `folder-merger` | 与空目录合并或使用系统命令 |

### 模式匹配

| Python | Rust工具 | 命令/配置 |
|--------|-----------|----------------|
| `fnmatch()` | `file-classifier` | `--pattern "*.txt"` |
| `re.match()` | `file-classifier` | 使用正则模式 |
| `pathlib.Path.glob()` | `file-classifier` | `--pattern "*.ext"` |

### 数据处理

| Python | Rust工具 | 命令/配置 |
|--------|-----------|----------------|
| `json.load()` | `batch-processor` | 使用自定义脚本 |
| `yaml.safe_load()` | `batch-processor` | 使用自定义脚本 |
| `csv.reader()` | `batch-processor` | 使用自定义脚本 |
| `pandas` | 自定义工具 | 开发自定义工具 |

### 并发

| Python | Rust工具 | 命令/配置 |
|--------|-----------|----------------|
| `threading` | `batch-processor` | `--parallel <N>` |
| `multiprocessing` | `batch-processor` | `--parallel <N>` |
| `asyncio` | 自定义工具 | 异步Rust实现 |

### 网络

| Python | Rust工具 | 命令/配置 |
|--------|-----------|----------------|
| `requests.get()` | 自定义工具 | `reqwest` crate |
| `aiohttp` | 自定义工具 | `tokio` + `reqwest` |
| `socket` | 自定义工具 | `tokio::net` |

---

## 🔨 分步迁移流程

### 步骤1：分析当前脚本
```bash
# 创建分析报告
python analyze_script.py your_script.py > analysis.txt

# 检查依赖
pip show $(pip freeze | cut -d'=' -f1) > dependencies.txt

# 性能分析
python -m cProfile -o profile.stats your_script.py
```

### 步骤2：映射到Rust工具
```bash
# 列出可用工具
cargo run -- --help

# 检查工具能力
cargo run -- file-classifier --help
cargo run -- file-mover --help
cargo run -- folder-merger --help
cargo run -- batch-processor --help

# 识别差距
# 如果工具不能满足需求 → 计划自定义工具
```

### 步骤3：创建工作流或命令
```bash
# 简单情况：创建命令
cat > migrate_command.sh << 'EOF'
#!/bin/bash
cargo run -- file-classifier \
  --source ./input \
  --dest ./output \
  --pattern "*.pdf,*.docx" \
  --recursive \
  --threads 4
EOF

# 复杂情况：创建工作流
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

### 步骤4：增量测试
```bash
# 使用试运行测试
cargo run -- file-classifier --source ./test --dest ./output --dry-run

# 测试小批量
cargo run -- batch-processor --source ./small --dest ./output --batch-size 10

# 验证结果
diff -r expected/ output/
```

### 步骤5：处理自定义逻辑
```bash
# 如果Python逻辑复杂：
# 1. 检查Rust工具是否可以扩展
# 2. 在 src/tools/ 中创建自定义工具
# 3. 注册到工具注册表
# 4. 在工作流中使用

# 示例：创建自定义工具
cat > src/tools/custom_processor.rs << 'EOF'
// 你的自定义实现
EOF
```

### 步骤6：性能测试
```bash
# 比较性能
time python your_script.py
time cargo run --release -- workflow execute workflow.yaml

# 如需分析
cargo build --release
/usr/bin/time -v ./target/release/rust-tool-v2 workflow execute workflow.yaml
```

### 步骤7：部署
```bash
# 构建release
cargo build --release

# 创建部署包
tar -czf rust-tool-v2.tar.gz target/release/rust-tool-v2 examples/ docs/

# 部署
# 复制二进制文件到目标系统
# 在目标系统上测试
```

---

## 📝 常见Python模式 → Rust解决方案

### 模式1：简单文件复制
**Python：**
```python
import shutil
shutil.copytree("source", "dest")
```

**Rust：**
```bash
# 选项1：使用file-mover并保留
 cargo run -- file-mover --source source --dest dest --preserve

# 选项2：使用folder-merger
cargo run -- folder-merger --source source --dest dest --strategy replace
```

### 模式2：条件处理
**Python：**
```python
for file in os.listdir("."):
    if file.endswith(".txt") and os.path.getsize(file) > 1024:
        process(file)
```

**Rust：**
```bash
# 使用file-classifier并指定模式
cargo run -- file-classifier --source . --dest ./processed --pattern "*.txt"

# 对于大小过滤：使用自定义工具或脚本
# 创建带自定义步骤的工作流
```

### 模式3：带进度的批处理
**Python：**
```python
from tqdm import tqdm
for item in tqdm(items):
    process(item)
```

**Rust：**
```bash
cargo run -- batch-processor --source ./data --dest ./output --progress
```

### 模式4：错误处理和日志
**Python：**
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

**Rust：**
```bash
# 启用日志
RUST_LOG=info cargo run -- workflow execute workflow.yaml

# 或在代码中：
# 使用tracing宏：info!(), error!(), debug!()
```

### 模式5：配置文件
**Python：**
```python
import json
with open("config.json") as f:
    config = json.load(f)
```

**Rust：**
```bash
# 使用配置文件
cargo run -- config show

# 或环境变量
export RUST_TOOL_V2_CONFIG=/path/to/config.toml
cargo run -- workflow execute workflow.yaml
```

---

## 🎯 迁移示例

### 示例1：文件夹分类器（65KB Python → Rust）

**原始Python（folder_classifier_v5_improved2.py）：**
```python
# 功能：AC自动机、中文拼音、多线程
# ~65KB，2.1秒启动，12.3秒分类
```

**Rust等效：**
```bash
# 单条命令
cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt,*.jpg,*.png" \
  --recursive \
  --threads 8 \
  --verbose

# 或工作流
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

**性能提升：**
- 启动：21倍更快
- 分类：5倍更快
- 内存：减少4倍

### 示例2：文件夹合并器（18KB Python → Rust）

**原始Python（mergeClassifierSimple.py）：**
```python
# 功能：合并、重复处理、冲突解决
# ~18KB，8.7秒文件操作
```

**Rust等效：**
```bash
# 交互式合并
cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --interactive

# 或批处理模式
cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --backup
```

**性能提升：**
- 文件操作：4倍更快
- 内存：减少4倍
- 二进制：自包含

---

## 🛠️ 自定义工具开发

### 何时创建自定义工具
- 现有工具未涵盖的逻辑
- 需要特定业务规则
- 与外部API集成
- 性能关键的自定义算法

### 开发步骤

#### 1. 创建工具文件
```bash
# 在 src/tools/ 中
cat > src/tools/custom_processor.rs << 'EOF'
use async_trait::async_trait;
use serde_json::Value;
use crate::error::{Result, WorkflowError};
use crate::tools::Tool;

pub struct CustomProcessor;

#[async_trait]
impl Tool for CustomProcessor {
    async fn execute(&self, params: Value, context: Context) -> Result<Value> {
        // 你的逻辑在这里
        let input = params["input"]
            .as_str()
            .ok_or(WorkflowError::tool("Missing input"))?;
        
        // 处理
        let result = process_input(input);
        
        Ok(serde_json::json!({
            "status": "success",
            "result": result
        }))
    }
}

fn process_input(input: &str) -> String {
    // 自定义处理逻辑
    input.to_uppercase()
}
EOF
```

#### 2. 注册工具
```bash
# 在 src/tools/registry.rs 中
pub fn register_custom_tools(registry: &mut ToolRegistry) -> Result<()> {
    registry.register(
        "custom-processor",
        Box::new(CustomProcessor),
        "自定义处理工具"
    )?;
    Ok(())
}
```

#### 3. 在工作流中使用
```yaml
name: custom-workflow
steps:
  - name: process
    tool: custom-processor
    params:
      input: "hello world"
```

---

## 📊 性能比较工具

### 基准测试脚本
```bash
cat > benchmark.sh << 'EOF'
#!/bin/bash
echo "性能比较"
echo "======================"

echo -e "\n1. Python脚本："
time python your_script.py

echo -e "\n2. Rust等效："
time cargo run --release -- workflow execute workflow.yaml

echo -e "\n3. 内存使用（Linux/macOS）："
/usr/bin/time -v python your_script.py 2>&1 | grep "Maximum resident"
/usr/bin/time -v cargo run --release -- workflow execute workflow.yaml 2>&1 | grep "Maximum resident"
EOF

chmod +x benchmark.sh
./benchmark.sh
```

---

## ✅ 迁移验证

### 检查清单
- [ ] 所有Python功能映射到Rust工具
- [ ] 工作流或命令已创建
- [ ] 使用 `--dry-run` 测试
- [ ] 使用小数据集测试
- [ ] 性能已基准测试
- [ ] 错误处理已验证
- [ ] 日志已配置
- [ ] 文档已更新
- [ ] 部署已测试

### 验证命令
```bash
# 1. 编译
cargo check
cargo build --release

# 2. 基本功能
cargo run -- workflow execute test.yaml

# 3. 性能测试
time cargo run --release -- workflow execute large.yaml

# 4. 错误处理
# 使用无效输入测试
cargo run -- workflow execute invalid.yaml  # 应该优雅地失败

# 5. 内存使用
/usr/bin/time -v cargo run --release -- workflow execute test.yaml
```

---

## 🚀 快速启动模板

### 模板1：文件组织
```bash
# Python等效：~20行
# Rust：1条命令

cargo run -- file-classifier \
  --source ./downloads \
  --dest ./organized \
  --pattern "*.pdf,*.docx,*.txt,*.jpg,*.png" \
  --recursive \
  --threads 8
```

### 模板2：文件夹同步
```bash
# Python等效：~30行
# Rust：1条命令

cargo run -- folder-merger \
  --source ./folder1 \
  --dest ./folder2 \
  --strategy keep-both \
  --duplicate-check \
  --backup
```

### 模板3：批处理
```bash
# Python等效：~50行带线程
# Rust：1条命令

cargo run -- batch-processor \
  --source ./data \
  --dest ./processed \
  --batch-size 100 \
  --parallel 8 \
  --progress
```

### 模板4：复杂工作流
```yaml
# Python等效：~100+行
# Rust：工作流YAML

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

## 📚 迁移资源

### 文档
- **[USER_GUIDE.md](USER_GUIDE.md)** - 使用示例
- **[API_INDEX.md](API_INDEX.md)** - 命令参考
- **[DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** - 自定义工具
- **[FILE_MANAGEMENT_TOOLS_GUIDE.md](FILE_MANAGEMENT_TOOLS_GUIDE.md)** - 文件操作

### 工具
- **[INDEX.md](INDEX.md)** - 完整导航
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - 常见问题

### 外部资源
- Rust书籍：https://doc.rust-lang.org/book/
- Tokio：https://tokio.rs/
- Serde：https://serde.rs/

---

## 🎯 迁移成功标准

### 性能
- [ ] 5倍以上执行速度
- [ ] 4倍以下内存使用
- [ ] <0.5秒启动时间

### 功能
- [ ] 所有Python功能都支持
- [ ] 等效的错误处理
- [ ] 等效的日志
- [ ] 等效的配置

### 质量
- [ ] 无编译警告
- [ ] 所有测试通过
- [ ] 代码已格式化
- [ ] 文档完整

### 部署
- [ ] 单一二进制部署
- [ ] 无外部依赖
- [ ] 跨平台兼容
- [ ] 自包含

---

## 📞 获取帮助

### 迁移问题
1. **查看 [TROUBLESHOOTING.md](TROUBLESHOOTING.md)** 了解常见问题
2. **查看 [API_INDEX.md](API_INDEX.md)** 了解命令选项
3. **查看 [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** 了解自定义工具
4. **使用 [INDEX.md](INDEX.md)** 查找相关文档

### 自定义开发
1. **阅读 [DEVELOPMENT_GUIDE.md](DEVELOPMENT_GUIDE.md)** 了解代码风格
2. **查看 [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md)** 了解扩展
3. **查看现有工具** 在 `src/tools/`

---
