---
version: v2.4.0
updated: 2026-02-22
---

# 测试用例 CSV 规范（简版）

目的：`sop-test-design-csv` 以 CSV 作为测试设计载体（TDD 路径），`sop-test-implementation` 只读 CSV 并实现测试代码。

---

## Header（必填）

```text
# version: v[major.minor.patch]
# updated: YYYY-MM-DD
# change: [brief]
# owner: sop-test-design-csv
```

## Columns（固定）

```csv
ID,模块,功能点,测试场景,前置条件,输入数据,预期输出,优先级,类型,状态,关联L2原子操作,版本,更新日期,备注
```

## 单行规则

- 输入数据/预期输出：JSON 字符串（单行）
- 优先级：P0/P1/P2
- 类型：正向/异常/边界
- 状态：待实现/已实现/待更新

