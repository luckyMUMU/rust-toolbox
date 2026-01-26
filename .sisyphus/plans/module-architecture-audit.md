# Module Architecture Audit Plan

## Context
- 目标：对当前代码库的架构设计进行全面审计，识别设计偏差、耦合点、性能瓶颈与可维护性改进点，制定分阶段的改进路线。
- 触发原因：现有设计存在巨型模块、职责不清、接口不统一、MCP 服务设计与实现不一致等问题，需通过系统化审计确保后续重构可控。 

## Scope
- IN：以下模块及其子模块：
- src/ (核心库)
- src/workflow/ (DAG 引擎及执行组件)
- src/tools/ (工具系统)
- src/plugins/file_management/ (文件管理插件)
- src/storage/ (存储层)
- OUT：界外的其他非核心模块、外部服务实现细节，及不涉及当前审计范围的插件实现细节。

## Key Findings (初步)
- MCP 服务实现为桩实现，与对外契约存在偏差，影响后续集成与测试。
- 巨型模块信号明显：如 file_management/utils.rs、UI 相关 layout/widgets 等，影响可测试性和演化成本。
- 模块边界未完全解耦，接口契约不统一，错误类型在跨模块间分散。
- Cursor/Copilot 规则缺失，未来自动化代理执行时可能缺乏执行约束与一致性。
- 测试覆盖量充足，但对大模块的端到端场景、故障注入、并发场景的覆盖需要更明确的验收标准。
- 性能观察点未在全局层面统一定义基线与指标（UI 路径、文件管理热点等）。
- 安全性与合规性需要明确的审计日志策略、数据脱敏与最小权限策略。

## Objectives（目标）
- 将巨型模块拆分为职责单一的子模块，降低耦合，提升可测试性与可维护性。
- 统一对外契约和错误模型，建立集中化的错误与日志策略。
- 明确 MVP 的 MCP 服务边界与契约，便于后续插件与外部系统对接。
- 落地统一的模块级 AGENTS.md 模板与 Cursor/Copilot 规则，提升自动化执行的一致性。
- 提升可观测性：定义基线性能、日志、追踪指标，便于后续性能优化。

## Plan Overview（总体方案）
1) 模块级补充细则落地（L3 级别，中文文档，统一模板）
- 为以下模块追加“模块级补充细则”段落：src/、src/workflow/、src/tools/、src/plugins/file_management/、src/storage/
- 增加 Cursor/Copilot 规则段落，统一写入 AGENTS.md。

2) MCP MVP 舍得实现与契约文档
- 明确 MVP 的对外 API、输入输出、错误契约、测试用例，以及对后续插件对接的扩展点。
- 提供一个 MVP 的对外示例接口文档（OpenAPI/RustDoc 风格示例）。

3) 体系化的模块重组计划
- 将巨型模块拆分为更小的 crate/模块，建立公共的错误类型 crate，统一接口契约。
- 将共用的工具算法（如 Aho-Corasick、模板展开）放入独立的公共 crate，减少重复实现。

4) 测试与观测性
- 设计覆盖每个核心路径的测试策略（TDD/手动混合），定义回归测试集。
- 为关键路径建立基线基准，收集延迟、内存、并发等性能指标。
- 统一日志与追踪标识，方便跨模块追踪。

5) Cursor/Copilot 集成
- 将规则合并到模块 AGENTS.md 模板，确保代理执行可控。
- 设计自动化审阅触发点，确保变更可追溯。

## Delivery & Milestones（分阶段产出）
- 1) 模块级 AGENTS.md 补充细则：完成 5 个模块的追加文档。
- 2) Module Architecture Audit 草案：完成 .sisyphus/plans/module-architecture-audit.md 的初稿。
- 3) Draft 文档：在 .sisyphus/drafts/module-architecture-audit.md 写入工作内存草案与初步的决策点。
- 4) Patch 应用：将草案转化为正式计划文本并合并至仓库计划中（.sisyphus/plans/）。
- 5) 评审与修订：依据 Momus/Metis 反馈进行修订，直到 OKAY。

## Deliverables（产出）
- src/AGENTS.md、src/workflow/AGENTS.md、src/tools/AGENTS.md、src/plugins/file_management/AGENTS.md、src/storage/AGENTS.md 的模块级补充细则段落。
- .sisyphus/plans/module-architecture-audit.md：完整计划草案（L3 颗粒度）
- .sisyphus/drafts/module-architecture-audit.md：工作内存草案，含决策点与背景
- Cursor/Copilot 合并规则文本（嵌入 AGENTS.md 模板）

## Verification Strategy（验证策略）
- 基于 TDD 的分阶段验收：每一个改动对应测试用例的新增/更新，测试通过后才进入下一阶段。
- 以实证为导向的验证集合：构建、测试、格式化、静态检查、文档一致性。要求 IC 级别（OKAY）前不提交。
- 评审与回滚策略：Momus 评审直至 OKAY；如有拒绝，迭代修复并重新提交。

## Risks & Mitigations（风险与缓解）
- 风险：模块拆分可能引入接口兼容性问题。
  缓解：先定义公共接口与错误类型，再逐步抽象。
- 风险：MCP MVP 不完善导致后续对接困难。
  缓解：先定义 MVP 的最小集合及最清晰的契约。
- 风险：Cursor/Copilot 规则未落地导致执行偏差。
  缓解：将规则嵌入模块模板，形成强约束。

## Open Questions（待解答的问题）
- 是否要对模块进行分阶段的里程碑划分？若需要，请给出里程碑名称与完成条件。
- 是否希望增加对外部 API/契约的补充文档模板？
- 对于大型 UI 模块，是否需要额外的分离策略与子 crate？

## Plan Status & Next Steps（计划状态与后续步骤）
- 待你确认：上述草案结构与内容分配是否符合期望？
- 一旦确认，我将把此草案转化为正式的提交补丁，执行 .sisyphus/plans 与 .sisyphus/drafts 的同步更新。
