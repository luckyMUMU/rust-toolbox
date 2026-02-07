# 参考文档

## 文档放置规则

| 目录 | 用途 | 权限 |
|------|------|------|
| `/docs` | 项目设计文档 | 动态创建更新 |
| `/docs/参考/` | SOP参考文档 | **非指定不变更** |
| `/docs/01_requirements/` | PRD | Analyst创建 |
| `/docs/02_logical_workflow/` | 架构设计 | Prometheus创建 |
| `src/**/design.md` | 实现设计 | Oracle创建 |

---

## design.md规则

| 复杂度 | 行数 | 要求 |
|--------|------|------|
| 低 | <100 | 省略，代码注释 |
| 中 | 100-500 | 简要design.md+接口契约 |
| 高 | >500 | 完整design.md+详细契约 |

**必须包含**:
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

## 模板

| 模板 | 用途 | 角色 |
|------|------|------|
| [架构设计](document_templates/architecture_design.md) | 技术无关架构 | Prometheus |
| [实现设计](document_templates/implementation_design.md) | 项目特定实现 | Oracle |

---

## 交互格式

| 格式 | 用途 | 角色 |
|------|------|------|
| [Supervisor报告](interaction_formats/supervisor_report.md) | 进度/熔断/决策 | Supervisor |

---

## 规范

### 伪代码规范
- `UPPER_SNAKE_CASE` 表示原子操作
- 4空格缩进表示逻辑层次
- 注释说明"为什么"
- 避免特定语言语法

### 状态标记
- `[进行中]` - 正在处理
- `[已完成]` - 处理完成
- `[待审批]` - 等待审批
- `[已归档]` - 已归档
