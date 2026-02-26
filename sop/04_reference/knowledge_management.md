---
version: v2.9.0
updated: 2026-02-24
---

# 参考资料与知识沉淀规范

---

## 概述

本文档规范如何在 AI 辅助开发流程中管理参考资料和知识沉淀，确保：
- 用户输入得到妥善保存和引用
- 外部获取的知识可追溯
- 参考资料与设计决策保持一致
- 冲突时能够及时发现并处理

---

## 文档层级关系

```
L1: 概念层 (01_concept_overview.md)
    ↓
L2: 逻辑工作流 (02_logical_workflow/*.md)
    ↓
L3: 技术规格 (src/**/design.md)
    ↓
L4: 决策参考 (04_context_reference/)
    ├── adr_*.md              # 架构决策记录
    ├── decisions/            # 决策记录（触发 `[USER_DECISION]` 时必须落盘）
    ├── rag/                  # 参考资料 (RAG - Retrieval Augmented Generation)
    │   ├── user_input/       # 用户提供的资料
    │   ├── external/         # 外部获取的知识
    │   └── project/          # 项目沉淀知识
    └── knowledge_management.md  # 本文件
```

---

## RAG 目录结构

```
docs/04_context_reference/
├── adr_*.md                    # 架构决策记录
├── decisions/                  # 决策记录（RECORD_DECISION 产物）
├── knowledge_management.md     # 本规范文件
└── rag/                        # 参考资料根目录
    ├── README.md              # RAG 使用指南
    ├── user_input/            # 用户提供的资料
    │   ├── requirements/      # 需求文档
    │   ├── designs/           # 设计稿/原型
    │   └── references/        # 其他参考资料
    ├── external/              # 外部获取的知识
    │   ├── tech_docs/         # 技术文档
    │   ├── api_specs/         # API规范
    │   └── best_practices/    # 最佳实践
    └── project/               # 项目沉淀知识
        ├── patterns.md        # 设计模式
        ├── lessons.md         # 经验教训
        └── decisions.md       # 决策记录摘要（可选）
```

---

## 决策记录（Decision Record）

当触发 `[USER_DECISION]` 且原因是“来源/依赖缺口”或“冲突无法消解”时，必须落盘决策记录文件，并在后续产物中引用。

- 命令：`RECORD_DECISION(topic, decision)`（参见 05_constraints/command_dictionary.md）
- 模板：04_reference/interaction_formats/source_dependency.md
- 逻辑目录：`docs/04_context_reference/decisions/`
- 命名建议：`[YYYYMMDD]_[topic].md`

---

## 用户输入处理

### 接收的资料类型

| 类型 | 示例 | 保存位置 |
|------|------|----------|
| 需求文档 | PRD、MRD、业务规则 | `rag/user_input/requirements/` |
| 设计稿 | Figma、Sketch、原型 | `rag/user_input/designs/` |
| 参考资料 | 技术文档、规范、链接 | `rag/user_input/references/` |
| 示例代码 | 参考实现、开源项目 | `rag/user_input/references/code/` |

### 处理流程

```
接收用户输入
    ↓
分类判断
    ├─ 临时参考 → 直接使用，不保存
    ├─ 项目参考 → 保存到 rag/user_input/
    └─ 架构参考 → 保存并提取到 ADR
    ↓
命名规范
    ├─ 需求: [YYYYMMDD]_[source]_[brief].md
    ├─ 设计: [YYYYMMDD]_design_[module].md
    └─ 参考: [YYYYMMDD]_ref_[topic].md
    ↓
更新索引
    ↓
在设计中引用
```

### 命名规范

**文件命名格式**: `[YYYYMMDD]_[source]_[brief].[ext]`

**示例**:
- `20260208_user_payment_requirements_v2.md`
- `20260208_design_user_module.fig`
- `20260208_ref_postgresql_best_practices.md`

---

## 外部知识获取

### 获取方式

| 方式 | 示例 | 保存位置 |
|------|------|----------|
| 网络搜索 | 技术对比、最佳实践 | `rag/external/tech_docs/` |
| 文档查询 | API文档、官方指南 | `rag/external/api_specs/` |
| 代码参考 | 开源项目、示例代码 | `rag/external/best_practices/` |

### 沉淀原则

1. **必要性**: 只沉淀对项目长期有价值的知识
2. **可追溯**: 记录知识来源（URL、文档、时间）
3. **可更新**: 定期审查和更新沉淀的知识
4. **可查找**: 在索引中登记所有参考资料

### 处理流程

```
获取外部知识
    ↓
价值评估
    ├─ 临时使用 → 直接引用，不保存
    ├─ 项目参考 → 保存到 rag/external/
    └─ 架构决策 → 保存并关联 ADR
    ↓
提取关键信息
    ↓
保存到 RAG
    ↓
在设计中引用
```

---

## 参考资料引用规范

### 引用时机

**必须引用 RAG**:
- 架构设计阶段（sop-architecture-design）
- 实现设计阶段（sop-implementation-designer）
- 架构审查阶段（sop-architecture-reviewer）

**建议引用 RAG**:
- 需求分析阶段（sop-requirement-analyst）
- 代码实现阶段（sop-code-implementation）

### 引用格式

在 design.md 或 ADR 中添加"参考资料"章节：

```markdown
## 参考资料

### 架构决策 (ADR)
| ADR | 决策内容 | 影响 | 状态 |
|-----|----------|------|------|
| [ADR-001] | [描述] | [本目录] | [已接受] |

### 用户输入 (RAG)
| 来源 | 类型 | 内容摘要 | 链接 |
|------|------|----------|------|
| [20260208_user_req] | 需求 | 支付模块需求v2 | [rag/user_input/...] |

### 外部知识 (RAG)
| 来源 | 类型 | 内容摘要 | 链接 |
|------|------|----------|------|
| [20260208_postgres_doc] | 技术文档 | 连接池配置 | [rag/external/...] |
```

---

## 冲突处理流程

### 冲突类型

| 类型 | 示例 | 处理方式 |
|------|------|----------|
| ADR vs 设计 | ADR选择MySQL，设计选择PostgreSQL | `[USER_DECISION]` |
| RAG vs 设计 | 参考文档建议A，设计采用B | `[USER_DECISION]` |
| 用户输入 vs ADR | 用户新需求与已接受ADR冲突 | `[USER_DECISION]` |
| RAG 内部冲突 | 两份参考文档建议矛盾 | `[USER_DECISION]` |

### 处理流程

```
发现冲突
    ↓
标记冲突点
    ↓
生成对比报告
```markdown
## 冲突报告

### 冲突点
[描述冲突的具体内容]

### 来源A
- 来源: [ADR-001 / RAG文件]
- 内容: [具体内容]
- 理由: [原始理由]

### 来源B
- 来源: [当前设计]
- 内容: [具体内容]
- 理由: [设计理由]

### 建议
- 选项1: [遵循ADR/RAG]
- 选项2: [更新设计]
- 选项3: [更新ADR/RAG]
```
    ↓
标记 `[USER_DECISION]`
    ↓
等待用户决策
    ↓
根据决策执行
    ├─ 遵循ADR/RAG → 更新设计
    ├─ 更新ADR → 修改ADR文档
    └─ 更新RAG → 修改RAG文档
```

---

## 维护规则

### 定期审查

| 频率 | 内容 | 执行 Skill |
|------|------|--------|
| 每周 | 检查新增RAG文件 | sop-document-sync |
| 每月 | 审查RAG文件有效性 | sop-document-sync |
| 每季度 | 更新项目知识库 | sop-document-sync |

### 清理规则

- **临时参考**: 不保存，使用后丢弃
- **过期参考**: 标记 `[DEPRECATED]`，保留但不再引用
- **错误参考**: 删除并记录原因

---

## 工具与模板

### RAG 文件模板

**用户输入模板**:
```markdown
# [标题]

## 来源
- 提供方: [用户/团队]
- 日期: [YYYY-MM-DD]
- 类型: [需求/设计/参考]

## 内容摘要
[简要描述]

## 原始内容
[完整内容或链接]

## 影响范围
- [模块A]
- [模块B]

## 关联文档
- [ADR-001]
- [design.md]

## 状态
- [x] 已审阅
- [ ] 已纳入设计
- [ ] 已过期
```

**外部知识模板**:
```markdown
# [标题]

## 来源
- URL: [链接]
- 获取日期: [YYYY-MM-DD]
- 类型: [技术文档/API规范/最佳实践]

## 关键信息
[提取的关键内容]

## 适用场景
- [场景A]
- [场景B]

## 关联决策
- [ADR-002]

## 状态
- [x] 已验证
- [ ] 已应用
- [ ] 已过期
```

---

## 相关文档

- [ADR 模板](04_reference/document_templates/adr.md)
- [架构设计 Skill](skills/sop-architecture-design/SKILL.md)
- [实现设计 Skill](skills/sop-implementation-designer/SKILL.md)
- [RAG README](04_reference/rag/README.md)
