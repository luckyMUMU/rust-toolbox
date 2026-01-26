# 全局规则 - Global Rules

## 语言规则 - Language Rules

### 回复语言 - Response Language
**默认使用中文回复** - Default to Chinese responses

- **中文用户**: 使用简体中文回复
- **英文用户**: 使用英文回复
- **混合场景**: 根据用户输入语言自动切换

**示例**:
```
用户: "帮我分析代码"
回复: "好的，我来分析代码..."

用户: "Analyze the code"
回复: "I'll analyze the code..."
```

### 代码注释 - Code Comments
- **新代码**: 使用中文注释
- **现有代码**: 保持原有注释语言
- **文档字符串**: 使用中文描述功能

**示例**:
```rust
/// 计算两个数的和
/// 
/// # 参数
/// * `a` - 第一个数
/// * `b` - 第二个数
/// 
/// # 返回值
/// 两个数的和
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## 文档创建规则 - Documentation Creation Rules

### 何时创建文档 - When to Create Documentation

**必须创建文档**:
1. **新模块**: 创建模块时必须有 AGENTS.md
2. **复杂功能**: 功能复杂度 > 5 分时
3. **公共API**: 所有公共接口必须有文档
4. **架构变更**: 任何架构调整都需要更新文档
5. **用户请求**: 用户明确要求创建文档

**文档类型**:
- **AGENTS.md**: 模块开发指南
- **README.md**: 用户文档
- **API文档**: Rust doc comments
- **教程**: 使用示例和最佳实践

### 文档位置 - Documentation Locations

**层级结构**:
```
项目根目录/
├── AGENTS.md                    # 项目总览
├── GLOBAL_RULES.md             # 全局规则（本文件）
├── README.md                   # 用户文档
├── docs/                       # 详细文档
│   ├── ARCHITECTURE.md        # 架构设计
│   ├── API_REFERENCE.md       # API参考
│   └── TUTORIALS.md           # 使用教程
├── src/
│   ├── AGENTS.md              # 核心模块指南
│   ├── workflow/
│   │   ├── AGENTS.md         # 工作流引擎
│   │   └── executor/
│   │       └── AGENTS.md     # 执行器指南
│   └── plugins/
│       ├── AGENTS.md         # 插件系统
│       └── file_management/
│           └── AGENTS.md     # 文件管理插件
├── examples/
│   ├── AGENTS.md             # 示例指南
│   └── templates/
│       └── AGENTS.md         # 模板指南
└── tests/
    └── AGENTS.md             # 测试指南
```

### 文档内容要求 - Documentation Requirements

**AGENTS.md 必须包含**:
1. **模块概述**: 一句话描述模块功能
2. **关键组件**: 主要结构体和特征
3. **使用示例**: 代码示例
4. **设计模式**: 使用的设计模式
5. **最佳实践**: 使用建议
6. **注意事项**: 常见陷阱
7. **相关文档**: 链接到其他文档

**示例模板**:
```markdown
# 模块名称 - Module Name

## 概述 - Overview
一句话描述模块功能。

## 关键组件 - Key Components
- `StructName`: 功能描述
- `TraitName`: 功能描述

## 使用示例 - Usage Example
```rust
// 代码示例
```

## 设计模式 - Design Patterns
- **Builder Pattern**: 用于...
- **Strategy Pattern**: 用于...

## 最佳实践 - Best Practices
1. 建议1
2. 建议2

## 注意事项 - Important Notes
- 注意1
- 注意2

## 相关文档 - See Also
- [链接](path/to/doc.md)
```

## 文档更新规则 - Documentation Update Rules

### 代码变更时 - When Code Changes

**必须更新**:
1. **公共API变更**: 更新 API 文档
2. **新增功能**: 更新模块文档
3. **架构调整**: 更新架构文档
4. **Bug修复**: 更新相关文档
5. **性能优化**: 更新性能文档

**更新流程**:
1. 读取现有文档
2. 分析代码变更
3. 更新文档内容
4. 验证链接和示例
5. 运行文档检查

### 文档质量检查 - Documentation Quality Checks

**完整性**:
- [ ] 模块概述清晰
- [ ] 所有公共项都有文档
- [ ] 代码示例可运行
- [ ] 链接有效
- [ ] 无拼写错误

**一致性**:
- [ ] 语言风格一致
- [ ] 格式统一
- [ ] 术语一致
- [ ] 链接格式统一

**可读性**:
- [ ] 结构清晰
- [ ] 层次分明
- [ ] 示例恰当
- [ ] 语言简洁

## 文档创建流程 - Documentation Creation Workflow

### 步骤1: 分析需求 - Analyze Requirements
```bash
# 1. 确定文档范围
- 模块功能分析
- 用户群体分析
- 使用场景分析

# 2. 确定文档类型
- API文档
- 用户指南
- 开发指南
- 参考手册
```

### 步骤2: 创建文档结构 - Create Structure
```bash
# 1. 创建目录
mkdir -p docs/
mkdir -p src/module_name/

# 2. 创建文档文件
touch docs/ARCHITECTURE.md
touch src/module_name/AGENTS.md
```

### 步骤3: 编写内容 - Write Content
```bash
# 1. 编写概述
# 2. 编写组件说明
# 3. 添加代码示例
# 4. 添加最佳实践
# 5. 添加注意事项
```

### 步骤4: 验证文档 - Validate Documentation
```bash
# 1. 检查链接
# 2. 验证代码示例
# 3. 拼写检查
# 4. 格式检查
```

### 步骤5: 发布文档 - Publish Documentation
```bash
# 1. 提交到版本控制
git add docs/ src/module_name/AGENTS.md
git commit -m "docs: 添加模块文档"

# 2. 更新根文档
git add AGENTS.md
git commit -m "docs: 更新项目文档"
```

## 文档示例 - Documentation Examples

### 模块文档示例 - Module Documentation Example

**src/workflow/AGENTS.md**:
```markdown
# Workflow Engine - 工作流引擎

## 概述 - Overview
基于DAG的分布式工作流执行引擎，支持并行执行、错误恢复和状态持久化。

## 核心组件 - Core Components

### WorkflowDefinition
工作流定义结构，包含节点和边：
```rust
pub struct WorkflowDefinition {
    pub name: String,
    pub version: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}
```

### DagScheduler
DAG调度器，负责：
- 拓扑排序
- 依赖解析
- 并行分组
- 关键路径分析

### RefactoredWorkflowEngine
重构后的工作流引擎，特点：
- 真正的并行执行
- 单一职责原则
- 检查点支持
- 错误恢复

## 执行流程 - Execution Flow

1. **验证**: 检查工作流定义
2. **调度**: 确定执行顺序
3. **执行**: 运行节点（支持并行）
4. **监控**: 实时状态更新
5. **持久化**: 保存状态
6. **完成**: 记录结果

## 使用示例 - Usage Example

```rust
use workflow_toolkit::workflow::{WorkflowDefinition, RefactoredWorkflowEngine};

// 创建工作流定义
let workflow = WorkflowDefinition {
    name: "data-processing".to_string(),
    version: "1.0.0".to_string(),
    nodes: vec![/* 节点定义 */],
    edges: vec![/* 边定义 */],
};

// 创建引擎
let engine = RefactoredWorkflowEngine::new(
    config,
    tool_registry,
    state_manager,
)?;

// 执行工作流
let result = engine.execute_workflow(workflow).await?;
```

## 设计模式 - Design Patterns

### Executor Chain
使用责任链模式处理横切关注点：
```
ToolComponent → RetryExecutor → CacheExecutor → AuditExecutor → BasicExecutor
```

### Component System
组件化架构，支持：
- Tool: 执行工具
- Parallel: 并行执行
- Condition: 条件分支
- Loop: 循环执行
- Switch: 多路分支

## 最佳实践 - Best Practices

1. **使用检查点**: 对长时间运行的工作流启用检查点
2. **配置重试策略**: 根据操作类型设置合适的重试策略
3. **启用审计日志**: 生产环境必须启用审计日志
4. **监控性能**: 使用 PerformanceManager 监控工作流性能
5. **错误处理**: 实现适当的错误恢复策略

## 注意事项 - Important Notes

- **并发限制**: 默认最多4个并发工作流，可通过配置调整
- **检查点间隔**: 默认5分钟，可根据需求调整
- **内存使用**: 长时间运行的工作流可能占用较多内存
- **错误隔离**: 单个节点失败不会停止整个工作流

## 相关文档 - See Also

- [Tools AGENTS.md](../tools/AGENTS.md) - 工具系统
- [Storage AGENTS.md](../storage/AGENTS.md) - 持久化存储
- [Performance AGENTS.md](../performance/AGENTS.md) - 性能优化
- [Examples](../../examples/AGENTS.md) - 使用示例
```

### API文档示例 - API Documentation Example

**Rust doc comments**:
```rust
/// 执行工作流
/// 
/// 这个函数是工作流引擎的主要入口点。它接受工作流定义，
/// 验证其有效性，然后执行所有节点。
/// 
/// # 参数
/// * `definition` - 工作流定义，包含节点和边
/// 
/// # 返回值
/// `Result<WorkflowExecution, WorkflowError>` - 执行结果
/// 
/// # 错误
/// - `WorkflowError::ValidationError`: 工作流定义无效
/// - `WorkflowError::ToolNotFound`: 工具不存在
/// - `WorkflowError::ExecutionTimeout`: 执行超时
/// 
/// # 示例
/// ```rust
/// use workflow_toolkit::workflow::{WorkflowDefinition, WorkflowEngine};
/// 
/// let engine = WorkflowEngine::new()?;
/// let workflow = WorkflowDefinition::from_file("workflow.yaml")?;
/// let result = engine.execute_workflow(workflow).await?;
/// ```
/// 
/// # 注意
/// 这个函数是异步的，需要在 tokio 运行时中调用。
pub async fn execute_workflow(
    &self,
    definition: WorkflowDefinition,
) -> Result<WorkflowExecution, WorkflowError> {
    // 实现代码
}
```

## 文档维护 - Documentation Maintenance

### 定期检查 - Regular Checks
- **每周**: 检查文档完整性
- **每月**: 更新过时内容
- **每季度**: 全面审查文档

### 版本控制 - Version Control
- **Git Hooks**: 提交前检查文档
- **CI/CD**: 自动验证文档
- **版本标记**: 文档随版本发布

### 反馈机制 - Feedback Mechanism
- **Issue追踪**: 文档问题使用 issue 跟踪
- **用户反馈**: 收集用户文档反馈
- **贡献指南**: 鼓励社区贡献文档

## 工具和脚本 - Tools and Scripts

### 文档检查脚本 - Documentation Check Script
```bash
#!/bin/bash
# check-docs.sh

echo "检查文档完整性..."

# 检查所有 AGENTS.md 文件
find . -name "AGENTS.md" -not -path "*/target/*" | while read file; do
    echo "检查: $file"
    
    # 检查文件大小
    size=$(wc -c < "$file")
    if [ $size -lt 100 ]; then
        echo "  ⚠️  文件过小: $size 字节"
    fi
    
    # 检查代码示例
    if grep -q '```rust' "$file"; then
        echo "  ✓ 包含 Rust 代码示例"
    fi
    
    # 检查链接
    if grep -q '\[.*\](.*\.md)' "$file"; then
        echo "  ✓ 包含文档链接"
    fi
done

echo "文档检查完成"
```

### 文档生成脚本 - Documentation Generation Script
```bash
#!/bin/bash
# generate-docs.sh

echo "生成文档..."

# 生成 API 文档
cargo doc --no-deps --open

# 生成架构图
echo "生成架构图..."
cat > docs/architecture-diagram.txt << 'EOF'
┌─────────────────────────────────────────────────────────┐
│                    User Interfaces                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │   CLI    │  │   TUI    │  │  MCP Server (stub)   │  │
│  └──────────┘  └──────────┘  └──────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    Application Layer                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │ Workflow │  │   Tools  │  │     Plugins          │  │
│  │  Engine  │  │ Registry │  │  (Multi-language)    │  │
│  └──────────┘  └──────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────┘
EOF

echo "文档生成完成"
```

## 质量标准 - Quality Standards

### 内容质量 - Content Quality
- **准确性**: 信息准确无误
- **完整性**: 覆盖所有重要方面
- **时效性**: 及时更新
- **一致性**: 术语和格式统一

### 可读性 - Readability
- **结构清晰**: 层次分明，逻辑清晰
- **语言简洁**: 避免冗余，表达准确
- **示例恰当**: 代码示例具有代表性
- **格式规范**: 遵循 Markdown 规范

### 可维护性 - Maintainability
- **易于更新**: 结构便于修改
- **版本控制**: 与代码同步
- **自动化**: 尽可能自动化检查
- **反馈机制**: 收集用户反馈

## 总结 - Summary

### 核心原则 - Core Principles
1. **中文优先**: 默认使用中文回复
2. **文档驱动**: 代码变更必须更新文档
3. **质量第一**: 文档质量与代码质量同等重要
4. **持续改进**: 定期审查和更新文档

### 检查清单 - Checklist
- [ ] 使用中文回复用户
- [ ] 代码变更时更新文档
- [ ] 创建新模块时创建 AGENTS.md
- [ ] 文档包含使用示例
- [ ] 文档包含最佳实践
- [ ] 文档包含注意事项
- [ ] 文档链接有效
- [ ] 代码示例可运行

### 相关文档 - Related Documents
- [AGENTS.md](AGENTS.md) - 项目总览
- [README.md](README.md) - 用户文档
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - 架构设计
- [docs/API_REFERENCE.md](docs/API_REFERENCE.md) - API参考

---

**最后更新**: 2026-01-24  
**版本**: 1.0.0  
**维护者**: 开发团队
