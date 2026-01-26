# 任务 1.1: 修复所有 unwrap() 使用

## 📋 任务概述
**目标**: 消除 870 处 `unwrap()`/`expect()` 调用  
**影响**: 提升生产环境稳定性，防止运行时 panic  
**预计工时**: 2-3 天  
**优先级**: 高（阻塞其他任务）

---

## 🎯 验证标准

### 验证命令
```bash
# 统计当前 unwrap() 数量
grep -rn "\.unwrap()" src/ | wc -l
# 预期: < 100

# 统计当前 expect() 数量
grep -rn "\.expect(" src/ | wc -l
# 预期: < 10

# 编译验证
cargo check
# 预期: 无编译错误

# 测试验证
cargo test --lib
# 预期: 所有测试通过

# 代码质量验证
cargo clippy
# 预期: 无新警告
```

### 验证清单
- [ ] unwrap/expect 使用减少 90% 以上（从 870 到 < 100）
- [ ] 所有错误都有适当的上下文信息
- [ ] `cargo check` 通过
- [ ] `cargo test --lib` 通过
- [ ] `cargo clippy` 无新警告

---

## 📊 当前状态

### 统计数据
- **当前 unwrap() 数量**: 868 个
- **当前 expect() 数量**: 7 个
- **总计**: 875 个
- **目标**: < 100 个
- **需要修复**: 775+ 个

### 分类处理策略
1. **可恢复错误**: 使用 `?` 操作符
2. **不可恢复错误**: 使用 `expect()` 并添加详细错误信息
3. **确定性操作**: 保留 `unwrap()`（如 `Option::unwrap()` 在确定有值时）

---

## 📝 修复指南

### 1. RwLock/RwLockWriteGuard unwrap() 修复

**位置**: `src/config.rs:541-542`

**当前代码**:
```rust
let mut config = self.config.write().unwrap();
let mut sources = self.sources.write().unwrap();
```

**修复方案**:
```rust
let mut config = self.config.write().map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to acquire config write lock: {}",
        e
    ))
})?;
let mut sources = self.sources.write().map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to acquire sources write lock: {}",
        e
    ))
})?;
```

**说明**: RwLock 的 `write()` 可能失败（死锁或毒化），需要错误处理

---

### 2. TempDir 创建 unwrap() 修复

**位置**: `src/config.rs:803`

**当前代码**:
```rust
let temp_dir = TempDir::new().unwrap();
```

**修复方案**:
```rust
let temp_dir = TempDir::new().map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to create temporary directory: {}",
        e
    ))
})?;
```

**说明**: 文件系统操作可能失败，需要错误处理

---

### 3. 文件写入 unwrap() 修复

**位置**: `src/config.rs:815, 838`

**当前代码**:
```rust
std::fs::write(&config_file, initial_config).unwrap();
```

**修复方案**:
```rust
std::fs::write(&config_file, initial_config).map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to write config file: {}",
        e
    ))
})?;
```

**说明**: 文件 I/O 操作可能失败，需要错误处理

---

### 4. 字符串解析 unwrap() 修复

**位置**: `src/core/version.rs:102, 106`

**当前代码**:
```rust
let version_part = parts.next().unwrap();
let version_numbers = parts.next().unwrap();
```

**修复方案**:
```rust
let version_part = parts.next().ok_or_else(|| {
    crate::WorkflowError::validation("Invalid version format: missing version part")
})?;

let version_numbers = parts.next().ok_or_else(|| {
    crate::WorkflowError::validation("Invalid version format: missing version numbers")
})?;
```

**说明**: Iterator::next() 返回 Option，需要处理 None 情况

---

### 5. Semaphore acquire unwrap() 修复

**位置**: `src/interfaces/cli/app.rs:1323`

**当前代码**:
```rust
let _permit = semaphore.acquire().await.unwrap();
```

**修复方案**:
```rust
let _permit = semaphore.acquire().await.map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to acquire semaphore permit: {}",
        e
    ))
})?;
```

**说明**: Semaphore 的 acquire() 可能失败（关闭或取消），需要错误处理

---

### 6. JSON/YAML 解析 unwrap() 修复

**位置**: `src/interfaces/cli/output.rs:459, 470`

**当前代码**:
```rust
let _: serde_json::Value = serde_json::from_str(&output).unwrap();
let _: serde_json::Value = serde_yaml::from_str(&output).unwrap();
```

**修复方案**:
```rust
let _: serde_json::Value = serde_json::from_str(&output).map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to parse JSON output: {}",
        e
    ))
})?;

let _: serde_json::Value = serde_yaml::from_str(&output).map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to parse YAML output: {}",
        e
    ))
})?;
```

**说明**: 序列化/反序列化可能失败，需要错误处理

---

### 7. 测试中的 unwrap() 修复

**位置**: `src/interfaces/mcp_test.rs:14`

**当前代码**:
```rust
let tools = server.list_tools().await.unwrap();
```

**修复方案**:
```rust
let tools = server.list_tools().await.expect("Failed to list tools");
```

**说明**: 测试中的 unwrap() 可以替换为 expect()，提供更好的错误消息

---

## 🎯 修复优先级

### 高优先级（必须修复）
1. **生产代码中的 unwrap()** - 影响稳定性
2. **并发相关的 unwrap()** - 可能导致死锁
3. **文件 I/O 的 unwrap()** - 可能导致数据丢失

### 中优先级（应该修复）
1. **配置相关的 unwrap()** - 影响配置加载
2. **网络相关的 unwrap()** - 影响网络操作
3. **解析相关的 unwrap()** - 影响数据处理

### 低优先级（可以修复）
1. **测试中的 unwrap()** - 仅影响测试
2. **示例中的 unwrap()** - 仅影响示例
3. **文档中的 unwrap()** - 仅影响文档

---

## 📝 记录学习

### 学习点 1: RwLock 错误处理
**问题**: RwLock 的 `write()` 可能失败
**原因**: 死锁或毒化（poisoned）
**解决方案**: 使用 `map_err()` 转换为 WorkflowError
**示例**:
```rust
self.config.write().map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to acquire config write lock: {}",
        e
    ))
})?;
```

### 学习点 2: Iterator 错误处理
**问题**: Iterator::next() 返回 Option
**原因**: 可能没有下一个元素
**解决方案**: 使用 `ok_or_else()` 转换为 Result
**示例**:
```rust
let version_part = parts.next().ok_or_else(|| {
    crate::WorkflowError::validation("Invalid version format: missing version part")
})?;
```

### 学习点 3: 文件 I/O 错误处理
**问题**: 文件操作可能失败
**原因**: 权限、磁盘空间、文件不存在等
**解决方案**: 使用 `map_err()` 提供详细错误信息
**示例**:
```rust
std::fs::write(&config_file, content).map_err(|e| {
    crate::WorkflowError::workflow_execution(format!(
        "Failed to write config file: {}",
        e
    ))
})?;
```

### 学习点 4: 测试中的 expect()
**问题**: 测试中的 unwrap() 失败时信息不足
**原因**: unwrap() 只显示 "called `Result::unwrap()` on an `Err` value"
**解决方案**: 使用 expect() 提供详细错误信息
**示例**:
```rust
let tools = server.list_tools().await.expect("Failed to list tools");
```

---

## 📊 修复进度跟踪

### 文件修复统计
| 文件 | unwrap() 数量 | 状态 | 修复时间 |
|------|--------------|------|----------|
| src/config.rs | 10+ | ⬜ 待修复 | - |
| src/core/version.rs | 5+ | ⬜ 待修复 | - |
| src/interfaces/cli/app.rs | 2+ | ⬜ 待修复 | - |
| src/interfaces/cli/output.rs | 2+ | ⬜ 待修复 | - |
| src/interfaces/mcp_test.rs | 1+ | ⬜ 待修复 | - |
| ... | ... | ... | - |

### 总体进度
- **已修复**: 0 个
- **待修复**: 875+ 个
- **完成度**: 0%

---

## 🚀 执行步骤

### 步骤 1: 分类扫描
```bash
# 扫描所有 unwrap() 使用
grep -rn "\.unwrap()" src/ > unwrap_list.txt

# 扫描所有 expect() 使用
grep -rn "\.expect(" src/ > expect_list.txt

# 分类统计
cat unwrap_list.txt | wc -l  # 总数
cat unwrap_list.txt | grep "RwLock" | wc -l  # RwLock 相关
cat unwrap_list.txt | grep "TempDir" | wc -l  # TempDir 相关
cat unwrap_list.txt | grep "fs::" | wc -l  # 文件 I/O 相关
cat unwrap_list.txt | grep "semaphore" | wc -l  # Semaphore 相关
```

### 步骤 2: 按优先级修复
1. 修复生产代码中的 unwrap()
2. 修复并发相关的 unwrap()
3. 修复文件 I/O 的 unwrap()
4. 修复配置相关的 unwrap()
5. 修复网络相关的 unwrap()
6. 修复解析相关的 unwrap()
7. 修复测试中的 unwrap()

### 步骤 3: 验证修复
```bash
# 统计剩余 unwrap() 数量
grep -rn "\.unwrap()" src/ | wc -l
# 预期: < 100

# 编译验证
cargo check

# 测试验证
cargo test --lib

# 代码质量验证
cargo clippy
```

### 步骤 4: 更新文档
- 更新 AGENTS.md 文件
- 添加错误处理最佳实践
- 记录学习点

---

## 📚 参考资料

### Rust 最佳实践
- [Rust Error Handling](https://doc.rust-lang.org/stable/rust-by-example/error.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Error Handling](https://tokio.rs/tokio/tutorial/error)

### 本项目参考
- `src/error.rs` - 错误类型定义
- `src/config.rs:275-277` - 已修复的 unwrap 示例
- `src/config.rs:287-290` - RwLock unwrap 修复示例

---

## ✅ 完成检查清单

- [ ] unwrap/expect 使用减少 90% 以上（从 870 到 < 100）
- [ ] 所有错误都有适当的上下文信息
- [ ] `cargo check` 通过
- [ ] `cargo test --lib` 通过
- [ ] `cargo clippy` 无新警告
- [ ] 更新 AGENTS.md 文档
- [ ] 记录学习点到 notepad

---

**任务创建时间**: 2026-01-25  
**任务状态**: 待执行  
**预计工时**: 2-3 天  
**依赖**: 无
