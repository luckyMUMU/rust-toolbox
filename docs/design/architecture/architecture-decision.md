# Architecture Decision Records (ADR)（架构决策记录）

> **项目**: Workflow Toolkit  
> **版本**: v0.1.0  
> **最后更新**: 2026-02-05

---

## ADR-001: 采用 Rust 作为核心开发语言

### Status（状态）
Accepted（已接受）

### Context（背景）
需要选择一种系统编程语言来实现高性能的工作流引擎，要求：
- 高并发处理能力
- 内存安全保证
- 优秀的性能表现
- 丰富的生态系统

### Decision（决策）
选择 **Rust** 作为核心开发语言。

### Rationale（理由）
1. **Memory Safety（内存安全）**: 所有权系统（Ownership System）消除数据竞争和空指针问题
2. **Zero-Cost Abstractions（零成本抽象）**: 高性能同时保持代码可读性
3. **Concurrency Safety（并发安全）**: 编译时保证线程安全
4. **Ecosystem（生态系统）**: 丰富的异步运行时（Tokio）、序列化（Serde）等库
5. **Cross-Platform（跨平台）**: 支持 Windows、Linux、macOS

### Alternatives（替代方案）
- **Go**: 开发速度快，但性能略低，缺乏内存安全保证
- **C++**: 性能优秀，但内存安全问题多，开发效率低
- **Java**: 生态成熟，但启动慢，内存占用高

### Consequences（后果）
- ✅ 高性能、低延迟
- ✅ 内存安全，减少运行时错误
- ✅ 优秀的并发处理能力
- ❌ 学习曲线陡峭
- ❌ 开发速度相对较慢

---

## ADR-002: 采用 Enum（枚举）而非 Trait 实现工具系统

### Status（状态）
Accepted（已接受）

### Context（背景）
工具系统需要支持多种类型的工具（Native、Python、Node.js、Docker、WASM），需要决定使用 Trait 还是 Enum（枚举）来实现。

### Decision（决策）
采用 **Enum（枚举）** 方式实现工具系统，而非 Trait Object（Trait 对象）。

### Rationale（理由）
1. **Performance（性能）**: Enum 避免动态分发（Dynamic Dispatch）开销
2. **Type Safety（类型安全）**: 编译时确定所有可能的工具类型
3. **Serialization（序列化）**: Enum 更容易序列化和反序列化
4. **Pattern Matching（模式匹配）**: 清晰的 `match` 语法处理不同类型

```rust
// 采用的方式
pub enum Tool {
    Native(Arc<NativeTool>),
    Python(Arc<PythonTool>),
    NodeJs(Arc<NodeJsTool>),
    Docker(Arc<DockerTool>),
    Wasm(Arc<WasmTool>),
    Composed(Arc<ComposedTool>),
}
```

### Alternatives（替代方案）
- **Trait Object（Trait 对象）**: 更灵活，但性能开销大，难以序列化
- **Generics（泛型）**: 编译时膨胀，不适合动态工具加载

### Consequences（后果）
- ✅ 更好的性能（零成本抽象）
- ✅ 编译时类型安全
- ✅ 易于序列化
- ❌ 添加新类型需要修改枚举定义
- ❌ 代码耦合度略高

---

## ADR-003: 采用 DAG（有向无环图）而非 State Machine（状态机）实现工作流

### Status（状态）
Accepted（已接受）

### Context（背景）
工作流引擎需要支持复杂的执行顺序和依赖关系。

### Decision（决策）
采用 **DAG（Directed Acyclic Graph，有向无环图）** 模型实现工作流，使用 `petgraph` 库。

### Rationale（理由）
1. **Expressiveness（表达能力）**: DAG 可以表达复杂的依赖关系
2. **Parallel Execution（并行执行）**: 自动识别可以并行执行的节点
3. **Visualization（可视化）**: 易于理解和可视化
4. **Mature Library（成熟库）**: `petgraph` 提供稳定的图算法实现

### Alternatives（替代方案）
- **State Machine（状态机）**: 简单但难以表达复杂依赖
- **Sequential Execution（顺序执行）**: 最简单但无法并行
- **Rule Engine（规则引擎）**: 灵活但难以控制执行流程

### Consequences（后果）
- ✅ 强大的表达能力
- ✅ 自动并行化
- ✅ 可视化友好
- ❌ 需要处理循环检测
- ❌ 调试复杂工作流较困难

---

## ADR-004: 采用 Layered Architecture（分层架构）

### Status（状态）
Accepted（已接受）

### Context（背景）
需要设计一个可维护、可测试、可扩展的系统架构。

### Decision（决策）
采用 **Layered Architecture（分层架构）**：

```
Interface Layer (CLI/TUI/MCP)
    ↓
Application Layer (UseCase/Service)
    ↓
Domain Layer (Model/Port)
    ↓
Infrastructure Layer (Persistence/Plugin)
    ↓
Adapter Layer (DTO/适配器)
```

### Rationale（理由）
1. **Separation of Concerns（关注点分离）**: 每层职责清晰
2. **Testability（可测试性）**: 每层可以独立测试
3. **Replaceability（可替换性）**: 可以替换某层实现而不影响其他层
4. **Domain-Driven（领域驱动）**: 核心业务逻辑在领域层

### Alternatives（替代方案）
- **Microkernel（微内核）**: 更灵活但复杂度高
- **Event-Driven（事件驱动）**: 松耦合但难以追踪流程
- **Pipe-Filter（管道过滤器）**: 适合数据处理但不适合复杂业务

### Consequences（后果）
- ✅ 清晰的职责边界
- ✅ 易于测试
- ✅ 支持多种接口（CLI/TUI/MCP）
- ❌ 层间转换有性能开销
- ❌ 过度设计风险

---

## ADR-005: 采用 Middleware Pattern（中间件模式）实现 Cross-Cutting Concerns（横切关注点）

### Status（状态）
Accepted（已接受）

### Context（背景）
需要在工具执行中实现缓存、重试、超时、日志等横切关注点（Cross-Cutting Concerns）。

### Decision（决策）
采用 **Middleware Pattern（中间件模式）** 实现横切关注点。

### Rationale（理由）
1. **Composability（可组合）**: 中间件可以灵活组合
2. **Reusability（可复用）**: 同一中间件可用于多个工具
3. **Testability（可测试）**: 中间件可以独立测试
4. **Clarity（清晰）**: 业务逻辑和横切关注点分离

```rust
pub trait Middleware {
    async fn execute(
        &self,
        ctx: &mut MiddlewareContext,
        next: Next<'_>,
    ) -> Result<ToolOutput>;
}
```

### Alternatives（替代方案）
- **Decorator Pattern（装饰器模式）**: 每个组合都需要新类
- **AOP（面向切面编程）**: Rust 不支持原生 AOP
- **Template Method（模板方法）**: 不够灵活

### Consequences（后果）
- ✅ 高度可组合
- ✅ 横切关注点分离
- ✅ 易于扩展新功能
- ❌ 调用栈较深
- ❌ 调试时需要追踪中间件链

---

## ADR-006: 支持 Multi-Language Plugins（多语言插件）

### Status（状态）
Accepted（已接受）

### Context（背景）
需要支持用户使用不同语言编写插件。

### Decision（决策）
支持 **Multi-Language Plugins（多语言插件）**：Native (Rust)、Python、Node.js、Docker、WASM。

### Rationale（理由）
1. **User-Friendly（用户友好）**: 用户可以使用熟悉的语言
2. **Ecosystem Reuse（生态复用）**: 利用各语言的生态系统
3. **Isolation（隔离性）**: 外部进程/容器提供隔离
4. **Performance Choice（性能选择）**: 用户可以根据性能需求选择语言

### Alternatives（替代方案）
- **Rust Only（仅 Rust）**: 性能最好但门槛高
- **Scripting Languages Only（仅脚本语言）**: 易用但性能差
- **WASM Unified（WASM 统一）**: 理想但生态不成熟

### Consequences（后果）
- ✅ 低门槛，高灵活性
- ✅ 利用各语言生态
- ✅ 良好的隔离性
- ❌ 维护复杂度高
- ❌ 性能不一致
- ❌ 需要处理多种运行时

---

## ADR-007: 采用 Tokio 作为 Async Runtime（异步运行时）

### Status（状态）
Accepted（已接受）

### Context（背景）
需要选择一个异步运行时（Async Runtime）来处理并发。

### Decision（决策）
采用 **Tokio** 作为异步运行时（Async Runtime）。

### Rationale（理由）
1. **Ecosystem（生态）**: Rust 异步生态的事实标准
2. **Performance（性能）**: 高效的调度器（Scheduler）实现
3. **Feature-Rich（功能丰富）**: 提供网络、定时器、通道等
4. **Documentation（文档）**: 优秀的文档和社区支持

### Alternatives（替代方案）
- **async-std**: API 更简洁但生态较小
- **smol**: 轻量但功能有限
- **Custom（自定义）**: 维护成本高

### Consequences（后果）
- ✅ 丰富的生态支持
- ✅ 高性能调度
- ✅ 大量现成组件
- ❌ 编译时间较长
- ❌ 学习曲线较陡

---

## ADR-008: 采用 Layered Configuration System（分层配置系统）

### Status（状态）
Accepted（已接受）

### Context（背景）
需要管理不同环境（开发、测试、生产）的配置。

### Decision（决策）
采用 **Layered Configuration System（分层配置系统）**：
1. Default Configuration（默认配置，代码内置）
2. Configuration File（配置文件，`config/default.toml`）
3. Environment Variables（环境变量，`WORKFLOW_TOOLKIT_*`）
4. Command-Line Arguments（命令行参数，最高优先级）

### Rationale（理由）
1. **Flexibility（灵活性）**: 不同环境使用不同配置
2. **12-Factor（12 因素应用）**: 符合云原生应用最佳实践
3. **Usability（易用性）**: 用户可以选择最方便的方式
4. **Security（安全性）**: 敏感信息通过环境变量传递

### Alternatives（替代方案）
- **Single Configuration File（单一配置文件）**: 简单但不灵活
- **Database Configuration（数据库配置）**: 需要数据库连接才能启动
- **Configuration Center（配置中心）**: 增加外部依赖

### Consequences（后果）
- ✅ 灵活的配置管理
- ✅ 符合云原生最佳实践
- ✅ 敏感信息安全
- ❌ 配置来源多，需要理解优先级
- ❌ 配置验证复杂

---

## ADR-009: 采用 Pseudocode Specification（伪代码规范）描述业务逻辑

### Status（状态）
Accepted（已接受）

### Context（背景）
需要一种方式来描述业务逻辑，既能让开发人员理解，又能被 LLM 解析。

### Decision（决策）
采用 **Structured Pseudocode（结构化伪代码）** 规范描述业务逻辑，参考 `document_llm_GUIDE.md`。

### Rationale（理由）
1. **Language-Independent（语言无关）**: 不绑定特定编程语言
2. **Easy to Understand（易理解）**: 类似自然语言的语法
3. **LLM-Friendly（LLM 友好）**: 结构化格式便于 LLM 解析
4. **Architecture Decoupling（架构解耦）**: 逻辑与实现分离

### Specification（规范）
- 使用 `FUNCTION`、`IF`、`FOR`、`SWITCH` 等关键字
- 使用 `UPPER_SNAKE_CASE` 表示原子操作
- 4 空格缩进表示层级
- 注释说明"为什么"而非"是什么"

### Alternatives（替代方案）
- **Flowchart（流程图）**: 可视化但不便于版本控制
- **UML**: 标准但复杂
- **Natural Language（自然语言）**: 灵活但不够精确

### Consequences（后果）
- ✅ 语言无关，架构与实现解耦
- ✅ 易于理解和维护
- ✅ LLM 友好
- ❌ 需要维护伪代码和实际代码的一致性
- ❌ 无法直接执行

---

## ADR-010: 采用 Tiered Documentation Architecture（分级文档架构）

### Status（状态）
Accepted（已接受）

### Context（背景）
文档庞大且复杂，需要更好的组织结构。

### Decision（决策）
采用 **Four-Tier Documentation Architecture（四级文档架构）**：

```
docs/
├── INDEX.md                    # 文档入口
├── specs/                      # P1/P2: Specifications（规范文档）
│   ├── system-spec.md          # 系统规范
│   └── api-contract.md         # API 契约
├── design/                     # Design Documents（设计文档）
│   ├── logical-workflow/       # 逻辑工作流
│   ├── architecture/           # 架构设计
│   ├── adr/                    # 架构决策记录
│   └── modules/                # 模块设计
├── 01_constitution/            # P0: Constitution（工程宪章）
└── 05_constraints/             # P3: Constraints（实现约束）
```

### Rationale（理由）
1. **Progressive Disclosure（渐进式披露）**: 读者可以根据需要深入
2. **Separation of Concerns（关注点分离）**: 不同类型信息分开存放
3. **LLM-Friendly（LLM 友好）**: 结构化便于 LLM 学习
4. **Easy Maintenance（易于维护）**: 修改时知道去哪里找

### Alternatives（替代方案）
- **Single README（单一 README）**: 简单但难以维护
- **Wiki**: 灵活但缺乏版本控制
- **Auto-Generation（自动生成）**: 方便但缺乏上下文

### Consequences（后果）
- ✅ 清晰的文档结构
- ✅ 渐进式学习路径
- ✅ 易于维护
- ❌ 需要维护多个文件
- ❌ 需要更新索引

---

## Pending Decisions（待决策事项）

### 是否采用 WebAssembly 作为主要插件格式？
- **Under Consideration（考虑中）**: WASM 提供良好的隔离性和性能
- **Blockers（阻碍）**: 生态不成熟，调试困难

### 是否支持 Distributed Execution（分布式执行）？
- **Under Consideration（考虑中）**: 未来扩展需要
- **Blockers（阻碍）**: 增加复杂度，当前单机足够

### 是否采用 gRPC 作为主要通信协议？
- **Under Consideration（考虑中）**: 高性能，强类型
- **Blockers（阻碍）**: 增加复杂度，HTTP/REST 足够当前需求
