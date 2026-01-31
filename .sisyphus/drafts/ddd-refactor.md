# DDD 架构重构计划草案

## 项目概述
- **代码规模**: 163个Rust文件，约110,000+行代码
- **当前结构**: 混合架构，缺乏清晰的分层
- **目标**: 按照DDD（领域驱动设计）进行重构

## DDD分层目标

### 1. 适配层 (Adapter/Presentation Layer)
**职责**: 用户接口、输入输出适配
- **CLI**: 命令行界面 (`src/interfaces/cli/`)
- **TUI**: 终端用户界面 (`src/interfaces/tui/`)
- **MCP**: 模型上下文协议接口 (`src/interfaces/mcp*.rs`)

### 2. 应用层 (Application Layer)
**职责**: 用例编排、工作流协调
- **Workflow**: 工作流执行引擎 (`src/workflow/`)
- **Use Cases**: 具体的业务用例

### 3. 领域层 (Domain Layer)
**职责**: 核心业务逻辑、领域模型
- **Tools**: 工具系统 (`src/tools/`)
- **Core**: 核心领域类型 (`src/core/`)
- **Plugins**: 插件抽象接口

### 4. 基础设施层 (Infrastructure Layer)
**职责**: 技术实现、外部依赖
- **Storage**: 存储实现 (`src/storage/`)
- **Performance**: 性能优化实现
- **Plugin Implementations**: 具体插件实现 (Python, Node.js, Docker等)

## 当前架构问题识别

### 问题1: 领域逻辑泄露到接口层
- TUI组件中包含业务逻辑 (`src/interfaces/tui/widgets/`)
- CLI代码直接处理工作流执行细节

### 问题2: 基础设施与领域层混合
- `src/plugins/file_management/` 混合了领域逻辑和文件系统实现
- 存储层代码直接暴露实现细节

### 问题3: 缺乏清晰的边界
- `src/core/` 同时包含领域类型和基础设施配置
- 模块间依赖关系复杂且循环

### 问题4: 重复代码
- `src/plugins/file_management/ac_automaton.rs` 与 `src/tools/algo/ac_automaton.rs` 疑似重复
- `src/workflow/engine.rs` 和 `src/workflow/engine_legacy.rs` 并存

## 已确定策略

### 重构方式
- **渐进式重构**: 建立新的分层目录结构，逐步迁移代码，降低风险
- **激进代码清理**: 全面审计，删除重复、未使用和过时代码

### 依赖管理
- **依赖注入框架**: 使用 shaku 或类似 Rust DI 容器管理依赖
- **特质对象**: 层间通过 trait 接口交互，实现依赖倒置

### 测试策略
- **契约测试**: 定义层间接口契约，确保层间交互正确
- **分层测试**: 每层的单元测试 + 跨层集成测试

## 目标分层结构

```
src/
├── adapter/                    # 适配层 (原 interfaces)
│   ├── cli/                    # CLI接口
│   ├── tui/                    # TUI接口
│   ├── mcp/                    # MCP接口
│   └── dto/                    # 数据传输对象
│
├── application/                # 应用层
│   ├── workflow/               # 工作流编排
│   │   ├── orchestrator.rs     # 工作流编排器
│   │   ├── executor.rs         # 执行器协调
│   │   └── port/               # 输出端口（领域层接口）
│   ├── usecase/                # 具体用例
│   └── service/                # 应用服务
│
├── domain/                     # 领域层
│   ├── model/                  # 领域模型
│   │   ├── workflow.rs         # 工作流聚合根
│   │   ├── tool.rs             # 工具实体
│   │   ├── plugin.rs           # 插件实体
│   │   └── value_object.rs     # 值对象
│   ├── repository/             # 仓储接口（端口）
│   ├── service/                # 领域服务
│   └── event/                  # 领域事件
│
├── infrastructure/             # 基础设施层
│   ├── persistence/            # 持久化实现
│   │   ├── storage/            # 存储实现
│   │   └── cache/              # 缓存实现
│   ├── plugin/                 # 插件实现
│   │   ├── native.rs
│   │   ├── python.rs
│   │   ├── nodejs.rs
│   │   └── docker.rs
│   ├── external/               # 外部服务
│   └── config/                 # 配置管理
│
└── di/                         # 依赖注入容器
    ├── container.rs
    └── module.rs
```

## 代码清理清单

### 重复代码
- [x] `src/plugins/file_management/ac_automaton.rs` ↔ `src/tools/algo/ac_automaton.rs`
- [x] `src/workflow/engine.rs` ↔ `src/workflow/engine_legacy.rs`
- [ ] 检查其他重复模式

### 待删除文件
- [ ] `src/workflow/engine_legacy.rs` (被 engine.rs 替代)
- [ ] `src/plugins/file_management/ac_automaton.rs` (使用 tools 版本)
- [ ] `src/interfaces/mcp_server.rs` (stub实现)

### 待合并模块
- [ ] 性能相关代码 (`src/performance/`) 合并到基础设施层
- [ ] 存储代码 (`src/storage/`) 作为基础设施层持久化实现

## 研究任务

- [x] 分析模块依赖图
- [ ] 识别循环依赖 (进行中)
- [x] 查找重复代码
- [ ] 评估测试覆盖率
- [ ] 设计契约测试
