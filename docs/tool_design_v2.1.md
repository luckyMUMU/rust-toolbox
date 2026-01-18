# R-Flow 工具矩阵：原子化拆解与组合定义

## 1. 全量工具设计清单

| 工具 ID | 类别 | 级别 | 功能简述 |
| --- | --- | --- | --- |
| **`ac-manager`** | 算法 | **简单** | **AC自动机管理器**：负责初始化、构建 Fail 指针及内存生命周期管理。使用DAT算法 |
| **`ac-pattern-pusher`** | 算法 | **简单** | **模式串加载器**：向指定自动机实例中动态添加或删除敏感词/模式串。 |
| **`ac-matcher`** | 算法 | **简单** | **字符串匹配器**：执行多模式搜索，返回匹配位置及元数据。 |
| **`sensitive-word-filter`** | 业务 | **复杂** | **复杂工具**：由 `ac-manager` -> `ac-pattern-pusher` -> `ac-matcher` 编排而成。 |
| **`fs-scanner`** | 文件 | **简单** | **目录扫描器**：仅负责读取磁盘结构并转化为内存中的文件树列表。 |
| **`logic-classifier`** | 文件 | **简单** | **分类逻辑计算**：纯函数，根据预设规则（正则/扩展名）计算每个文件的目标路径。 |
| **`fs-executor`** | 文件 | **简单** | **物理执行器**：负责执行真正的 `move`, `copy` 或 `delete` 磁盘操作（有副作用）。 |
| **`smart-organizer`** | 业务 | **复杂** | **复杂工具**：编排 `fs-scanner` -> `logic-classifier` -> `batch-confirmer` -> `fs-executor`。 |
| **`data-slot-cache`** | 数据 | **简单** | **上下文持久化**：将当前 Context Slot 的快照写入 LanceDB 或 Redis。 |
| **`json-transformer`** | 数据 | **简单** | **结构转换器**：使用 JQ 语法或 Rust 逻辑重组 JSON 数据结构。 |
| **`batch-confirmer`** | 交互 | **简单** | **拦截确认器**：在 TUI 或 MCP 端挂起流程，等待人工授权执行。 |
| **`mcp-bridge`** | 插件 | **简单** | **协议转换桥**：将外部 MCP Server 的 Tool 转换为 R-Flow 内部可调用的 Node。 |
| **`python-runtime`** | 脚本 | **简单** | **Python 隔离环境**：在独立进程/沙箱中运行 Python 脚本段并返回结果。 |
| **`wasm-executor`** | 脚本 | **简单** | **WASM 沙箱**：以接近原生的性能安全运行编译为 WASM 的自定义逻辑。 |

---

## 2. 核心拆解逻辑深度解析

### 2.1 AC 自动机：从“单体”到“服务化”

传统的 AC 自动机实现通常是一个封闭的类。在 R-Flow 中，我们将其拆解为三个步骤，这使得我们可以实现**“一次构建，多次匹配”**或**“动态更新词库”**而无需重启流程。

* **`ac-manager`**: 创建线程安全的 `Arc<AcTrie>`。
* **`ac-pattern-pusher`**: 监听数据库或配置文件，实时更新 `Arc` 中的模式串。
* **`ac-matcher`**: 作为并发执行节点，在 `WHEN` 语义下并行扫描多个大文件。

### 2.2 文件夹整理：计算与执行的分离

原本的 `folder-classifier` 被拆解为：

1. **感知 (Scanner)**：只看文件在哪里。
2. **思考 (Classifier)**：只决定文件该去哪里（输出为 `MovePlan` 对象）。
3. **干预 (Confirmer)**：给人看的中间层，确保 AI 没乱移动系统文件。
4. **执行 (Executor)**：最底层的物理操作。

> **设计优势**：你可以将 `logic-classifier` 替换为 `llm-classifier`（调用大模型分类），而底层的扫描和执行代码一行都不用改。

---

## 3. 编排示例：复杂工具的“乐高式”搭建

以下是使用这些拆解后的简单工具构建的 **“企业级敏感词实时监控流”**：

```yaml
id: "realtime_security_audit"
name: "复杂工具：企业安全审计"
category: "COMPLEX"

chain:
  - THEN:
      - node: ac-manager              # 初始化自动机内核
      - node: ac-pattern-pusher       # 加载最新的非法关键词库
      - WHEN:                         # 并行处理
          - node: fs-scanner          # 扫描文档目录
          - node: mcp-bridge          # 通过 MCP 监听 Slack 消息流
      - node: ac-matcher              # 执行多模匹配
      - SWITCH:                       # 结果判断
          on: "${context.has_violation}"
          to:
            - CASE: 
                value: true
                node: batch-confirmer # 发现违规，挂起并等待人工处置
            - CASE:
                value: false
                node: data-slot-cache # 正常，记录审计日志

```

---

## 4. 后续开发建议

1. **强类型上下文合约**：由于拆解后工具变多，建议为每个简单工具定义 `InputSchema` 和 `OutputSchema`。例如 `ac-matcher` 声明它需要上下文中有 `pattern_trie` 这个键。
2. **生命周期钩子**：利用 Rust 的 `Drop` Trait 为 `ac-manager` 实现自动资源回收，确保在大规模并发下内存不泄露。

