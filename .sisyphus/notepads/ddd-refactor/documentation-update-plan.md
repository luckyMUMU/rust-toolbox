# 文档更新计划 - DDD架构重构后

## 需要创建的新文档

### 1. src/adapter/AGENTS.md
**内容要点**:
- 适配层架构说明（CLI/TUI/MCP）
- DDD架构中的位置: `adapter → application → domain ← infrastructure`
- 目录结构: cli/, tui/, mcp/, dto/
- 设计原则: 无业务逻辑、DTO隔离、向后兼容
- 迁移状态: 结构已建立，待功能迁移

### 2. src/application/AGENTS.md
**内容要点**:
- 应用层架构说明（用例/编排）
- 目录结构: port/, workflow/, usecase/, service/
- 职责: 工作流编排、用例实现
- 设计原则: 通过端口与领域层交互

### 3. src/domain/AGENTS.md
**内容要点**:
- 领域层架构说明（核心层）
- 目录结构: model/, port/
- 领域模型: ToolInfo, PluginInfo, ExecutionContext, WorkflowConfig
- 领域端口: ToolRegistry, PluginManager, Repository
- 设计原则: 无外部依赖、富领域模型
- 实现状态: ✅ 已完成

### 4. src/infrastructure/AGENTS.md
**内容要点**:
- 基础设施层架构说明
- 目录结构: persistence/, plugin/, cache/, external/, config/
- 实现领域端口: 仓储实现、插件注册表、缓存
- 技术实现: 存储后端、缓存策略

### 5. src/di/AGENTS.md
**内容要点**:
- 依赖注入容器说明
- shaku框架集成
- 模块: container.rs, module.rs, provider.rs
- 职责: 依赖生命周期管理

## 需要更新的现有文档

### 1. ./AGENTS.md (根文档)
**更新内容**:
- 添加DDD架构说明
- 更新模块结构图，包含新层
- 添加依赖方向: `adapter → application → domain ← infrastructure`
- 更新完成的功能列表

### 2. src/AGENTS.md
**更新内容**:
- 更新模块结构，包含 adapter, application, domain, infrastructure, di
- 更新架构图
- 添加DDD分层说明

### 3. src/workflow/AGENTS.md
**更新内容**:
- 添加与新架构的集成说明
- 更新引擎架构说明
- 说明RefactoredWorkflowEngine的位置

### 4. src/tools/AGENTS.md
**更新内容**:
- 添加与domain层的ToolRegistry端口的关系
- 说明algo/模块（ac_automaton）

### 5. src/plugins/AGENTS.md
**更新内容**:
- 添加与domain层的PluginManager端口的关系
- 说明file_management迁移状态
- 删除重复的ac_automaton引用

### 6. src/interfaces/AGENTS.md
**更新内容**:
- 添加迁移到adapter/的计划说明
- 说明向后兼容策略

### 7. 其他AGENTS.md文件
- 更新交叉引用
- 添加DDD架构上下文

## 实施步骤

1. 创建新的AGENTS.md文件（adapter, application, domain, infrastructure, di）
2. 更新根AGENTS.md
3. 更新src/AGENTS.md
4. 更新各模块AGENTS.md
5. 统一格式和风格

## 验收标准

- [ ] 所有新层都有AGENTS.md文档
- [ ] 根AGENTS.md反映新架构
- [ ] 所有现有AGENTS.md更新完成
- [ ] 文档之间交叉引用正确
- [ ] 中英文对照完整
