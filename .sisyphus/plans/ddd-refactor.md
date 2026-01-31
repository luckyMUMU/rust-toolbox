# DDD架构重构计划

## TL;DR

> **目标**: 按照DDD分层架构重构110,000+行Rust代码库，建立清晰的适配层、应用层、领域层和基础设施层，同时精简重复和遗留代码。
> 
> **策略**: 渐进式重构 + 激进代码清理 + 依赖注入框架(shaku) + 契约测试
> 
> **预估任务数**: 15个主要任务
> 
> **并行执行**: 3个波次
> **关键路径**: 建立新目录结构 → 迁移领域层 → 重构工作流引擎 → 适配层迁移 → 集成测试

---

## 上下文

### 原始需求
用户希望按照DDD设计重构当前代码库：
- 工作流位于应用层，负责业务逻辑编排
- 工具实现等位于领域层
- Plugin及外部依赖等位于基础设施层
- CLI、TUI、MCP等作为适配层
- 尽可能精简代码，不影响功能

### 访谈总结
**关键决策**:
- 重构方式: 渐进式重构（风险较低，逐步迁移）
- 代码清理: 激进策略（全面审计，删除重复/未使用/过时代码）
- 依赖管理: 依赖注入框架（shaku）管理依赖生命周期
- 测试策略: 契约测试（定义层间接口契约）

### 研究发现
**当前架构问题**:
1. **层间耦合**: `RefactoredWorkflowEngine` 直接依赖 `StateManager`（基础设施）
2. **职责混合**: `core` 模块混合了领域类型和配置/监控类型
3. **重复代码**: 
   - `engine.rs` 和 `engine_legacy.rs` 重复
   - `ac_automaton.rs` 在两个目录都有
4. **依赖方向混乱**: 接口层直接访问基础设施层

---

## 工作范围

### 包含 (IN SCOPE)
1. 建立新的DDD分层目录结构
2. 迁移和重构领域层（domain/）
3. 重构应用层（application/）- 工作流编排
4. 适配层重构（adapter/）- CLI、TUI、MCP
5. 基础设施层重构（infrastructure/）- 存储、插件、外部服务
6. 依赖注入容器实现（di/）
7. 删除重复和遗留代码
8. 契约测试实现

### 明确排除 (OUT OF SCOPE / Guardrails)
1. **不改变业务逻辑**: 仅重构架构，不修改功能行为
2. **不改外部API**: CLI/TUI/MCP接口保持兼容
3. **不新增功能**: 不添加DDD之外的特性
4. **保留工作流格式**: workflow定义文件格式不变
5. **逐步迁移**: 不一次性删除旧代码，直到新代码验证通过

### 代码精简目标
**删除清单**:
- `src/workflow/engine_legacy.rs` → 使用 `engine.rs`
- `src/plugins/file_management/ac_automaton.rs` → 使用 `src/tools/algo/ac_automaton.rs`
- 检查并删除未使用的配置类型（如AuthConfig如果未被使用）
- 合并重复的error类型定义

---

## 目标架构

```
src/
├── adapter/                    # 适配层 (原 interfaces)
│   ├── cli/                    # CLI接口适配器
│   ├── tui/                    # TUI接口适配器
│   ├── mcp/                    # MCP接口适配器
│   └── dto/                    # 数据传输对象
│
├── application/                # 应用层
│   ├── port/                   # 领域层接口（输出端口）
│   │   ├── repository.rs       # 仓储接口
│   │   ├── tool_registry.rs    # 工具注册表接口
│   │   └── plugin_manager.rs   # 插件管理器接口
│   ├── workflow/               # 工作流编排
│   │   ├── orchestrator.rs     # 工作流编排器
│   │   ├── executor.rs         # 执行器协调
│   │   └── command/            # 工作流命令
│   ├── usecase/                # 具体用例
│   │   ├── execute_workflow.rs
│   │   ├── manage_plugins.rs
│   │   └── system_monitoring.rs
│   └── service/                # 应用服务
│       └── workflow_service.rs
│
├── domain/                     # 领域层
│   ├── model/                  # 领域模型
│   │   ├── workflow.rs         # 工作流聚合根
│   │   ├── tool.rs             # 工具实体和值对象
│   │   ├── plugin.rs           # 插件实体
│   │   ├── execution.rs        # 执行上下文和状态
│   │   └── value_object.rs     # 共享值对象
│   ├── repository/             # 仓储接口（端口）
│   │   ├── workflow_repo.rs
│   │   ├── execution_repo.rs
│   │   └── plugin_repo.rs
│   ├── service/                # 领域服务
│   │   └── workflow_validator.rs
│   └── event/                  # 领域事件
│       └── workflow_events.rs
│
├── infrastructure/             # 基础设施层
│   ├── persistence/            # 持久化实现
│   │   ├── storage/            # 存储实现（原 src/storage/）
│   │   │   ├── file_storage.rs
│   │   │   ├── memory_cache.rs
│   │   │   └── state_manager.rs
│   │   └── repository/         # 仓储实现
│   │       ├── workflow_repo_impl.rs
│   │       └── execution_repo_impl.rs
│   ├── plugin/                 # 插件实现（原 src/plugins/）
│   │   ├── native.rs
│   │   ├── python.rs
│   │   ├── nodejs.rs
│   │   ├── docker.rs
│   │   ├── registry_impl.rs    # PluginManager实现
│   │   └── file_management/    # 文件管理插件
│   ├── external/               # 外部服务
│   │   ├── http_client.rs
│   │   └── docker_client.rs
│   ├── cache/                  # 缓存实现（原 performance/cache.rs）
│   │   └── moka_cache.rs
│   └── config/                 # 配置管理
│       └── app_config.rs
│
└── di/                         # 依赖注入容器
    ├── container.rs            # DI容器实现（使用shaku）
    ├── module.rs               # 模块定义
    └── provider.rs             # 依赖提供者
```

---

## 依赖规则 (Dependency Rules)

```
adapter → application → domain ← infrastructure
```

**依赖方向**:
1. **适配层** 依赖 **应用层** 的接口
2. **应用层** 依赖 **领域层** 的模型和端口
3. **基础设施层** 依赖 **领域层** 的接口（实现它们）
4. **禁止循环依赖**: 任何层都不能反向依赖

**依赖注入策略**:
- 使用 `shaku` 框架管理依赖生命周期
- 领域层定义 trait 接口（端口）
- 基础设施层提供具体实现
- 在 `di/` 模块组装所有依赖

---

## 执行策略

### 波次1: 基础结构 (并行执行) ✅
- [x] 1.1 建立新的目录结构
- [x] 1.2 设置依赖注入框架 (shaku)
- [x] 1.3 定义领域层端口（接口）

### 波次2: 领域层迁移 (依赖波次1) ✅
- [x] 2.1 迁移领域模型（从core/）
- [x] 2.2 重构工具系统到领域层
- [x] 2.3 重构插件抽象接口

### 波次3: 基础设施层实现 (依赖波次2) ✅
- [x] 3.1 实现仓储层（原storage/）
- [x] 3.2 重构插件实现（原plugins/）
- [x] 3.3 重构缓存和性能优化

### 波次4: 应用层重构 (依赖波次3) ✅
- [x] 4.1 重构工作流编排器
- [x] 4.2 实现用例层
- [x] 4.3 重构工作流执行器

### 波次5: 适配层迁移 (依赖波次4) ✅
- [x] 5.1 重构CLI适配器
- [x] 5.2 重构TUI适配器
- [x] 5.3 重构MCP适配器

### 波次6: 清理与测试 ✅
- [x] 6.1 删除遗留代码
- [x] 6.2 契约测试实现
- [x] 6.3 集成测试与验证

### 波次7: 编译修复 ✅
- [x] 7.1 修复execution_manager兼容性
- [x] 7.2 添加缺失的引擎方法
- [x] 7.3 确保cargo check通过

---

## TODOs

### 波次1: 基础结构

- [x] 1.1 建立DDD分层目录结构 ✅

  **What to do**:
  - 创建新的目录结构: adapter/, application/, domain/, infrastructure/, di/
  - 创建每个层的mod.rs文件
  - 在Cargo.toml添加shaku依赖
  - 保持现有src/结构不动，建立新的模块树

  **Status**: ✅ 已完成
  - 提交: `d66d4a9 feat(ddd): establish DDD layer directory structure`

  **Acceptance Criteria**:
  - [x] 新目录结构已创建
  - [x] `cargo check` 能通过（新模块为空但编译通过）
  - [x] shaku 已添加到 Cargo.toml

---

- [x] 1.2 ✅ 设置依赖注入框架 (shaku)

  **What to do**:
  - 在Cargo.toml添加 `shaku = "0.6"`
  - 创建 `src/di/container.rs` - DI容器主实现
  - 创建 `src/di/module.rs` - 模块定义
  - 创建 `src/di/provider.rs` - 依赖提供者 trait
  - 实现基本的Provider trait，支持单例和瞬态生命周期

  **Must NOT do**:
  - 不要集成所有依赖（这是基础框架）
  - 不要删除现有的手动依赖传递

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉依赖注入概念，了解Rust trait系统
  - **Reason**: 需要理解shaku的工作原理并实现适配层

  **Parallelization**:
  - **Can Run In Parallel**: YES (与1.1)
  - **Parallel Group**: Wave 1
  - **Blocks**: 2.3, 3.x, 4.x, 5.x
  - **Blocked By**: None

  **References**:
  - Shaku docs: https://docs.rs/shaku/latest/shaku/
  - Example pattern: `src/plugins/manager.rs:42` - 当前手动依赖管理

  **Acceptance Criteria**:
  - [x] shaku 依赖已添加
  - [x] DI容器基本结构实现
  - [x] 示例：能用DI容器创建一个简单的服务
  - [x] `cargo test di::` 通过基础测试

  **Commit**: YES (groups with 1.1)
  - Message: `feat(di): setup dependency injection container with shaku`

---

- [x] 1.3 ✅ 定义领域层端口（接口）

  **What to do**:
  - 分析现有trait定义，提取领域层端口
  - 创建 `src/domain/port/tool_registry.rs` - 提取ToolRegistry trait
  - 创建 `src/domain/port/plugin_manager.rs` - 定义PluginManager接口
  - 创建 `src/domain/port/repository.rs` - 定义仓储接口
  - 确保这些trait不依赖任何基础设施类型

  **Must NOT do**:
  - 不要包含基础设施实现细节（如StateManager）
  - 不要依赖外部crate（如tokio可以在trait中使用，但moka不行）

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉trait设计，了解端口适配器模式
  - **Reason**: 需要仔细设计接口边界

  **Parallelization**:
  - **Can Run In Parallel**: YES (与1.1, 1.2)
  - **Parallel Group**: Wave 1
  - **Blocks**: 2.2, 2.3, 3.x, 4.x
  - **Blocked By**: None

  **References**:
  - `src/tools/registry.rs:18-71` - ToolRegistry trait（要迁移）
  - `src/plugins/manager.rs:42-73` - PluginManager结构（提取接口）
  - `src/storage/state_manager.rs` - StateManager（不能出现在领域层端口）

  **Acceptance Criteria**:
  - [x] 领域端口trait已定义
  - [x] 不包含任何基础设施依赖
  - [x] 使用async-trait支持异步方法
  - [x] 单元测试验证接口编译通过

  **Commit**: YES (groups with 1.x)
  - Message: `feat(domain): define domain layer ports and interfaces`

---

### 波次2: 领域层迁移

- [x] 2.1 ✅ 迁移领域模型（从core/）

  **What to do**:
  - 分析 `src/core/mod.rs` 中的所有类型
  - 将领域相关的类型迁移到 `src/domain/model/`
    - `ToolInfo`, `PluginInfo` → `src/domain/model/tool.rs`, `plugin.rs`
    - `ExecutionContext`, `ExecutionStatus` → `src/domain/model/execution.rs`
    - `WorkflowId` 等值对象 → `src/domain/model/value_object.rs`
  - 将基础设施配置类型移动到 `src/infrastructure/config/`
    - `AuthConfig`, `RateLimitConfig` → 移出领域层
  - 保持向后兼容：在core/中re-export（暂时）

  **Must NOT do**:
  - 不要删除原文件（只添加新文件）
  - 不要修改类型的内部实现
  - 不要移动序列化/反序列化逻辑

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 领域建模经验，能识别领域vs基础设施关注点
  - **Reason**: 这是关键的领域边界划分

  **Parallelization**:
  - **Can Run In Parallel**: NO (需要完成2.1后才能并行其他领域任务)
  - **Parallel Group**: Wave 2
  - **Blocks**: 2.2, 2.3, 3.x
  - **Blocked By**: 1.x

  **References**:
  - `src/core/mod.rs:55-415` - 所有要分析的类型
  - `src/error.rs` - WorkflowError类型（保持原位置，跨层共享）

  **Acceptance Criteria**:
  - [x] 领域模型类型已迁移到domain/model/
  - [x] 基础设施类型已标记为待迁移
  - [x] `cargo test` 通过（向后兼容re-export工作）
  - [x] 契约测试：验证序列化/反序列化行为不变

  **Commit**: YES
  - Message: `refactor(domain): migrate core domain models from core/ to domain/`

---

- [x] 2.2 ✅ 重构工具系统到领域层

  **What to do**:
  - 迁移 `ToolRegistry` trait 到 `src/domain/port/tool_registry.rs`
  - 迁移 `ToolNode` trait 到 `src/domain/model/tool.rs`
  - 迁移工具相关的值对象（ToolVersion, Dependency等）
  - 保留 `BasicToolRegistry` 实现在基础设施层（后续任务）
  - 更新引用：确保现有代码通过端口使用工具系统

  **Must NOT do**:
  - 不要移动BasicToolRegistry实现（它属于基础设施层）
  - 不要修改ToolNode trait的方法签名

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 熟悉trait提取和重构，了解Rust模块系统
  - **Reason**: 需要保持向后兼容同时重构接口

  **Parallelization**:
  - **Can Run In Parallel**: YES (与2.3)
  - **Parallel Group**: Wave 2
  - **Blocks**: 3.1, 4.1
  - **Blocked By**: 1.3, 2.1

  **References**:
  - `src/tools/registry.rs:18-71` - ToolRegistry trait
  - `src/tools/node.rs:14-60` - ToolNode trait
  - `src/tools/mod.rs:49-64` - 模块re-export结构

  **Acceptance Criteria**:
  - [x] ToolRegistry trait 已定义在domain层
  - [x] ToolNode trait 已定义在domain层
  - [x] 现有工具实现仍然工作（通过适配）
  - [x] 契约测试：验证工具执行行为不变

  **Commit**: YES
  - Message: `refactor(domain): extract tool system traits to domain layer`

---

- [x] 2.3 ✅ 重构插件抽象接口

  **What to do**:
  - 提取 `Plugin` trait 到 `src/domain/model/plugin.rs`
  - 创建 `PluginManager` trait 在 `src/domain/port/plugin_manager.rs`
  - 迁移插件相关的配置和状态类型
  - 确保插件接口不依赖具体运行时（Python/Docker等）

  **Must NOT do**:
  - 不要移动具体插件实现（NativePlugin, PythonPlugin等）
  - 不要修改Plugin trait的生命周期方法

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉插件架构设计
  - **Reason**: 需要抽象出通用的插件管理接口

  **Parallelization**:
  - **Can Run In Parallel**: YES (与2.2)
  - **Parallel Group**: Wave 2
  - **Blocks**: 3.2
  - **Blocked By**: 1.3, 2.1

  **References**:
  - `src/plugins/types.rs` - Plugin trait定义
  - `src/plugins/manager.rs:42-73` - PluginManager结构
  - `src/core/mod.rs:84-103` - PluginInfo, PluginType

  **Acceptance Criteria**:
  - [x] Plugin trait 已定义在domain层
  - [x] PluginManager 端口已定义
  - [x] 插件类型系统已迁移
  - [x] 契约测试：验证插件加载行为不变

  **Commit**: YES (groups with 2.2)
  - Message: `refactor(domain): extract plugin abstractions to domain layer`

---

### 波次3: 基础设施层实现

- [x] 3.1 ✅ 实现仓储层（原storage/）

  **What to do**:
  - 实现 `WorkflowRepository` trait 在 `src/infrastructure/persistence/repository/workflow_repo_impl.rs`
  - 实现 `ExecutionRepository` trait 在 `src/infrastructure/persistence/repository/execution_repo_impl.rs`
  - 迁移现有 `FileStorage`, `SimpleMemoryCache` 等实现
  - 确保实现依赖领域层的端口接口
  - 将 `StateManager` 重构为仓储实现

  **Must NOT do**:
  - 不要修改存储格式（保持向后兼容）
  - 不要在仓储中引入业务逻辑

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 熟悉仓储模式，了解Rust文件操作
  - **Reason**: 需要仔细处理持久化逻辑

  **Parallelization**:
  - **Can Run In Parallel**: YES (与3.2, 3.3)
  - **Parallel Group**: Wave 3
  - **Blocks**: 4.1
  - **Blocked By**: 2.x

  **References**:
  - `src/storage/backends.rs:78-129` - FileStorage实现
  - `src/storage/state_manager.rs` - StateManager实现
  - `src/workflow/state/checkpoint.rs` - 检查点逻辑

  **Acceptance Criteria**:
  - [x] 仓储实现完成
  - [x] 能通过DI容器注入
  - [x] 存储格式与原实现兼容
  - [x] 集成测试：验证存储/读取工作流状态

  **Commit**: YES
  - Message: `refactor(infra): implement repository pattern in infrastructure layer`

---

- [x] 3.2 ✅ 重构插件实现（原plugins/）

  **What to do**:
  - 迁移具体插件实现到 `src/infrastructure/plugin/`
  - 实现 `PluginManager` trait（使用shaku注入）
  - 迁移 `NativePlugin`, `PythonPlugin`, `NodeJsPlugin`, `DockerPlugin`
  - 重构 `file_management` 插件，删除重复的 `ac_automaton.rs`
  - 更新插件注册逻辑，通过端口与领域层交互

  **Must NOT do**:
  - 不要修改插件执行逻辑（只移动位置）
  - 不要删除file_management功能（只删除重复代码）

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 熟悉插件生命周期管理，了解多种运行时
  - **Reason**: 涉及多种插件类型的迁移

  **Parallelization**:
  - **Can Run In Parallel**: YES (与3.1, 3.3)
  - **Parallel Group**: Wave 3
  - **Blocks**: 4.1
  - **Blocked By**: 2.3

  **References**:
  - `src/plugins/native.rs` - NativePlugin实现
  - `src/plugins/python.rs` - PythonPlugin实现
  - `src/plugins/file_management/ac_automaton.rs` - 要删除的重复代码
  - `src/plugins/file_management/registry.rs` - FileManagementToolRegistry

  **Acceptance Criteria**:
  - [x] 所有插件实现已迁移到infrastructure/plugin/
  - [x] 重复的ac_automaton.rs已删除
  - [x] 插件可以通过DI容器加载
  - [x] 集成测试：验证所有插件类型工作正常

  **Commit**: YES
  - Message: `refactor(infra): migrate plugin implementations to infrastructure layer`

---

- [x] 3.3 ✅ 重构缓存和性能优化

  **What to do**:
  - 迁移 `CacheManager` 到 `src/infrastructure/cache/`
  - 重构性能监控代码，使其通过端口与领域层交互
  - 确保缓存实现满足领域层定义的缓存端口
  - 保留moka集成，但封装在基础设施层

  **Must NOT do**:
  - 不要暴露moka的具体类型到领域层
  - 不要修改缓存策略逻辑

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉缓存策略，了解moka
  - **Reason**: 需要封装第三方库依赖

  **Parallelization**:
  - **Can Run In Parallel**: YES (与3.1, 3.2)
  - **Parallel Group**: Wave 3
  - **Blocks**: 4.1
  - **Blocked By**: 2.x

  **References**:
  - `src/performance/cache.rs:120-275` - CacheManager实现
  - `src/workflow/result_cache.rs` - 工作流结果缓存

  **Acceptance Criteria**:
  - [x] 缓存实现已迁移到infrastructure/cache/
  - [x] 缓存端口已定义在domain层
  - [x] 性能监控不直接依赖基础设施实现

  **Commit**: YES (groups with 3.x)
  - Message: `refactor(infra): migrate caching and performance to infrastructure layer`

---

### 波次4: 应用层重构

- [x] 4.1 ✅ 重构工作流编排器

  **What to do**:
  - 创建 `src/application/workflow/orchestrator.rs` - 工作流编排器
  - 重构 `RefactoredWorkflowEngine`，移除对基础设施的直接依赖
  - 通过端口使用：Repository, ToolRegistry, PluginManager, AuditLogger
  - 将编排逻辑与执行逻辑分离
  - 实现工作流命令模式（ExecuteWorkflow, PauseWorkflow等）

  **Must NOT do**:
  - 不要修改DAG调度逻辑
  - 不要修改并行执行语义
  - 不要引入新的工作流特性

  **Recommended Agent Profile**:
  - **Category**: ultrabrain
  - **Skills**: 深入理解工作流引擎，熟悉命令模式
  - **Reason**: 这是最复杂的重构任务，需要重新设计依赖关系

  **Parallelization**:
  - **Can Run In Parallel**: NO (关键路径)
  - **Parallel Group**: Wave 4
  - **Blocks**: 4.2, 4.3, 5.x
  - **Blocked By**: 3.x

  **References**:
  - `src/workflow/engine.rs:54-82` - RefactoredWorkflowEngine
  - `src/workflow/scheduler.rs` - DagScheduler
  - `src/workflow/execution_manager.rs` - ExecutionManager

  **Acceptance Criteria**:
  - [x] 工作流编排器不直接依赖基础设施
  - [x] 通过DI容器注入所有依赖
  - [x] 编排器实现领域层定义的端口
  - [x] 集成测试：完整工作流执行测试通过

  **Commit**: YES
  - Message: `refactor(app): restructure workflow orchestrator with DDD layering`

---

- [x] 4.2 ✅ 实现用例层

  **What to do**:
  - 创建 `src/application/usecase/` 目录
  - 实现具体用例：
    - `execute_workflow.rs` - 执行工作流用例
    - `manage_plugins.rs` - 管理插件用例
    - `system_monitoring.rs` - 系统监控用例
  - 每个用例封装一个完整的业务流程
  - 用例通过编排器与领域层交互

  **Must NOT do**:
  - 不要用例直接访问基础设施
  - 不要在一个用例中混合多个业务概念

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉用例驱动开发
  - **Reason**: 将业务逻辑组织成清晰的用例

  **Parallelization**:
  - **Can Run In Parallel**: YES (与4.3)
  - **Parallel Group**: Wave 4
  - **Blocks**: 5.x
  - **Blocked By**: 4.1

  **References**:
  - `src/interfaces/cli/app.rs` - CLI中的用例逻辑
  - `src/interfaces/tui/app.rs` - TUI中的用例逻辑

  **Acceptance Criteria**:
  - [x] 主要用例已实现
  - [x] 用例通过端口与下层交互
  - [x] 集成测试：验证用例行为

  **Commit**: YES
  - Message: `feat(app): implement use case layer with concrete business scenarios`

---

- [x] 4.3 ✅ 重构工作流执行器

  **What to do**:
  - 重构 `src/workflow/executor/` 模块
  - 将执行策略（basic, retry, cache, audit）迁移到应用层
  - 确保执行器通过ToolRegistry端口调用工具
  - 保持执行器链式结构

  **Must NOT do**:
  - 不要修改执行策略的行为
  - 不要将执行器移出应用层（它是编排的一部分）

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉装饰器/链式模式
  - **Reason**: 需要重新组织执行器但不改变行为

  **Parallelization**:
  - **Can Run In Parallel**: YES (与4.2)
  - **Parallel Group**: Wave 4
  - **Blocks**: 5.x
  - **Blocked By**: 4.1

  **References**:
  - `src/workflow/executor/basic.rs` - 基础执行器
  - `src/workflow/executor/retry.rs` - 重试执行器
  - `src/workflow/executor/cache.rs` - 缓存执行器
  - `src/workflow/executor/audit.rs` - 审计执行器

  **Acceptance Criteria**:
  - [x] 执行器链已重构到应用层
  - [x] 通过端口调用工具
  - [x] 集成测试：执行器链测试通过

  **Commit**: YES (groups with 4.x)
  - Message: `refactor(app): migrate workflow executors to application layer`

---

### 波次5: 适配层迁移

- [x] 5.1 ✅ 重构CLI适配器

  **What to do**:
  - 迁移 `src/interfaces/cli/` 到 `src/adapter/cli/`
  - 将CLI逻辑改为调用应用层的用例
  - 移除CLI中的业务逻辑（移入用例层）
  - 创建DTO用于CLI与用例层之间的数据传输
  - 保持CLI命令和参数不变（向后兼容）

  **Must NOT do**:
  - 不要修改CLI命令名称和参数
  - 不要删除CLI的功能

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉CLI设计，了解适配器模式
  - **Reason**: 需要保持CLI兼容同时重构内部实现

  **Parallelization**:
  - **Can Run In Parallel**: YES (与5.2, 5.3)
  - **Parallel Group**: Wave 5
  - **Blocks**: 6.x
  - **Blocked By**: 4.x

  **References**:
  - `src/interfaces/cli/app.rs` - CLI应用
  - `src/interfaces/cli/commands.rs` - CLI命令定义
  - `src/interfaces/cli/output.rs` - 输出格式化

  **Acceptance Criteria**:
  - [x] CLI代码已迁移到adapter/cli/
  - [x] CLI通过用例层执行业务逻辑
  - [x] CLI命令和参数保持不变
  - [x] 集成测试：所有CLI命令测试通过

  **Commit**: YES
  - Message: `refactor(adapter): migrate CLI to adapter layer with use case integration`

---

- [x] 5.2 ✅ 重构TUI适配器

  **What to do**:
  - 迁移 `src/interfaces/tui/` 到 `src/adapter/tui/`
  - 将TUI的业务逻辑移入应用层用例
  - TUI只负责：渲染、用户输入、事件处理
  - 通过用例层获取数据和执行业务操作
  - 保持TUI界面和交互不变

  **Must NOT do**:
  - 不要修改TUI布局和主题
  - 不要在TUI中保留业务逻辑

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 熟悉TUI架构，了解ratatui
  - **Reason**: TUI是最大最复杂的接口层（25个文件）

  **Parallelization**:
  - **Can Run In Parallel**: YES (与5.1, 5.3)
  - **Parallel Group**: Wave 5
  - **Blocks**: 6.x
  - **Blocked By**: 4.x

  **References**:
  - `src/interfaces/tui/app.rs` - TUI应用
  - `src/interfaces/tui/widgets/` - 8个widget组件
  - `src/interfaces/tui/layout.rs` - 布局管理（3,275行）

  **Acceptance Criteria**:
  - [x] TUI代码已迁移到adapter/tui/
  - [x] TUI通过用例层获取数据
  - [x] TUI界面和行为保持不变
  - [x] 集成测试：TUI交互测试通过

  **Commit**: YES
  - Message: `refactor(adapter): migrate TUI to adapter layer with clean separation`

---

- [x] 5.3 ✅ 重构MCP适配器

  **What to do**:
  - 迁移 `src/interfaces/mcp*.rs` 到 `src/adapter/mcp/`
  - 目前MCP是stub实现，需要完成适配器实现
  - 通过用例层处理MCP请求
  - 实现MCP协议到内部用例的映射

  **Must NOT do**:
  - 不要暴露内部领域模型给MCP协议
  - 不要在MCP适配器中引入业务逻辑

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 了解MCP协议（Model Context Protocol）
  - **Reason**: 需要理解MCP协议映射

  **Parallelization**:
  - **Can Run In Parallel**: YES (与5.1, 5.2)
  - **Parallel Group**: Wave 5
  - **Blocks**: 6.x
  - **Blocked By**: 4.x

  **References**:
  - `src/interfaces/mcp.rs` - MCP接口定义
  - `src/interfaces/mcp_server.rs` - MCP服务器（stub）

  **Acceptance Criteria**:
  - [x] MCP适配器已实现（或stub已迁移）
  - [x] MCP通过用例层处理请求

  **Commit**: YES (groups with 5.x)
  - Message: `refactor(adapter): migrate MCP to adapter layer`

---

### 波次6: 清理与测试

- [x] 6.1 ✅ 删除遗留代码

  **What to do**:
  - 删除 `src/workflow/engine_legacy.rs`（确认engine.rs已完全替代）
  - 删除 `src/plugins/file_management/ac_automaton.rs`（使用tools/algo/版本）
  - 删除旧的 `src/interfaces/` 目录（迁移完成后）
  - 删除未使用的配置类型（如未使用的AuthConfig）
  - 清理 `src/core/` 中已迁移到domain/的类型（移除重复）
  - 更新 `src/lib.rs` 的模块声明

  **Must NOT do**:
  - 不要删除还有引用的地方
  - 不要删除测试文件
  - 确保所有功能都有替代实现

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉代码清理，了解Rust模块系统
  - **Reason**: 需要谨慎删除，确保不破坏功能

  **Parallelization**:
  - **Can Run In Parallel**: NO（必须在最后）
  - **Parallel Group**: Wave 6
  - **Blocks**: None
  - **Blocked By**: 5.x

  **References**:
  - `src/workflow/engine_legacy.rs` - 待删除
  - `src/plugins/file_management/ac_automaton.rs` - 待删除
  - `src/core/mod.rs` - 检查待清理的类型

  **Acceptance Criteria**:
  - [x] 所有遗留代码已删除
  - [x] `cargo build` 通过
  - [x] 代码行数减少（目标：减少10-15%）

  **Commit**: YES
  - Message: `chore(cleanup): remove legacy code and duplicates`

---

- [x] 6.2 ✅ 实现契约测试

  **What to do**:
  - 创建 `tests/contract/` 目录
  - 实现层间契约测试：
    - `domain_application_contract.rs` - 领域层与应用层契约
    - `application_adapter_contract.rs` - 应用层与适配层契约
    - `domain_infrastructure_contract.rs` - 领域层与基础设施层契约
  - 使用Pact或自定义契约测试框架
  - 验证：接口兼容性、数据序列化、错误传播

  **Must NOT do**:
  - 不要测试具体实现（只测试接口契约）
  - 不要引入外部契约测试服务（使用本地验证）

  **Recommended Agent Profile**:
  - **Category**: unspecified-medium
  - **Skills**: 熟悉契约测试概念
  - **Reason**: 需要定义和验证层间契约

  **Parallelization**:
  - **Can Run In Parallel**: YES (与6.1, 6.3)
  - **Parallel Group**: Wave 6
  - **Blocks**: None
  - **Blocked By**: 5.x

  **References**:
  - `tests/` 目录现有测试结构
  - `src/domain/port/` - 所有端口接口

  **Acceptance Criteria**:
  - [x] 契约测试框架已建立
  - [x] 主要层间接口都有契约测试
  - [x] 契约测试在CI中运行

  **Commit**: YES
  - Message: `test(contract): implement contract tests between layers`

---

- [x] 6.3 ✅ 集成测试与验证

  **What to do**:
  - 运行完整的集成测试套件
  - 验证所有CLI命令工作正常
  - 验证TUI交互正常
  - 验证工作流执行（包括并行、重试、检查点）
  - 验证所有插件类型（Native, Python, Node.js, Docker）
  - 性能对比：重构前后性能不应下降

  **Must NOT do**:
  - 不要跳过失败的测试（修复或标记原因）
  - 不要降低测试覆盖率

  **Recommended Agent Profile**:
  - **Category**: unspecified-high
  - **Skills**: 熟悉集成测试，了解性能测试
  - **Reason**: 最终验证，确保重构成功

  **Parallelization**:
  - **Can Run In Parallel**: YES (与6.1, 6.2)
  - **Parallel Group**: Wave 6
  - **Blocks**: None
  - **Blocked By**: 5.x

  **References**:
  - `tests/` 所有测试文件
  - `examples/` 用于手动验证的示例

  **Acceptance Criteria**:
  - [x] 所有测试通过：`cargo test`
  - [x] 手动验证CLI/TUI/工作流执行
  - [x] 性能基准对比完成
  - [x] 文档已更新（AGENTS.md等）

  **Commit**: YES (groups with 6.x)
  - Message: `test(integration): complete integration testing and verification`

---

## 依赖矩阵

| 任务 | 依赖 | 阻塞 | 并行组 |
|------|------|------|--------|
| 1.1 目录结构 | None | 所有任务 | Wave 1 |
| 1.2 DI框架 | None | 2.3, 3.x, 4.x, 5.x | Wave 1 |
| 1.3 领域端口 | None | 2.2, 2.3, 3.x, 4.x | Wave 1 |
| 2.1 领域模型 | 1.x | 2.2, 2.3, 3.x | Wave 2 |
| 2.2 工具系统 | 1.3, 2.1 | 3.1, 4.1 | Wave 2 |
| 2.3 插件抽象 | 1.3, 2.1 | 3.2 | Wave 2 |
| 3.1 仓储实现 | 2.x | 4.1 | Wave 3 |
| 3.2 插件实现 | 2.3 | 4.1 | Wave 3 |
| 3.3 缓存重构 | 2.x | 4.1 | Wave 3 |
| 4.1 工作流编排 | 3.x | 4.2, 4.3, 5.x | Wave 4 |
| 4.2 用例层 | 4.1 | 5.x | Wave 4 |
| 4.3 执行器 | 4.1 | 5.x | Wave 4 |
| 5.1 CLI适配器 | 4.x | 6.x | Wave 5 |
| 5.2 TUI适配器 | 4.x | 6.x | Wave 5 |
| 5.3 MCP适配器 | 4.x | 6.x | Wave 5 |
| 6.1 代码清理 | 5.x | None | Wave 6 |
| 6.2 契约测试 | 5.x | None | Wave 6 |
| 6.3 集成验证 | 5.x | None | Wave 6 |

## 关键路径

**关键路径**（决定总工期）: 
```
1.1 → 1.2 → 1.3 → 2.1 → 2.2 → 3.1 → 4.1 → 4.2 → 5.1 → 6.3
```

**预期并行收益**: 
- Wave 1: 3个任务并行（节省2个任务时间）
- Wave 2: 3个任务并行，但2.2/2.3依赖2.1（节省1个任务时间）
- Wave 3: 3个任务完全并行（节省2个任务时间）
- Wave 4: 3个任务，4.2/4.3依赖4.1（节省1个任务时间）
- Wave 5: 3个任务完全并行（节省2个任务时间）
- Wave 6: 3个任务完全并行（节省2个任务时间）

**总任务**: 18个
**关键路径任务**: 10个
**预估节省**: ~40%时间（通过并行执行）

## Commit策略

| 波次 | Commit Message Pattern | Scope |
|------|------------------------|-------|
| Wave 1 | `chore(ddd): ...` 或 `feat(di): ...` | 基础结构 |
| Wave 2 | `refactor(domain): ...` 或 `feat(domain): ...` | 领域层 |
| Wave 3 | `refactor(infra): ...` | 基础设施层 |
| Wave 4 | `refactor(app): ...` 或 `feat(app): ...` | 应用层 |
| Wave 5 | `refactor(adapter): ...` | 适配层 |
| Wave 6 | `chore(cleanup): ...` 或 `test(contract): ...` | 清理和测试 |

## 成功标准

### 架构验证
- [x] 依赖方向正确：adapter → app → domain ← infra
- [x] 无循环依赖：`cargo tree` 验证
- [x] 层间通过端口交互：无直接类型依赖
- [x] DI容器管理所有依赖：无手动new()基础设施

### 功能验证
- [x] 所有现有测试通过：`cargo test`
- [x] CLI向后兼容：所有命令正常工作
- [x] TUI正常工作：交互测试通过
- [x] 工作流执行正常：包括并行、重试、检查点
- [x] 所有插件类型工作：Native, Python, Node.js, Docker

### 代码质量验证
- [x] 代码行数减少：目标-10-15%（通过删除重复/遗留代码）
- [x] 重复代码消除：通过 `cargo dudupes` 或人工审查
- [x] 模块边界清晰：每层职责单一
- [x] 契约测试覆盖：主要层间接口都有契约测试

### 性能验证
- [x] 工作流执行性能不下降：基准测试对比
- [x] 内存使用不增加：通过 `valgrind` 或 `heaptrack`
- [x] 编译时间不显著增加：`cargo build --release` 时间对比

---

## 风险评估与缓解

### 高风险项
1. **4.1 工作流编排器重构**：核心引擎重构，影响所有功能
   - **缓解**：详细的前期设计审查，保持原有测试通过
   
2. **5.2 TUI适配器迁移**：最大最复杂的组件（25个文件，3000+行）
   - **缓解**：分阶段迁移，每次迁移一个widget
   
3. **6.3 集成验证**：可能发现难以预料的回归问题
   - **缓解**：建立完整的回归测试套件，CI中运行

### 中风险项
1. **1.2 DI框架设置**：shaku的学习曲线
   - **缓解**：先在小范围试点，验证后再全面使用
   
2. **3.x 基础设施层迁移**：大量代码移动，可能引入错误
   - **缓解**：每次迁移一个组件，详细测试

### 低风险项
1. **1.1 目录结构建立**：纯新增操作
2. **6.1 代码清理**：删除操作，影响范围明确

---

## 后续建议

### 重构后优化
1. **聚合根优化**：评估Workflow聚合根是否需要进一步拆分
2. **事件溯源**：考虑引入事件溯源模式增强可审计性
3. **CQRS**：读写分离，优化查询性能
4. **微服务拆分**：如果系统继续增长，考虑按限界上下文拆分

### 技术债务清理
1. **Async_trait移除**：使用Rust原生async trait（稳定后）
2. **WASM支持恢复**：重新启用WASM插件支持
3. **MCP完整实现**：完成MCP服务器完整功能

---

## 下一步行动

1. **审查计划**：确认本计划满足所有需求
2. **选择模式**：
   - 标准模式：直接执行本计划
   - 高精度模式：提交Momus审查后再执行
3. **开始执行**：运行 `/start-work` 启动Sisyphus执行

**计划文件**: `.sisyphus/plans/ddd-refactor.md`
**建议执行顺序**: 按波次顺序，波次内并行执行

---

*计划生成时间*: 2026-01-31
*基于*: 渐进式重构 + 激进代码清理 + 依赖注入框架 + 契约测试
*预估工作量*: 18个主要任务，关键路径10个任务
