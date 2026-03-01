# Workflow Toolkit 工作流编写指南

## 文档元数据

- **版本**: v1.0.0
- **最后更新**: 2026-03-01
- **状态**: 已批准

## 1. 工作流概述

### 1.1 什么是工作流

工作流是由节点和边组成的有向图，定义了任务的执行顺序和条件。Workflow Toolkit 使用 YAML 格式定义工作流。

### 1.2 工作流结构

```yaml
apiVersion: workflow.kit/v1
kind: Workflow
metadata:
  name: workflow-name
  description: 工作流描述
  version: "1.0.0"
spec:
  nodes:
    - id: node1
      # 节点配置
    - id: node2
      # 节点配置
  edges:
    - source: node1
      target: node2
```

## 2. 基本概念

### 2.1 节点 (Node)

节点是工作流的基本执行单元，代表一个具体的操作。

**节点类型**:
| 类型 | 描述 |
|------|------|
| tool | 工具节点，执行一个工具 |
| condition | 条件节点，根据条件选择分支 |
| loop | 循环节点，重复执行一组节点 |
| parallel | 并行节点，并行执行多个分支 |
| switch | 切换节点，根据值选择分支 |
| checkpoint | 检查点节点，保存执行状态 |

### 2.2 边 (Edge)

边定义了节点之间的连接关系和执行顺序。

```yaml
edges:
  - source: node1
    target: node2
    condition: "${node1.result == 'success'}"  # 可选条件
```

### 2.3 数据上下文

工作流执行过程中维护一个数据上下文，用于存储和传递数据。

**访问数据**:
- `${input.xxx}` - 访问输入参数
- `${node_id.result}` - 访问节点执行结果
- `${context.xxx}` - 访问上下文变量

## 3. 工具节点

### 3.1 基本工具节点

```yaml
- id: process
  name: 处理数据
  type: tool
  config:
    tool_name: text_processor
    parameters:
      text: "${input.text}"
      operation: uppercase
```

### 3.2 带超时的工具节点

```yaml
- id: slow_process
  name: 慢处理
  type: tool
  config:
    tool_name: slow_processor
    parameters:
      data: "${input.data}"
    timeout: 300s
```

### 3.3 带重试的工具节点

```yaml
- id: unreliable_process
  name: 不可靠处理
  type: tool
  config:
    tool_name: unreliable_processor
    parameters:
      data: "${input.data}"
    retry:
      max_retries: 3
      backoff: exponential
      initial_delay: 1s
```

## 4. 条件节点

### 4.1 基本条件节点

```yaml
- id: check_result
  name: 检查结果
  type: condition
  config:
    expression: "${process.result == 'success'}"
    true_branch: success_handler
    false_branch: failure_handler
```

### 4.2 复杂条件表达式

```yaml
- id: complex_check
  name: 复杂检查
  type: condition
  config:
    expression: "${process.score > 80 && process.status == 'completed'}"
    true_branch: high_score_handler
    false_branch: low_score_handler
```

### 4.3 条件表达式语法

| 操作符 | 描述 | 示例 |
|--------|------|------|
| `==` | 等于 | `${a == b}` |
| `!=` | 不等于 | `${a != b}` |
| `>` | 大于 | `${a > b}` |
| `<` | 小于 | `${a < b}` |
| `>=` | 大于等于 | `${a >= b}` |
| `<=` | 小于等于 | `${a <= b}` |
| `&&` | 逻辑与 | `${a && b}` |
| `\|\|` | 逻辑或 | `${a \|\| b}` |
| `!` | 逻辑非 | `${!a}` |
| `contains` | 包含 | `${list contains item}` |
| `matches` | 正则匹配 | `${text matches '^\\d+$'}` |

## 5. 循环节点

### 5.1 基本循环节点

```yaml
- id: process_items
  name: 处理项目列表
  type: loop
  config:
    items: "${input.items}"
    item_var: item
    index_var: index
    body:
      - id: process_item
        type: tool
        config:
          tool_name: item_processor
          parameters:
            item: "${item}"
            index: "${index}"
```

### 5.2 并行循环

```yaml
- id: parallel_process
  name: 并行处理
  type: loop
  config:
    items: "${input.items}"
    item_var: item
    parallel: true
    max_parallel: 5
    body:
      - id: process_item
        type: tool
        config:
          tool_name: item_processor
          parameters:
            item: "${item}"
```

### 5.3 循环控制

```yaml
- id: controlled_loop
  name: 受控循环
  type: loop
  config:
    items: "${input.items}"
    item_var: item
    body:
      - id: check_item
        type: condition
        config:
          expression: "${item.status == 'skip'}"
          true_branch: skip_item
          false_branch: process_item
      - id: skip_item
        type: tool
        config:
          tool_name: skip_handler
          parameters:
            item: "${item}"
          # 返回 ComponentOutput::Skip
      - id: process_item
        type: tool
        config:
          tool_name: item_processor
          parameters:
            item: "${item}"
```

## 6. 并行节点

### 6.1 基本并行节点

```yaml
- id: parallel_tasks
  name: 并行任务
  type: parallel
  config:
    branches:
      - name: branch1
        nodes:
          - id: task1
            type: tool
            config:
              tool_name: task1_processor
      - name: branch2
        nodes:
          - id: task2
            type: tool
            config:
              tool_name: task2_processor
      - name: branch3
        nodes:
          - id: task3
            type: tool
            config:
              tool_name: task3_processor
```

### 6.2 等待策略

```yaml
- id: parallel_with_strategy
  name: 带策略的并行
  type: parallel
  config:
    wait_strategy: any  # all | any | n
    wait_count: 2       # n 策略时使用
    branches:
      - name: fast_task
        nodes:
          - id: fast
            type: tool
            config:
              tool_name: fast_processor
      - name: slow_task
        nodes:
          - id: slow
            type: tool
            config:
              tool_name: slow_processor
```

**等待策略说明**:
| 策略 | 描述 |
|------|------|
| `all` | 等待所有分支完成（默认） |
| `any` | 任一分支完成即继续 |
| `n` | 指定数量的分支完成即继续 |

## 7. 切换节点

### 7.1 基本切换节点

```yaml
- id: route_by_type
  name: 按类型路由
  type: switch
  config:
    expression: "${input.type}"
    cases:
      - value: "text"
        target: text_handler
      - value: "image"
        target: image_handler
      - value: "video"
        target: video_handler
    default: unknown_handler
```

### 7.2 复杂切换

```yaml
- id: complex_switch
  name: 复杂切换
  type: switch
  config:
    expression: "${process.category}"
    cases:
      - value: "A"
        target: category_a_handler
      - value: "B"
        target: category_b_handler
      - value: "C"
        target: category_c_handler
    default: default_handler
```

## 8. 检查点节点

### 8.1 基本检查点

```yaml
- id: save_checkpoint
  name: 保存检查点
  type: checkpoint
  config:
    name: after_data_load
    save_context: true
```

### 8.2 从检查点恢复

```bash
workflow-toolkit workflow execute --resume exec-123 --checkpoint after_data_load workflow.yaml
```

## 9. 工作流配置

### 9.1 并发配置

```yaml
spec:
  concurrency:
    max_parallel_nodes: 10
    max_parallel_tools: 5
```

### 9.2 重试配置

```yaml
spec:
  retry:
    max_retries: 3
    backoff: exponential
    initial_delay: 1s
    max_delay: 60s
    retryable_errors:
      - "TIMEOUT"
      - "NETWORK_ERROR"
```

### 9.3 超时配置

```yaml
spec:
  timeout: 3600s
```

### 9.4 检查点配置

```yaml
spec:
  checkpoint:
    enabled: true
    interval: 60s
    directory: ./checkpoints
```

### 9.5 资源限制

```yaml
spec:
  resource_limits:
    max_memory: 1GB
    max_cpu: 2.0
    max_execution_time: 3600s
```

## 10. 完整示例

### 10.1 数据处理工作流

```yaml
apiVersion: workflow.kit/v1
kind: Workflow
metadata:
  name: data-processing
  description: 数据处理工作流
  version: "1.0.0"
  tags:
    - data
    - processing
spec:
  concurrency:
    max_parallel_nodes: 5
  retry:
    max_retries: 3
    backoff: exponential
  timeout: 1800s
  checkpoint:
    enabled: true
    interval: 120s
  
  nodes:
    # 1. 加载数据
    - id: load_data
      name: 加载数据
      type: tool
      config:
        tool_name: data_loader
        parameters:
          source: "${input.source}"
          format: "${input.format}"
    
    # 2. 检查点
    - id: checkpoint_after_load
      name: 数据加载检查点
      type: checkpoint
      config:
        name: after_load
        save_context: true
    
    # 3. 验证数据
    - id: validate_data
      name: 验证数据
      type: tool
      config:
        tool_name: data_validator
        parameters:
          data: "${load_data.result}"
    
    # 4. 条件分支
    - id: check_validation
      name: 检查验证结果
      type: condition
      config:
        expression: "${validate_data.valid == true}"
        true_branch: process_data
        false_branch: handle_error
    
    # 5. 处理数据
    - id: process_data
      name: 处理数据
      type: loop
      config:
        items: "${load_data.result.items}"
        item_var: item
        parallel: true
        max_parallel: 3
        body:
          - id: transform_item
            name: 转换项目
            type: tool
            config:
              tool_name: item_transformer
              parameters:
                item: "${item}"
    
    # 6. 合并结果
    - id: merge_results
      name: 合并结果
      type: tool
      config:
        tool_name: result_merger
        parameters:
          results: "${process_data.results}"
    
    # 7. 导出结果
    - id: export_results
      name: 导出结果
      type: tool
      config:
        tool_name: data_exporter
        parameters:
          data: "${merge_results.result}"
          destination: "${input.destination}"
    
    # 错误处理
    - id: handle_error
      name: 处理错误
      type: tool
      config:
        tool_name: error_handler
        parameters:
          error: "${validate_data.errors}"
  
  edges:
    - source: load_data
      target: checkpoint_after_load
    - source: checkpoint_after_load
      target: validate_data
    - source: validate_data
      target: check_validation
    - source: check_validation
      target: process_data
      condition: "${validate_data.valid == true}"
    - source: check_validation
      target: handle_error
      condition: "${validate_data.valid == false}"
    - source: process_data
      target: merge_results
    - source: merge_results
      target: export_results
```

### 10.2 审批工作流

```yaml
apiVersion: workflow.kit/v1
kind: Workflow
metadata:
  name: approval-workflow
  description: 审批工作流
  version: "1.0.0"
spec:
  nodes:
    # 1. 提交申请
    - id: submit_request
      name: 提交申请
      type: tool
      config:
        tool_name: request_submitter
        parameters:
          applicant: "${input.applicant}"
          content: "${input.content}"
    
    # 2. 人工审批
    - id: human_approval
      name: 人工审批
      type: tool
      config:
        tool_name: human_decision
        parameters:
          request_id: "${submit_request.request_id}"
          approvers: "${input.approvers}"
          timeout: 86400s
    
    # 3. 检查审批结果
    - id: check_approval
      name: 检查审批结果
      type: condition
      config:
        expression: "${human_approval.decision == 'approved'}"
        true_branch: process_approved
        false_branch: process_rejected
    
    # 4. 处理批准
    - id: process_approved
      name: 处理批准
      type: tool
      config:
        tool_name: approval_processor
        parameters:
          request_id: "${submit_request.request_id}"
    
    # 5. 处理拒绝
    - id: process_rejected
      name: 处理拒绝
      type: tool
      config:
        tool_name: rejection_processor
        parameters:
          request_id: "${submit_request.request_id}"
          reason: "${human_approval.reason}"
  
  edges:
    - source: submit_request
      target: human_approval
    - source: human_approval
      target: check_approval
    - source: check_approval
      target: process_approved
      condition: "${human_approval.decision == 'approved'}"
    - source: check_approval
      target: process_rejected
      condition: "${human_approval.decision == 'rejected'}"
```

## 11. 最佳实践

### 11.1 命名规范

- 节点 ID 使用 snake_case
- 节点名称使用中文描述
- 工作流名称使用 kebab-case

### 11.2 错误处理

```yaml
# 使用条件节点处理错误
- id: check_result
  type: condition
  config:
    expression: "${previous_node.status == 'success'}"
    true_branch: success_handler
    false_branch: error_handler
```

### 11.3 性能优化

```yaml
# 使用并行处理提高性能
- id: parallel_process
  type: parallel
  config:
    wait_strategy: all
    branches:
      - name: branch1
        nodes: [...]
      - name: branch2
        nodes: [...]
```

### 11.4 可维护性

```yaml
# 使用检查点便于恢复
- id: checkpoint
  type: checkpoint
  config:
    name: important_checkpoint
    save_context: true

# 添加清晰的描述
metadata:
  description: |
    这是一个数据处理工作流，包含以下步骤：
    1. 加载数据
    2. 验证数据
    3. 处理数据
    4. 导出结果
```

## 12. 变更历史

| 版本 | 日期 | 变更内容 | 变更人 |
|------|------|----------|--------|
| v1.0.0 | 2026-03-01 | 初始版本 | Architecture Team |
