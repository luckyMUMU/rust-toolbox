---
version: v2.9.0
updated: 2026-02-24
---

# 实现设计模板 (L3: 技术规格)

**层级**: L3 - 技术规格  
**位置**: `src/**/design.md` 或 `docs/03_technical_spec/[module].md`  
**创建者**: sop-implementation-designer  
**规范**: 将L2伪代码映射为具体技术实现

---

## 质量门控检查清单

> 在完成本阶段设计后，必须确认以下检查项：

| 检查项 | 通过标准 | 状态 |
|--------|----------|------|
| 任务覆盖完整 | 所有L2原子操作已映射到L3实现 | [ ] |
| 依赖无循环 | 目录依赖图中无循环依赖 | [ ] |
| 任务可独立验证 | 每个任务有明确的验收标准 | [ ] |
| 接口契约完整 | 输入/输出/异常定义完整 | [ ] |
| 技术选型合理 | 有明确的选型理由 | [ ] |

**门控失败处理**：若任一检查项未通过，应记录失败原因并返回修正。

---

## 输入/输出契约定义

### 输入契约

| 契约项 | 说明 |
|--------|------|
| 前置依赖 | L2架构设计文档、需求文档(FRD/MRD) |
| 输入数据 | 业务需求、技术约束、现有代码上下文 |
| 环境依赖 | 开发环境、技术栈版本、外部服务 |

### 输出契约

| 契约项 | 说明 |
|--------|------|
| 输出数据 | design.md文档、接口定义、数据模型 |
| 交付物 | 可执行的任务清单、验收标准 |
| 验收标准 | 门控检查项全部通过、用户确认 |

---

## 文件结构

# [模块] 实现设计

## 1. 技术选型

### 0. 来源与依赖声明
> 必须引用 [Source and Dependency](04_reference/interaction_formats/source_dependency.md) 标准格式

| 组件 | 选择 | 版本 | 理由 |
|------|------|------|------|
| 语言 | [语言] | [版本] | [理由] |
| 框架 | [框架] | [版本] | [理由] |
| 存储 | [数据库] | [版本] | [理由] |
| 缓存 | [缓存] | [版本] | [理由] |

## 2. L2→L3 映射
基于: [L2逻辑设计文档链接]

| L2 原子操作/片段 | L3 实现 | 技术细节 |
|-----------|---------|----------|
| `VALIDATE_INPUT` | `InputValidator.validate()` | 使用 Joi/Yup 验证 |
| `PROCESS_TYPE_A` | `TypeAProcessor.process()` | 异步处理，线程池 |
| `PREPROCESS_DATA` | `DataTransformer.normalize()` | 数据清洗逻辑 |

## 3. 领域模型

### 实体定义
| 实体 | 属性 | 类型 | 约束 |
|------|------|------|------|
| [Entity] | [field] | [type] | [constraint] |

### 类/接口定义
```[language]
interface [Name] {
    [returnType] [method]([params]);
}

class [ClassName] implements [Interface] {
    // 实现L2的某个原子操作
}
```

## 4. 接口契约

### 输入
| 参数 | 类型 | 必填 | 验证规则 | 说明 |
|------|------|------|----------|------|
| [param] | [type] | [Y/N] | [rule] | [desc] |

### 输出
| 返回 | 类型 | 说明 |
|------|------|------|
| [return] | [type] | [desc] |

### 异常
| 类型 | 触发条件 | 处理 | HTTP码 |
|------|----------|------|--------|
| [Type] | [condition] | [handle] | [code] |

## 5. 数据模型

### 数据库表
| 表名 | 字段 | 类型 | 索引 | 说明 |
|------|------|------|------|------|
| [table] | [field] | [type] | [index] | [desc] |

### API 定义
```yaml
# OpenAPI/Swagger 片段
paths:
  /api/[endpoint]:
    [method]:
      parameters:
        - name: [param]
          type: [type]
      responses:
        [code]:
          description: [desc]
```

## 6. 目录依赖

> 仅在多目录项目中填写。定义当前目录与其他目录的依赖关系。

| 依赖目录 | 依赖类型 | 说明 | 状态 |
|----------|----------|------|------|
| [dir_path] | 显式/隐式/父子 | [description] | [DIR_COMPLETED]/[DIR_WORKING]/- |

**依赖类型说明**：
- **显式依赖**：design.md 中声明的依赖接口
- **隐式依赖**：代码中的 import/require
- **父子依赖**：目录层级关系

## 7. 任务清单

### 任务粒度指导

- **建议单任务时长**：不超过 4 小时
- **任务拆分原则**：
  - 每个任务应对应一个可独立验证的产出物
  - 任务间依赖关系清晰，避免循环依赖
  - 高优先级任务优先执行

### 任务规范文件（推荐）

> 对于复杂模块，建议使用与 trae spec 对齐的任务规范文件结构：
> - `spec.md`：任务规范说明
> - `tasks.md`：任务列表与依赖
> - `checklist.md`：验证检查清单
>
> CMD: `TASK_SPEC_CREATE(dir, spec_name)` 创建任务规范文件

### 任务状态说明

| 状态 | 标记 | 含义 | CMD |
|------|------|------|-----|
| 待处理 | `[ ]` | 任务尚未开始 | - |
| 进行中 | `[-]` | 任务正在执行 | `TASK_START(task_id)` |
| 已完成 | `[x]` | 任务已完成并通过验证 | `TASK_COMPLETE(task_id)` |
| 已阻塞 | `[!]` | 任务被阻塞，需外部依赖 | `TASK_BLOCK(task_id, reason)` |
| 已归档 | `[archived]` | 任务已归档，不在活跃清单显示 | `TASK_ARCHIVE(dir)` |

### 活跃任务

| 任务ID | 描述 | 状态 | 依赖 | 产出物 | 完成日期 |
|--------|------|------|------|--------|----------|
| T001 | [任务1: 对应L2的某个原子操作] | `[ ]` | - | [文件名] | - |
| T002 | [任务2] | `[ ]` | T001 | [文件名] | - |
| T003 | [任务3] | `[ ]` | - | [文件名] | - |

### 归档任务

> 归档操作由 `sop-document-sync` 在目录归档时自动执行。当目录状态变为 `[DIR_COMPLETED]` 时，已完成任务（状态 `[x]`）将移至此章节。

| 任务ID | 描述 | 原状态 | 归档日期 | 归档原因 |
|--------|------|--------|----------|----------|
| - | - | - | - | - |

## 8. 测试策略
- 单元测试: [覆盖目标，对应L2逻辑分支]
- 集成测试: [场景]
- 性能测试: [指标]

## 9. 分层验收清单

**验收流程**: L1 → L1审查 → L2 → L2审查 → L3 → L3审查 → L4 → L4审查

### L1 - 单元/函数级别验收

**验收命令**:
```bash
# 根据技术栈选择
pytest tests/acceptance/l1/ -v --cov=src --cov-report=term-missing
```

**验收标准**:
- [ ] 所有单元测试通过
- [ ] 代码覆盖率 >= 80%
- [ ] 无lint错误
- [ ] 无type错误

**审查检查点** (Oracle审查):
- [ ] 接口实现符合本design.md定义
- [ ] 异常处理完整
- [ ] 日志记录规范
- [ ] 单元测试覆盖所有分支

### L2 - 模块级别验收

**验收命令**:
```bash
pytest tests/acceptance/l2/ -v
```

**验收标准**:
- [ ] 模块集成测试通过
- [ ] 模块间接口调用正常
- [ ] 模块性能达标

**审查检查点** (Oracle审查):
- [ ] 模块设计符合本design.md
- [ ] 模块间依赖正确
- [ ] 模块边界清晰
- [ ] 接口契约满足

### L3 - 功能级别验收

**验收命令**:
```bash
pytest tests/acceptance/l3/ -v
```

**验收标准**:
- [ ] 功能验收测试通过
- [ ] 符合FRD需求
- [ ] 用户场景覆盖

**审查检查点** (`sop-code-review` 审查):
- [ ] 功能实现符合本design.md
- [ ] 符合对应FRD需求
- [ ] 用户场景完整覆盖
- [ ] 业务规则正确实现

### L4 - 系统级别验收

**验收命令**:
```bash
pytest tests/acceptance/l4/ -v
```

**验收标准**:
- [ ] 端到端测试通过
- [ ] 系统性能达标
- [ ] 架构约束满足

**审查检查点** (`sop-code-review` 审查):
- [ ] 符合架构设计文档
- [ ] 符合本design.md整体设计
- [ ] 系统级约束满足
- [ ] 性能指标达标
- [ ] 可扩展性满足

## 10. 状态
| 状态 | 阶段 | 日期 |
|------|------|------|
| `[WAITING_FOR_DESIGN]` | [阶段] | [日期] |
| `[USER_DECISION]` | [阶段] | [日期] |
| `[已完成]` | [阶段] | [日期] |

---

## L3层约束

✅ **必须**:
- 明确技术栈（语言、框架、版本）
- 映射L2伪代码到具体实现
- 定义数据模型和接口契约
- 包含可执行的任务清单

❌ **禁止**:
- 重复描述L2已定义的逻辑
- 写具体业务代码实现
- 遗漏与L2的映射关系

---

## L2→L3 映射示例

### L2 伪代码
```pseudo
FUNCTION process_order(order):
    VALIDATE_ORDER order
    
    IF order.amount > 1000:
        REQUIRE_APPROVAL order
    END IF
    
    SAVE_ORDER order
    SEND_NOTIFICATION order.user_id
    
    RETURN order.id
END FUNCTION
```

### L3 实现映射
| L2 原子操作 | L3 实现 | 技术选择 |
|-------------|---------|----------|
| `VALIDATE_ORDER` | `OrderValidator.validate()` | Class-validator + DTO |
| `REQUIRE_APPROVAL` | `ApprovalService.request()` | 状态机 + 消息队列 |
| `SAVE_ORDER` | `OrderRepository.save()` | TypeORM + PostgreSQL |
| `SEND_NOTIFICATION` | `NotificationService.send()` | RabbitMQ + WebSocket |

---

## 与L2/L4的关系

| 层级 | 文件 | 内容 | 产出 Skill |
|------|------|------|--------|
| L2 | `.md` | 逻辑工作流 | sop-architecture-design |
| L3 | `design.md` | 技术规格 | sop-implementation-designer |
| L4 | `adr_*.md` | 决策背景 | sop-architecture-design / sop-implementation-designer |

👉 L3 必须引用对应的 L2 文档  
👉 L3 的关键技术决策可记录到 L4 ADR
