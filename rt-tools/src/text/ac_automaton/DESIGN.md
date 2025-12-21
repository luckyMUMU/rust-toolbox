# 模块名称：AC 自动机文本匹配工具

## 1. 目标 (Goal)
- **核心功能**：实现 Aho-Corasick 算法，用于高效的多模式字符串匹配，支持同时在多个文本中搜索多个模式。
- **非目标**：不支持正则表达式模式（未来增强），不支持实时流处理（未来增强）。

## 2. 核心定义 (Definitions)
- **AcAutomatonTool**：工具的核心实现结构体，实现了 `rt_core::tool::Tool` trait。
- **AcAutomatonImpl**：内部实现结构体，管理：
  - Aho-Corasick 自动机：底层模式匹配引擎
  - 模式存储：线程安全的活动模式存储
  - 构建器配置：自动机配置（大小写敏感性等）
- **AcAutomatonInput**：输入参数结构体，包含操作类型、模式列表、文本列表和配置选项。
- **AcAutomatonOutput**：输出结果结构体，包含操作成功状态、匹配结果、当前模式列表和执行时间。
- **MatchResult**：匹配结果结构体，包含匹配的模式、起始位置和结束位置。

## 3. 算法与逻辑设计 (Algorithm & Logic)

### 核心流程
1. **操作解析**：解析输入参数，识别要执行的操作类型（add、remove、list、match）。
2. **模式管理**：
   - **添加模式**：验证模式非空且唯一，添加到模式存储中，触发自动机重建。
   - **移除模式**：验证确认标志，从模式存储中移除模式，触发自动机重建。
   - **列出模式**：返回当前加载的所有模式。
3. **自动机构建**：
   - 基于当前模式列表构建 Aho-Corasick 自动机。
   - 仅在模式变更时重建（延迟重建优化）。
4. **文本匹配**：
   - **单文本匹配**：对单个文本执行高效匹配。
   - **并行匹配**：异步并行处理多个文本。
   - **结果聚合**：合并多个匹配操作的结果。
5. **结果输出**：返回匹配结果、当前模式列表和执行时间。

### 算法细节
- **Aho-Corasick 算法**：
  - 构建一个包含所有模式的前缀树（Trie）。
  - 为每个节点添加失败指针，实现高效的多模式匹配。
  - 时间复杂度：O(m + n + z)，其中 m 是所有模式的总长度，n 是文本长度，z 是匹配数量。

- **延迟重建优化**：
  - 仅在模式添加或移除后重建自动机。
  - 避免不必要的计算，提高性能。

- **并行处理**：
  - 使用 tokio 异步运行时实现并行文本匹配。
  - 每个文本在独立的任务中处理。
  - 结果通过通道聚合。

### 伪代码
```
class AcAutomatonTool:
    def __init__(self):
        self.patterns = set()
        self.automaton = None
        self.config = Config()
    
    def add_patterns(self, new_patterns):
        # 验证并去重
        valid_patterns = [p for p in new_patterns if p and p not in self.patterns]
        if not valid_patterns:
            return
        # 添加到模式集合
        self.patterns.update(valid_patterns)
        # 触发自动机重建
        self._rebuild_automaton()
    
    def remove_patterns(self, patterns_to_remove, confirm):
        if not confirm:
            raise Error("Remove operation requires confirmation")
        # 从模式集合中移除
        self.patterns.difference_update(patterns_to_remove)
        # 触发自动机重建
        self._rebuild_automaton()
    
    def match_texts(self, texts, ignore_case, parallel):
        if not self.automaton:
            return empty_result
        
        if parallel:
            # 并行处理多个文本
            tasks = [self._match_single_text(text, ignore_case) for text in texts]
            results = await asyncio.gather(*tasks)
        else:
            # 串行处理多个文本
            results = [self._match_single_text(text, ignore_case) for text in texts]
        
        return self._aggregate_results(results)
    
    def _rebuild_automaton(self):
        # 仅在模式变更时重建
        if not self.patterns:
            self.automaton = None
            return
        # 构建 Aho-Corasick 自动机
        self.automaton = AhoCorasickBuilder()
            .ascii_case_insensitive(self.config.ignore_case)
            .build(self.patterns)
    
    def _match_single_text(self, text, ignore_case):
        # 使用 Aho-Corasick 自动机进行匹配
        matches = []
        for match in self.automaton.find_iter(text):
            matches.append({
                "pattern": self.patterns[match.pattern()],
                "start": match.start(),
                "end": match.end()
            })
        return matches
```

### 复杂度分析
- **时间复杂度**：
  - **模式添加**：O(m)，其中 m 是所有模式的总长度。
  - **自动机构建**：O(m)，用于构建失败函数。
  - **文本匹配**：O(n + z)，其中 n 是文本长度，z 是匹配数量。
  - **并行匹配**：接近线性加速比，与 CPU 核心数量相关。

- **空间复杂度**：
  - **模式存储**：O(m)，用于存储模式字符串。
  - **自动机结构**：O(m)，用于状态机。
  - **匹配结果**：O(z)，用于存储匹配位置。

## 4. 接口契约 (Interface)

### 输入 Schema
```json
{
  "action": "match",
  "patterns": ["pattern1"],
  "texts": ["text to search"],
  "confirm": false,
  "ignore_case": false,
  "parallel": false
}
```

### 字段描述
- **action**：要执行的操作类型（`add`、`remove`、`list`、`match`、`save`、`load`）。
- **patterns**：要添加/移除或用于匹配的模式字符串数组。
- **texts**：要搜索的文本字符串数组（仅用于 `match` 操作）。
- **confirm**：`remove` 操作所需的确认标志，防止意外删除。
- **ignore_case**：启用大小写不敏感匹配（重建自动机）。
- **parallel**：对多个文本使用并行处理。

### 输出 Schema
```json
{
  "success": true,
  "message": "操作成功",
  "results": [
    {
      "pattern": "pattern1",   // 匹配的模式
      "start": 0,              // 文本中的起始位置
      "end": 8                 // 文本中的结束位置
    }
  ],
  "patterns": ["pattern1"],    // 当前模式列表（用于 list 操作）
  "elapsed_ms": 15            // 执行时间（毫秒）
}
```

### 错误处理策略

#### 输入验证错误
- **空模式**：模式不能为空字符串。
- **重复模式**：尝试添加已存在的模式。
- **缺少确认**：移除操作需要明确的确认。
- **无效操作**：不支持的操作类型。

#### 运行时错误
- **自动机构建失败**：构建 Aho-Corasick 自动机时出现问题。
- **并行执行失败**：异步任务执行中的错误。
- **内存分配**：大型模式集的内存不足情况。

#### 错误恢复
- **优雅降级**：失败的操作不影响现有模式。
- **状态保留**：错误后自动机状态保持一致。
- **详细错误消息**：用于调试的本地化错误描述。

## 5. 变更记录 (Status)
> 格式：[状态] | 变更描述 | 日期

### 当前变更
- `[已完成]`：更新文档结构，统一语言为中文，添加变更记录，重命名为小写 | 2025-12-21

### 历史记录
- `[已完成]`：初始设计文档创建 | 2025-12-20

## 附加信息

### 功能特性

#### 核心功能
- **多模式匹配**：在单次文本遍历中搜索多个模式
- **模式管理**：添加、移除和列出自动机中的模式
- **大小写敏感性控制**：支持大小写敏感和大小写不敏感匹配
- **并行处理**：多个文本的可选并行匹配
- **性能指标**：执行时间跟踪，用于性能分析

#### 支持的操作
1. **添加模式** (`add`)：向自动机添加一个或多个模式
2. **移除模式** (`remove`)：移除模式，需要确认
3. **列出模式** (`list`)：显示所有当前加载的模式
4. **匹配文本** (`match`)：对输入文本执行模式匹配
5. **保存/加载** (`save`/`load`)：持久化操作（计划用于未来实现）

### 架构组件

#### 模式管理
- **验证**：确保模式非空且唯一
- **去重**：防止重复模式注册
- **原子操作**：线程安全的模式添加和移除

#### 匹配引擎
- **单文本匹配**：单个文本的高效匹配
- **并行处理**：多个文本的异步并行匹配
- **结果聚合**：合并多个匹配操作的结果

### 优化特性
- **延迟自动机重建**：仅在模式变更时重建
- **异步并行处理**：非阻塞并行文本处理
- **高效内存布局**：每个模式的最小内存开销

### 国际化

#### 支持的语言环境
- **英语 (`en`)**：主要开发语言
- **中文 (`zh-CN`)**：简体中文翻译

#### 本地化元素
- **显示名称**：用户界面中的工具名称
- **描述**：工具用途和功能
- **用户指南**：全面的使用说明
- **字段标题**：输入/输出字段标签
- **操作标签**：操作类型描述
- **错误消息**：本地化错误描述

### 测试策略

#### 单元测试
- **模式管理**：添加、移除、列出操作
- **匹配准确性**：验证正确的模式检测
- **大小写敏感性**：测试大小写敏感和不敏感模式
- **错误条件**：测试所有错误场景
- **性能**：基准测试执行时间

#### 集成测试
- **工具注册**：验证与 rt-core 的正确集成
- **模式验证**：测试输入/输出模式合规性
- **异步操作**：测试并行处理功能
- **本地化**：确保所有语言环境正确加载

#### 性能测试
- **大型模式集**：使用数千个模式进行测试
- **长文本**：在大型文档上验证性能
- **并行扩展**：使用多个文本测量加速比
- **内存使用**：监控内存消耗模式

### 未来增强

#### 计划功能
- **模式持久化**：将模式集保存到文件/从文件加载
- **正则表达式支持**：扩展到字面字符串模式之外
- **流处理**：以块为单位处理非常大的文本
- **高级统计**：详细的匹配统计和分析

#### API 扩展
- **批处理操作**：批量模式管理操作
- **配置文件**：不同用例的命名模式集
- **导出格式**：多种输出格式（CSV、XML 等）
- **集成钩子**：实时处理的回调

### 依赖关系

#### 核心依赖
- **aho-corasick**：高效的 Aho-Corasick 实现
- **tokio**：异步运行时，用于并行处理
- **serde**：输入/输出处理的序列化
- **schemars**：JSON 模式生成

#### 开发依赖
- **rt-core**：核心工具 trait 和错误类型
- **futures**：异步操作的未来工具
- **serde_json**：JSON 处理和模式处理

### 兼容性

#### 平台支持
- **Windows**：完整支持，使用原生编译
- **Linux**：完整支持，使用原生编译
- **macOS**：完整支持，使用原生编译

#### Rust 版本
- **最低要求**：Rust 2021 Edition
- **推荐版本**：最新稳定 Rust 版本
- **特性**：使用 async/await、const generics 和其他现代特性