# Workflow Toolkit 文档中心

## 项目概述

Workflow Toolkit 是一个 Rust 实现的工作流工具包，提供插件化架构、工具执行引擎和工作流编排能力。

## 文档目录结构

```
docs/
├── index.md                    # 文档入口（本文件）
├── 01_concept_overview.md      # 概念概述
├── 01_requirements/            # 需求文档
├── 02_logical_workflow/        # 逻辑工作流伪代码
├── 03_technical_spec/          # 技术规范
├── 04_context_reference/       # 上下文参考
├── archive/                    # 归档文档
└── 参考/                       # 参考资料
```

---

## 章节导航

### 1. 概念概述

- [01_concept_overview.md](./01_concept_overview.md) - 项目核心概念与架构概述

### 2. 需求文档

| 文档 | 说明 |
|------|------|
| [workflow_toolkit_prd.md](./01_requirements/workflow_toolkit_prd.md) | 产品需求文档 |

### 3. 逻辑工作流

工作流伪代码描述核心业务逻辑：

| 文档 | 说明 |
|------|------|
| [error_handling.pseudo](./02_logical_workflow/error_handling.pseudo) | 错误处理逻辑 |
| [plugin_loading.pseudo](./02_logical_workflow/plugin_loading.pseudo) | 插件加载逻辑 |
| [tool_execution.pseudo](./02_logical_workflow/tool_execution.pseudo) | 工具执行逻辑 |
| [workflow_execution.pseudo](./02_logical_workflow/workflow_execution.pseudo) | 工作流执行逻辑 |

### 4. 技术规范

#### 接口定义

| 文档 | 说明 |
|------|------|
| [interfaces.md](./03_technical_spec/interfaces.md) | 核心接口规范 |

#### API 参考

| 文档 | 说明 |
|------|------|
| [CLI_REFERENCE.md](./03_technical_spec/api/CLI_REFERENCE.md) | 命令行接口参考 |
| [RUST_SDK_REFERENCE.md](./03_technical_spec/api/RUST_SDK_REFERENCE.md) | Rust SDK 参考 |

### 5. 上下文参考

#### 架构决策记录 (ADR)

| 文档 | 说明 |
|------|------|
| [architecture_decision.md](./04_context_reference/architecture_decision.md) | 架构决策总览 |
| [adr_di_001_自建DI容器选择.md](./04_context_reference/adr/adr_di_001_自建DI容器选择.md) | DI 容器选择决策 |
| [adr_workflow_002_采用JoinSet并行执行.md](./04_context_reference/adr/adr_workflow_002_采用JoinSet并行执行.md) | JoinSet 并行执行决策 |
| [adr_plugin_003_WASM沙箱隔离策略.md](./04_context_reference/adr/adr_plugin_003_WASM沙箱隔离策略.md) | WASM 沙箱隔离策略 |
| [adr_tools_004_Schema验证策略.md](./04_context_reference/adr/adr_tools_004_Schema验证策略.md) | Schema 验证策略 |
| [template.md](./04_context_reference/adr/template.md) | ADR 模板 |

### 6. 归档文档

历史版本文档和迁移指南，详见 [archive/DOCS_README.md](./archive/DOCS_README.md)。

### 7. 参考资料

| 文档 | 说明 |
|------|------|
| [SPEC.md](./参考/SPEC.md) | 规格说明 |

---

## 快速开始指南

### 新读者

1. 从 [概念概述](./01_concept_overview.md) 了解项目核心概念
2. 阅读 [产品需求文档](./01_requirements/workflow_toolkit_prd.md) 理解项目目标
3. 查看 [接口规范](./03_technical_spec/interfaces.md) 了解技术细节

### 开发者

1. 阅读 [架构决策记录](./04_context_reference/architecture_decision.md) 了解设计背景
2. 参考 [逻辑工作流](./02_logical_workflow/) 理解业务逻辑
3. 使用 [API 参考](./03_technical_spec/api/) 进行开发集成

### 维护者

1. 参考 [ADR 模板](./04_context_reference/adr/template.md) 记录架构决策
2. 遵循文档结构规范更新相关章节
3. 将过时文档移至 [archive](./archive/) 目录

---

## 文档约定

- 文件命名使用小写字母和下划线
- 目录使用数字前缀排序
- ADR 文档遵循 `adr_领域_编号_标题.md` 格式
- 伪代码文件使用 `.pseudo` 扩展名
