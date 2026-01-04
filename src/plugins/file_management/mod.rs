//! File Management Plugin Module
//! 
//! This module contains all components for the file management plugin including
//! tools, utilities, and error handling.

pub mod ac_automaton;
pub mod classification_tool;
pub mod error;
pub mod plugin;
pub mod registry;
pub mod rule_config;
pub mod text_processor_tool;
pub mod utils;

// Re-export main plugin components
pub use plugin::{
    FileManagementPlugin, FileManagementPluginBuilder, FileManagementConfig,
};
pub use error::{FileManagementError, FileManagementResult};
pub use registry::FileManagementToolRegistry;
pub use text_processor_tool::{
    TextProcessorTool, TextProcessorParams, TextProcessorResult,
    TextOperation, ChineseProcessingConfig, TextOutputFormat,
};
pub use classification_tool::{
    ClassificationTool, ClassificationEngine, ClassificationParams, ClassificationResult,
    ClassificationRules, ClassificationRule, ClassificationCandidate, ClassificationStatus,
    ClassificationOutputFormat,
};
// pub use rule_config::{
//     RuleConfigLoader, KeywordCombination, EnhancedClassificationRule,
// };
pub use utils::{
    FileOperationManager, TextProcessor, PathUtils, ValidationUtils,
    ExperimentalMode, HumanDecisionContext, TextNormalizationConfig,
    PinyinStyle, PinyinResult, ChineseTextType, MixedTextResult,
    FolderMerger, FolderMergerConfig, FolderComparisonResult, CommonFolderInfo,
    FolderLocationInfo, UniqueFolderInfo, MergeDirection, MergeRecommendation,
    MergeStrategy, DuplicateHandling, FolderMergeResult, SingleFolderMergeResult,
    FolderMergeError, MergeOperationStats,
};
pub use ac_automaton::{
    Pattern, AutomatonNode, PatternMatch, AutomatonConfig, AutomatonStats,
    AutomatonError, AutomatonResult,
};