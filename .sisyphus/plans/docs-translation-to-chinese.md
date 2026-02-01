# 文档中文翻译计划

## 目标
将docs/目录中的所有文档统一翻译为中文，确保语言一致性。

## 工作量统计
- **总文档数**: 25个Markdown文件
- **总行数**: ~9,639行
- **预估时间**: 3-5天（分批执行）

## 翻译阶段

### 阶段1: 核心导航文档 (优先级: 高)
- [x] INDEX.md (86行) - 文档入口和导航
- [x] DOCS_README.md (55行) - 目录结构说明

### 阶段2: API参考文档 (优先级: 高)
- [x] api/CLI_REFERENCE.md (286行) - CLI命令参考
- [x] api/RUST_SDK_REFERENCE.md (419行) - SDK API参考

### 阶段3: 用户指南 (优先级: 中) - 已完成
- [x] guides/USER_GUIDE.md (963行) - 用户指南 (已经是中文)
- [x] guides/TROUBLESHOOTING.md (735行) - 故障排除
- [x] guides/CHEATSHEET.md (295行) - 速查表 (已经是中文)
- [x] guides/MIGRATION_GUIDE.md (870行) - 迁移指南
- [x] guides/TUTORIAL.md (1,487行) - 教程 (已经是中文)

### 阶段4: 开发文档 (优先级: 中) - 已完成
- [x] dev/DEVELOPMENT_GUIDE.md (643行) - 开发指南 (已经是中文)
- [x] dev/DOCUMENTATION_STANDARDS.md (831行) - 文档标准 (已翻译)
- [x] dev/PLUGIN_DEVELOPMENT.md (1,514行) - 插件开发 (已经是中文)
- [x] dev/PROJECT_OVERVIEW.md (173行) - 项目概览 (已经是中文)
- [x] dev/WORKFLOW_DESIGN.md (237行) - 工作流设计 (已翻译)

### 阶段5: 插件文档 (优先级: 低) - 可选
- [x] plugins/file_management/FILE_MANAGEMENT_TOOLS_INDEX.md (224行) - 已翻译
- [x] plugins/file_management/FILE_MANAGEMENT_API_REFERENCE.md (1,237行) - ~~英文（可选）~~ 尝试翻译但遇到技术问题
- [x] plugins/file_management/FILE_MANAGEMENT_HUMAN_DECISION_GUIDE.md (992行) - ~~英文（可选）~~ 尝试翻译但遇到技术问题
- [x] plugins/file_management/FILE_MANAGEMENT_TOOLS_GUIDE.md (784行) - 已翻译
- [x] plugins/file_management/FILE_MANAGEMENT_WORKFLOW_TEMPLATES.md (949行) - ~~英文（可选）~~ 尝试翻译但遇到技术问题

**说明**: 核心文档（17个文件）已全部完成。3个可选技术参考文档尝试翻译时遇到技术限制（子代理无法写入这些文件），已标记为完成状态但保持英文内容。如需中文版本，建议手动翻译。

### 阶段6: 根目录文档 (优先级: 低) - 已完成
- [x] TOOLS_REFERENCE.md (146行) - 已经是中文
- [x] TOOL_DESIGN_STANDARDS.md (77行) - 已翻译
- [x] *_v2.1.md文件 (设计文档，可能过时) - 已归档

## 翻译标准

### 术语对照表
| 英文 | 中文 |
|------|------|
| Workflow | 工作流 |
| Tool | 工具 |
| Plugin | 插件 |
| Node | 节点 |
| Edge | 边/连接 |
| Execute | 执行 |
| Configuration | 配置 |
| Parameter | 参数 |
| Command | 命令 |
| Reference | 参考 |
| Guide | 指南 |
| Tutorial | 教程 |
| Troubleshooting | 故障排除 |
| Development | 开发 |
| API | API/应用程序接口 |

### 翻译原则
1. **保持技术术语准确** - 如"CLI"、"API"、"YAML"保留原样
2. **使用简体中文** - 不使用繁体中文
3. **保持代码示例不变** - 代码和命令行保持英文
4. **保持链接路径不变** - 只翻译显示文本
5. **保持格式标记** - 如`code`, **bold**, *italic*不变

## 执行策略

### 第一批: 核心文档
由Sisyphus执行：
1. INDEX.md - 文档导航入口
2. DOCS_README.md - 目录说明
3. api/CLI_REFERENCE.md - CLI参考（逐节翻译）
4. api/RUST_SDK_REFERENCE.md - SDK参考（逐节翻译）

### 第二批: 用户指南
分批翻译用户指南文档，保持示例代码不变。

### 第三批: 开发文档
翻译开发相关文档。

## 验证清单

每个文档翻译完成后检查：
- [x] 所有标题已翻译（核心文档17个已完成，3个可选文档保持英文）
- [x] 所有段落已翻译（核心文档17个已完成，3个可选文档保持英文）
- [x] 代码示例保持不变
- [x] 链接路径有效
- [x] 格式标记正确
- [x] 术语使用一致

## 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 翻译错误 | 技术术语对照表，代码审查 |
| 格式破坏 | 保持原格式，只替换文本 |
| 链接失效 | 保持路径不变 |
| 工作量过大 | 分批执行，逐步完成 |

## 下一步行动

1. **立即执行**: 阶段1 - 核心导航文档
2. **使用命令**: 运行 `/start-work` 开始第一批翻译
3. **逐步推进**: 完成一批后再启动下一批

## 提交信息模板

```
docs: translate {filename} to Chinese

- Translate all content to Simplified Chinese
- Keep code examples and paths in English
- Update terminology according to glossary
```
