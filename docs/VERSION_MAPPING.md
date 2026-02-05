# 版本映射表

> **项目**: Workflow Toolkit  
> **当前版本**: v0.1.0  
> **最后更新**: 2026-02-05

---

## 版本历史

| 版本 | 日期 | 代码标签 | 文档变更 | 主要特性 |
|------|------|----------|----------|----------|
| v0.1.0 | 2026-02-05 | `v0.1.0` | 初始文档结构 | 基础工作流引擎、多语言插件支持 |

---

## 代码-文档版本关联

### 核心模块

| 模块 | 代码路径 | 文档路径 | 最后更新版本 |
|------|----------|----------|--------------|
| 工作流引擎 | `src/workflow/` | `docs/02_logical_workflow/workflow_execution.pseudo` | v0.1.0 |
| 工具系统 | `src/tools/` | `docs/02_logical_workflow/tool_execution.pseudo` | v0.1.0 |
| 插件系统 | `src/plugins/` | `docs/02_logical_workflow/plugin_loading.pseudo` | v0.1.0 |
| 错误处理 | `src/error.rs` | `docs/02_logical_workflow/error_handling.pseudo` | v0.1.0 |
| 接口契约 | `src/domain/port/` | `docs/03_technical_spec/interfaces.md` | v0.1.0 |
| 架构决策 | - | `docs/04_context_reference/architecture_decision.md` | v0.1.0 |

### 接口层

| 接口 | 代码路径 | 文档路径 | 最后更新版本 |
|------|----------|----------|--------------|
| CLI | `src/interfaces/cli/` | `docs/api/CLI_REFERENCE.md` | v0.1.0 |
| TUI | `src/interfaces/tui/` | `docs/guides/USER_GUIDE.md` | v0.1.0 |
| MCP Server | `src/interfaces/mcp.rs` | `docs/api/RUST_SDK_REFERENCE.md` | v0.1.0 |

---

## 文档版本标记规范

所有文档文件应包含版本标记：

```markdown
> **版本**: v0.1.0  
> *最后更新：2026-02-05*
```

### 版本标记位置

- **L2 逻辑层**: 文件头部注释
- **L3 规格层**: 文档标题下方
- **L4 决策层**: 文档标题下方
- **API 文档**: 文档标题下方
- **指南文档**: 文档标题下方

---

## 版本更新检查清单

当发布新版本时，请检查以下项目：

### 代码更新
- [ ] 更新 `Cargo.toml` 中的版本号
- [ ] 创建 Git 标签
- [ ] 更新 `CHANGELOG.md`

### 文档更新
- [ ] 更新本文档的版本历史表格
- [ ] 更新所有文档中的版本标记
- [ ] 更新代码-文档映射表
- [ ] 检查文档链接有效性

### 验证
- [ ] 运行 `cargo test` 确保所有测试通过
- [ ] 运行 `cargo doc` 确保文档生成成功
- [ ] 验证所有文档链接有效

---

## 文档结构版本

### L2: 逻辑流转层

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `workflow_execution.pseudo` | v0.1.0 | 2026-02-05 |
| `tool_execution.pseudo` | v0.1.0 | 2026-02-05 |
| `plugin_loading.pseudo` | v0.1.0 | 2026-02-05 |
| `error_handling.pseudo` | v0.1.0 | 2026-02-05 |

### L3: 技术规格层

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `interfaces.md` | v0.1.0 | 2026-02-05 |

### L4: 决策参考层

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `architecture_decision.md` | v0.1.0 | 2026-02-05 |

### API 参考

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `CLI_REFERENCE.md` | v0.1.0 | 2026-02-05 |
| `RUST_SDK_REFERENCE.md` | v0.1.0 | 2026-02-05 |
| `TOOLS_REFERENCE.md` | v0.1.0 | 2026-02-05 |
| `TOOL_DESIGN_STANDARDS.md` | v0.1.0 | 2026-02-05 |

### 开发文档

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `DEVELOPMENT_GUIDE.md` | v0.1.0 | 2026-02-05 |
| `PLUGIN_DEVELOPMENT.md` | v0.1.0 | 2026-02-05 |
| `PROJECT_OVERVIEW.md` | v0.1.0 | 2026-02-05 |
| `DOCUMENTATION_STANDARDS.md` | v0.1.0 | 2026-02-05 |
| `WORKFLOW_DESIGN.md` | v0.1.0 | 2026-02-05 |

### 用户指南

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `USER_GUIDE.md` | v0.1.0 | 2026-02-05 |
| `CHEATSHEET.md` | v0.1.0 | 2026-02-05 |
| `TUTORIAL.md` | v0.1.0 | 2026-02-05 |
| `TROUBLESHOOTING.md` | v0.1.0 | 2026-02-05 |
| `MIGRATION_GUIDE.md` | v0.1.0 | 2026-02-05 |

### 插件文档

| 文档 | 版本 | 最后更新 |
|------|------|----------|
| `FILE_MANAGEMENT_TOOLS_INDEX.md` | v0.1.0 | 2026-02-05 |
| `FILE_MANAGEMENT_TOOLS_GUIDE.md` | v0.1.0 | 2026-02-05 |
| `FILE_MANAGEMENT_API_REFERENCE.md` | v0.1.0 | 2026-02-05 |
| `FILE_MANAGEMENT_HUMAN_DECISION_GUIDE.md` | v0.1.0 | 2026-02-05 |
| `FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md` | v0.1.0 | 2026-02-05 |

---

## 文档维护责任

| 文档类型 | 维护者 | 更新频率 |
|----------|--------|----------|
| L2 逻辑层 | 架构师 | 架构变更时 |
| L3 规格层 | 技术负责人 | API 变更时 |
| L4 决策层 | 技术负责人 | 决策变更时 |
| API 文档 | 核心团队 | 每个版本 |
| 开发文档 | 核心团队 | 每个版本 |
| 用户指南 | 文档团队 | 每个版本 |
| 插件文档 | 插件维护者 | 插件更新时 |
