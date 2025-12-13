# Rust 工具箱项目规范 (Project Rules)

最后更新：2025-12-13
版本控制：Git

## 1. 项目框架 (Project Framework)

### 1.1 核心框架与版本
| 框架/库 | 版本 | 用途 |
| :--- | :--- | :--- |
| **Rust** | 1.70+ | 编程语言 |
| **Tokio** | 1.0+ | 异步运行时 (Async Runtime) |
| **Clap** | 4.0+ | 命令行参数解析 (CLI) |
| **Serde** | 1.0+ | 序列化/反序列化 |
| **Schemars** | 0.8+ | JSON Schema 生成 |
| **Eframe/Egui** | 0.29+ | 即时模式 GUI 框架 |
| **Tracing** | 0.1+ | 日志与追踪 |

### 1.2 核心依赖关系
- **rt-core**: 基础库，无上层依赖。
- **rt-tools**: 依赖 `rt-core`，实现具体工具逻辑。
- **rt-cli**: 依赖 `rt-core`, `rt-tools`，提供命令行入口。
- **rt-gui**: 依赖 `rt-core`, `rt-tools`，提供图形界面入口。

### 1.3 升级与兼容性策略
- **语义化版本**: 严格遵循 Semantic Versioning 2.0.0。
- **锁定版本**: 生产环境依赖 `Cargo.lock` 锁定版本。
- **升级流程**: 修改 `Cargo.toml` -> `cargo update` -> 全量测试 (`cargo test`) -> 提交。

## 2. 测试框架要求 (Test Framework Requirements)

### 2.1 覆盖率标准
- **单元测试 (Unit Tests)**: 核心业务逻辑 (`rt-core`, `rt-tools`) 覆盖率需达到 **80%** 以上。
- **集成测试 (Integration Tests)**: 关键 CLI 命令和完整工具流程必须有集成测试覆盖。

### 2.2 编写规范
- **位置**: 单元测试置于源码文件底部 `#[cfg(test)] mod tests { ... }` 中。
- **断言**: 使用 `assert!`, `assert_eq!`, `assert_ne!` 等标准宏。
- **验收标准**: 所有测试用例必须通过 (`cargo test` 返回 0)。

### 2.3 测试环境与数据
- **环境**: 测试必须支持在本地开发环境独立运行，不依赖外部不可控服务。
- **数据**: 测试数据应包含在源码库中 (`tests/data/`) 或在测试代码中动态生成，禁止使用硬编码的绝对路径。

### 2.4 测试报告
- **输出格式**: 标准 Cargo Test 输出。
- **CI 集成**: (未来规划) 在 CI 环境中生成 JUnit 格式报告。

## 3. API 使用限制 (API Usage Limits)

### 3.1 禁止使用的 API
| 禁止 API | 替代方案 | 原因 |
| :--- | :--- | :--- |
| `unwrap()`, `expect()` | `Result<T, E>` + `?` 操作符 | 防止生产环境 Panic，强制错误处理 |
| `std::fs` (在 async fn 中) | `tokio::fs` | 防止阻塞异步运行时线程 |
| `std::thread::sleep` | `tokio::time::sleep` | 防止阻塞异步运行时线程 |
| `println!` (用于日志) | `tracing::info!` 等 | 统一日志管理和格式 |
| `unsafe` 块 | Safe Rust 封装 | 避免内存安全问题，除非绝对必要且经严格审查 |

### 3.2 审计与违规处理
- **审计机制**:
  - **Clippy**: CI/Commit Hook 强制运行 `cargo clippy -- -D warnings`。
  - **Code Review**: 人工审查重点关注 `unwrap` 和 `unsafe` 使用。
- **违规处理**:
  - 发现违规代码必须立即修复，否则禁止合并代码。
  - 紧急修复 (Hotfix) 允许临时豁免，但必须添加 `// TODO: FIX` 注释并创建后续修复任务。
