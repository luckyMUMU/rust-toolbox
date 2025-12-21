use async_trait::async_trait;
use rt_core::{Tool, CoreError, Result, Locale};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use schemars::JsonSchema;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use std::sync::{Arc, RwLock};
use crate::utils::ToolI18n;
use futures::future::try_join_all;

/// 匹配结果
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MatchResult {
    /// 匹配到的模式串
    pattern: String,
    /// 起始索引
    start: usize,
    /// 结束索引
    end: usize,
}

/// AC自动机输入参数
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AcAutomatonInput {
    /// 操作类型: add, remove, list, match, save, load
    action: String,
    /// 模式串列表
    patterns: Vec<String>,
    /// 待匹配文本列表
    texts: Option<Vec<String>>,
    /// 确认删除标志
    confirm: Option<bool>,
    /// 是否忽略大小写
    ignore_case: Option<bool>,
    /// 是否启用并行匹配
    parallel: Option<bool>,
}

/// AC自动机输出结果
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AcAutomatonOutput {
    /// 是否成功
    success: bool,
    /// 消息
    message: String,
    /// 匹配结果列表
    results: Option<Vec<MatchResult>>,
    /// 模式串列表
    patterns: Option<Vec<String>>,
    /// 操作耗时（毫秒）
    elapsed_ms: u64,
}

/// AC自动机内部实现
struct AcAutomatonImpl {
    /// 底层aho-corasick自动机
    automaton: RwLock<Option<AhoCorasick>>,
    /// 模式串集合
    patterns: RwLock<Vec<String>>,
    /// 构建器配置
    builder: RwLock<AhoCorasickBuilder>,
}

impl AcAutomatonImpl {
    /// 创建新的AC自动机实例
    fn new() -> Self {
        Self {
            automaton: RwLock::new(None),
            patterns: RwLock::new(Vec::new()),
            builder: RwLock::new(AhoCorasickBuilder::new()),
        }
    }
    
    /// 添加单个模式串
    fn add_pattern(&self, pattern: &str) -> Result<()> {
        // 输入验证
        if pattern.is_empty() {
            return Err(CoreError::InvalidInput("模式串不能为空".to_string()));
        }
        
        // 检查是否重复
        let patterns = self.patterns.read().unwrap();
        if patterns.contains(&pattern.to_string()) {
            return Err(CoreError::InvalidInput(format!("模式串 '{}' 已存在", pattern)));
        }
        drop(patterns);
        
        // 添加模式串
        let mut patterns = self.patterns.write().unwrap();
        patterns.push(pattern.to_string());
        drop(patterns);
        
        // 重建自动机
        self.rebuild()
    }
    
    /// 批量添加模式串
    fn add_patterns(&self, patterns: &[String]) -> Result<()> {
        for pattern in patterns {
            self.add_pattern(pattern)?;
        }
        Ok(())
    }
    
    /// 删除单个模式串
    fn remove_pattern(&self, pattern: &str, confirm: bool) -> Result<()> {
        // 检查确认标志
        if !confirm {
            return Err(CoreError::InvalidInput("删除操作需要确认".to_string()));
        }
        
        // 检查模式串是否存在
        let patterns = self.patterns.read().unwrap();
        if !patterns.contains(&pattern.to_string()) {
            return Err(CoreError::InvalidInput(format!("模式串 '{}' 不存在", pattern)));
        }
        drop(patterns);
        
        // 删除模式串
        let mut patterns = self.patterns.write().unwrap();
        patterns.retain(|p| p != pattern);
        drop(patterns);
        
        // 重建自动机
        self.rebuild()
    }
    
    /// 批量删除模式串
    fn remove_patterns(&self, patterns: &[String], confirm: bool) -> Result<()> {
        for pattern in patterns {
            self.remove_pattern(pattern, confirm)?;
        }
        Ok(())
    }
    
    /// 单文本匹配
    fn match_text(&self, text: &str) -> Vec<MatchResult> {
        let automaton = self.automaton.read().unwrap();
        let patterns = self.patterns.read().unwrap();
        
        match &*automaton {
            Some(ac) => {
                ac.find_iter(text)
                    .map(|mat| MatchResult {
                        pattern: patterns[mat.pattern()].clone(),
                        start: mat.start(),
                        end: mat.end(),
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }
    
    /// 重建自动机
    fn rebuild(&self) -> Result<()> {
        let patterns = self.patterns.read().unwrap();
        let builder = self.builder.read().unwrap();
        
        let automaton = builder.build(patterns.iter().map(|p| p.as_str()))
            .map_err(|e| CoreError::ToolFailure(format!("构建自动机失败: {}", e)))?;
        
        let mut automaton_lock = self.automaton.write().unwrap();
        *automaton_lock = Some(automaton);
        
        Ok(())
    }
    
    /// 设置是否忽略大小写
    fn set_ignore_case(&self, ignore_case: bool) -> Result<()> {
        let mut builder = self.builder.write().unwrap();
        if ignore_case {
            builder.ascii_case_insensitive(true);
        } else {
            builder.ascii_case_insensitive(false);
        }
        drop(builder);
        
        // 重建自动机
        self.rebuild()
    }
}

/// AC自动机工具
pub struct AcAutomatonTool {
    /// 国际化资源
    i18n: ToolI18n,
    /// 自动机实例
    automaton: Arc<AcAutomatonImpl>,
}

// 注册工具
crate::register_tool!(AcAutomatonTool);

// 测试模块
#[cfg(test)]
mod tests;

impl AcAutomatonTool {
    /// 创建新的AC自动机工具实例
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for AcAutomatonTool {
    /// 创建默认的AC自动机工具实例
    fn default() -> Self {
        Self {
            i18n: ToolI18n::new(
                include_str!("locales/tool.en.json"),
                include_str!("locales/tool.zh-CN.json"),
            ).expect("Failed to load i18n resources for ac_automaton tool"),
            automaton: Arc::new(AcAutomatonImpl::new()),
        }
    }
}

#[async_trait]
impl Tool for AcAutomatonTool {
    /// 获取工具名称
    fn name(&self) -> &str {
        "text.ac_automaton"
    }
    
    /// 获取工具显示名称
    fn display_name(&self, locale: Locale) -> String {
        self.i18n.display_name(locale).to_string()
    }
    
    /// 获取工具描述
    fn description(&self, locale: Locale) -> String {
        self.i18n.description(locale).to_string()
    }
    
    /// 获取输入参数Schema
    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(AcAutomatonInput)).unwrap();
        
        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.input_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
            
            // 添加enum约束 for action field
            if let Some(action_schema) = props.get_mut("action") {
                let actions = vec!["add", "remove", "list", "match", "save", "load"];
                action_schema["enum"] = json!(actions);

                let mut labels = serde_json::Map::new();
                for action in &actions {
                    if let Some(label) = self.i18n.extra(action, locale) {
                         labels.insert(action.to_string(), json!(label));
                    }
                }
                action_schema["x-enum-labels"] = Value::Object(labels);
            }
        }
        
        schema
    }
    
    /// 获取输出结果Schema
    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(AcAutomatonOutput)).unwrap();

        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.output_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
        }
        
        schema
    }
    
    /// 获取用户指南
    fn user_guide(&self, locale: Locale) -> String {
        self.i18n.user_guide(locale).to_string()
    }
    
    /// 执行工具
    async fn run(&self, input: Value) -> Result<Value> {
        let start_time = std::time::Instant::now();
        let args: AcAutomatonInput = serde_json::from_value(input)
            .map_err(|e| CoreError::InvalidInput(format!("解析输入参数失败: {}", e)))?;
        
        let mut success = true;
        let mut message = "操作成功".to_string();
        let mut results = None;
        let mut patterns = None;
        
        // 处理操作
        match args.action.as_str() {
            "add" => {
                if let Err(e) = self.automaton.add_patterns(&args.patterns) {
                    success = false;
                    message = e.to_string();
                }
            }
            "remove" => {
                let confirm = args.confirm.unwrap_or(false);
                if let Err(e) = self.automaton.remove_patterns(&args.patterns, confirm) {
                    success = false;
                    message = e.to_string();
                }
            }
            "list" => {
                let pat = self.automaton.patterns.read().unwrap().clone();
                patterns = Some(pat);
            }
            "match" => {
                if let Some(texts) = args.texts {
                    let parallel = args.parallel.unwrap_or(false);
                    
                    if parallel {
                        // 并行匹配
                        let automaton = self.automaton.clone();
                        let tasks: Vec<_> = texts.into_iter().map(|text| {
                            let automaton = automaton.clone();
                            tokio::spawn(async move {
                                automaton.match_text(&text)
                            })
                        }).collect();
                        
                        let match_results = try_join_all(tasks).await
                            .map_err(|e| CoreError::ToolFailure(format!("并行匹配执行失败: {}", e)))?;
                        results = Some(match_results.into_iter().flatten().collect());
                    } else {
                        // 串行匹配
                        let mut match_results = Vec::new();
                        for text in texts {
                            match_results.extend(self.automaton.match_text(&text));
                        }
                        results = Some(match_results);
                    }
                }
            }
            "save" => {
                // 暂不实现持久化，后续扩展
                message = "保存操作尚未实现".to_string();
            }
            "load" => {
                // 暂不实现持久化，后续扩展
                message = "加载操作尚未实现".to_string();
            }
            _ => {
                success = false;
                message = format!("无效操作: {}", args.action);
            }
        }
        
        // 处理构建选项
        if let Some(ignore_case) = args.ignore_case {
            if let Err(e) = self.automaton.set_ignore_case(ignore_case) {
                success = false;
                message = e.to_string();
            }
        }
        
        let elapsed_ms = start_time.elapsed().as_millis() as u64;
        
        let output = AcAutomatonOutput {
            success,
            message,
            results,
            patterns,
            elapsed_ms,
        };
        
        Ok(serde_json::to_value(output)
            .map_err(|e| CoreError::ToolFailure(e.to_string()))?)
    }
}