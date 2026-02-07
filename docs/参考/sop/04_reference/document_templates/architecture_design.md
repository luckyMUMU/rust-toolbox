# 架构设计文档模板

**使用角色**: Prometheus  
**文档位置**: `docs/02_logical_workflow/[module_name].pseudo`  
**编写原则**: 技术无关、逻辑抽象、可复用

---

## 文档结构

```markdown
# [模块名称] 逻辑设计

## 1. 核心概念
## 2. 逻辑流程 (伪代码)
## 3. 接口定义
## 4. 设计决策
```

---

## 完整模板

```markdown
# [模块名称] 逻辑设计

---

## 1. 核心概念

### 1.1 一句话定义
[用一句话概括该模块的核心功能]

### 1.2 解决痛点
- **问题1**: [描述问题]
- **问题2**: [描述问题]

### 1.3 核心概念定义
- **[概念A]**: [定义说明]
- **[概念B]**: [定义说明]

---

## 2. 逻辑流程 (伪代码)

### 2.1 主流程
```pseudo
FUNCTION main_process(input):
    INITIALIZE context
    VALIDATE input_format
    
    IF input.type == TYPE_A:
        result = process_type_a(input)
    ELSE IF input.type == TYPE_B:
        result = process_type_b(input)
    ELSE:
        RAISE invalid_type_error
    END IF
    
    RETURN result
END FUNCTION
```

### 2.2 子流程 A
```pseudo
FUNCTION process_type_a(input):
    PRE_PROCESS input
    TRANSFORM data
    VALIDATE output
    RETURN output
END FUNCTION
```

### 2.3 异常处理
```pseudo
TRY:
    result = main_process(input)
CATCH validation_error:
    LOG error
    RETURN error_response
CATCH system_error:
    LOG critical_error
    TRIGGER alert
    RAISE
END TRY
```

---

## 3. 接口定义

### 3.1 输入
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| field_a | String | 是 | [说明] |
| field_b | Integer | 否 | [说明] |

### 3.2 输出
| 字段 | 类型 | 说明 |
|------|------|------|
| result | Object | [说明] |
| status | String | [说明] |

### 3.3 错误码
| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| E001 | [错误说明] | [处理建议] |
| E002 | [错误说明] | [处理建议] |

---

## 4. 设计决策 (ADR)

### 4.1 决策: [决策主题]

**背景**: [决策背景]

**选项**:
- **选项A**: [描述]
- **选项B**: [描述]

**决策**: 选择 [选项]

**理由**:
- [理由1]
- [理由2]

**风险**:
- [风险1] → [缓解措施]
- [风险2] → [缓解措施]

**影响**:
- [影响1]
- [影响2]

---

## 5. 非功能性需求

### 5.1 性能要求
- **响应时间**: [要求]
- **吞吐量**: [要求]
- **并发数**: [要求]

### 5.2 可靠性要求
- **可用性**: [要求]
- **容错性**: [要求]

### 5.3 扩展性
- [扩展方向1]
- [扩展方向2]

---

## 6. 依赖关系

### 6.1 内部依赖
- [模块X]: [依赖说明]
- [模块Y]: [依赖说明]

### 6.2 外部依赖
- [外部系统/库]: [依赖说明]

---

## 7. 状态记录

- `[设计中]` | [描述] | [日期]
- `[已完成]` | [历史记录]

---

## 附录

### A. 术语表
- **[术语]**: [定义]

### B. 参考资料
- [参考链接1]
- [参考链接2]
```

---

## 伪代码规范

### 命名规范
- **原子操作**: 使用 `UPPER_SNAKE_CASE`
  - 例: `VALIDATE_INPUT`, `FETCH_DATA`
- **函数**: 使用 `lower_snake_case`
  - 例: `process_data`, `calculate_result`

### 控制结构
```pseudo
IF condition:
    action
ELSE IF other_condition:
    other_action
ELSE:
    default_action
END IF

FOR EACH item IN collection:
    process(item)
END FOR

WHILE condition:
    action
END WHILE

TRY:
    risky_operation
CATCH error:
    handle_error
FINALLY:
    cleanup
END TRY
```

### 注释规范
- 说明"为什么"而非"是什么"
- 复杂逻辑需要注释
- 关键决策点需要注释

---

## 质量检查清单

- [ ] 技术无关性：不绑定特定技术栈
- [ ] 完整性：覆盖所有核心逻辑
- [ ] 清晰性：伪代码易于理解
- [ ] 一致性：术语和概念一致
- [ ] 可追溯性：决策有明确理由

---

👉 [返回参考文档首页](../index.md)
