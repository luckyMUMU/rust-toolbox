# 质量红线 (Quality Redlines)

> **版本**: v1.0.0  
> **创建日期**: 2026-03-01  
> **最后更新**: 2026-03-01  
> **状态**: Active  
> **级别**: P0 级（不可违背）

---

## 1. 概述

本文档定义了 Workflow Toolkit 项目的质量红线，这些是**不可违背**的约束。任何违反质量红线的代码都**不得提交**，已提交的必须**立即修复**。

---

## 2. 安全红线

### 2.1 禁止硬编码密钥

**规则**: 严禁在代码中硬编码任何敏感信息

**违规示例**:
```rust
// ❌ 禁止
const API_KEY: &str = "sk-1234567890abcdef";
const DB_PASSWORD: &str = "password123";
```

**正确做法**:
```rust
// ✅ 推荐
let api_key = std::env::var("API_KEY")
    .map_err(|_| ConfigError::MissingEnvVar("API_KEY"))?;
```

**验证方式**: 
- 代码审查
- 静态分析工具（grep 搜索 "password", "secret", "key" 等关键词）

### 2.2 禁止强制解包

**规则**: 严禁使用 `unwrap()` 或 `expect()`（测试代码除外）

**违规示例**:
```rust
// ❌ 禁止
let value = option.unwrap();
let result = some_result.expect("This should not fail");
```

**正确做法**:
```rust
// ✅ 推荐
let value = option.ok_or(Error::MissingValue)?;
let result = some_result.map_err(|e| Error::Conversion(e))?;
```

**验证方式**: 
- Clippy lint: `unwrap_used`, `expect_used`
- 代码审查

### 2.3 禁止未验证的输入

**规则**: 所有外部输入必须经过验证和规范化

**违规示例**:
```rust
// ❌ 禁止
let file_path = user_input;
std::fs::read_to_string(&file_path)?;
```

**正确做法**:
```rust
// ✅ 推荐
let file_path = validate_and_normalize_path(user_input)
    .map_err(|e| Error::InvalidPath(e))?;
if !file_path.starts_with("/allowed/base") {
    return Err(Error::PathTraversal);
}
```

**验证方式**: 
- 代码审查
- 模糊测试

---

## 3. 性能红线

### 3.1 API 响应时间

**规则**: 所有 API 接口的响应时间不得超过 500ms（P95）

**测量方式**:
- 性能测试套件
- 生产环境监控

**违规处理**:
- 性能测试不通过不得发布
- 生产环境 P95 超过 500ms 需立即优化

### 3.2 内存泄漏

**规则**: 严禁内存泄漏

**验证方式**:
- 长时间运行测试（24 小时+）
- 内存分析工具（Valgrind、heaptrack）

**违规处理**:
- 发现内存泄漏立即修复
- 无法定位的泄漏需架构审查

---

## 4. 代码质量红线

### 4.1 代码覆盖率

**规则**: 单元测试覆盖率不得低于 80%

**测量方式**:
- `cargo tarpaulin` 或 `cargo-llvm-cov`

**违规处理**:
- CI 检查覆盖率，不通过禁止合并

### 4.2 编译警告

**规则**: 严禁编译警告

**验证方式**:
- CI 构建使用 `RUSTFLAGS="-D warnings"`

**违规处理**:
- 编译警告即失败

### 4.3 文档完整性

**规则**: 所有公共 API 必须有文档注释

**违规示例**:
```rust
// ❌ 禁止
pub fn process_data(input: String) -> Result<String> {
    // ...
}
```

**正确做法**:
```rust
// ✅ 推荐
/// 处理输入数据并返回结果
/// 
/// # 参数
/// * `input` - 输入数据字符串
/// 
/// # 返回
/// 处理后的结果字符串
/// 
/// # 错误
/// * `Error::InvalidInput` - 输入格式错误
pub fn process_data(input: String) -> Result<String> {
    // ...
}
```

**验证方式**:
- `cargo doc` 检查未文档化的公共项

---

## 5. 架构红线

### 5.1 禁止循环依赖

**规则**: 严禁模块间循环依赖

**验证方式**:
- 架构审查
- 依赖分析工具

### 5.2 分层依赖规则

**规则**: 依赖只能从外层指向内层，禁止反向依赖

**依赖方向**:
```
Interfaces → Application → Domain ← Infrastructure
```

**违规示例**:
```rust
// ❌ 禁止：领域层依赖基础设施层
use crate::infrastructure::DatabaseConnection;

pub struct User {
    db: DatabaseConnection, // 错误！
}
```

**正确做法**:
```rust
// ✅ 推荐：领域层定义抽象接口
pub trait UserRepository {
    fn find_by_id(&self, id: u64) -> Result<User>;
}

// 基础设施层实现接口
impl UserRepository for DatabaseConnection {
    fn find_by_id(&self, id: u64) -> Result<User> {
        // ...
    }
}
```

**验证方式**:
- 架构审查
- 依赖分析

---

## 6. 测试红线

### 6.1 测试失败处理

**规则**: 任何测试失败必须立即修复，禁止跳过失败测试

**违规处理**:
- CI 测试失败禁止合并
- 生产环境发现 bug 需补充测试用例

### 6.2 测试独立性

**规则**: 测试用例必须独立，不得相互依赖

**违规示例**:
```rust
// ❌ 禁止
#[test]
fn test_create_user() { /* ... */ }

#[test]
fn test_delete_user() {
    // 依赖 test_create_user 先执行
    let user = get_user_from_previous_test(); // 错误！
}
```

**正确做法**:
```rust
// ✅ 推荐
#[test]
fn test_delete_user() {
    // 自己创建测试数据
    let user = create_test_user();
    // ...
}
```

---

## 7. 文档红线

### 7.1 先写规范后写代码

**规则**: 任何功能开发必须先写规范文档

**违规处理**:
- 无规范文档的代码 PR 直接拒绝

### 7.2 文档更新同步

**规则**: 代码变更后必须同步更新相关文档

**违规处理**:
- 文档与代码不一致需立即修复

---

## 8. 熔断机制

### 8.1 自动熔断

以下情况触发自动熔断：

1. **安全红线违反**: 发现硬编码密钥
2. **性能红线违反**: P95 响应时间 > 1s
3. **测试红线违反**: 关键测试用例失败
4. **架构红线违反**: 循环依赖

### 8.2 熔断处理

触发熔断后：

1. 立即停止所有开发活动
2. 技术负责人组织紧急修复
3. 修复完成后需技术委员会审批
4. 更新质量红线文档（如需要）

---

## 9. 违规记录

| 日期 | 违规类型 | 描述 | 处理措施 | 状态 |
|------|----------|------|----------|------|
| - | - | - | - | - |

---

## 10. 变更历史

| 版本 | 日期 | 变更人 | 变更描述 |
|------|------|--------|----------|
| v1.0.0 | 2026-03-01 | Workflow Toolkit Team | 初始版本 |

---

*本文档是 P0 级质量红线，所有项目成员必须严格遵守。违反质量红线的代码不得提交、不得合并、不得发布。*
