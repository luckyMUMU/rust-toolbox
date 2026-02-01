//! Classification module for file management plugin
//!
//! Contains tools for file and folder classification.

pub mod classification_flow;
pub mod classification_tool;
pub mod rule_config;

// Re-export classification components
pub use classification_flow::{
    AmbiguityDetectorTool, AutomatonBuilderTool, DirectoryScannerTool, ExperimentalCheckTool,
    FolderNamePreprocessorTool, ParallelMatcherTool, ReportGeneratorTool, ResultMergerTool,
    RuleLoaderTool, RulePreprocessorTool, ScoreCalculatorTool,
};
pub use classification_tool::{
    ClassificationCandidate, ClassificationEngine, ClassificationOutputFormat,
    ClassificationParams, ClassificationResult, ClassificationRule, ClassificationRules,
    ClassificationStatus, ClassificationTool,
};
