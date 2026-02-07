# 架构设计模板

**位置**: `docs/02_logical_workflow/[module].pseudo`  
**创建者**: Prometheus

---

## 结构

```markdown
# [模块] 逻辑设计

## 1. 核心概念
- 定义: [一句话]
- 痛点: [问题列表]
- 概念: [术语定义]

## 2. 逻辑流程
```pseudo
FUNCTION main(input):
    VALIDATE input
    IF condition:
        result = process_a(input)
    ELSE:
        result = process_b(input)
    RETURN result
END FUNCTION
```

## 3. 接口定义
### 输入
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| field | String | 是 | [说明] |

### 输出
| 字段 | 类型 | 说明 |
|------|------|------|
| result | Object | [说明] |

### 错误码
| 码 | 说明 | 处理 |
|----|------|------|
| E001 | [说明] | [处理] |

## 4. 设计决策
| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| [主题] | A/B | [选择] | [理由] |
```

---

## 伪代码规范

- **原子操作**: `UPPER_SNAKE_CASE` (例: `VALIDATE_INPUT`)
- **函数**: `lower_snake_case` (例: `process_data`)
- **缩进**: 4空格
- **注释**: 说明"为什么"

---

## 控制结构

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
