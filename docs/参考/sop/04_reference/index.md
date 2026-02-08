# 参考文档

## 渐进式披露层级 (L1-L4)

| 层级 | 目录 | 内容 | 格式 | 创建者 |
|------|------|------|------|--------|
| L1 | `01_concept_overview.md` | 核心概念 | Markdown | - |
| L2 | `02_logical_workflow/` | 逻辑工作流 | `.pseudo` | Prometheus |
| L3 | `03_technical_spec/` / `src/**/design.md` | 技术规格 | Markdown/YAML | Oracle |
| L4 | `04_context_reference/` | 决策参考 | `adr_*.md` | Prometheus/Oracle |

---

## 文档放置规则

| 目录 | 用途 | 权限 |
|------|------|------|
| `/docs` | 项目设计文档 | 动态创建更新 |
| `/docs/参考/` | SOP参考文档 | **非指定不变更** |
| `/docs/01_requirements/` | PRD (L1) | Analyst创建 |
| `/docs/02_logical_workflow/` | 架构设计 (L2) | Prometheus创建 |
| `/docs/03_technical_spec/` | 技术规格 (L3) | Oracle创建 |
| `/docs/04_context_reference/` | 决策参考 (L4) | Prometheus/Oracle创建 |
| `src/**/design.md` | 实现设计 (L3) | Oracle创建 |

---

## L2: 逻辑工作流 (`.pseudo`)

**创建者**: Prometheus  
**规范**: 技术无关伪代码

### 伪代码规范
- **原子操作**: `UPPER_SNAKE_CASE` (例: `VALIDATE_INPUT`)
- **函数**: `lower_snake_case` (例: `process_data`)
- **缩进**: 4空格
- **注释**: 说明"为什么"

### 控制结构
```pseudo
IF condition:
    action
ELSE:
    default_action
END IF

FOR EACH item IN collection:
    process(item)
END FOR

TRY:
    operation
CATCH error:
    handle_error
END TRY
```

---

## L3: 技术规格

**创建者**: Oracle  
**规范**: 将L2伪代码映射为具体技术实现

### design.md规则
| 复杂度 | 行数 | 要求 |
|--------|------|------|
| 低 | <100 | 省略，代码注释 |
| 中 | 100-500 | 简要+接口契约 |
| 高 | >500 | 完整+详细契约 |

### 必须包含
```markdown
## 接口契约

### 输入
| 参数 | 类型 | 说明 |
|------|------|------|
| [param] | [type] | [desc] |

### 输出
| 返回值 | 类型 | 说明 |
|--------|------|------|
| [return] | [type] | [desc] |

### 依赖
- 依赖模块: [name]
- 依赖接口: [iface]
```

---

## L4: 决策参考 (ADR)

**创建者**: Prometheus / Oracle  
**规范**: 记录关键决策的背景和理由

### ADR编号
```
ADR-[模块]-[序号]: [标题]
```

### 何时创建
- 引入新技术栈/框架
- 核心架构模式变更
- 重大接口设计决策
- 性能优化策略选择

---

## 模板

| 模板 | 层级 | 用途 | 角色 |
|------|------|------|------|
| [PRD](document_templates/prd.md) | L1 | 需求文档 | Analyst |
| [架构设计](document_templates/architecture_design.md) | L2 | 逻辑工作流 | Prometheus |
| [实现设计](document_templates/implementation_design.md) | L3 | 技术规格 | Oracle |
| [ADR](document_templates/adr.md) | L4 | 决策参考 | Prometheus/Oracle |

---

## 交互格式

| 格式 | 用途 | 角色 |
|------|------|------|
| [Supervisor报告](interaction_formats/supervisor_report.md) | 进度/熔断/决策 | Supervisor |

---

## 状态标记

- `[进行中]` - 正在处理
- `[已完成]` - 处理完成
- `[待审批]` - 等待审批
- `[已归档]` - 已归档
