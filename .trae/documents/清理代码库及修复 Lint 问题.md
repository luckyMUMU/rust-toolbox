# 清理代码库计划

我们将执行以下步骤来清理和维护代码库：

## 1. 修复损坏的测试文件
- **目标**: `tests/tui_performance_benchmark_tests.rs`
- **动作**: 修复因截断导致的代码错误，补全 `BenchmarkResult` 结构体。

## 2. 修复代码异味 (Clippy Lint Fixes)
- **Workflow 模块**:
  - 移除 `src/workflow/` 下多个文件中冗余的 `.into()` 错误转换调用 (definition.rs, engine.rs, scheduler.rs, validator.rs, execution_manager.rs)。
  - 简化 `src/workflow/scheduler.rs` 中的布尔逻辑。
  - 在 `src/workflow/result_cache.rs` 中为 `CacheKey` 实现标准的 `Display` trait，替代非标准的 `to_string` 方法。
  - 抑制 `src/workflow/result_cache.rs` 中 `cache_node_result` 函数参数过多的警告 (暂不重构以保持兼容性)。
- **TUI 界面**:
  - 优化 `src/interfaces/tui/widgets/system_status.rs` 中不必要的 `vec!` 宏使用，改用静态数组。

## 3. 代码格式化 (Code Formatting)
- **动作**: 运行 `cargo fmt` 自动修复以下文件的代码风格问题：
  - `examples/experimental_classify.rs`
  - `src/interfaces/tui/virtualization.rs`
  - `src/plugins/file_management/classification_tool.rs`
  - `src/plugins/file_management/rule_config.rs`
  - `src/tools/mod.rs`

## 4. 清理临时文件
- **动作**: 将根目录下的临时报告和输出文件移动到 `.temp/` 文件夹（如果不存在则创建）：
  - `check_errors.txt`, `check_output.txt`
  - `python_output.txt`, `rust_output*.txt`
  - `compare_results.py`, `comparison_report.md`
  - `DOCS_COMPLETION_REPORT.md`, `PROJECT_ORGANIZATION_COMPLETE.md`

## 5. 验证
- **动作**: 运行 `cargo check` 确保所有修改无误且编译通过。
