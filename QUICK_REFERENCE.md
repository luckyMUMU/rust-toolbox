# 快速参考 - Quick Reference

## 语言规则 - Language Rules

### 回复语言
**默认使用中文回复**

- 中文用户 → 使用简体中文
- 英文用户 → 使用英文
- 混合场景 → 自动切换

**示例**:
```
用户: "帮我分析代码"
回复: "好的，我来分析代码..."

用户: "Analyze the code"
回复: "I'll analyze the code..."
```

## 文档创建规则 - Documentation Rules

### 何时创建文档
1. **新模块** → 必须有 AGENTS.md
2. **复杂功能** → 复杂度 > 5 分
3. **公共API** → 所有公共接口
4. **架构变更** → 任何调整
5. **用户请求** → 明确要求

### 文档类型
- **AGENTS.md** - 模块开发指南
- **README.md** - 用户文档
- **API文档** - Rust doc comments
- **教程** - 使用示例

### 文档创建流程
```
1. 分析需求
   ↓
2. 创建结构
   ↓
3. 编写内容
   ↓
4. 验证文档
   ↓
5. 发布文档
```

## 常用命令 - Common Commands

### 开发 - Development
```bash
cargo build                    # 构建 debug
cargo build --release          # 构建 release
cargo test                     # 运行所有测试
cargo clippy                   # 代码检查
cargo fmt                      # 代码格式化
```

### 运行 - Running
```bash
cargo run -- --help            # 显示 CLI 帮助
cargo run -- workflow execute <file>  # 执行工作流
cargo run -- tui               # 启动 TUI
cargo run -- server            # 启动 MCP 服务器（stub）
```

### 测试 - Testing
```bash
cargo test -- --nocapture       # 显示输出
cargo test -- --test-threads=1 # 单线程
cargo test property_tests       # 属性测试
cargo test --lib                # 库测试
```

## 文档位置 - Documentation Locations

### 核心文档
- **[GLOBAL_RULES.md](GLOBAL_RULES.md)** - 全局规则（必读）
- **[AGENTS.md](AGENTS.md)** - 项目总览
- **[README.md](README.md)** - 用户文档

### 模块文档
- **[src/AGENTS.md](src/AGENTS.md)** - 核心库
- **[src/workflow/AGENTS.md](src/workflow/AGENTS.md)** - 工作流引擎
- **[src/tools/AGENTS.md](src/tools/AGENTS.md)** - 工具系统
- **[src/plugins/AGENTS.md](src/plugins/AGENTS.md)** - 插件系统
- **[src/storage/AGENTS.md](src/storage/AGENTS.md)** - 存储层
- **[src/performance/AGENTS.md](src/performance/AGENTS.md)** - 性能优化
- **[src/interfaces/AGENTS.md](src/interfaces/AGENTS.md)** - 用户接口
- **[src/interfaces/cli/AGENTS.md](src/interfaces/cli/AGENTS.md)** - CLI
- **[src/interfaces/tui/AGENTS.md](src/interfaces/tui/AGENTS.md)** - TUI
- **[src/interfaces/tui/widgets/AGENTS.md](src/interfaces/tui/widgets/AGENTS.md)** - TUI 组件

### 其他文档
- **[examples/AGENTS.md](examples/AGENTS.md)** - 示例指南
- **[workflows/templates/AGENTS.md](workflows/templates/AGENTS.md)** - 模板指南
- **[tests/AGENTS.md](tests/AGENTS.md)** - 测试指南

## 文档质量检查 - Documentation Quality Check

### 完整性
- [ ] 模块概述清晰
- [ ] 所有公共项都有文档
- [ ] 代码示例可运行
- [ ] 链接有效
- [ ] 无拼写错误

### 一致性
- [ ] 语言风格一致
- [ ] 格式统一
- [ ] 术语一致
- [ ] 链接格式统一

### 可读性
- [ ] 结构清晰
- [ ] 层次分明
- [ ] 示例恰当
- [ ] 语言简洁

## 常见任务 - Common Tasks

### 创建新模块
```bash
# 1. 创建模块目录
mkdir -p src/my_module

# 2. 创建 AGENTS.md
cat > src/my_module/AGENTS.md << 'EOF'
# src/my_module/ - 模块名称

## 概述
一句话描述模块功能。

## 关键组件
- `StructName`: 功能描述

## 使用示例
\`\`\`rust
// 代码示例
\`\`\`

## 最佳实践
1. 建议1
2. 建议2

## 注意事项
- 注意1
- 注意2

## 相关文档
- [链接](path/to/doc.md)
EOF

# 3. 创建模块文件
touch src/my_module/mod.rs

# 4. 更新父模块的 AGENTS.md
# 5. 更新根 AGENTS.md
```

### 更新现有文档
```bash
# 1. 读取现有文档
cat src/module/AGENTS.md

# 2. 分析代码变更
# 3. 更新文档内容
# 4. 验证链接
# 5. 运行文档检查
```

### 运行文档检查
```bash
# 检查所有 AGENTS.md 文件
find . -name "AGENTS.md" -not -path "*/target/*" | while read file; do
    echo "检查: $file"
    # 检查文件大小
    size=$(wc -c < "$file")
    if [ $size -lt 100 ]; then
        echo "  ⚠️  文件过小: $size 字节"
    fi
done
```

## 文档模板 - Documentation Templates

### 模块文档模板
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

### API 文档模板
```rust
/// 功能描述
/// 
/// 详细描述函数的功能和用途。
/// 
/// # 参数
/// * `param1` - 参数1描述
/// * `param2` - 参数2描述
/// 
/// # 返回值
/// 返回值描述
/// 
/// # 错误
/// - `ErrorType`: 错误情况描述
/// 
/// # 示例
/// ```rust
/// // 代码示例
/// ```
/// 
/// # 注意
/// 重要注意事项
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // 实现代码
}
```

## 检查清单 - Checklist

### 代码变更时
- [ ] 使用中文回复用户
- [ ] 更新相关文档
- [ ] 验证代码示例
- [ ] 检查链接有效性
- [ ] 运行文档检查

### 创建新模块时
- [ ] 创建 AGENTS.md
- [ ] 包含模块概述
- [ ] 包含使用示例
- [ ] 包含最佳实践
- [ ] 包含注意事项
- [ ] 更新父模块文档
- [ ] 更新根文档

### 发布前
- [ ] 所有文档完整
- [ ] 所有链接有效
- [ ] 代码示例可运行
- [ ] 拼写检查通过
- [ ] 格式检查通过

## 相关资源 - Related Resources

### 内部文档
- [GLOBAL_RULES.md](GLOBAL_RULES.md) - 详细规则
- [AGENTS.md](AGENTS.md) - 项目总览
- [README.md](README.md) - 用户文档

### 外部资源
- Rust 官方文档: https://doc.rust-lang.org/
- Tokio 文档: https://tokio.rs/
- Clap 文档: https://clap.rs/
- Ratatui 文档: https://ratatui.rs/

## 总结 - Summary

### 核心原则
1. **中文优先**: 默认使用中文回复
2. **文档驱动**: 代码变更必须更新文档
3. **质量第一**: 文档质量与代码质量同等重要
4. **持续改进**: 定期审查和更新文档

### 快速行动
- **需要帮助**: 查看 [GLOBAL_RULES.md](GLOBAL_RULES.md)
- **创建文档**: 使用模板和检查清单
- **验证质量**: 运行文档检查
- **寻求反馈**: 创建 issue 或 PR

---

**最后更新**: 2026-01-24  
**版本**: 1.0.0
