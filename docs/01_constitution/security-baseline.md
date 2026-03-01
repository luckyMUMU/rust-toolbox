# 安全基线 (Security Baseline)

> **版本**: v1.0.0  
> **创建日期**: 2026-03-01  
> **最后更新**: 2026-03-01  
> **状态**: Active  
> **级别**: P0 级（不可违背）

---

## 1. 概述

本文档定义了 Workflow Toolkit 项目的安全基线要求，这些是**必须满足**的最低安全标准。任何安全基线不达标都不得发布。

---

## 2. 输入验证

### 2.1 所有外部输入必须验证

**规则**: 所有来自外部的输入（用户输入、文件、网络、环境变量）必须经过验证

**验证维度**:
- 类型检查
- 长度限制
- 格式验证
- 范围检查

**示例**:
```rust
// ✅ 推荐：完整验证
pub fn validate_input(input: &str) -> Result<ValidatedInput> {
    // 长度检查
    if input.len() > 10 * 1024 {
        return Err(Error::InputTooLong);
    }
    
    // 格式检查
    if !input.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(Error::InvalidFormat);
    }
    
    // 内容检查
    if input.contains("..") || input.contains('/') {
        return Err(Error::PathTraversal);
    }
    
    Ok(ValidatedInput(input.to_string()))
}
```

### 2.2 字符串长度限制

**规则**: 所有字符串输入必须有长度限制

**默认限制**:
- 普通字段：10KB
- 文件路径：1KB
- 配置项：100KB
- 日志消息：10KB

---

## 3. 路径安全

### 3.1 路径规范化

**规则**: 所有文件路径必须规范化，防止路径遍历攻击

**示例**:
```rust
// ✅ 推荐：路径规范化
pub fn normalize_path(base: &Path, user_path: &str) -> Result<PathBuf> {
    let full_path = base.join(user_path);
    let canonical = full_path.canonicalize()
        .map_err(|_| Error::PathNotFound)?;
    
    // 确保在基目录内
    if !canonical.starts_with(base) {
        return Err(Error::PathTraversal);
    }
    
    Ok(canonical)
}
```

### 3.2 路径白名单

**规则**: 文件访问必须在白名单目录内

**配置示例**:
```toml
[security]
allowed_paths = [
    "/data/workflows",
    "/data/plugins",
    "/tmp"
]
```

---

## 4. 插件安全

### 4.1 沙箱隔离

**规则**: 所有插件必须在沙箱中执行

**隔离措施**:
- CPU 限制（cgroups）
- 内存限制
- 文件系统隔离
- 网络隔离（默认关闭）

**资源配置**:
```rust
pub struct PluginResourceLimit {
    pub max_memory_mb: u64,      // 默认：512MB
    pub max_cpu_percent: u8,     // 默认：50%
    pub max_file_descriptors: u64, // 默认：100
    pub allow_network: bool,     // 默认：false
}
```

### 4.2 插件权限最小化

**规则**: 插件只能访问必需的资源

**权限模型**:
```rust
pub struct PluginPermissions {
    pub read_paths: Vec<PathBuf>,
    pub write_paths: Vec<PathBuf>,
    pub allow_exec: bool,
    pub allow_network: bool,
    pub allow_env: bool,
}
```

---

## 5. 数据安全

### 5.1 敏感数据加密

**规则**: 敏感数据必须加密存储

**加密范围**:
- API 密钥
- 数据库密码
- 用户凭据
- 个人身份信息（PII）

**加密标准**:
- 对称加密：AES-256-GCM
- 非对称加密：RSA-2048 或 Ed25519
- 哈希：SHA-256 或 Argon2

### 5.2 内存安全

**规则**: 敏感数据使用后立即清除

**示例**:
```rust
// ✅ 推荐：安全清除
use zeroize::Zeroize;

let mut password = get_password();
use_password(&password);
password.zeroize(); // 立即清除
```

---

## 6. 认证与授权

### 6.1 API 认证

**规则**: 所有 API 接口必须要求认证（公开接口除外）

**认证方式**:
- Bearer Token（JWT）
- API Key
- OAuth 2.0

### 6.2 权限检查

**规则**: 所有操作必须检查权限

**权限模型**: RBAC（基于角色的访问控制）

```rust
pub enum Role {
    Admin,
    Developer,
    Viewer,
}

pub fn check_permission(role: Role, action: Action) -> Result<()> {
    match (role, action) {
        (Role::Admin, _) => Ok(()),
        (Role::Developer, Action::Read) => Ok(()),
        (Role::Developer, Action::Write) => Ok(()),
        (Role::Viewer, Action::Read) => Ok(()),
        _ => Err(Error::PermissionDenied),
    }
}
```

---

## 7. 日志安全

### 7.1 敏感信息脱敏

**规则**: 日志中不得包含敏感信息

**脱敏范围**:
- 密码
- API 密钥
- Token
- 个人身份信息

**示例**:
```rust
// ✅ 推荐：脱敏
tracing::info!("User login attempt: user={}", mask_username(&username));

fn mask_username(username: &str) -> String {
    if username.len() <= 2 {
        "**".to_string()
    } else {
        format!("{}{}**", &username[0..1], &username[1..2])
    }
}
```

### 7.2 审计日志

**规则**: 关键操作必须记录审计日志

**审计范围**:
- 用户登录/登出
- 权限变更
- 配置修改
- 数据删除

---

## 8. 错误处理

### 8.1 错误信息不泄露

**规则**: 错误信息不得泄露系统细节

**违规示例**:
```rust
// ❌ 禁止：泄露堆栈信息
return Err(format!("Database error at line {}: {}", line, error));
```

**正确做法**:
```rust
// ✅ 推荐：通用错误信息
return Err(Error::DatabaseError);
```

### 8.2 错误日志完整

**规则**: 服务端错误日志必须包含完整上下文

**日志内容**:
- 错误类型
- 时间戳
- 请求 ID
- 用户 ID（如适用）
- 相关参数（脱敏后）

---

## 9. 依赖安全

### 9.1 依赖审查

**规则**: 所有第三方依赖必须经过安全审查

**审查内容**:
- 许可证兼容性
- 已知漏洞（CVE）
- 维护活跃度
- 代码质量

### 9.2 依赖更新

**规则**: 定期更新依赖，修复安全漏洞

**更新频率**:
- 严重漏洞：24 小时内
- 高危漏洞：1 周内
- 中危漏洞：1 月内

---

## 10. 安全测试

### 10.1 模糊测试

**规则**: 关键组件必须通过模糊测试

**测试范围**:
- 输入解析器
- 序列化/反序列化
- 网络协议处理

### 10.2 渗透测试

**规则**: 发布前必须进行渗透测试

**测试内容**:
- OWASP Top 10
- 路径遍历
- 注入攻击
- 权限提升

---

## 11. 安全合规

### 11.1 安全清单

发布前必须完成以下检查：

- [ ] 所有输入已验证
- [ ] 路径已规范化
- [ ] 插件沙箱已启用
- [ ] 敏感数据已加密
- [ ] 日志已脱敏
- [ ] 认证授权已实现
- [ ] 依赖已审查
- [ ] 安全测试已通过

### 11.2 安全事件响应

**响应流程**:
1. 发现安全漏洞
2. 立即报告安全负责人
3. 评估影响范围
4. 制定修复方案
5. 紧急修复发布
6. 事后分析与改进

---

## 12. 变更历史

| 版本 | 日期 | 变更人 | 变更描述 |
|------|------|--------|----------|
| v1.0.0 | 2026-03-01 | Workflow Toolkit Team | 初始版本 |

---

*本文档是 P0 级安全基线，所有功能必须满足这些要求才能发布。*
