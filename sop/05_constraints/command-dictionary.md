---
version: v3.0.0
updated: 2026-02-28
---

# 命令字典

> **定义**: 工作流中所有命令的统一定义

---

## 工作流命令

### 流程控制命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/start` | 无 | 启动工作流 | `/start` |
| `/pause` | 无 | 暂停工作流 | `/pause` |
| `/resume` | 无 | 恢复工作流 | `/resume` |
| `/cancel` | 无 | 取消工作流 | `/cancel` |
| `/status` | 无 | 查看工作流状态 | `/status` |

---

## 阶段 0 命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/spec-weight` | 需求描述 | 评估规范重量 | `/spec-weight 实现用户登录功能` |
| `/confirm-weight` | heavy/light/fast | 确认规范重量 | `/confirm-weight heavy` |

---

## 阶段 1 命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/propose` | 需求描述 | 创建需求提案 | `/propose 实现用户登录功能` |
| `/design` | 模块名 | 生成设计文档 | `/design user-auth` |
| `/confirm-design` | 无 | 确认设计文档 | `/confirm-design` |
| `/reject-design` | 原因 | 拒绝设计文档 | `/reject-design 需要增加安全设计` |

---

## 阶段 2 命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/implement` | 模块名 | 开始实现 | `/implement user-auth` |
| `/test` | 测试范围 | 运行测试 | `/test unit` |
| `/verify` | 约束层级 | 验证约束 | `/verify p0` |
| `/review` | 无 | 请求代码审查 | `/review` |
| `/approve` | 无 | 批准代码变更 | `/approve` |
| `/reject` | 原因 | 拒绝代码变更 | `/reject 需要优化性能` |

---

## 阶段 3 命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/sync` | 无 | 同步文档 | `/sync` |
| `/deliver` | 无 | 确认交付 | `/deliver` |

---

## 阶段 4 命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/archive` | 无 | 创建归档记录 | `/archive` |
| `/upgrade` | 目标层级 | 升级为重规范 | `/upgrade p1` |
| `/changelog` | 变更类型 | 更新变更日志 | `/changelog added` |

---

## 契约命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/contract-validate` | 契约文件 | 验证契约 | `/contract-validate stage-1-contract.yaml` |
| `/contract-show` | 阶段 ID | 显示契约内容 | `/contract-show stage-1` |

---

## 约束命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/constraint-check` | 约束层级 | 检查约束 | `/constraint-check p0` |
| `/constraint-report` | 无 | 生成约束报告 | `/constraint-report` |

---

## 规范命令

| 命令 | 参数 | 描述 | 示例 |
|------|------|------|------|
| `/spec-create` | 规范名称 | 创建规范文档 | `/spec-create user-auth-spec` |
| `/spec-review` | 规范名称 | 审查规范文档 | `/spec-review user-auth-spec` |
| `/spec-approve` | 规范名称 | 批准规范文档 | `/spec-approve user-auth-spec` |

---

## 快捷命令

| 命令 | 等价操作 | 描述 |
|------|----------|------|
| `/go` | `/start` → `/confirm-weight` → `/confirm-design` → `/approve` → `/deliver` → `/archive` | 快速完成流程 |
| `/redo` | 返回上一阶段 | 重做当前阶段 |
| `/skip` | 跳过当前步骤（仅限快速路径） | 跳过非必需步骤 |

---

## 命令响应格式

### 成功响应

```json
{
  "status": "success",
  "command": "/confirm-design",
  "message": "设计文档已确认",
  "next_action": "进入阶段 2：实现与验证"
}
```

### 失败响应

```json
{
  "status": "failed",
  "command": "/confirm-design",
  "error": "设计文档未通过审查",
  "issues": ["缺少安全设计", "接口定义不完整"],
  "next_action": "请修正设计文档后重新提交"
}
```

---

## 版本历史

| 版本 | 日期 | 变更说明 |
|------|------|----------|
| v3.0.0 | 2026-02-28 | 初始版本，命令字典定义 |

---

**文档所有者**: 质量团队  
**最后审核**: 2026-02-28
