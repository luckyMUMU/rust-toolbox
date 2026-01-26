# Rust 工具库代码优化工作计划

## 📋 项目概述

**目标**: 优化 Rust 工具库代码质量、性能和可维护性  
**代码库**: workflow-toolkit (124 个 Rust 文件, 94,312 行代码)  
**预计工期**: 30-40 小时  
**优先级**: 高（影响生产环境稳定性）

---

## 🎯 核心优化目标

### 1. 代码质量提升
- 修复所有 `unwrap()` 使用（858 处 → < 100 处）
- 统一错误处理模式
- 提升代码健壮性

### 2. 安全修复
- ✅ 修复硬编码 JWT 密钥
- ✅ 修复 CORS 配置
- 消除安全漏洞

### 3. 结构优化
- 拆分超大文件（4 个文件 >2,500 行）
- 降低单文件复杂度
- 提升可维护性

### 4. 性能优化
- 优化异步代码
- 改进缓存策略
- 减少锁竞争

### 5. 测试完善
- 提升测试覆盖率（70% → 85%）
- 添加错误场景测试
- 配置 CI/CD 自动化

### 6. 文档完善
- 更新所有 AGENTS.md 文件
- 添加安全指南
- 完善 API 文档

---

## 📊 代码库分析结果

### 复杂度热点
| 文件 | 行数 | 复杂度 | 问题 |
|------|------|--------|------|
| `src/plugins/file_management/utils.rs` | 3,604 | 🔴 极高 | 需立即拆分 |
| `src/interfaces/tui/widgets/plugin_manager.rs` | 3,553 | 🔴 极高 | 需立即拆分 |
| `src/interfaces/tui/layout.rs` | 3,275 | 🔴 极高 | 需立即拆分 |
| `src/workflow/engine.rs` | 2,404 | 🟡 高 | 考虑拆分 |

### 代码质量问题统计
- **unwrap/expect 使用**: 858 个 ⚠️
- **异步函数**: 1,230 个 ✅
- **公共类型定义**: 870 个 ✅
- **智能指针使用**: 593 处 ✅
- **集合类型使用**: 673 处 ✅
- **并发控制**: 91 处 ✅
- **缓存相关**: 994 处 ⚠️
- **Trait 实现**: 351 个 ✅
- **有测试的文件**: 59 个 ⚠️

---

## 📋 工作计划

### 阶段 1: 紧急修复（高优先级）

#### 任务 1.1：修复所有 unwrap() 使用
**目标**: 消除 858 处 `unwrap()` 调用  
**影响**: 提升生产环境稳定性，防止运行时 panic

**实施步骤**:
1. 使用 grep/ast-grep 扫描所有 unwrap 使用
2. 分类处理：
   - 可恢复错误：使用 `?` 操作符
   - 不可恢复错误：使用 `expect()` 并添加详细错误信息
   - 确定性操作：保留 `unwrap()`（如 `Option::unwrap()` 在确定有值时）
3. 创建错误处理辅助宏
4. 运行测试验证

**验收标准**:
- unwrap/expect 使用减少 90% 以上
- 所有错误都有适当的上下文信息
- 测试通过率 100%

**预计工时**: 2-3 天  
**依赖**: 无

---

#### 任务 1.2：统一错误处理模式
**目标**: 创建统一的错误处理工具和模式

**实施步骤**:
1. 分析现有错误类型
2. 创建错误处理宏
3. 统一错误消息格式
4. 添加错误上下文

**验收标准**:
- 所有错误都有清晰的上下文
- 错误消息格式统一
- 错误类型层次清晰

**预计工时**: 1-2 天  
**依赖**: 任务 1.1

---

#### 任务 1.3：修复安全问题 - 硬编码密钥
**目标**: 消除硬编码 JWT 密钥

**实施步骤**:
1. 从环境变量 `WORKFLOW_TOOLKIT_JWT_SECRET` 读取密钥
2. 未设置时生成随机 32 字节密钥
3. 添加日志记录
4. 更新配置文档

**验收标准**:
- JWT 密钥从环境变量加载
- 生成随机密钥作为后备
- 无硬编码密钥

**预计工时**: 0.5 天  
**依赖**: 无

**状态**: ✅ 已完成

---

#### 任务 1.4：修复安全问题 - CORS 配置
**目标**: 限制 CORS 来源，避免使用通配符

**实施步骤**:
1. 从环境变量 `WORKFLOW_TOOLKIT_CORS_ORIGINS` 读取允许的来源
2. 默认使用 `localhost` 和 `127.0.0.1`
3. 验证来源格式
4. 添加日志记录

**验收标准**:
- CORS 配置限制为可信来源
- 无通配符 CORS 配置
- 默认值安全

**预计工时**: 0.5 天  
**依赖**: 无

**状态**: ✅ 已完成

---

### 阶段 2: 结构优化

#### 任务 2.1：拆分 `src/plugins/file_management/utils.rs`
**目标**: 将 3,604 行的超大文件拆分为多个模块

**拆分方案**:
```
src/plugins/file_management/utils.rs (3,604 行)
├── file_classifier.rs (800 行) - 文件分类逻辑
├── batch_processor.rs (700 行) - 批量处理
├── text_analyzer.rs (600 行) - 文本分析
├── ai_integration.rs (500 行) - AI 集成
├── human_decision.rs (400 行) - 人工决策
├── file_operations.rs (400 行) - 文件操作
└── utils_types.rs (204 行) - 类型定义
```

**验收标准**:
- 所有功能正常
- 单文件行数 < 1,000 行
- 测试覆盖率 > 80%
- 无编译错误

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

#### 任务 2.2：拆分 `src/interfaces/tui/widgets/plugin_manager.rs`
**目标**: 将 3,553 行的超大文件拆分为多个模块

**拆分方案**:
```
src/interfaces/tui/widgets/plugin_manager.rs (3,553 行)
├── plugin_list.rs (800 行) - 插件列表显示
├── plugin_details.rs (700 行) - 插件详情
├── plugin_actions.rs (600 行) - 插件操作
├── plugin_config.rs (500 行) - 插件配置
├── plugin_ui.rs (800 行) - UI 组件和渲染
└── plugin_types.rs (353 行) - 类型定义
```

**验收标准**:
- 所有功能正常
- 单文件行数 < 1,000 行
- UI 渲染性能无退化
- 测试覆盖率 > 80%

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

#### 任务 2.3：拆分 `src/interfaces/tui/layout.rs`
**目标**: 将 3,275 行的超大文件拆分为多个模块

**拆分方案**:
```
src/interfaces/tui/layout.rs (3,275 行)
├── layout_calculator.rs (800 行) - 布局计算逻辑
├── responsive_design.rs (700 行) - 响应式设计
├── widget_positioning.rs (800 行) - 组件定位
├── constraint_solver.rs (600 行) - 约束求解
└── layout_types.rs (375 行) - 类型定义
```

**验收标准**:
- 布局计算正确
- 响应式设计正常
- 单文件行数 < 1,000 行
- 测试覆盖率 > 80%

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

#### 任务 2.4：拆分 `src/workflow/engine.rs`
**目标**: 将 2,404 行的超大文件拆分为多个模块

**拆分方案**:
```
src/workflow/engine.rs (2,404 行)
├── engine_core.rs (600 行) - 核心执行逻辑
├── engine_state.rs (500 行) - 状态管理
├── engine_recovery.rs (600 行) - 错误恢复
├── engine_cache.rs (400 行) - 缓存管理
└── engine_types.rs (304 行) - 类型定义
```

**验收标准**:
- 工作流执行正常
- 状态管理正确
- 错误恢复功能正常
- 单文件行数 < 1,000 行
- 测试覆盖率 > 80%

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

### 阶段 3: 性能优化

#### 任务 3.1：优化异步代码性能
**目标**: 提升并发性能，减少异步开销

**实施步骤**:
1. 分析热点异步函数
2. 使用 `tokio::select!` 优化并发任务
3. 使用 `tokio::spawn` 处理独立任务
4. 优化任务调度策略
5. 添加性能基准测试

**验收标准**:
- 并发性能提升 > 15%
- 无死锁或竞态条件
- 基准测试通过
- 内存使用优化

**预计工时**: 2-3 天  
**依赖**: 任务 1.1

---

#### 任务 3.2：优化缓存策略
**目标**: 提升缓存命中率，减少重复计算

**实施步骤**:
1. 分析现有缓存使用模式
2. 优化 Moka 缓存配置
3. 实现多级缓存策略
4. 添加缓存预热机制
5. 监控缓存命中率

**验收标准**:
- 缓存命中率 > 60%
- 减少重复计算 50%+
- 无内存泄漏
- 基准测试通过

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

#### 任务 3.3：减少锁竞争
**目标**: 优化并发控制，减少锁等待

**实施步骤**:
1. 分析锁使用热点
2. 使用读写锁替代互斥锁
3. 实现无锁数据结构
4. 优化锁粒度
5. 添加锁竞争监控

**验收标准**:
- 锁竞争减少 30%+
- 并发性能提升
- 无死锁
- 基准测试通过

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

### 阶段 4: 测试完善

#### 任务 4.1：提升测试覆盖率
**目标**: 将测试覆盖率从 70% 提升到 85%

**实施步骤**:
1. 分析现有测试覆盖情况
2. 为关键模块添加单元测试
3. 添加集成测试
4. 添加性能基准测试
5. 配置 CI/CD 自动化测试

**验收标准**:
- 测试覆盖率 > 85%
- 所有新模块有测试
- 基准测试通过
- CI/CD 自动运行

**预计工时**: 3-4 天  
**依赖**: 任务 1.1

---

#### 任务 4.2：添加错误场景测试
**目标**: 测试各种错误场景和边界条件

**实施步骤**:
1. 测试超时场景
2. 测试并发冲突
3. 测试资源耗尽
4. 测试网络异常
5. 测试数据损坏

**验收标准**:
- 所有错误场景有测试
- 边界条件覆盖完整
- 测试通过率 100%

**预计工时**: 2 天  
**依赖**: 任务 4.1

---

### 阶段 5: 文档完善

#### 任务 5.1：更新 AGENTS.md 文档
**目标**: 更新所有 AGENTS.md 文件，反映代码改进

**实施步骤**:
1. 更新根 AGENTS.md
2. 更新模块特定 AGENTS.md
3. 添加改进说明
4. 更新代码示例
5. 验证链接有效性

**验收标准**:
- 文档覆盖率 > 80%
- 所有公共 API 有文档
- 示例代码可运行
- 链接有效

**预计工时**: 2 天  
**依赖**: 任务 1.1

---

#### 任务 5.2：添加安全指南
**目标**: 创建安全配置指南

**内容**:
- JWT 密钥管理
- CORS 配置
- 认证配置
- 安全最佳实践

**验收标准**:
- 安全指南完整
- 配置示例可运行
- 最佳实践清晰

**预计工时**: 1 天  
**依赖**: 任务 5.1

---

## 📅 时间估算

| 阶段 | 任务 | 时间（小时） |
|------|------|-------------|
| 阶段 1 | 修复高优先级问题 | 8-12 |
| 阶段 2 | 结构优化 | 8-10 |
| 阶段 3 | 性能优化 | 6-8 |
| 阶段 4 | 测试完善 | 5-6 |
| 阶段 5 | 文档完善 | 3-4 |
| **总计** | | **30-40 小时** |

---

## 🎯 验证标准

### 安全验证
- [ ] JWT 密钥从环境变量加载
- [ ] 生成随机密钥作为后备
- [ ] CORS 配置限制为可信来源
- [ ] 无硬编码密钥
- [ ] 无通配符 CORS 配置

### 代码质量验证
- [ ] unwrap/expect 使用 < 100 处
- [ ] 所有文件行数 < 1,000 行
- [ ] 无重复代码
- [ ] 所有公共 API 有文档
- [ ] 错误处理完整

### 功能验证
- [ ] 超时包装器正常工作
- [ ] 工具注册表线程安全
- [ ] 批量执行性能优化
- [ ] 错误场景正确处理

### 测试验证
- [ ] 集成测试通过
- [ ] 错误场景测试通过
- [ ] 并发测试通过
- [ ] 性能测试通过

### 文档验证
- [ ] AGENTS.md 文档更新
- [ ] 安全指南创建
- [ ] 示例代码可运行
- [ ] 链接有效

---

## 📚 参考资料

### Rust 最佳实践
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Best Practices](https://tokio.rs/tokio/topics/bridging)
- [Error Handling in Rust](https://doc.rust-lang.org/stable/rust-by-example/error.html)

### 性能优化
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Performance](https://tokio.rs/tokio/topics/performance)
- [Cargo Bench Guide](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)

### 代码质量
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rust Patterns](https://rust-unofficial.github.io/patterns/)
- [Clean Code in Rust](https://github.com/mre/idiomatic-rust)

### 本项目文档
- [AGENTS.md](../AGENTS.md) - 项目概述
- [src/interfaces/cli/AGENTS.md](../src/interfaces/cli/AGENTS.md) - CLI 文档
- [GLOBAL_RULES.md](../GLOBAL_RULES.md) - 全局规则

---

## 🚀 开始执行

要开始执行此计划，请运行：

```bash
/start-work
```

这将：
1. 注册此计划为当前工作
2. 跟踪进度 across sessions
3. 支持中断后自动继续

---

## 📋 TODOs

> Implementation + Test = ONE Task. Never separate.
> Specify parallelizability for EVERY task.

### 阶段 1: 紧急修复（高优先级）

- [ ] **任务 1.1：修复所有 unwrap() 使用**

  **What to do**:
  - 使用 grep/ast-grep 扫描所有 unwrap 使用
  - 分类处理：可恢复错误使用 `?`，不可恢复错误使用 `expect()`
  - 创建错误处理辅助宏
  - 运行测试验证

  **Must NOT do**:
  - 不要删除错误处理逻辑
  - 不要忽略编译错误

  **Parallelizable**: NO (基础任务，其他任务依赖)

  **References**:
  - `src/config.rs:275-277` - 已修复的 unwrap 示例
  - `src/config.rs:287-290` - RwLock unwrap 修复示例
  - `src/error.rs` - 错误类型定义

  **Acceptance Criteria**:
  - [ ] unwrap/expect 使用减少 90% 以上（从 858 到 < 100）
  - [ ] 所有错误都有适当的上下文信息
  - [ ] `cargo test` 通过率 100%
  - [ ] `cargo clippy` 无新警告

  **Manual Execution Verification**:
  - [ ] 运行命令: `grep -rn "\.unwrap()" src/ | wc -l`
  - [ ] 预期输出: < 100
  - [ ] 运行命令: `cargo test`
  - [ ] 预期输出: 所有测试通过

  **Commit**: YES
  - Message: `fix: remove unwrap() calls and add proper error handling`
  - Files: `src/**/*.rs`
  - Pre-commit: `cargo test`

---

- [ ] **任务 1.2：统一错误处理模式**

  **What to do**:
  - 分析现有错误类型
  - 创建错误处理宏
  - 统一错误消息格式
  - 添加错误上下文

  **Must NOT do**:
  - 不要改变错误类型的公共 API
  - 不要删除现有的错误变体

  **Parallelizable**: YES (与任务 1.3, 1.4 并行)

  **References**:
  - `src/error.rs` - 现有错误类型
  - `src/core.rs` - 核心类型定义

  **Acceptance Criteria**:
  - [ ] 所有错误都有清晰的上下文
  - [ ] 错误消息格式统一
  - [ ] 错误类型层次清晰
  - [ ] `cargo test` 通过

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo build`
  - [ ] 预期: 无编译错误
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `refactor: unify error handling patterns`
  - Files: `src/error.rs, src/core.rs`
  - Pre-commit: `cargo test`

---

- [ ] **任务 1.3：修复安全问题 - 硬编码密钥**

  **What to do**:
  - 从环境变量 `WORKFLOW_TOOLKIT_JWT_SECRET` 读取密钥
  - 如果未设置，生成 32 字节的随机密钥
  - 记录密钥（仅开发环境）
  - 更新配置文档

  **Must NOT do**:
  - 不要硬编码任何密钥
  - 不要在生产环境记录密钥

  **Parallelizable**: YES (与任务 1.2, 1.4 并行)

  **References**:
  - `src/interfaces/cli/app.rs:1075-1076` - 当前硬编码位置
  - `src/config.rs` - 配置管理

  **Acceptance Criteria**:
  - [ ] JWT 密钥从环境变量加载
  - [ ] 生成随机密钥作为后备
  - [ ] 无硬编码密钥
  - [ ] `cargo test` 通过

  **Manual Execution Verification**:
  - [ ] 运行命令: `export WORKFLOW_TOOLKIT_JWT_SECRET="test-secret"`
  - [ ] 运行命令: `cargo run -- server --auth`
  - [ ] 预期: 服务器启动正常，使用环境变量密钥
  - [ ] 运行命令: `unset WORKFLOW_TOOLKIT_JWT_SECRET && cargo run -- server --auth`
  - [ ] 预期: 服务器启动正常，生成随机密钥

  **Commit**: YES
  - Message: `fix: remove hardcoded JWT secret, use environment variable`
  - Files: `src/interfaces/cli/app.rs`
  - Pre-commit: `cargo test`

  **Status**: ✅ 已完成

---

- [ ] **任务 1.4：修复安全问题 - CORS 配置**

  **What to do**:
  - 从环境变量 `WORKFLOW_TOOLKIT_CORS_ORIGINS` 读取允许的来源
  - 如果未设置，默认使用 `localhost` 和 `127.0.0.1`
  - 验证来源格式
  - 添加日志记录

  **Must NOT do**:
  - 不要使用通配符 `*`
  - 不要允许所有来源

  **Parallelizable**: YES (与任务 1.2, 1.3 并行)

  **References**:
  - `src/interfaces/cli/app.rs:1081, 1090` - 当前 CORS 配置位置
  - `src/config.rs` - 配置管理

  **Acceptance Criteria**:
  - [ ] CORS 配置限制为可信来源
  - [ ] 无通配符 CORS 配置
  - [ ] 默认值安全
  - [ ] `cargo test` 通过

  **Manual Execution Verification**:
  - [ ] 运行命令: `export WORKFLOW_TOOLKIT_CORS_ORIGINS="localhost,127.0.0.1"`
  - [ ] 运行命令: `cargo run -- server`
  - [ ] 预期: 服务器启动正常，CORS 配置正确
  - [ ] 运行命令: `curl -H "Origin: http://malicious.com" -I http://localhost:8080`
  - [ ] 预期: CORS 头不包含 malicious.com

  **Commit**: YES
  - Message: `fix: restrict CORS origins, remove wildcard`
  - Files: `src/interfaces/cli/app.rs`
  - Pre-commit: `cargo test`

  **Status**: ✅ 已完成

---

### 阶段 2: 结构优化

- [ ] **任务 2.1：拆分 `src/plugins/file_management/utils.rs`**

  **What to do**:
  - 分析功能模块
  - 创建新模块文件
  - 移动相关代码
  - 更新导入引用
  - 编写单元测试
  - 更新文档

  **Must NOT do**:
  - 不要改变功能逻辑
  - 不要删除公共 API

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/plugins/file_management/utils.rs:1-3604` - 超大文件
  - `src/plugins/file_management/registry.rs` - 现有模块结构

  **Acceptance Criteria**:
  - [ ] 所有功能正常
  - [ ] 单文件行数 < 1,000 行
  - [ ] 测试覆盖率 > 80%
  - [ ] 无编译错误

  **Manual Execution Verification**:
  - [ ] 运行命令: `wc -l src/plugins/file_management/*.rs`
  - [ ] 预期: 所有文件 < 1,000 行
  - [ ] 运行命令: `cargo test --lib file_management`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `refactor: split file_management/utils.rs into modules`
  - Files: `src/plugins/file_management/*.rs`
  - Pre-commit: `cargo test --lib file_management`

---

- [ ] **任务 2.2：拆分 `src/interfaces/tui/widgets/plugin_manager.rs`**

  **What to do**:
  - 分析插件管理器的功能模块
  - 创建新模块文件
  - 移动相关代码
  - 更新 widget trait 实现
  - 编写单元测试
  - 更新文档

  **Must NOT do**:
  - 不要改变 UI 渲染逻辑
  - 不要删除 widget 接口

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/interfaces/tui/widgets/plugin_manager.rs:1-3553` - 超大文件
  - `src/interfaces/tui/widgets/` - 现有 widget 结构

  **Acceptance Criteria**:
  - [ ] 所有功能正常
  - [ ] 单文件行数 < 1,000 行
  - [ ] UI 渲染性能无退化
  - [ ] 测试覆盖率 > 80%

  **Manual Execution Verification**:
  - [ ] 运行命令: `wc -l src/interfaces/tui/widgets/*.rs`
  - [ ] 预期: 所有文件 < 1,000 行
  - [ ] 运行命令: `cargo test --lib tui`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `refactor: split tui/plugin_manager.rs into modules`
  - Files: `src/interfaces/tui/widgets/*.rs`
  - Pre-commit: `cargo test --lib tui`

---

- [ ] **任务 2.3：拆分 `src/interfaces/tui/layout.rs`**

  **What to do**:
  - 分析布局系统的功能模块
  - 创建新模块文件
  - 移动相关代码
  - 更新布局计算逻辑
  - 编写单元测试
  - 更新文档

  **Must NOT do**:
  - 不要改变布局计算结果
  - 不要删除公共布局接口

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/interfaces/tui/layout.rs:1-3275` - 超大文件
  - `src/interfaces/tui/` - 现有 TUI 结构

  **Acceptance Criteria**:
  - [ ] 布局计算正确
  - [ ] 响应式设计正常
  - [ ] 单文件行数 < 1,000 行
  - [ ] 测试覆盖率 > 80%

  **Manual Execution Verification**:
  - [ ] 运行命令: `wc -l src/interfaces/tui/*.rs`
  - [ ] 预期: 所有文件 < 1,000 行
  - [ ] 运行命令: `cargo test --lib tui`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `refactor: split tui/layout.rs into modules`
  - Files: `src/interfaces/tui/layout*.rs`
  - Pre-commit: `cargo test --lib tui`

---

- [ ] **任务 2.4：拆分 `src/workflow/engine.rs`**

  **What to do**:
  - 分析引擎的功能模块
  - 创建新模块文件
  - 移动相关代码
  - 更新引擎 trait 实现
  - 编写单元测试
  - 更新文档

  **Must NOT do**:
  - 不要改变工作流执行逻辑
  - 不要删除公共引擎接口

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/workflow/engine.rs:1-2404` - 超大文件
  - `src/workflow/` - 现有工作流结构

  **Acceptance Criteria**:
  - [ ] 工作流执行正常
  - [ ] 状态管理正确
  - [ ] 错误恢复功能正常
  - [ ] 单文件行数 < 1,000 行
  - [ ] 测试覆盖率 > 80%

  **Manual Execution Verification**:
  - [ ] 运行命令: `wc -l src/workflow/*.rs`
  - [ ] 预期: 所有文件 < 1,000 行
  - [ ] 运行命令: `cargo test --lib workflow`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `refactor: split workflow/engine.rs into modules`
  - Files: `src/workflow/engine*.rs`
  - Pre-commit: `cargo test --lib workflow`

---

### 阶段 3: 性能优化

- [ ] **任务 3.1：优化异步代码性能**

  **What to do**:
  - 分析热点异步函数
  - 使用 `tokio::select!` 优化并发任务
  - 使用 `tokio::spawn` 处理独立任务
  - 优化任务调度策略
  - 添加性能基准测试

  **Must NOT do**:
  - 不要改变异步语义
  - 不要引入竞态条件

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/workflow/executor/` - 执行器代码
  - `src/tools/registry.rs` - 工具注册表
  - `src/performance/` - 性能监控

  **Acceptance Criteria**:
  - [ ] 并发性能提升 > 15%
  - [ ] 无死锁或竞态条件
  - [ ] 基准测试通过
  - [ ] 内存使用优化

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo test --release performance_benchmark`
  - [ ] 预期: 性能指标提升
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `perf: optimize async code and task scheduling`
  - Files: `src/workflow/executor/*.rs, src/tools/registry.rs`
  - Pre-commit: `cargo test --release`

---

- [ ] **任务 3.2：优化缓存策略**

  **What to do**:
  - 分析现有缓存使用模式
  - 优化 Moka 缓存配置
  - 实现多级缓存策略
  - 添加缓存预热机制
  - 监控缓存命中率

  **Must NOT do**:
  - 不要改变缓存语义
  - 不要引入缓存一致性问题

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/performance/cache.rs` - 缓存实现
  - `src/storage/` - 存储层

  **Acceptance Criteria**:
  - [ ] 缓存命中率 > 60%
  - [ ] 减少重复计算 50%+
  - [ ] 无内存泄漏
  - [ ] 基准测试通过

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo test --release cache`
  - [ ] 预期: 缓存命中率提升
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `perf: optimize cache strategy and hit rate`
  - Files: `src/performance/cache.rs, src/storage/*.rs`
  - Pre-commit: `cargo test --release`

---

- [ ] **任务 3.3：减少锁竞争**

  **What to do**:
  - 分析锁使用热点
  - 使用读写锁替代互斥锁
  - 实现无锁数据结构
  - 优化锁粒度
  - 添加锁竞争监控

  **Must NOT do**:
  - 不要改变并发语义
  - 不要引入死锁

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `src/tools/registry.rs` - 工具注册表
  - `src/performance/` - 性能监控

  **Acceptance Criteria**:
  - [ ] 锁竞争减少 30%+
  - [ ] 并发性能提升
  - [ ] 无死锁
  - [ ] 基准测试通过

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo test --release concurrency`
  - [ ] 预期: 并发性能提升
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `perf: reduce lock contention and optimize concurrency`
  - Files: `src/tools/registry.rs, src/performance/*.rs`
  - Pre-commit: `cargo test --release`

---

### 阶段 4: 测试完善

- [ ] **任务 4.1：提升测试覆盖率**

  **What to do**:
  - 分析现有测试覆盖情况
  - 为关键模块添加单元测试
  - 添加集成测试
  - 添加性能基准测试
  - 配置 CI/CD 自动化测试

  **Must NOT do**:
  - 不要删除现有测试
  - 不要降低测试标准

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `tests/` - 现有测试目录
  - `src/*/AGENTS.md` - 模块文档

  **Acceptance Criteria**:
  - [ ] 测试覆盖率 > 85%
  - [ ] 所有新模块有测试
  - [ ] 基准测试通过
  - [ ] CI/CD 自动运行

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo test --coverage`
  - [ ] 预期: 覆盖率 > 85%
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `test: improve test coverage to 85%`
  - Files: `tests/*.rs, src/**/tests/*.rs`
  - Pre-commit: `cargo test`

---

- [ ] **任务 4.2：添加错误场景测试**

  **What to do**:
  - 测试超时场景
  - 测试并发冲突
  - 测试资源耗尽
  - 测试网络异常
  - 测试数据损坏

  **Must NOT do**:
  - 不要测试生产环境
  - 不要破坏现有测试

  **Parallelizable**: NO (依赖任务 4.1)

  **References**:
  - `tests/integration_tests.rs` - 集成测试
  - `src/error.rs` - 错误类型

  **Acceptance Criteria**:
  - [ ] 所有错误场景有测试
  - [ ] 边界条件覆盖完整
  - [ ] 测试通过率 100%

  **Manual Execution Verification**:
  - [ ] 运行命令: `cargo test error`
  - [ ] 预期: 所有错误测试通过
  - [ ] 运行命令: `cargo test`
  - [ ] 预期: 所有测试通过

  **Commit**: YES
  - Message: `test: add error scenario tests`
  - Files: `tests/error_scenarios.rs`
  - Pre-commit: `cargo test`

---

### 阶段 5: 文档完善

- [ ] **任务 5.1：更新 AGENTS.md 文档**

  **What to do**:
  - 更新根 AGENTS.md
  - 更新模块特定 AGENTS.md
  - 添加改进说明
  - 更新代码示例
  - 验证链接有效性

  **Must NOT do**:
  - 不要删除现有文档
  - 不要改变文档结构

  **Parallelizable**: NO (依赖任务 1.1)

  **References**:
  - `AGENTS.md` - 根文档
  - `src/*/AGENTS.md` - 模块文档

  **Acceptance Criteria**:
  - [ ] 文档覆盖率 > 80%
  - [ ] 所有公共 API 有文档
  - [ ] 示例代码可运行
  - [ ] 链接有效

  **Manual Execution Verification**:
  - [ ] 运行命令: `find . -name "AGENTS.md" | wc -l`
  - [ ] 预期: >= 15 个文件
  - [ ] 运行命令: `cargo doc --open`
  - [ ] 预期: 文档生成成功

  **Commit**: YES
  - Message: `docs: update AGENTS.md with improvements`
  - Files: `AGENTS.md, src/*/AGENTS.md`
  - Pre-commit: `cargo doc`

---

- [ ] **任务 5.2：添加安全指南**

  **What to do**:
  - 创建安全配置指南
  - 添加 JWT 密钥管理说明
  - 添加 CORS 配置说明
  - 添加认证配置说明
  - 添加安全最佳实践

  **Must NOT do**:
  - 不要包含敏感信息
  - 不要提供默认密钥

  **Parallelizable**: NO (依赖任务 5.1)

  **References**:
  - `docs/` - 文档目录
  - `src/interfaces/cli/app.rs` - 安全配置位置

  **Acceptance Criteria**:
  - [ ] 安全指南完整
  - [ ] 配置示例可运行
  - [ ] 最佳实践清晰

  **Manual Execution Verification**:
  - [ ] 运行命令: `ls docs/security.md`
  - [ ] 预期: 文件存在
  - [ ] 运行命令: `cat docs/security.md`
  - [ ] 预期: 内容完整

  **Commit**: YES
  - Message: `docs: add security configuration guide`
  - Files: `docs/security.md`
  - Pre-commit: `cargo doc`

---

## 📊 成功标准

### 验证命令
```bash
# 安全验证
grep -r "unwrap()" src/ | wc -l  # 预期: < 100
grep -r "default_secret_key" src/  # 预期: 无结果
grep -r '"*"' src/interfaces/cli/app.rs  # 预期: 无结果

# 代码质量验证
cargo clippy  # 预期: 无警告
cargo fmt -- --check  # 预期: 无格式问题
cargo check  # 预期: 无编译错误

# 测试验证
cargo test  # 预期: 所有测试通过
cargo test --coverage  # 预期: 覆盖率 > 85%

# 性能验证
cargo test --release performance_benchmark  # 预期: 性能提升
```

### 最终检查清单
- [ ] 所有 "Must Have" 功能存在
- [ ] 所有 "Must NOT Have" 功能不存在
- [ ] 所有测试通过
- [ ] 文档完整
- [ ] 安全问题修复

---

## 🚀 开始执行

要开始执行此计划，请运行：

```bash
/start-work
```

这将：
1. 注册此计划为当前工作
2. 跟踪进度 across sessions
3. 支持中断后自动继续

---

**计划创建时间**: 2026-01-25  
**计划版本**: v1.0  
**计划状态**: 待执行  
**下一步**: 运行 `/start-work` 开始执行
