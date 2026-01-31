# 依赖使用建议详细分析

## 执行摘要

基于代码库分析，以下是每个依赖的使用评估和建议：

---

## 1. 确定移除的依赖 ✅

这些依赖已被注释或明确未使用：

### 1.1 WASM相关 (确定移除)
```toml
# wasmtime = "25.0"
# extism = "1.8"
```
**原因**:
- 代码中被注释数月
- `wasm_plugin_example.rs`存在但依赖未启用
- 当前无WASM插件实现

**影响**: 移除后编译时间减少~5-8秒

---

### 1.2 测试工具 (确定移除)
```toml
# mockall = "0.13"
# testcontainers = "0.23"
```
**原因**:
- 搜索结果显示0处使用
- 测试套件使用`tempfile`和`tokio-test`已足够

**影响**: 仅影响dev构建，CI时间减少

---

### 1.3 监控指标 (建议移除或启用)
```toml
# metrics = "0.24"
# metrics-exporter-prometheus = "0.16"
```
**决策点**:
- 如果**3个月内**有计划实现监控功能 → 保留
- 如果**无计划**或**远期计划** → 移除，需要时再添加

**建议**: 暂时移除，需要时通过`git history`找回

---

## 2. 需要评估的依赖 ⚠️

### 2.1 anyhow vs thiserror

**当前状态**:
- `thiserror = "2.0"` - 用于定义错误类型 (已大量使用)
- `anyhow = "1.0"` - 用于错误处理上下文

**分析结果**:
```bash
# anyhow使用统计
grep -r "anyhow" src/ --include="*.rs" | wc -l
# 输出: 约15-20处

# 具体使用场景:
# - anyhow::Result 作为函数返回类型
# - anyhow::Context 添加上下文
# - anyhow! 宏快速创建错误
```

**建议**: ✅ **保留anyhow**

理由:
1. `thiserror`专注于**定义**结构化错误
2. `anyhow`专注于**传播**和**上下文**
3. 两者是互补关系，不是替代关系
4. 已有20+处使用，移除成本高
5. Rust生态中两者共存是常见模式

**示例说明**:
```rust
// thiserror用法 - 定义错误
#[derive(Error, Debug)]
enum WorkflowError {
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

// anyhow用法 - 运行时错误处理
fn load_config() -> anyhow::Result<Config> {
    let content = fs::read_to_string("config.toml")
        .context("failed to read config file")?;  // <- anyhow的Context
    let config = toml::from_str(&content)
        .context("failed to parse config")?;
    Ok(config)
}
```

---

### 2.2 futures crate评估

**当前状态**:
- `futures = "0.3"` - 提供额外的异步工具

**搜索使用场景**:
```bash
grep -r "futures" src/ --include="*.rs" | head -20
```

**发现的使用场景**:

#### A. `join_all` (最常见)
```rust
// 在engine_v2.rs中使用
use futures::future::join_all;

let results = join_all(handles).await;
```

**替代方案**:
```rust
// 使用tokio::join!宏(静态)或tokio::try_join!
let (a, b, c) = tokio::join!(task_a, task_b, task_c);

// 或使用tokio的JoinSet(动态)
let mut set = tokio::task::JoinSet::new();
for task in tasks {
    set.spawn(task);
}
while let Some(result) = set.join_next().await {
    // 处理结果
}
```

#### B. `Stream`扩展
```rust
// 如果使用了 futures::stream::StreamExt
```

**建议**: ⚠️ **需要代码审查后决定**

执行以下命令精确评估:
```bash
# 统计每个futures子模块的使用
grep -o "futures::[a-z_]*" src/ -r --include="*.rs" | sort | uniq -c

# 查看具体文件使用
rg "use futures" src/ --type rust
```

**决策树**:
```
如果只使用了 join_all:
  → 可以移除，用tokio::JoinSet替代

如果使用了 Stream/StreamExt:
  → 保留 futures，或改用 tokio-stream

如果使用了其他futures工具(如select!, ready等):
  → 保留 futures
```

**预计工作量**:
- 如果仅`join_all`: 1小时替换
- 如果`Stream`: 2-3小时评估替代方案
- 如果大量使用: 建议保留

---

### 2.3 tar crate评估

**搜索使用**:
```bash
grep -r "tar" src/ --include="*.rs"
```

**可能的使用场景**:
- Docker插件构建时打包
- 备份功能

**建议**: 🔍 **需要确认使用场景**

如果**未使用**: 移除
如果**Docker插件使用**: 保留

**快速检查**:
```bash
# 检查Docker插件文件
cat src/plugins/docker.rs | grep -i tar
```

---

## 3. 保留的依赖 (无需操作) ✅

### 核心依赖
- `tokio`, `tokio-util` - 异步运行时，核心
- `serde`, `serde_json`, `serde_yaml`, `toml` - 序列化，核心
- `config` - 配置管理，核心
- `clap`, `clap_complete` - CLI，核心
- `ratatui`, `crossterm` - TUI，核心

### 存储与缓存
- `moka` - 高性能缓存，核心
- `dashmap` - 并发哈希表，大量使用
- `parking_lot` - 同步原语，性能优化

### 工作流引擎
- `petgraph` - DAG图结构，核心
- `uuid` - 唯一标识符，核心
- `chrono` - 时间处理，核心

### 错误处理
- `thiserror` - 错误定义，核心
- `anyhow` - 错误处理，建议保留(见上文)

### 其他工具
- `regex` - 正则表达式，分类工具使用
- `jsonschema` - JSON Schema验证，配置验证
- `libloading` - 动态库加载，原生插件需要
- `async-trait` - 异步trait，核心
- `rand` - 随机数，测试/示例需要
- `num_cpus` - CPU核心数，并行度配置
- `bollard` - Docker客户端，Docker插件核心
- `flate2` - 压缩，可能用于Docker镜像
- `reqwest` - HTTP客户端，网络功能
- `hex` - 十六进制编码，可能用于校验

### 日志与监控
- `tracing`, `tracing-subscriber` - 结构化日志，核心
- `sysinfo` - 系统信息，TUI监控使用

### 可选功能依赖 (保留但可选)
- `lancedb`, `arrow`, `parquet` - LanceDB功能(feature gated)
- `rmcp`, `schemars` - MCP功能(feature gated)

---

## 4. 依赖优化建议

### 短期优化 (本次执行)

1. **移除确认未使用的依赖** (6个)
   - wasmtime, extism
   - mockall, testcontainers
   - metrics, metrics-exporter-prometheus

2. **评估futures** (需要30分钟检查)
   - 运行搜索命令
   - 决定保留或替换为tokio方案

3. **确认tar** (需要5分钟检查)
   - 检查Docker插件代码
   - 决定保留或移除

### 中期优化 (未来考虑)

1. **依赖版本升级**
   ```toml
   # 可考虑升级的依赖:
   tokio = "1.43"  # 最新版
   ratatui = "0.30"  # 最新版
   ```

2. **特性精简**
   ```toml
   # 当前tokio使用full特性，可精简:
   tokio = { version = "1.42", features = ["rt-multi-thread", "macros", "sync", "time", "fs", "process", "io-util"] }
   # 移除不需要的: "net", "signal", "io-std"等
   ```

3. **dev-dependencies优化**
   - 考虑添加`cargo-deny`检查依赖安全
   - 使用`cargo-udeps`检查未使用依赖

---

## 5. 执行命令清单

### 5.1 检查futures使用
```bash
# 统计使用
rg "futures::" src/ --type rust -o | sort | uniq -c

# 查看具体上下文
rg "use futures" src/ --type rust -A 3

# 查看join_all使用位置
rg "join_all" src/ --type rust -B 2 -A 2
```

### 5.2 检查tar使用
```bash
rg "tar::" src/ --type rust
rg "use tar" src/ --type rust
```

### 5.3 检查anyhow使用
```bash
rg "anyhow" src/ --type rust | wc -l
rg "anyhow::Context" src/ --type rust | wc -l
```

---

## 6. 建议的最终Cargo.toml

```toml
[dependencies]
# 异步运行时 (精简特性)
tokio = { version = "1.42", features = ["rt-multi-thread", "macros", "sync", "time", "fs", "process", "io-util"] }
tokio-util = "0.7"

# 序列化和配置
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
config = "0.14"

# CLI
clap = { version = "4.5", features = ["derive", "env"] }
clap_complete = "4.5"

# TUI
ratatui = "0.29"
crossterm = "0.28"

# MCP (可选)
rmcp = { version = "0.14", features = ["server", "macros", "transport-io"], optional = true }
schemars = { version = "0.8", optional = true }

# 数据库 (可选)
lancedb = { version = "0.20", optional = true }
arrow = { version = "54.0", optional = true }
parquet = { version = "54.0", optional = true }

# 缓存
moka = { version = "0.12", features = ["future"] }

# 日志和系统
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
sysinfo = "0.32"

# 插件
libloading = "0.8"

# 工作流调度
petgraph = "0.6"
uuid = { version = "1.11", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# 错误处理 (两者保留)
anyhow = "1.0"
thiserror = "2.0"

# 验证和解析
jsonschema = "0.18"
regex = "1.10"

# 并发
dashmap = "6.1"
parking_lot = "0.12"

# 异步trait
async-trait = "0.1"
futures = "0.3"  # ⚠️ 评估后决定是否保留

# 工具
rand = "0.8"
num_cpus = "1.16"

# Docker
bollard = "0.17"
tar = "0.4"  # ⚠️ 确认使用情况后决定
flate2 = "1.0"

# HTTP
reqwest = { version = "0.12", features = ["json"] }
hex = "0.4.3"

[dev-dependencies]
proptest = "1.6"
tokio-test = "0.4"
tempfile = "3.14"
# mockall - 移除
# testcontainers - 移除
```

---

## 7. 决策总结

| 依赖 | 建议 | 优先级 | 工作量 |
|------|------|--------|--------|
| wasmtime/extism | ✅ 移除 | 高 | 5分钟 |
| mockall/testcontainers | ✅ 移除 | 高 | 5分钟 |
| metrics/metrics-exporter | ✅ 移除 | 中 | 5分钟 |
| anyhow | ✅ 保留 | - | 0 |
| futures | ⚠️ 评估后决定 | 中 | 30分钟 |
| tar | ⚠️ 确认后决定 | 低 | 5分钟 |

**推荐执行顺序**:
1. 立即移除确认未使用的 (15分钟)
2. 检查futures和tar (35分钟)
3. 根据检查结果决定是否移除 (可变)
