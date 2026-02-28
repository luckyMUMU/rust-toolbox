---
version: v3.0.0
updated: 2026-02-28
---

# 模板索引

> **用途**: 提供各类文档和契约的标准模板

---

## 概述

本目录存放所有模板文件，包括文档模板、契约模板、报告模板。

---

## 文档模板

**目录**: [documents/](documents/)

| 模板 | 用途 | 使用场景 |
|------|------|----------|
| [proposal.md](documents/proposal.md) | 需求提案模板 | 轻规范路径 |
| [confirmation.md](documents/confirmation.md) | 技术确认模板 | 轻规范路径 |
| [archive.md](documents/archive.md) | 归档记录模板 | 阶段 4 |
| [design.md](documents/design.md) | 实现设计模板 | 阶段 1 |

---

## 契约模板

**目录**: [contracts/](contracts/)

| 模板 | 用途 | 使用场景 |
|------|------|----------|
| [stage-contract.yaml](contracts/stage-contract.yaml) | 阶段契约模板 | 所有阶段 |
| [api-contract.yaml](contracts/api-contract.yaml) | API 契约模板 | P2 级规范 |
| [data-model.yaml](contracts/data-model.yaml) | 数据模型模板 | P2 级规范 |

---

## 报告模板

**目录**: [reports/](reports/)

| 模板 | 用途 | 使用场景 |
|------|------|----------|
| [review-report.md](reports/review-report.md) | 审查报告模板 | 代码/架构审查 |
| [constraint-report.md](reports/constraint-report.md) | 约束验证报告模板 | 约束验证 |

---

## 使用方法

1. 复制模板文件到目标目录
2. 替换 `{占位符}` 为实际内容
3. 删除不需要的部分
4. 确保符合相关约束

---

## 版本历史

| 版本 | 日期 | 变更说明 |
|------|------|----------|
| v3.0.0 | 2026-02-28 | 初始版本，模板索引 |

---

**文档所有者**: 文档团队  
**最后审核**: 2026-02-28
