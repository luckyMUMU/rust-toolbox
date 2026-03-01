# ADR-Tools-004: 工具 Schema 验证策略

## 状态
- [x] 已接受
- [ ] 已废弃
- [ ] 已替代

## 背景 (Context)

### 问题描述
工具执行需要验证输入输出的正确性，确保参数类型、格式和约束条件符合预期。需要设计一个灵活且高效的 Schema 验证系统。

### 约束条件
- 必须支持 JSON Schema 标准
- 必须支持自定义验证规则
- 必须提供清晰的错误消息
- 验证性能不能显著影响执行效率

### 影响范围
- `src/tools/schema_validator.rs` - Schema 验证器
- 所有工具的输入输出验证

## 决策 (Decision)

### 选择的方案
采用 **基于 jsonschema 库的 Schema 验证系统**，包括：
1. Schema 编译和缓存
2. 输入验证
3. 输出验证
4. 错误收集和报告

### 决策理由
jsonschema 库是 Rust 生态中成熟的 JSON Schema 验证库，支持完整的 JSON Schema 规范。

## 选项对比 (Options Considered)

| 选项 | 优点 | 缺点 | 结论 |
|------|------|------|------|
| **jsonschema 库** (已选择) | 功能完整、性能好、社区活跃 | 额外依赖 | ✅ 选择 |
| 自定义验证 | 无依赖、完全可控 | 开发成本高、功能有限 | ❌ 不选 |
| valico | 功能丰富 | 维护不活跃 | ❌ 不选 |

## 详细设计

### 1. Schema 验证器

```rust
pub struct SchemaValidator {
    config: SchemaValidatorConfig,
    compiled_schemas: HashMap<String, Arc<CompiledSchema>>,
}

impl SchemaValidator {
    pub fn validate_input(&mut self, tool_name: &str, schema: &InputSchema, input: &ToolInput) -> Result<SchemaValidationResult>;
    pub fn validate_output(&mut self, tool_name: &str, schema: &OutputSchema, output: &ToolOutput) -> Result<SchemaValidationResult>;
}
```

### 2. 验证结果

```rust
pub struct SchemaValidationResult {
    pub is_valid: bool,
    pub errors: Vec<SchemaValidationError>,
    pub warnings: Vec<String>,
}

pub struct SchemaValidationError {
    pub path: String,           // 错误路径
    pub message: String,        // 错误消息
    pub error_type: SchemaErrorType,  // 错误类型
    pub actual_value: Option<Value>,  // 实际值
    pub expected_value: Option<Value>, // 期望值
}
```

### 3. 错误类型

```rust
pub enum SchemaErrorType {
    TypeMismatch,       // 类型不匹配
    RequiredMissing,    // 必填字段缺失
    ValueOutOfRange,    // 值超出范围
    PatternMismatch,    // 模式不匹配
    LengthExceeded,     // 长度超出限制
    EnumMismatch,       // 枚举值不匹配
    AdditionalProperty, // 额外属性
    DependencyMissing,  // 依赖缺失
    FormatError,        // 格式错误
}
```

## 影响 (Consequences)

### 正面影响
- ✅ 完整的 JSON Schema 支持
- ✅ 清晰的中文错误消息
- ✅ Schema 编译缓存提高性能
- ✅ 支持严格模式和宽松模式

### 负面影响/风险
- ⚠️ 额外依赖 jsonschema 库 → 已在 Cargo.toml 中
- ⚠️ 验证可能影响执行性能 → 通过缓存缓解

### 技术债务
- 无明显技术债务

## 实施计划

### 已完成
- [x] 实现 `SchemaValidator` 核心逻辑
- [x] 实现输入验证
- [x] 实现输出验证
- [x] 实现错误类型分类
- [x] 实现 `SchemaRegistry`

### 后续优化
- [ ] 添加自定义验证函数支持
- [ ] 添加 Schema 自动生成

## 决策记录

| 日期 | 决策人 | 动作 | 说明 |
|------|--------|------|------|
| 2026-02-20 | Architecture Team | 创建 | 设计工具 Schema 验证策略 |
| 2026-03-01 | Architecture Team | 迁移 | 从 .temp/ADR/ 迁移到正式文档目录 |
