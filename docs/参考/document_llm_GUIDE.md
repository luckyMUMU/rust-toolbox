基于伪代码逻辑与分级目录的 LLM 技术规范

---

### 一、 分级存储架构 (Storage Hierarchy)

文档不再是一个巨大的 `README.md`，而是根据“披露深度”拆分为不同的文件/文件夹：

Plaintext

```
docs/
├── 01_concept_overview.md   # L1: 核心概念与价值（极简文字）
├── 02_logical_workflow/     # L2: 逻辑工作流（本层级只写伪代码）
│   ├── auth_flow.pseudo     # 鉴权逻辑伪代码
│   └── data_pipeline.pseudo  # 数据处理逻辑伪代码
├── 03_technical_spec/       # L3: 技术规格（接口、数据结构定义）
│   ├── api_schema.yaml      # OpenAPI/Swagger 定义
│   └── data_models.json     # 数据模型定义
└── 04_context_reference/    # L4: 决策背景与长尾细节
    └── architecture_decision.md # 架构设计决策记录 (ADR)
```

---

### 二、 每一层的内容分配规范

#### L1: 核心概念层 (Concept Layer)

- **存放位置：** 根目录 `README.md` 或 `01_overview.md`。
    
- **内容：** * 一句话定义该模块。
    
    - 解决的核心痛点。
        
    - **禁止：** 出现任何代码、路径或配置。
        

#### L2: 逻辑流转层 (Logic Layer) —— **核心改进点**

- **存放位置：** `02_logical_workflow/` 文件夹。
    
- **编写原则：使用“结构化伪代码”**。
    
- **规范：**
    
    - **忽略语法：** 不使用特定的编程语言（如 Python 或 JS），使用 `IF`, `THEN`, `FOR EACH`, `TRY-CATCH` 等通用描述。
        
    - **屏蔽实现：** 不要写 `db.connect()`，而是写 `CONNECT_TO_DATABASE`。
        
    - **示例：**
        
        Plaintext
        
        ```
        // 伪代码示例：用户意图识别流程
        FUNCTION identify_intent(user_input):
            INITIALIZE context_buffer
            PRE_PROCESS user_input (remove_noise, normalize)
        
            IF user_input contains "urgent":
                SET priority = HIGH
            ELSE:
                SET priority = NORMAL
        
            RETURN CALL_LLM_MODEL(prompt_template, user_input)
        END FUNCTION
        ```
        

#### L3: 技术规格层 (Spec Layer)

- **存放位置：** `03_technical_spec/`。
    
- **内容：** * **数据合同：** 定义输入/输出的具体字段、类型（Int, String, Boolean）。
    
    - **状态码定义：** 逻辑流转中可能出现的错误代码及其含义。
        
    - **禁止：** 描述业务逻辑，只定义数据边界。
        

#### L4: 决策参考层 (Context Layer)

- **存放位置：** `04_context_reference/`。
    
- **内容：** * **Why：** 为什么选择伪代码中的逻辑 A 而不是 B？
    
    - **限制：** 当前逻辑在并发超过 1000 时可能失效。
        
    - **历史：** 旧版本逻辑的废弃原因。
        

---

### 三、 伪代码编写的三项准则

为了确保伪代码既能被人读懂，也能被 LLM 精准解析，建议遵循以下标准：

1. **原子化操作命名：** 使用 `UPPER_SNAKE_CASE` 表示底层原子操作（如 `FETCH_USER_PROFILE`），这些操作在实际代码中可能对应一个复杂的函数。
    
2. **缩进体现分层：** 严格使用 4 空格缩进，表示逻辑的嵌套关系，这有助于 LLM 构建抽象语法树。
    
3. **注释意图而非步骤：** 注释应说明“为什么这里需要分支”，而不是解释“这是一个 If 语句”。
    

---

### 四、 这种规范的优势

1. **架构与实现解耦：** 即使你的项目从 Python 迁移到了 Go，`02_logical_workflow` 下的伪代码文档完全不需要修改。
    
2. **降低 LLM 的 Token 噪音：** LLM 在学习你的技术文档时，不需要被大量的 `import`, `try...except`, `logger.info` 干扰，能更直接地捕获业务逻辑。
    
3. **渐进式认知：** * 开发者想看逻辑？去 L2 看伪代码。
    
    - 开发者要写代码？去 L3 看字段定义。
        
    - 开发者遇到 Bug？去 L4 看设计背景。
    