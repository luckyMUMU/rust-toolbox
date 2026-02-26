# docs/ 和 design.md 文档更新与补全规范

## Why

项目的 `docs/` 目录和各模块的 `design.md` 文档存在以下问题需要解决：
1. **索引文件缺失**：`docs/index.md`、`docs/03_technical_spec/index.md`、`docs/04_context_reference/index.md` 不存在
2. **子模块 design.md 缺失**：workflow 和 plugins 的子模块缺少 design.md
3. **文档版本不一致**：API 参考文档更新时间滞后，伪代码文件版本过旧
4. **内容与代码不同步**：部分接口定义可能与实际代码实现不一致
5. **与 SOP 对齐**：需要确保文档结构与 SOP 定义的目录映射一致

## What Changes

### 索引文件创建
- 创建 `docs/index.md` 作为文档入口导航
- 创建 `docs/03_technical_spec/index.md` 作为技术规范索引
- 创建 `docs/04_context_reference/index.md` 作为上下文参考索引

### 子模块 design.md 补全（基于代码实现）
- 创建 `src/workflow/component/design.md` - 基于 `component/mod.rs` 的实际实现
- 创建 `src/workflow/executor/design.md` - 基于 `executor/mod.rs` 的实际实现
- 创建 `src/workflow/context/design.md` - 基于 `context/mod.rs` 的实际实现
- 创建 `src/workflow/state/design.md` - 基于 `state/mod.rs` 的实际实现
- 创建 `src/plugins/file_management/design.md` - 基于 `file_management/mod.rs` 的实际实现

### 版本同步
- 更新 CLI_REFERENCE.md 最后更新时间
- 更新 RUST_SDK_REFERENCE.md 最后更新时间
- 更新伪代码文件版本号

### 内容同步（代码一致性验证）
- 验证 WorkflowDefinition 结构与 `workflow/definition.rs` 一致
- 验证 WorkflowEngine trait 与 `workflow/engine.rs` 一致
- 验证 Component trait 与 `workflow/component/mod.rs` 一致
- 验证 Executor trait 与 `workflow/executor/mod.rs` 一致
- 验证 DataContext 与 `workflow/context/mod.rs` 一致
- 验证 ExecutionTracker 与 `workflow/state/mod.rs` 一致

### SOP 对齐
- 确保文档结构与 `sop/04_reference/document_directory_mapping.md` 一致
- 遵循 `sop/04_reference/design_guide.md` 的设计文档规范

## Impact

### Affected specs
- docs/ 目录结构
- 各模块 design.md 文件

### Affected code
- 约 15 个文档文件需要创建或更新

## ADDED Requirements

### Requirement: 文档索引完整性
系统应提供完整的文档导航入口，确保用户能够从顶层索引逐级导航到具体文档。

#### Scenario: 入口导航
- **WHEN** 用户访问 docs/ 目录
- **THEN** 应能找到 index.md 作为导航入口

### Requirement: 子模块设计文档
每个具有独立职责的子模块应有对应的 design.md 文档描述其设计。

#### Scenario: 子模块设计
- **WHEN** 子模块包含多个源文件
- **THEN** 应有 design.md 描述模块职责、结构和接口

### Requirement: 文档版本同步
文档版本和更新时间应与代码实现保持同步。

#### Scenario: 版本检查
- **WHEN** 代码接口发生变化
- **THEN** 相关文档应同步更新

### Requirement: 代码一致性
文档描述的接口和数据结构应与实际代码实现一致。

#### Scenario: 接口一致性
- **WHEN** 文档描述 trait 或 struct
- **THEN** 其方法签名和字段应与代码定义一致

## MODIFIED Requirements

### Requirement: API 参考更新
更新 CLI_REFERENCE.md 和 RUST_SDK_REFERENCE.md 以反映当前 API 状态。

## REMOVED Requirements

无移除需求。
