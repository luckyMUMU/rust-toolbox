# 全面代码审查与文件结构整理 Spec

## Why

代码库存在以下需要改进的问题：
1. **代码质量**：发现 835 处 `unwrap()`/`expect()` 调用，违反项目规范"严禁 unwrap/expect"
2. **未完成功能**：40+ 处 TODO 标记，部分核心功能未实现
3. **架构重叠**：`adapter/` 和 `interfaces/` 目录存在职责重叠，部分模块为空占位
4. **安全风险**：31 处 unsafe 代码块需要审查，native 插件缺少签名验证
5. **性能隐患**：批量操作未并发化，锁竞争风险

## What Changes

### 代码质量修复
- 移除运行时代码中的 `unwrap()`/`expect()` 调用
- 改用 `?` 操作符或 `map_err()` 进行错误传播
- 为错误添加上下文信息

### 架构整理
- 完成 `adapter/` 模块迁移或移除空模块
- 合并 `application/port` 和 `domain/port` 重复定义
- 统一 metrics 收集到 `performance/` 模块

### 安全加固
- 审查 unsafe 代码块，添加安全注释
- 为 native 插件添加签名验证机制
- 强化生产环境 JWT 密钥配置要求

### 性能优化
- 批量操作并发化
- 评估锁竞争热点，考虑使用 `DashMap`

## Impact

- Affected specs: 工具系统、插件系统、工作流引擎、存储系统
- Affected code: 
  - `src/tools/types.rs` - 移除 unwrap
  - `src/workflow/engine.rs` - 实现缺失功能
  - `src/di/container.rs` - 修复依赖解析
  - `src/storage/backends.rs` - 批量操作优化
  - `src/adapter/` - 架构整理

## ADDED Requirements

### Requirement: 代码质量标准

系统 SHALL 遵循以下代码质量标准：
- 运行时代码禁止使用 `unwrap()` 和 `expect()`
- 测试代码可使用 `unwrap()` 但应优先使用 `assert!`
- 所有错误必须提供上下文信息

#### Scenario: unwrap 移除验证
- **WHEN** 执行代码审查
- **THEN** 运行时代码中无 `unwrap()`/`expect()` 调用

### Requirement: 架构一致性

系统 SHALL 保持架构一致性：
- 消除 `adapter/` 和 `interfaces/` 目录重叠
- 空模块应实现或移除
- 端口接口统一放置在 `domain/port/`

#### Scenario: 架构整理验证
- **WHEN** 检查目录结构
- **THEN** 无空模块占位，无职责重叠

### Requirement: 安全代码审查

系统 SHALL 对 unsafe 代码进行安全审查：
- 所有 unsafe 块必须有安全注释
- native 插件必须有签名验证
- 生产环境强制配置 JWT 密钥

#### Scenario: unsafe 代码审查
- **WHEN** 检查 unsafe 代码块
- **THEN** 每个块都有 `# Safety` 注释说明安全性保证

## MODIFIED Requirements

### Requirement: 错误处理规范

原有：使用 `thiserror` 进行错误定义
修改为：使用 `thiserror` 进行错误定义，所有错误传播使用 `?` 操作符，禁止 panic

### Requirement: 性能优化

原有：批量操作顺序执行
修改为：批量操作使用 `futures::join!` 或 `tokio::join!` 并发执行

## REMOVED Requirements

### Requirement: 空模块占位
**Reason**: 空模块增加维护负担且无实际功能
**Migration**: 实现功能或删除模块
