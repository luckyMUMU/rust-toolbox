# Workflow Toolkit - 项目概览

## 项目简介

**Workflow Toolkit** 是一个基于Rust开发的多接口工作流执行系统，提供灵活的工作流定义、执行和管理能力。

### 核心特性

✅ **多接口支持**
- CLI (命令行接口) - 适合脚本和自动化
- TUI (终端用户界面) - 交互式操作
- MCP服务器 - IDE和外部工具集成

✅ **DAG工作流引擎**
- 有向无环图执行模型
- 并行执行和条件分支
- 检查点和恢复机制

✅ **多语言插件系统**
- Native (Rust动态库)
- Python (ML/数据处理)
- Node.js (JavaScript工具)
- Docker (隔离环境)
- WASM (便携代码)

✅ **高性能存储**
- LanceDB向量数据库
- Moka高性能缓存
- 自动备份和恢复

✅ **全面的监控**
- 实时系统监控
- 性能分析和优化
- 审计日志记录

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    用户接口层                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │     CLI     │  │     TUI     │  │    MCP Server       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   应用逻辑层                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Workflow   │  │    Tool     │  │   File Management  │  │
│  │   Engine    │  │  Registry   │  │      Tools          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    插件系统层                               │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────┐  │
│  │ Native  │ │ Python  │ │ Node.js │ │ Docker  │ │ WASM  │  │
│  │ Plugins │ │ Plugins │ │ Plugins │ │ Plugins │ │Plugins│  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └───────┘  │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                   基础设施层                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Storage   │  │    Cache    │  │    Performance      │  │
│  │  (LanceDB)  │  │   (Moka)    │  │     Monitoring      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 技术栈

| 组件 | 技术 | 版本 | 用途 |
|------|------|------|------|
| **语言** | Rust | 1.70+ | 核心开发 |
| **异步运行时** | tokio | 1.42 | 异步执行 |
| **CLI框架** | clap | 4.5 | 命令行接口 |
| **TUI框架** | ratatui | 0.29 | 终端界面 |
| **序列化** | serde | 1.0 | 数据格式化 |
| **错误处理** | thiserror | 2.0 | 结构化错误 |
| **DAG处理** | petgraph | 0.6 | 图算法 |
| **并发集合** | dashmap | 6.1 | 线程安全Map |
| **缓存** | moka | 0.12 | 高性能缓存 |
| **存储** | lancedb | 0.20 | 向量数据库 |
| **日志** | tracing | 0.1 | 结构化日志 |
| **插件** | libloading | 0.8 | 动态库加载 |

## 核心组件

### 1. 工作流引擎 (Workflow Engine)

**功能**: DAG执行、调度、状态管理

**关键类型**:
- `WorkflowDefinition`: 工作流定义
- `DagScheduler`: DAG调度器
- `DefaultWorkflowEngine`: 执行引擎
- `ExecutionManager`: 执行管理

**特点**:
- 拓扑排序确定执行顺序
- 信号量控制并发
- 检查点每5分钟自动保存
- 支持暂停/恢复/停止

### 2. 工具系统 (Tool System)

**功能**: 工具注册、执行、验证

**关键类型**:
- `ToolNode`: 工具节点trait
- `ToolRegistry`: 工具注册表
- `BasicTool`: 标准工具实现
- `BasicToolRegistry`: DashMap注册表

**特点**:
- JSON Schema参数验证
- 异步执行
- 版本管理
- 依赖解析

### 3. 插件系统 (Plugin System)

**功能**: 多语言插件支持

**关键类型**:
- `Plugin`: 插件trait
- `PluginManager`: 插件管理器
- `RuntimeManager`: 运行时管理
- `IntegratedPluginSystem`: 集成系统

**支持类型**:
- **Native**: Rust动态库 (libloading)
- **Python**: Python脚本 (subprocess)
- **Node.js**: JavaScript模块 (subprocess)
- **Docker**: 容器化 (bollard)
- **WASM**: WebAssembly (wasmtime/extism)

### 4. 存储层 (Storage Layer)

**功能**: 持久化、缓存、备份

**关键类型**:
- `StorageBackend`: 存储后端trait
- `StateManager`: 状态管理器
- `FileStorage`: 文件存储
- `SimpleMemoryCache`: 内存缓存
- `BackupManager`: 备份管理

**特点**:
- LanceDB向量存储
- Moka LRU缓存
- 自动备份
- TTL支持

### 5. 用户接口 (Interfaces)

#### CLI (Command-Line Interface)
```bash
workflow-toolkit workflow execute file.yaml
workflow-toolkit tool list
workflow-toolkit plugin install plugin.so
```

#### TUI (Terminal User Interface)
```bash
workflow-toolkit tui
# 交互式界面，支持键盘导航
```

#### MCP (Model Context Protocol)
```bash
workflow-toolkit server --http-port 8080
# 提供JSON-RPC接口
```

## 数据流

### 工作流执行流程

```
用户输入 (CLI/TUI/MCP)
    ↓
配置加载 (ConfigManager)
    ↓
工作流定义 (YAML/JSON)
    ↓
验证 (WorkflowValidator)
    - DAG结构检查
    - 工具存在性验证
    - 依赖关系验证
    ↓
调度 (DagScheduler)
    - 拓扑排序
    - 并行节点识别
    - 执行顺序确定
    ↓
执行 (DefaultWorkflowEngine)
    - 信号量控制
    - 节点执行
    - 错误处理
    - 检查点保存
    ↓
状态管理 (StateManager)
    - 持久化状态
    - 缓存结果
    ↓
审计日志 (AuditLogger)
    - 记录操作
    - 错误详情
    ↓
输出结果
```

### 插件加载流程

```
配置发现
    ↓
PluginManager.load_plugin()
    ↓
RuntimeManager.create_runtime()
    ↓
类型判断
    ├─ Native → libloading (.so/.dll)
    ├─ Python → subprocess (python3)
    ├─ Node.js → subprocess (node)
    ├─ Docker → bollard (docker)
    └─ WASM → wasmtime (wasm)
    ↓
符号/接口验证
    ↓
注册到ToolRegistry
    ↓
工作流可用
```

## 项目统计

### 代码统计
- **总文件数**: ~150 Rust文件
- **代码行数**: ~109,000行
- **测试文件**: 15个
- **测试数量**: 298个可用测试

### 文档统计
- **设计文档**: 6个 (约1,400行)
- **AGENTS.md**: 17个 (约1,800行)
- **用户指南**: 2个 (约800行)
- **总计**: ~4,000行文档

### 功能统计
- **工作流节点类型**: 5种 (Tool, Condition, Loop, Parallel, Checkpoint)
- **插件类型**: 5种 (Native, Python, Node.js, Docker, WASM)
- **存储后端**: 2种 (LanceDB, FileStorage)
- **用户接口**: 3种 (CLI, TUI, MCP)

## 使用场景

### 场景1: 数据处理管道
```yaml
# 从CSV到报告的完整流程
nodes:
  - load → validate → transform → analyze → report
```
**用途**: ETL处理、数据分析

### 场景2: 文件整理
```yaml
# 自动整理下载文件夹
nodes:
  - scan → classify → organize → notify
```
**用途**: 文件管理、自动化整理

### 场景3: API同步
```yaml
# 从API同步数据
nodes:
  - fetch → parse → store → backup → notify
```
**用途**: 数据同步、备份

### 场景4: ML管道
```yaml
# 机器学习流程
nodes:
  - preprocess → train → evaluate → deploy → monitor
```
**用途**: AI/ML工作流

### 场景5: CI/CD自动化
```yaml
# 自动化部署
nodes:
  - build → test → package → deploy → verify
```
**用途**: DevOps自动化

## 性能指标

### 构建性能
- **Debug check**: ~0.56s
- **Test compile**: ~0.67s
- **Full build**: ~17s (storage tests)

### 运行时性能
- **工具执行**: 微秒级
- **工作流调度**: 毫秒级
- **并发支持**: 数百个工作流
- **内存使用**: 可配置限制

### 测试覆盖
- **核心模块**: 100%测试覆盖
- **集成测试**: 端到端验证
- **属性测试**: 随机化验证

## 文档结构

```
rust-tool-v2/
├── README.md                    # 项目概述
├── PROJECT_OVERVIEW.md          # 本文件 - 项目概览
├── DEVELOPMENT_GUIDE.md         # 开发指南
├── USER_GUIDE.md                # 用户指南
├── DESIGN.md                    # 主设计文档
├── IMPLEMENTATION_VERIFICATION.md # 验证报告
│
├── AGENTS.md                    # 根级开发指南
├── docs/AGENTS.md               # 文档模块指南
├── src/AGENTS.md                # 核心库指南
├── src/workflow/AGENTS.md       # 工作流指南
├── src/plugins/AGENTS.md        # 插件指南
├── src/tools/AGENTS.md          # 工具指南
├── src/storage/AGENTS.md        # 存储指南
├── src/performance/AGENTS.md    # 性能指南
├── src/interfaces/AGENTS.md     # 接口指南
├── src/interfaces/cli/AGENTS.md # CLI指南
├── src/interfaces/tui/AGENTS.md # TUI指南
├── src/interfaces/tui/widgets/AGENTS.md # 组件指南
├── tests/AGENTS.md              # 测试指南
├── examples/AGENTS.md           # 示例指南
├── examples/templates/AGENTS.md # 模板指南
├── openspec/AGENTS.md           # 规范指南
│
├── src/workflow/DESIGN.md       # 工作流设计
├── src/plugins/DESIGN.md        # 插件设计
├── src/tools/DESIGN.md          # 工具设计
├── src/storage/DESIGN.md        # 存储设计
├── src/interfaces/cli/DESIGN.md # CLI设计
│
├── examples/                    # 示例代码
│   ├── comprehensive_workflow_example.rs
│   ├── python_plugin_example.rs
│   ├── file_management_example.rs
│   └── templates/
│
└── config/default.toml          # 默认配置
```

## 快速参考

### 常用命令

```bash
# 构建和测试
cargo check                    # 快速类型检查
cargo build                    # 调试构建
cargo test                     # 运行测试

# 工作流管理
workflow-toolkit workflow execute file.yaml
workflow-toolkit workflow list
workflow-toolkit workflow status <id>

# 工具管理
workflow-toolkit tool list
workflow-toolkit tool execute echo --params '{"message": "Hi"}'

# 插件管理
workflow-toolkit plugin list
workflow-toolkit plugin install plugin.so

# TUI界面
workflow-toolkit tui

# MCP服务器
workflow-toolkit server --http-port 8080

# 系统维护
workflow-toolkit system monitor --watch
workflow-toolkit system health
```

### 配置位置

```bash
# 系统级
/etc/workflow-toolkit/config.toml

# 用户级
~/.config/workflow-toolkit/config.toml

# 项目级
./workflow-toolkit.toml

# 环境变量
WORKFLOW_TOOLKIT_LOGGING__LEVEL=debug
```

### 日志和调试

```bash
# 启用详细日志
RUST_LOG=debug workflow-toolkit workflow execute file.yaml

# 查看日志
tail -f logs/workflow-toolkit.log

# 性能分析
export WORKFLOW_TOOLKIT_PERFORMANCE__ENABLE_PROFILING=true
```

## 学习路径

### 初学者
1. 阅读 **USER_GUIDE.md** 了解基本使用
2. 查看 **examples/** 中的示例
3. 尝试运行简单工作流
4. 使用 TUI 界面熟悉功能

### 开发者
1. 阅读 **DEVELOPMENT_GUIDE.md**
2. 理解 **DESIGN.md** 中的架构
3. 查看 **AGENTS.md** 文件
4. 运行测试了解实现
5. 尝试添加新工具或插件

### 高级用户
1. 深入理解工作流引擎
2. 自定义插件开发
3. 性能调优
4. 分布式部署

## 贡献指南

### 报告问题
1. 检查现有 issues
2. 提供最小复现示例
3. 包含日志和配置
4. 描述预期行为

### 提交代码
1. Fork 仓库
2. 创建特性分支
3. 编写测试
4. 运行 `cargo fmt && cargo clippy`
5. 提交 Pull Request

### 文档贡献
1. 更新相关 AGENTS.md
2. 添加使用示例
3. 保持一致性
4. 检查拼写和语法

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。

## 联系和支持

- **文档**: 查看 `docs/` 目录
- **示例**: 查看 `examples/` 目录
- **测试**: 运行 `cargo test`
- **社区**: 参与 GitHub Discussions

---

## 总结

Workflow Toolkit 是一个功能完整、架构清晰、文档详尽的工作流执行系统。它提供了：

✅ **完整的功能集**: 从工作流定义到执行监控  
✅ **灵活的扩展性**: 插件系统支持多语言  
✅ **优秀的性能**: 异步执行、并发控制、缓存优化  
✅ **全面的文档**: 设计文档、开发指南、用户指南  
✅ **丰富的测试**: 单元测试、集成测试、属性测试  

**开始使用**:
```bash
# 1. 构建项目
cargo build --release

# 2. 查看帮助
./target/release/workflow-toolkit --help

# 3. 运行示例
./target/release/workflow-toolkit workflow execute examples/hello-world.yaml

# 4. 启动TUI
./target/release/workflow-toolkit tui
```

**深入学习**:
- 开发者: `DEVELOPMENT_GUIDE.md` + `AGENTS.md` 文件
- 用户: `USER_GUIDE.md`
- 架构: `DESIGN.md` + `PROJECT_OVERVIEW.md`
