//! Composable Tool Example
//!
//! This example demonstrates how to use the composable tool system
//! to build complex workflows by combining atomic tools.
//!
//! # Features Demonstrated
//!
//! - Tool chains (sequential execution)
//! - Conditional tools (branching logic)
//! - Parallel tools (concurrent execution)
//! - Custom atomic tool implementation
//! - Tool composition with EL expressions
//!
//! # Usage
//!
//! ```bash
//! cargo run --example composable_tool_example
//! ```

use std::sync::Arc;
use workflow_toolkit::core::{ExecutionContext, ToolInfo};
use workflow_toolkit::error::Result;
use workflow_toolkit::tools::composable::{
    ComposableTool, ToolChain, ConditionalTool, ParallelTools, ToolComposer,
};
use workflow_toolkit::tools::ToolNode;
use async_trait::async_trait;
use serde_json::{json, Value};
use chrono::Utc;
use std::collections::HashMap;

/// Example atomic tool: File scanner
struct FileScanner {
    extensions: Vec<String>,
}

impl FileScanner {
    fn new(extensions: Vec<String>) -> Self {
        Self { extensions }
    }
}

#[async_trait]
impl ToolNode for FileScanner {
    async fn execute(&self, _params: Value, _context: ExecutionContext) -> Result<Value> {
        println!("📁 Scanning for files with extensions: {:?}", self.extensions);
        
        // Simulate file scanning
        let files = vec![
            "document1.pdf",
            "image1.jpg",
            "data1.json",
            "report1.pdf",
        ];
        
        Ok(json!({
            "files": files,
            "count": files.len(),
            "extensions": self.extensions
        }))
    }
    
    fn name(&self) -> &str {
        "file-scanner"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        format!("Scan files with extensions: {:?}", self.extensions)
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {}
            }),
            return_schema: json!({}),
            category: Some("scanner".to_string()),
            tags: vec!["file".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

/// Example atomic tool: Content analyzer
struct ContentAnalyzer;

#[async_trait]
impl ToolNode for ContentAnalyzer {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let files = params.get("files")
            .and_then(|f| f.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        println!("🔍 Analyzing content of {} files", files);
        
        // Simulate content analysis
        let analysis = json!({
            "total_files": files,
            "pdf_count": 2,
            "image_count": 1,
            "json_count": 1,
            "average_size": "2.5MB"
        });
        
        Ok(analysis)
    }
    
    fn name(&self) -> &str {
        "content-analyzer"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        "Analyze file content and metadata".to_string()
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "files": { "type": "array" }
                }
            }),
            return_schema: json!({}),
            category: Some("analyzer".to_string()),
            tags: vec!["content".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

/// Example atomic tool: Confidence scorer
struct ConfidenceScorer {
    threshold: f64,
}

impl ConfidenceScorer {
    fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl ToolNode for ConfidenceScorer {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        let total_files = params.get("total_files")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        
        // Calculate confidence based on file types
        let confidence = if total_files > 0 {
            0.85 // Simulated confidence
        } else {
            0.0
        };
        
        println!("📊 Calculated confidence: {:.2} (threshold: {:.2})", 
                 confidence, self.threshold);
        
        Ok(json!({
            "confidence": confidence,
            "threshold": self.threshold,
            "is_high_confidence": confidence >= self.threshold
        }))
    }
    
    fn name(&self) -> &str {
        "confidence-scorer"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        format!("Score classification confidence (threshold: {})", self.threshold)
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {
                    "total_files": { "type": "integer" }
                }
            }),
            return_schema: json!({}),
            category: Some("scorer".to_string()),
            tags: vec!["confidence".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

/// Example atomic tool: High confidence processor
struct HighConfidenceProcessor;

#[async_trait]
impl ToolNode for HighConfidenceProcessor {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        println!("✅ Processing with high confidence strategy");
        
        Ok(json!({
            "strategy": "high_confidence",
            "processed": true,
            "auto_classified": params.get("total_files").unwrap_or(&json!(0))
        }))
    }
    
    fn name(&self) -> &str {
        "high-confidence-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        "Process files with high confidence auto-classification".to_string()
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {}
            }),
            return_schema: json!({}),
            category: Some("processor".to_string()),
            tags: vec!["high-confidence".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

/// Example atomic tool: Low confidence processor
struct LowConfidenceProcessor;

#[async_trait]
impl ToolNode for LowConfidenceProcessor {
    async fn execute(&self, params: Value, _context: ExecutionContext) -> Result<Value> {
        println!("⚠️ Processing with low confidence strategy (needs review)");
        
        Ok(json!({
            "strategy": "low_confidence",
            "processed": true,
            "needs_human_review": true,
            "pending_count": params.get("total_files").unwrap_or(&json!(0))
        }))
    }
    
    fn name(&self) -> &str {
        "low-confidence-processor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        "Process files with low confidence (requires human review)".to_string()
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {}
            }),
            return_schema: json!({}),
            category: Some("processor".to_string()),
            tags: vec!["low-confidence".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

/// Example atomic tool: File processor
struct FileProcessor {
    name: String,
}

impl FileProcessor {
    fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl ToolNode for FileProcessor {
    async fn execute(&self, _params: Value, _context: ExecutionContext) -> Result<Value> {
        println!("🔧 Processor '{}' executing...", self.name);
        
        // Simulate processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(json!({
            "processor": self.name,
            "status": "completed"
        }))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> String {
        format!("File processor: {}", self.name)
    }
    
    fn definition(&self) -> ToolInfo {
        ToolInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: self.description(),
            parameters_schema: json!({
                "type": "object",
                "properties": {}
            }),
            return_schema: json!({}),
            category: Some("processor".to_string()),
            tags: vec!["file".to_string()],
            dependencies: vec![],
            plugin_name: None,
            version_requirements: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    fn validate_parameters(&self, _params: &Value) -> Result<()> {
        Ok(())
    }
    
    fn get_info(&self) -> ToolInfo {
        self.definition()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Composable Tool System Example                         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Create execution context
    let context = ExecutionContext::default();

    // ============================================================
    // Example 1: Tool Chain (Sequential Execution)
    // ============================================================
    println!("\n📋 Example 1: Tool Chain (Sequential Execution)");
    println!("─────────────────────────────────────────────────────────────");
    
    let scanner = Arc::new(FileScanner::new(vec!["pdf".to_string(), "jpg".to_string()]));
    let analyzer = Arc::new(ContentAnalyzer);
    let scorer = Arc::new(ConfidenceScorer::new(0.8));
    
    let chain = ToolChain::new("scan-analyze-score", "Scan, analyze, and score files")
        .add_step("scan", scanner)
        .add_step("analyze", analyzer)
        .add_step("score", scorer);
    
    let result = chain.execute(json!({}), context.clone()).await?;
    println!("\nChain result:\n{}", serde_json::to_string_pretty(&result)?);

    // ============================================================
    // Example 2: Tool Composer with Registration
    // ============================================================
    println!("\n\n📋 Example 2: Tool Composer with Registration");
    println!("─────────────────────────────────────────────────────────────");
    
    let composer = ToolComposer::new()
        .register(
            "file_pipeline",
            Arc::new(
                ToolChain::new("file_pipeline", "Complete file processing pipeline")
                    .add_step("scan", Arc::new(FileScanner::new(vec!["pdf".to_string()])))
                    .add_step("analyze", Arc::new(ContentAnalyzer))
            )
        );
    
    println!("Registered tools: {:?}", composer.list_tools());
    
    // Execute the composed tool
    if let Some(tool) = composer.get_tool("file_pipeline") {
        let result = tool.execute(json!({}), context.clone()).await?;
        println!("\nPipeline result:\n{}", serde_json::to_string_pretty(&result)?);
    }

    // ============================================================
    // Example 3: Conditional Tool
    // ============================================================
    println!("\n\n📋 Example 3: Conditional Tool (Branching Logic)");
    println!("─────────────────────────────────────────────────────────────");
    
    let conditional = ConditionalTool::new(
        "smart-processor",
        "Process based on confidence score",
        "is_high_confidence",  // Note: without ${} prefix for simple variable lookup
        Arc::new(HighConfidenceProcessor),
    )
    .with_else_branch(Arc::new(LowConfidenceProcessor));
    
    // Test with high confidence
    println!("\n--- Testing with high confidence ---");
    let high_conf_result = conditional.execute(
        json!({ "is_high_confidence": true, "total_files": 10 }),
        context.clone()
    ).await?;
    println!("Result:\n{}", serde_json::to_string_pretty(&high_conf_result)?);
    
    // Test with low confidence
    println!("\n--- Testing with low confidence ---");
    let low_conf_result = conditional.execute(
        json!({ "is_high_confidence": false, "total_files": 5 }),
        context.clone()
    ).await?;
    println!("Result:\n{}", serde_json::to_string_pretty(&low_conf_result)?);

    // ============================================================
    // Example 4: Parallel Tools
    // ============================================================
    println!("\n\n📋 Example 4: Parallel Tools (Concurrent Execution)");
    println!("─────────────────────────────────────────────────────────────");
    
    let parallel = ParallelTools::new("parallel-processors", "Run multiple processors in parallel")
        .with_tool("pdf-processor", Arc::new(FileProcessor::new("pdf-processor")))
        .with_tool("image-processor", Arc::new(FileProcessor::new("image-processor")))
        .with_tool("text-processor", Arc::new(FileProcessor::new("text-processor")))
        .with_max_concurrency(3);
    
    let start = std::time::Instant::now();
    let parallel_result = parallel.execute(json!({}), context.clone()).await?;
    let elapsed = start.elapsed();
    
    println!("\nParallel execution completed in {:?}", elapsed);
    println!("Result:\n{}", serde_json::to_string_pretty(&parallel_result)?);

    // ============================================================
    // Example 5: Complex Composition
    // ============================================================
    println!("\n\n📋 Example 5: Complex Composition");
    println!("─────────────────────────────────────────────────────────────");
    
    let complex_composer = ToolComposer::new()
        // Chain for initial processing
        .register(
            "initial_scan",
            Arc::new(
                ToolChain::new("initial_scan", "Initial file scanning")
                    .add_step("scanner", Arc::new(FileScanner::new(vec!["pdf".to_string(), "json".to_string()])))
            )
        )
        // Conditional processing based on results
        .register(
            "smart_classifier",
            Arc::new(
                ConditionalTool::new(
                    "smart_classifier",
                    "Classify based on confidence",
                    "confidence > 0.7",  // Comparison expression
                    Arc::new(HighConfidenceProcessor),
                )
                .with_else_branch(Arc::new(LowConfidenceProcessor))
            )
        )
        // Parallel processing for different file types
        .register(
            "multi_processor",
            Arc::new(
                ParallelTools::new("multi_processor", "Process different file types in parallel")
                    .with_tool("processor_a", Arc::new(FileProcessor::new("pdf-extractor")))
                    .with_tool("processor_b", Arc::new(FileProcessor::new("metadata-analyzer")))
                    .with_tool("processor_c", Arc::new(FileProcessor::new("content-classifier")))
                    .with_max_concurrency(3)
            )
        );
    
    println!("\nComplex composition registered tools:");
    for (name, desc) in complex_composer.list_tools() {
        println!("  - {}: {}", name, desc);
    }

    // Execute complex composition
    println!("\n--- Executing initial_scan ---");
    if let Some(tool) = complex_composer.get_tool("initial_scan") {
        let result = tool.execute(json!({}), context.clone()).await?;
        println!("Result:\n{}", serde_json::to_string_pretty(&result)?);
    }
    
    println!("\n--- Executing smart_classifier (high confidence) ---");
    if let Some(tool) = complex_composer.get_tool("smart_classifier") {
        let result = tool.execute(json!({ "confidence": 0.85 }), context.clone()).await?;
        println!("Result:\n{}", serde_json::to_string_pretty(&result)?);
    }
    
    println!("\n--- Executing multi_processor ---");
    if let Some(tool) = complex_composer.get_tool("multi_processor") {
        let start = std::time::Instant::now();
        let result = tool.execute(json!({}), context.clone()).await?;
        let elapsed = start.elapsed();
        println!("Completed in {:?}", elapsed);
        println!("Result:\n{}", serde_json::to_string_pretty(&result)?);
    }

    // ============================================================
    // Summary
    // ============================================================
    println!("\n\n╔════════════════════════════════════════════════════════════╗");
    println!("║     Examples Completed Successfully!                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    println!("\n📚 Key Concepts Demonstrated:");
    println!("  1. ToolChain - Sequential execution of multiple tools");
    println!("  2. ConditionalTool - Branching logic based on conditions");
    println!("  3. ParallelTools - Concurrent execution with semaphore control");
    println!("  4. ToolComposer - Registration and management of composable tools");
    
    println!("\n🔧 Custom Tool Implementation:");
    println!("  - Implement ToolNode trait for atomic tools");
    println!("  - Use Arc<dyn ToolNode> for tool references");
    println!("  - Support JSON parameters and results");
    
    println!("\n📊 Performance Features:");
    println!("  - Configurable concurrency limits");
    println!("  - Async/await throughout");
    println!("  - Efficient resource utilization");

    Ok(())
}
