# 角色速查卡

> **使用对象**: AI Agent | **目标**: 快速理解角色职责

---

## 角色索引

| 角色 | 层级 | 一句话职责 | 详情 |
|------|------|-----------|------|
| Router | 规划 | 任务分诊与路径选择 | [→ Router](#router) |
| Explorer | 规划 | 代码审计与影响评估 | [→ Explorer](#explorer) |
| Analyst | 需求 | 需求分析与PRD生成 | [→ Analyst](#analyst) |
| Prometheus | 设计 | 架构设计与伪代码 | [→ Prometheus](#prometheus) |
| Skeptic | 设计 | 架构审查与质量把关 | [→ Skeptic](#skeptic) |
| Oracle | 设计 | 实现设计与技术选型 | [→ Oracle](#oracle) |
| Worker | 实现 | 编码实现与质量检查 | [→ Worker](#worker) |
| Librarian | 监管 | 文档维护与渐进披露 | [→ Librarian](#librarian) |
| Supervisor | 监管 | 进度监管与熔断决策 | [→ Supervisor](#supervisor) |

---

## Router

**层级**: 规划 | **触发**: 接收任务

| 属性 | 值 |
|------|-----|
| 输入 | 用户请求 |
| 输出 | 路径选择(快速/深度) |
| 下一步 | @对应角色 |

**决策规则**:
```
单文件+<30行+无逻辑变更 → 快速路径(Explorer→Worker→Librarian)
其他 → 深度路径(Analyst→...)
```

**完整提示词**: [prompts/router_prompt.md](prompts/router_prompt.md)

---

## Explorer

**层级**: 规划 | **触发**: 需审计

| 属性 | 值 |
|------|-----|
| 输入 | 代码库 |
| 输出 | 审计报告 |
| 下一步 | @对应角色 |

**关键动作**: 分析代码 → 评估影响面 → 识别风险

**完整定义**: [02_role_matrix/explorer.md](02_role_matrix/explorer.md)

---

## Analyst

**层级**: 需求 | **触发**: 需求不清

| 属性 | 值 |
|------|-----|
| 输入 | 用户对话 |
| 输出 | PRD文档 |
| 下一步 | @Prometheus |

**关键动作**:
1. 多轮对话澄清需求
2. 6维度分析(业务/用户/功能/技术/风险/验收)
3. 生成PRD: `docs/01_requirements/*.md`
4. 用户确认

**停止点**: `[WAITING_FOR_REQUIREMENTS]`

**完整提示词**: [prompts/analyst_prompt.md](prompts/analyst_prompt.md)

---

## Prometheus

**层级**: 设计 | **触发**: 需架构设计

| 属性 | 值 |
|------|-----|
| 输入 | PRD |
| 输出 | 架构文档 |
| 下一步 | @Skeptic |

**关键动作**:
1. 技术无关设计
2. 编写伪代码: `docs/02_logical_workflow/*.pseudo`
3. 定义接口

**停止点**: `[WAITING_FOR_ARCHITECTURE]`

**完整提示词**: [prompts/prometheus_prompt.md](prompts/prometheus_prompt.md)

---

## Skeptic

**层级**: 设计 | **触发**: 架构完成

| 属性 | 值 |
|------|-----|
| 输入 | 架构文档 |
| 输出 | 审查报告 |
| 下一步 | @Prometheus(继续) / @Oracle(通过) |

**关键动作**:
1. 6维度审查(完整性/一致性/可实现性/性能/安全/扩展性)
2. 提出问题(🔴严重/🟡一般/🟢建议)
3. 评估回复

**循环**: 最多3轮
**停止点**: `[ARCHITECTURE_PASSED]` / `[USER_DECISION]`

**完整提示词**: [prompts/skeptic_prompt.md](prompts/skeptic_prompt.md)

---

## Oracle

**层级**: 设计 | **触发**: 需实现设计

| 属性 | 值 |
|------|-----|
| 输入 | 架构文档 |
| 输出 | 实现设计 |
| 下一步 | @Worker |

**关键动作**:
1. 技术选型
2. 任务分解
3. 编写实现设计: `src/**/design.*`

**停止点**: `[WAITING_FOR_DESIGN]`

**完整提示词**: [prompts/oracle_prompt.md](prompts/oracle_prompt.md)

---

## Worker

**层级**: 实现 | **触发**: 需编码

| 属性 | 值 |
|------|-----|
| 输入 | 实现设计 |
| 输出 | 代码 |
| 下一步 | @Librarian |

**关键动作**:
1. 按设计编码
2. 运行测试
3. 质量检查

**约束**: 三错即停
**停止点**: 展示Diff

**完整提示词**: [prompts/worker_prompt.md](prompts/worker_prompt.md)

---

## Librarian

**层级**: 监管 | **触发**: 文档变更

| 属性 | 值 |
|------|-----|
| 输入 | 各类文档 |
| 输出 | 更新索引 |
| 下一步 | - |

**关键动作**:
1. 更新父级索引
2. 维护链接
3. 渐进披露(父目录只保留摘要+链接)
4. 状态标记: `[进行中]`→`[已完成]`

**完整定义**: [02_role_matrix/librarian.md](02_role_matrix/librarian.md)

---

## Supervisor

**层级**: 监管 | **触发**: 异常/进度报告

| 属性 | 值 |
|------|-----|
| 输入 | 全局状态 |
| 输出 | 报告/决策 |
| 下一步 | 用户 |

**关键动作**:
1. 监控进度
2. 检测异常
3. 触发熔断(Strike 3)

**熔断条件**: Worker连续3次失败

**完整定义**: [02_role_matrix/supervisor.md](02_role_matrix/supervisor.md)

---

## 角色关系图

```
Router(入口)
  ├─ 快速路径 → Explorer → Worker → Librarian
  └─ 深度路径 → Analyst → Prometheus ↔ Skeptic → Oracle → Worker → Librarian
                                              ↓
                                         Supervisor(全局监管)
```

---

## 文档类型速查

| 类型 | 位置 | 创建者 | 特点 |
|------|------|--------|------|
| PRD | `docs/01_requirements/*.md` | Analyst | 业务导向 |
| 架构 | `docs/02_logical_workflow/*.pseudo` | Prometheus | 技术无关 |
| 实现 | `src/**/design.*` | Oracle | 项目特定 |

---

**紧凑版**: [AGENT_SOP_COMPACT.md](AGENT_SOP_COMPACT.md) | **完整文档**: [AGENT_SOP.md](AGENT_SOP.md)
