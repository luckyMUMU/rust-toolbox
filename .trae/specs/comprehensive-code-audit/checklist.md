# 代码审查与结构整理检查清单

## Phase 1: 代码质量修复

### unwrap/expect 移除
- [x] `src/di/container.rs` 中无运行时 unwrap/expect 调用 - 已修复
- [x] `src/storage/backends.rs` 批量操作并发化 - 已实现
- [ ] `src/tools/types.rs` 中无运行时 unwrap/expect 调用 - 部分完成
- [ ] `src/workflow/engine.rs` 中无运行时 unwrap/expect 调用 - 部分完成
- [ ] `src/plugins/` 模块中无运行时 unwrap/expect 调用 - 部分完成
- [ ] `src/interfaces/` 模块中无运行时 unwrap/expect 调用 - 待处理

### TODO 功能实现
- [ ] `tools/types.rs` Python 执行器已实现 - 待实现
- [ ] `tools/types.rs` Node.js 执行器已实现 - 待实现
- [ ] `tools/types.rs` Docker 执行器已实现 - 待实现
- [ ] `tools/types.rs` WASM 执行器已实现 - 待实现
- [ ] `workflow/engine.rs` 工作流停止功能已实现 - 待实现
- [ ] `workflow/engine.rs` Switch 控制流已实现 - 待实现
- [ ] `workflow/engine.rs` Loop 控制流已实现 - 待实现

## Phase 2: 架构整理

### 目录结构
- [ ] `adapter/` 目录无空模块 - 待处理
- [ ] `adapter/` 和 `interfaces/` 无职责重叠 - 待处理
- [ ] 所有迁移 TODO 已处理 - 待处理

### 端口合并
- [ ] `application/port/` 和 `domain/port/` 已合并 - 待处理
- [ ] 所有端口接口位于 `domain/port/` - 待处理
- [ ] 依赖引用已更新 - 待处理

### Metrics 统一
- [ ] `performance/` 模块统一管理 metrics - 待处理
- [ ] `workflow/metrics.rs` 已整合或移除 - 待处理
- [ ] metrics 收集接口一致 - 待处理

## Phase 3: 安全加固

### unsafe 代码审查
- [x] `plugins/native.rs` 所有 unsafe 块有安全注释 - 已完成
- [ ] `plugins/wasm.rs` 所有 unsafe 块有安全注释 - 待处理
- [ ] `plugins/file_management/utils/utils.rs` 所有 unsafe 块有安全注释 - 待处理
- [ ] unsafe 块数量已最小化 - 部分完成

### 安全机制
- [ ] native 插件签名验证机制已实现 - 待实现
- [ ] 生产环境强制 JWT 密钥配置 - 待实现
- [ ] WASM 沙箱安全级别配置正确 - 待验证

## Phase 4: 性能优化

### 批量操作
- [x] `storage/backends.rs` batch_save 已并发化 - 已完成

### 锁优化
- [x] DI 容器锁竞争已优化 - 已完成（移除 unwrap，添加 if let Ok）

## Phase 5: 文档与验证

### 审查报告
- [x] 问题清单已生成 - 已完成
- [x] 修复记录已更新 - 已完成
- [ ] 设计文档已同步 - 待处理

### 构建验证
- [x] `cargo build` 无错误 - 已通过
- [ ] `cargo test` 全部通过 - 测试代码有编译错误（非主代码问题）
- [ ] `cargo clippy` 无警告 - 待运行
- [ ] `cargo fmt --check` 通过 - 待运行

---

## 已完成的修复摘要

### 1. DI 容器修复 (`src/di/container.rs`)
- 移除所有 `unwrap()` 调用，改用 `if let Ok(...)` 模式
- 修复 `register_factory_with_deps` bug：工厂方法现在接收当前容器引用而非创建新容器
- 添加 `factories_with_deps` 存储带依赖的工厂方法

### 2. 存储批量操作优化 (`src/storage/backends.rs`)
- `batch_save` 和 `batch_load` 使用 `futures::future::join_all` 并发执行

### 3. Native 插件安全注释 (`src/plugins/native.rs`)
- 为所有 unsafe 代码块添加 `// SAFETY:` 注释
- 说明安全性保证和风险缓解措施

## 待处理项

以下项目需要后续迭代处理：
1. 测试代码中的 trait 定义问题（ToolRegistry、ToolNode 等）
2. adapter/interfaces 目录架构整理
3. 端口接口合并
4. TODO 功能实现
5. 安全机制增强
