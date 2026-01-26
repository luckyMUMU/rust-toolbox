# Draft: Module Architecture Audit

## Requirements (confirmed)
- 对核心架构进行全面审计，覆盖以下模块：src、src/workflow、src/tools、src/plugins/file_management、src/storage。
- 目标：降低巨型模块的复杂度、统一接口和错误模型、提升可测试性与可维护性。
- 需要落地的执行策略包括：模块级 AGENTS.md 补充、MCP MVP 接口与契约、跨模块契约统一、Cursor/Copilot 规则嵌入等。

## Technical Decisions (已做初步结论)
- 统一错误系统：集中定义 WorkflowError，统一对外返回 Result<T, WorkflowError>。
- 将通用算法提取为独立 crate（如 Aho-Corasick、模板展开），以减少重复实现。
- 引入公共模块以承载跨模块契约（如共用的数据结构和序列化策略）。
- MCP MVP 将明确对外 API、输入输出、错误契约及示例。
- Cursor/Copilot 规则将直接合并到模块级 AGENTS.md 模板中。

## Research Findings (初步研究结果)
- MCP 服务设计存在偏差：当前文档常规化为“桩实现”，与集成阶段的对外契约不一致。
- 巨型模块倾向导致测试和维护成本上升，尤其 file_management/utils.rs、UI 组件等。
- 接口统一性薄弱，存在多处重复实现与不同错误类型，需要抽象与统一。
- 计划中缺少统一的观测性指标和基线，需纳入性能/日志/追踪目标。

## Open Questions (待确认)
- 是否将所有模块的公共接口都统一成同一版本的契约？是否需要对外暴露版本化 API？
- 是否需要先行分拆为子 crates，再在母模组中以引用形式组合？
- Cursor/Copilot 的嵌入程度是否需要有更严格的执行分层？

## Scope Boundaries
- IN：5 个核心模块及其子模块的审计与改进。
- OUT：非核心插件实现、第三方外部服务实现细节、与 UI 的外观/风格相关变更。

## Next Steps
- 以 Patch 的形式将模块级细则写入各 AGENTS.md（已开始执行）
- 生成正式的 Plan，并在 Metis/Momus 循环中迭代。
- 将草案转化为 .sisyphus/plans 的正式计划文本。
