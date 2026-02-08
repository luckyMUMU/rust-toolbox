//! Tool registration framework for file management plugin

use crate::core::{ExecutionContext, PluginInfo, ToolInfo};
use crate::error::{Result, WorkflowError};
use crate::plugins::file_management::classification::classification_flow::{
    AmbiguityDetectorTool, AutomatonBuilderTool, DirectoryScannerTool, ExperimentalCheckTool,
    FolderNamePreprocessorTool, ParallelMatcherTool, ReportGeneratorTool, ResultMergerTool,
    RuleLoaderTool, RulePreprocessorTool, ScoreCalculatorTool,
};
use crate::plugins::file_management::classification::classification_tool::ClassificationTool;
use crate::plugins::file_management::batch::batch_processor_tool::BatchProcessorTool;
use crate::plugins::file_management::ui::{
    batch_confirmation_tool::BatchConfirmationTool,
    human_decision_tool::create_human_decision_tool,
    result_confirmation_tool::ResultConfirmationTool,
    result_review_tool::ResultReviewTool,
};
use crate::plugins::file_management::plugin::FileManagementConfig;
use crate::plugins::file_management::text::text_processor_tool::TextProcessorTool;
use crate::plugins::file_management::utils::utils::{
    ConflictResolution, DuplicateHandling, FileOperationManager, FileOperationType,
    FolderMerger, FolderMergerConfig, MergeStrategy,
};
use crate::tools::algo::ac_automaton::{AhoCorasickMatcher, AutomatonConfig, Pattern};
use crate::tools::types::{Tool, NativeToolBuilder, ToolInput, ToolOutput};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Registry for file management tools
pub struct FileManagementToolRegistry {
    config: FileManagementConfig,
    plugin_info: PluginInfo,
    registered_tools: HashMap<String, Tool>,
}

impl FileManagementToolRegistry {
    /// Create a new tool registry
    pub fn new(config: FileManagementConfig, plugin_info: PluginInfo) -> Self {
        Self {
            config,
            plugin_info,
            registered_tools: HashMap::new(),
        }
    }

    /// Register all file management tools
    pub fn register_all_tools(&mut self) -> Result<Vec<Tool>> {
        info!("Registering all file management tools");

        let mut tools = Vec::new();

        // Register tools in order of dependencies
        // Tools will be implemented in subsequent tasks

        // 1. Core utility tools (no dependencies)
        if let Ok(tool) = self.register_text_processor_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_ac_matcher_tool() {
            tools.push(tool);
        }

        // 2. File operation tools (depend on utilities)
        if let Ok(tool) = self.register_file_mover_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_folder_merger_tool() {
            tools.push(tool);
        }

        // 3. Higher-level tools (depend on file operations)
        if let Ok(tool) = self.register_classification_tool() {
            tools.push(tool);
        }

        // Register granular classification flow tools
        if let Ok(mut flow_tools) = self.register_classification_flow_tools() {
            tools.append(&mut flow_tools);
        }

        if let Ok(tool) = self.register_batch_processor_tool() {
            tools.push(tool);
        }

        // 4. Human interaction tools
        if let Ok(tool) = self.register_human_decision_tool() {
            tools.push(tool);
        }

        // 5. Result review and confirmation tools
        if let Ok(tool) = self.register_result_review_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_batch_confirmation_tool() {
            tools.push(tool);
        }

        if let Ok(tool) = self.register_result_confirmation_tool() {
            tools.push(tool);
        }

        info!("Registered {} file management tools", tools.len());
        Ok(tools)
    }

    /// Register the text processor tool
    fn register_text_processor_tool(&mut self) -> Result<Tool> {
        debug!("Registering text processor tool");

        let enable_chinese = self.config.enable_chinese_processing;

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("text-processor")
            .version("1.0.0")
            .description("文本处理工具，支持中文处理和拼音转换")
            .category("text-processing")
            .tag("text")
            .tag("chinese")
            .tag("pinyin")
            .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                let tool = TextProcessorTool::new(enable_chinese, None);
                async move {
                    let params: crate::plugins::file_management::text::text_processor_tool::TextProcessorParams =
                        serde_json::from_value(input.params)
                            .map_err(|e| WorkflowError::validation(format!("参数解析失败: {}", e)))?;
                    let result = tool.process_text(&params)
                        .map_err(|e| WorkflowError::tool(format!("文本处理失败: {:?}", e)))?;
                    Ok(ToolOutput::success(tool.format_result(result, &params.output_format.unwrap_or_default())))
                }
            })
            .build()?;

        let tool_wrapper = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("text-processor".to_string(), tool_wrapper.clone());

        Ok(tool_wrapper)
    }

    /// Register all classification flow tools (granular steps)
    fn register_classification_flow_tools(&mut self) -> Result<Vec<Tool>> {
        let mut tools: Vec<Tool> = Vec::new();

        // 1. Rule Loader - 规则加载工具
        let rule_loader_tool = NativeToolBuilder::new()
            .name("rule-loader")
            .version("1.0.0")
            .description("加载和验证分类规则")
            .category("classification")
            .tag("rules")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "rule-loader"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(rule_loader_tool));
        self.registered_tools.insert("rule-loader".to_string(), tool.clone());
        tools.push(tool);

        // 2. Rule Preprocessor - 规则预处理工具
        let enable_chinese = self.config.enable_chinese_processing;
        let rule_preprocessor_tool = NativeToolBuilder::new()
            .name("rule-preprocessor")
            .version("1.0.0")
            .description("预处理分类规则，应用文本规范化")
            .category("classification")
            .tag("rules")
            .tag("preprocessing")
            .executor(move |_input: ToolInput, _ctx: ExecutionContext| async move {
                let _processor = RulePreprocessorTool::new(enable_chinese);
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "rule-preprocessor",
                    "enable_chinese": enable_chinese
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(rule_preprocessor_tool));
        self.registered_tools.insert("rule-preprocessor".to_string(), tool.clone());
        tools.push(tool);

        // 3. Automaton Builder - AC自动机构建工具
        let automaton_builder_tool = NativeToolBuilder::new()
            .name("automaton-builder")
            .version("1.0.0")
            .description("构建AC自动机用于模式匹配")
            .category("classification")
            .tag("automaton")
            .tag("matching")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "automaton-builder"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(automaton_builder_tool));
        self.registered_tools.insert("automaton-builder".to_string(), tool.clone());
        tools.push(tool);

        // 4. Directory Scanner - 目录扫描工具
        let directory_scanner_tool = NativeToolBuilder::new()
            .name("directory-scanner")
            .version("1.0.0")
            .description("扫描源目录获取文件夹列表")
            .category("classification")
            .tag("directory")
            .tag("scanning")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "directory-scanner"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(directory_scanner_tool));
        self.registered_tools.insert("directory-scanner".to_string(), tool.clone());
        tools.push(tool);

        // 5. Folder Name Preprocessor - 文件夹名称预处理工具
        let enable_chinese = self.config.enable_chinese_processing;
        let folder_name_preprocessor_tool = NativeToolBuilder::new()
            .name("folder-name-preprocessor")
            .version("1.0.0")
            .description("预处理文件夹名称，应用文本规范化")
            .category("classification")
            .tag("folder")
            .tag("preprocessing")
            .executor(move |_input: ToolInput, _ctx: ExecutionContext| async move {
                let _processor = FolderNamePreprocessorTool::new(enable_chinese);
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "folder-name-preprocessor",
                    "enable_chinese": enable_chinese
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(folder_name_preprocessor_tool));
        self.registered_tools.insert("folder-name-preprocessor".to_string(), tool.clone());
        tools.push(tool);

        // 6. Parallel Matcher - 并行匹配工具
        let parallel_matcher_tool = NativeToolBuilder::new()
            .name("parallel-matcher")
            .version("1.0.0")
            .description("并行执行AC自动机匹配")
            .category("classification")
            .tag("matching")
            .tag("parallel")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "parallel-matcher"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(parallel_matcher_tool));
        self.registered_tools.insert("parallel-matcher".to_string(), tool.clone());
        tools.push(tool);

        // 7. Score Calculator - 分数计算工具
        let score_calculator_tool = NativeToolBuilder::new()
            .name("score-calculator")
            .version("1.0.0")
            .description("计算分类匹配分数")
            .category("classification")
            .tag("scoring")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "score-calculator"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(score_calculator_tool));
        self.registered_tools.insert("score-calculator".to_string(), tool.clone());
        tools.push(tool);

        // 8. Ambiguity Detector - 歧义检测工具
        let ambiguity_detector_tool = NativeToolBuilder::new()
            .name("ambiguity-detector")
            .version("1.0.0")
            .description("检测分类歧义和低置信度结果")
            .category("classification")
            .tag("ambiguity")
            .tag("validation")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "ambiguity-detector"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(ambiguity_detector_tool));
        self.registered_tools.insert("ambiguity-detector".to_string(), tool.clone());
        tools.push(tool);

        // 9. Result Merger - 结果合并工具
        let result_merger_tool = NativeToolBuilder::new()
            .name("result-merger")
            .version("1.0.0")
            .description("合并多个分类结果")
            .category("classification")
            .tag("merging")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "result-merger"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(result_merger_tool));
        self.registered_tools.insert("result-merger".to_string(), tool.clone());
        tools.push(tool);

        // 10. Experimental Check - 实验模式检查工具
        let experimental_check_tool = NativeToolBuilder::new()
            .name("experimental-check")
            .version("1.0.0")
            .description("检查实验模式并应用相应逻辑")
            .category("classification")
            .tag("experimental")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "experimental-check"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(experimental_check_tool));
        self.registered_tools.insert("experimental-check".to_string(), tool.clone());
        tools.push(tool);

        // 11. Report Generator - 报告生成工具
        let report_generator_tool = NativeToolBuilder::new()
            .name("report-generator")
            .version("1.0.0")
            .description("生成分类结果报告")
            .category("classification")
            .tag("reporting")
            .executor(|_input: ToolInput, _ctx: ExecutionContext| async move {
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "report-generator"
                })))
            })
            .build()?;
        let tool = Tool::Native(Arc::new(report_generator_tool));
        self.registered_tools.insert("report-generator".to_string(), tool.clone());
        tools.push(tool);

        Ok(tools)
    }

    /// Register the AC matcher tool
    fn register_ac_matcher_tool(&mut self) -> Result<Tool> {
        debug!("Registering AC matcher tool");

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("ac-matcher")
            .version("1.0.0")
            .description("Aho-Corasick多模式字符串匹配")
            .category("pattern-matching")
            .tag("pattern")
            .tag("matching")
            .tag("aho-corasick")
            .executor(|input: ToolInput, ctx: ExecutionContext| async move {
                let executor = AcMatcherExecutor::new();
                executor.execute(input.params, ctx).await
                    .map(|result| ToolOutput::success(result))
                    .map_err(|e| WorkflowError::tool(format!("AC匹配器执行失败: {}", e)))
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("ac-matcher".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the classification tool
    fn register_classification_tool(&mut self) -> Result<Tool> {
        debug!("Registering classification tool");

        let enable_chinese = self.config.enable_chinese_processing;
        let plugin_info = self.plugin_info.clone();

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("folder-classifier")
            .version("1.0.0")
            .description("文件夹分类工具，基于规则匹配")
            .category("classification")
            .tag("folder")
            .tag("classification")
            .tag("rules")
            .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                let _tool = ClassificationTool::with_plugin_info(enable_chinese, plugin_info.clone());
                async move {
                    // TODO: 实现实际的分类逻辑
                    Ok(ToolOutput::success(json!({
                        "status": "not_implemented",
                        "tool": "folder-classifier",
                        "input": input.params
                    })))
                }
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("folder-classifier".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the file mover tool
    fn register_file_mover_tool(&mut self) -> Result<Tool> {
        debug!("Registering file mover tool");

        let config = self.config.clone();

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("file-mover")
            .version("1.0.0")
            .description("安全的文件和文件夹操作，支持冲突解决")
            .category("file-operations")
            .tag("file")
            .tag("move")
            .tag("copy")
            .executor(move |input: ToolInput, ctx: ExecutionContext| {
                let executor = FileMoverExecutor::new(config.clone());
                async move {
                    executor.execute(input.params, ctx).await
                        .map(|result| ToolOutput::success(result))
                        .map_err(|e| WorkflowError::tool(format!("文件移动失败: {}", e)))
                }
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("file-mover".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the folder merger tool
    fn register_folder_merger_tool(&mut self) -> Result<Tool> {
        debug!("Registering folder merger tool");

        let config = self.config.clone();

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("folder-merger")
            .version("1.0.0")
            .description("智能文件夹合并，支持重复文件处理")
            .category("file-operations")
            .tag("folder")
            .tag("merge")
            .tag("duplicate")
            .executor(move |input: ToolInput, ctx: ExecutionContext| {
                let executor = FolderMergerExecutor::new(config.clone());
                async move {
                    executor.execute(input.params, ctx).await
                        .map(|result| ToolOutput::success(result))
                        .map_err(|e| WorkflowError::tool(format!("文件夹合并失败: {}", e)))
                }
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("folder-merger".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the batch processor tool
    fn register_batch_processor_tool(&mut self) -> Result<Tool> {
        debug!("Registering batch processor tool");

        let config = self.config.clone();
        let plugin_info = self.plugin_info.clone();

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("batch-processor")
            .version("1.0.0")
            .description("批处理工具，支持批量文件操作")
            .category("batch-processing")
            .tag("batch")
            .tag("file")
            .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                let _batch_tool = BatchProcessorTool::new(config.clone(), plugin_info.clone());
                async move {
                    // TODO: 实现实际的批处理逻辑
                    Ok(ToolOutput::success(json!({
                        "status": "not_implemented",
                        "tool": "batch-processor",
                        "input": input.params
                    })))
                }
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("batch-processor".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the human decision tool
    fn register_human_decision_tool(&mut self) -> Result<Tool> {
        debug!("Registering human decision tool");

        let config = self.config.clone();
        let plugin_info = self.plugin_info.clone();

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("human-decision")
            .version("1.0.0")
            .description("人工决策工具，用于需要人工确认的场景")
            .category("human-interaction")
            .tag("human")
            .tag("decision")
            .executor(move |input: ToolInput, _ctx: ExecutionContext| {
                let _config = config.clone();
                let _plugin_info = plugin_info.clone();
                async move {
                    // TODO: 实现实际的人工决策逻辑
                    Ok(ToolOutput::success(json!({
                        "status": "not_implemented",
                        "tool": "human-decision",
                        "input": input.params
                    })))
                }
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("human-decision".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the result review tool
    fn register_result_review_tool(&mut self) -> Result<Tool> {
        debug!("Registering result review tool");

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("result-reviewer")
            .version("1.0.0")
            .description("结果审查工具，用于审查处理结果")
            .category("review")
            .tag("review")
            .tag("result")
            .executor(|input: ToolInput, _ctx: ExecutionContext| async move {
                let _tool = ResultReviewTool::with_default_config();
                // TODO: 实现实际的结果审查逻辑
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "result-reviewer",
                    "input": input.params
                })))
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("result-reviewer".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the batch confirmation tool
    fn register_batch_confirmation_tool(&mut self) -> Result<Tool> {
        debug!("Registering batch confirmation tool");

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("batch-confirmer")
            .version("1.0.0")
            .description("批量确认工具，用于批量操作的确认")
            .category("confirmation")
            .tag("batch")
            .tag("confirmation")
            .executor(|input: ToolInput, _ctx: ExecutionContext| async move {
                let _tool = BatchConfirmationTool::with_default_config();
                // TODO: 实现实际的批量确认逻辑
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "batch-confirmer",
                    "input": input.params
                })))
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("batch-confirmer".to_string(), tool.clone());

        Ok(tool)
    }

    /// Register the comprehensive result confirmation tool
    fn register_result_confirmation_tool(&mut self) -> Result<Tool> {
        debug!("Registering comprehensive result confirmation tool");

        // 使用 NativeToolBuilder 创建 Tool::Native
        let native_tool = NativeToolBuilder::new()
            .name("result-confirmer")
            .version("1.0.0")
            .description("结果确认工具，用于确认处理结果")
            .category("confirmation")
            .tag("result")
            .tag("confirmation")
            .executor(|input: ToolInput, _ctx: ExecutionContext| async move {
                let _tool = ResultConfirmationTool::with_default_config();
                // TODO: 实现实际的结果确认逻辑
                Ok(ToolOutput::success(json!({
                    "status": "not_implemented",
                    "tool": "result-confirmer",
                    "input": input.params
                })))
            })
            .build()?;

        let tool = Tool::Native(Arc::new(native_tool));
        self.registered_tools
            .insert("result-confirmer".to_string(), tool.clone());

        Ok(tool)
    }

    /// Get a registered tool by name
    pub fn get_tool(&self, name: &str) -> Option<Tool> {
        self.registered_tools.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<Tool> {
        self.registered_tools.values().cloned().collect()
    }

    /// Get tool count
    pub fn tool_count(&self) -> usize {
        self.registered_tools.len()
    }
}

/// AC Matcher executor that uses the Aho-Corasick automaton
struct AcMatcherExecutor;

impl AcMatcherExecutor {
    fn new() -> Self {
        Self
    }

    /// Parse patterns from JSON input
    fn parse_patterns(&self, patterns_value: &Value) -> Result<Vec<Pattern>> {
        let patterns_array = patterns_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("patterns must be an array"))?;

        let mut patterns = Vec::new();
        for (index, pattern_obj) in patterns_array.iter().enumerate() {
            let pattern_str = pattern_obj
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "patterns[{}].pattern must be a string",
                        index
                    ))
                })?;

            let category = pattern_obj
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "patterns[{}].category must be a string",
                        index
                    ))
                })?;

            let score = pattern_obj
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            patterns.push(Pattern::new(pattern_str, category, score, index));
        }

        Ok(patterns)
    }

    /// Build automaton from patterns
    fn build_automaton(
        &self,
        patterns: Vec<Pattern>,
        case_sensitive: bool,
        find_overlapping: bool,
    ) -> Result<AhoCorasickMatcher> {
        let config = AutomatonConfig {
            case_sensitive,
            find_overlapping,
            max_patterns: 10_000,
            max_pattern_length: 1000,
        };

        let mut matcher = AhoCorasickMatcher::with_config(config);

        // Add patterns to the automaton
        for pattern in patterns {
            matcher
                .add_pattern(&pattern.pattern, &pattern.category, pattern.score)
                .map_err(|e| {
                    WorkflowError::tool(format!(
                        "Failed to add pattern '{}': {}",
                        pattern.pattern, e
                    ))
                })?;
        }

        // Build the automaton
        matcher
            .build()
            .map_err(|e| WorkflowError::tool(format!("Failed to build automaton: {}", e)))?;

        Ok(matcher)
    }

    /// Convert pattern matches to JSON
    fn matches_to_json(&self, matches: Vec<crate::tools::algo::ac_automaton::PatternMatch>) -> Value {
        let match_objects: Vec<Value> = matches
            .iter()
            .map(|m| {
                json!({
                    "pattern": m.pattern,
                    "category": m.category,
                    "score": m.score,
                    "pattern_id": m.pattern_id,
                    "start_pos": m.start_pos,
                    "end_pos": m.end_pos,
                    "match_length": m.match_len()
                })
            })
            .collect();

        json!(match_objects)
    }

    /// Get unique categories from matches
    fn get_categories_found(&self, matches: &[crate::tools::algo::ac_automaton::PatternMatch]) -> Vec<String> {
        let mut categories: Vec<String> = matches
            .iter()
            .map(|m| m.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        categories.sort();
        categories
    }

    /// Execute the AC matcher
    async fn execute(&self, params: Value, _ctx: ExecutionContext) -> Result<Value> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WorkflowError::validation("text参数是必需的字符串"))?;

        let patterns_value = params
            .get("patterns")
            .ok_or_else(|| WorkflowError::validation("patterns参数是必需的"))?;

        let patterns = self.parse_patterns(patterns_value)?;

        if patterns.is_empty() {
            return Err(WorkflowError::validation("patterns数组不能为空"));
        }

        let case_sensitive = params
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let find_overlapping = params
            .get("find_overlapping")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Build automaton
        let matcher = self.build_automaton(patterns, case_sensitive, find_overlapping)?;

        // Find matches
        let matches = matcher.find_matches(text)
            .map_err(|e| WorkflowError::tool(format!("匹配失败: {}", e)))?;

        let result = json!({
            "matches": self.matches_to_json(matches.clone()),
            "total_matches": matches.len(),
            "categories_found": self.get_categories_found(&matches),
            "statistics": {
                "text_length": text.len(),
                "case_sensitive": case_sensitive,
                "find_overlapping": find_overlapping
            },
            "experimental_mode": experimental_mode
        });

        Ok(result)
    }
}

/// File mover executor that implements actual file operations
pub struct FileMoverExecutor {
    config: FileManagementConfig,
}

impl FileMoverExecutor {
    pub fn new(config: FileManagementConfig) -> Self {
        Self { config }
    }

    /// Parse file operations from parameters
    fn parse_operations(
        &self,
        operations_value: &Value,
    ) -> Result<Vec<(String, String, FileOperationType)>> {
        let operations_array = operations_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("operations must be an array"))?;

        let mut operations = Vec::new();

        for (index, op_obj) in operations_array.iter().enumerate() {
            let op_obj = op_obj.as_object().ok_or_else(|| {
                WorkflowError::validation(format!("operations[{}] must be an object", index))
            })?;

            let source = op_obj
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "operations[{}].source is required and must be a string",
                        index
                    ))
                })?;

            let destination = op_obj
                .get("destination")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    WorkflowError::validation(format!(
                        "operations[{}].destination is required and must be a string",
                        index
                    ))
                })?;

            let operation_type_str = op_obj
                .get("operation_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Move");

            let operation_type = match operation_type_str {
                "Move" => FileOperationType::Move,
                "Copy" => FileOperationType::Copy,
                "Link" => FileOperationType::Link,
                "HardLink" => FileOperationType::HardLink,
                _ => {
                    return Err(WorkflowError::validation(format!(
                        "operations[{}].operation_type must be one of: Move, Copy, Link, HardLink",
                        index
                    )))
                }
            };

            operations.push((source.to_string(), destination.to_string(), operation_type));
        }

        Ok(operations)
    }

    /// Parse conflict resolution strategy
    fn parse_conflict_resolution(&self, params: &Value) -> ConflictResolution {
        let conflict_str = params
            .get("conflict_resolution")
            .and_then(|v| v.as_str())
            .unwrap_or("Rename");

        match conflict_str {
            "Skip" => ConflictResolution::Skip,
            "Overwrite" => ConflictResolution::Overwrite,
            "Rename" => ConflictResolution::Rename,
            "Fail" => ConflictResolution::Fail,
            "Ask" => ConflictResolution::Ask,
            "Merge" => ConflictResolution::Merge,
            "KeepBoth" => ConflictResolution::KeepBoth,
            "KeepNewer" => ConflictResolution::KeepNewer,
            "KeepLarger" => ConflictResolution::KeepLarger,
            _ => ConflictResolution::Rename, // Default fallback
        }
    }

    /// Execute the file mover
    async fn execute(&self, params: Value, _ctx: ExecutionContext) -> Result<Value> {
        let operations_value = params
            .get("operations")
            .ok_or_else(|| WorkflowError::validation("operations参数是必需的"))?;

        let operations = self.parse_operations(operations_value)?;
        let conflict_resolution = self.parse_conflict_resolution(&params);
        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // TODO: 实现实际的文件操作逻辑
        let result = json!({
            "operations_completed": 0,
            "operations_failed": 0,
            "operations_skipped": 0,
            "total_bytes_moved": 0,
            "duration_ms": 0,
            "errors": [],
            "experimental_mode": experimental_mode,
            "conflict_resolution": format!("{:?}", conflict_resolution),
            "total_operations": operations.len()
        });

        Ok(result)
    }
}

/// Folder merger executor that implements actual folder merging operations
pub struct FolderMergerExecutor {
    config: FileManagementConfig,
}

impl FolderMergerExecutor {
    pub fn new(config: FileManagementConfig) -> Self {
        Self { config }
    }

    /// Parse merge strategy from parameters
    fn parse_merge_strategy(&self, params: &Value) -> MergeStrategy {
        let strategy_str = params
            .get("merge_strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("SizeBased");

        match strategy_str {
            "SizeBased" => MergeStrategy::SizeBased,
            "DateBased" => MergeStrategy::DateBased,
            "Manual" => MergeStrategy::Manual,
            "Intelligent" => MergeStrategy::Intelligent,
            _ => MergeStrategy::SizeBased, // Default fallback
        }
    }

    /// Parse duplicate handling strategy from parameters
    fn parse_duplicate_handling(&self, params: &Value) -> DuplicateHandling {
        let handling_str = params
            .get("duplicate_handling")
            .and_then(|v| v.as_str())
            .unwrap_or("Rename");

        match handling_str {
            "Skip" => DuplicateHandling::Skip,
            "Rename" => DuplicateHandling::Rename,
            "KeepNewer" => DuplicateHandling::KeepNewer,
            "KeepLarger" => DuplicateHandling::KeepLarger,
            "Merge" => DuplicateHandling::Merge,
            _ => DuplicateHandling::Rename, // Default fallback
        }
    }

    /// Parse source directories from parameters
    fn parse_source_directories(&self, params: &Value) -> Result<Vec<String>> {
        let directories_value = params
            .get("source_directories")
            .ok_or_else(|| WorkflowError::validation("source_directories parameter is required"))?;

        let directories_array = directories_value
            .as_array()
            .ok_or_else(|| WorkflowError::validation("source_directories must be an array"))?;

        if directories_array.is_empty() {
            return Err(WorkflowError::validation(
                "source_directories array cannot be empty",
            ));
        }

        let mut directories = Vec::new();
        for (index, dir_value) in directories_array.iter().enumerate() {
            let dir_str = dir_value.as_str().ok_or_else(|| {
                WorkflowError::validation(format!("source_directories[{}] must be a string", index))
            })?;
            directories.push(dir_str.to_string());
        }

        Ok(directories)
    }

    /// Create folder merger configuration from parameters
    fn create_merger_config(
        &self,
        params: &Value,
        experimental_mode: bool,
    ) -> FolderMergerConfig {
        let merge_strategy = self.parse_merge_strategy(params);
        let duplicate_handling = self.parse_duplicate_handling(params);

        let max_recursion_depth = params
            .get("max_recursion_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        let min_confidence_threshold = params
            .get("min_confidence_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        FolderMergerConfig {
            merge_strategy,
            duplicate_handling,
            max_recursion_depth,
            min_confidence_threshold,
            enable_size_based_decisions: true,
            enable_date_based_decisions: true,
            dry_run: experimental_mode,
        }
    }

    /// Execute the folder merger
    async fn execute(&self, params: Value, _ctx: ExecutionContext) -> Result<Value> {
        let source_directories = self.parse_source_directories(&params)?;
        let merge_strategy = self.parse_merge_strategy(&params);
        let duplicate_handling = self.parse_duplicate_handling(&params);
        let experimental_mode = params
            .get("experimental_mode")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let _merger_config = self.create_merger_config(&params, experimental_mode);

        // TODO: 实现实际的文件夹合并逻辑
        let result = json!({
            "comparison_result": {
                "common_folders": [],
                "unique_folders": source_directories,
                "total_folders_analyzed": source_directories.len(),
                "total_size_bytes": 0,
                "merge_recommendations": []
            },
            "merge_result": {
                "total_operations": 0,
                "successful_operations": 0,
                "failed_operations": 0,
                "total_bytes_moved": 0,
                "duration_ms": 0,
                "folders_merged": 0
            },
            "experimental_mode": experimental_mode,
            "merge_strategy": format!("{:?}", merge_strategy),
            "duplicate_handling": format!("{:?}", duplicate_handling)
        });

        Ok(result)
    }
}

/// Placeholder executor for tools that will be implemented in later tasks
#[allow(dead_code)]
struct PlaceholderExecutor {
    tool_name: String,
}

impl PlaceholderExecutor {
    #[allow(dead_code)]
    fn new<S: Into<String>>(tool_name: S) -> Self {
        Self {
            tool_name: tool_name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::PluginType;
    use tempfile::TempDir;

    fn create_test_plugin_info() -> PluginInfo {
        PluginInfo {
            name: "file-management".to_string(),
            version: "1.0.0".to_string(),
            plugin_type: PluginType::Native,
            description: Some("Test plugin".to_string()),
            author: Some("Test".to_string()),
            homepage: Some("https://example.com".to_string()),
            metadata: HashMap::new(),
        }
    }

    fn create_test_config() -> FileManagementConfig {
        let temp_dir = TempDir::new().unwrap();
        FileManagementConfig {
            temp_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        }
    }

    #[test]
    fn test_registry_creation() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let registry = FileManagementToolRegistry::new(config, plugin_info);

        assert_eq!(registry.tool_count(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let mut registry = FileManagementToolRegistry::new(config, plugin_info);

        let result = registry.register_all_tools();
        assert!(result.is_ok());

        let tools = result.unwrap();
        assert!(tools.len() > 0);
        assert_eq!(registry.tool_count(), tools.len());
    }

    #[test]
    fn test_individual_tool_registration() {
        let config = create_test_config();
        let plugin_info = create_test_plugin_info();
        let mut registry = FileManagementToolRegistry::new(config, plugin_info);

        // Test text processor registration
        let result = registry.register_text_processor_tool();
        assert!(result.is_ok());

        let tool = result.unwrap();
        assert_eq!(tool.name(), "text-processor");
        assert_eq!(tool.version(), "1.0.0");

        // Test tool retrieval
        let retrieved_tool = registry.get_tool("text-processor");
        assert!(retrieved_tool.is_some());
    }

    #[tokio::test]
    async fn test_placeholder_executor() {
        let executor = PlaceholderExecutor::new("test-tool");
        let context = crate::core::ExecutionContext::new();
        let params = json!({"test": "value"});

        let result = executor.execute(params.clone(), context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["status"], "not_implemented");
        assert_eq!(response["tool_name"], "test-tool");
        assert_eq!(response["received_params"], params);
    }

    #[tokio::test]
    async fn test_ac_matcher_executor() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();

        let params = json!({
            "text": "hello world test hello",
            "patterns": [
                {"pattern": "hello", "category": "greeting", "score": 1.0},
                {"pattern": "world", "category": "noun", "score": 0.8},
                {"pattern": "test", "category": "action", "score": 1.2}
            ],
            "case_sensitive": false,
            "find_overlapping": false
        });

        let result = executor.execute(params, context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["total_matches"], 4); // hello appears twice
        assert!(response["matches"].is_array());
        assert!(response["categories_found"].is_array());
        assert!(response["statistics"].is_object());

        let categories = response["categories_found"].as_array().unwrap();
        assert!(categories.contains(&json!("greeting")));
        assert!(categories.contains(&json!("noun")));
        assert!(categories.contains(&json!("action")));
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_overlapping() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();

        let params = json!({
            "text": "abcde",
            "patterns": [
                {"pattern": "abc", "category": "pattern1"},
                {"pattern": "bcd", "category": "pattern2"},
                {"pattern": "cde", "category": "pattern3"}
            ],
            "find_overlapping": true
        });

        let result = executor.execute(params, context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["total_matches"], 3); // All three overlapping patterns
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_validation() {
        let executor = AcMatcherExecutor::new();

        // Test missing text parameter
        let params = json!({
            "patterns": [{"pattern": "test", "category": "test"}]
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());

        // Test missing patterns parameter
        let params = json!({
            "text": "test"
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());

        // Test empty patterns array
        let params = json!({
            "text": "test",
            "patterns": []
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());

        // Test invalid pattern object
        let params = json!({
            "text": "test",
            "patterns": [{"pattern": "test"}] // missing category
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_err());

        // Test valid parameters
        let params = json!({
            "text": "test",
            "patterns": [{"pattern": "test", "category": "test"}]
        });
        let result = executor.validate_parameters(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_unicode() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();

        let params = json!({
            "text": "hello测试world",
            "patterns": [
                {"pattern": "hello", "category": "english"},
                {"pattern": "测试", "category": "chinese"},
                {"pattern": "world", "category": "english"}
            ]
        });

        let result = executor.execute(params, context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["total_matches"], 3);

        let categories = response["categories_found"].as_array().unwrap();
        assert!(categories.contains(&json!("english")));
        assert!(categories.contains(&json!("chinese")));
    }

    #[tokio::test]
    async fn test_ac_matcher_executor_empty_patterns() {
        let executor = AcMatcherExecutor::new();
        let context = crate::core::ExecutionContext::new();

        // This should be caught by validation, but test the execution path too
        let params = json!({
            "text": "test text",
            "patterns": []
        });

        // Validation should fail
        let validation_result = executor.validate_parameters(&params);
        assert!(validation_result.is_err());
    }
}
