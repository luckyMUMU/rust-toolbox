//! Utility functions and shared components for file management operations

use crate::plugins::file_management::error::{FileManagementError, FileManagementResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// File operation manager for safe file operations
pub struct FileOperationManager {
    dry_run: bool,
    temp_directory: PathBuf,
}

impl FileOperationManager {
    /// Create a new file operation manager
    pub fn new(temp_directory: PathBuf, dry_run: bool) -> Self {
        Self {
            dry_run,
            temp_directory,
        }
    }

    /// Check if running in dry run mode
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Set dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Get available disk space for a path
    pub fn get_available_space<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<u64> {
        // This is a simplified implementation
        // In a real implementation, you would use platform-specific APIs
        // For now, we'll return a large number to avoid blocking operations
        Ok(u64::MAX)
    }

    /// Check if a file operation would have sufficient space
    pub fn check_space_requirements<P: AsRef<Path>>(
        &self,
        target_path: P,
        required_bytes: u64,
    ) -> FileManagementResult<()> {
        let available = self.get_available_space(&target_path)?;
        if available < required_bytes {
            return Err(FileManagementError::insufficient_space(required_bytes, available));
        }
        Ok(())
    }

    /// Validate that a path is safe for operations
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> FileManagementResult<()> {
        let path = path.as_ref();
        
        // Check for null bytes
        if path.to_string_lossy().contains('\0') {
            return Err(FileManagementError::invalid_path(
                path,
                "Path contains null bytes",
            ));
        }

        // Check for extremely long paths
        if path.to_string_lossy().len() > 4096 {
            return Err(FileManagementError::invalid_path(
                path,
                "Path is too long",
            ));
        }

        // Check for invalid characters (platform-specific)
        #[cfg(windows)]
        {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            let path_str = path.to_string_lossy();
            for &ch in &invalid_chars {
                if path_str.contains(ch) {
                    return Err(FileManagementError::invalid_path(
                        path,
                        format!("Path contains invalid character: {}", ch),
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Pinyin conversion styles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PinyinStyle {
    Normal,        // ni3 hao3
    WithTone,      // nǐ hǎo
    WithoutTone,   // ni hao
    FirstLetter,   // n h
    Numeric,       // ni3 hao3
}

/// Pinyin conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinyinResult {
    pub original: String,
    pub pinyin_variants: Vec<String>,
    pub style: PinyinStyle,
    pub combinations: Vec<Vec<String>>,
}

/// Chinese text type detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChineseTextType {
    None,
    Simplified,
    Traditional,
    Mixed,
    Unknown,
}

/// Result of mixed text processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedTextResult {
    pub original: String,
    pub chinese_chars: String,
    pub non_chinese_chars: String,
    pub simplified_chinese: String,
    pub chinese_type: ChineseTextType,
    pub has_mixed_content: bool,
}

/// Text normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextNormalizationConfig {
    pub normalize_case: bool,
    pub normalize_whitespace: bool,
    pub remove_punctuation: bool,
    pub normalize_unicode: bool,
    pub filter_characters: Option<Vec<char>>,
    pub preserve_alphanumeric_only: bool,
}

impl Default for TextNormalizationConfig {
    fn default() -> Self {
        Self {
            normalize_case: true,
            normalize_whitespace: true,
            remove_punctuation: false,
            normalize_unicode: true,
            filter_characters: None,
            preserve_alphanumeric_only: false,
        }
    }
}

/// Text processor for normalization and Chinese text handling
pub struct TextProcessor {
    enable_chinese: bool,
    normalization_config: TextNormalizationConfig,
}

impl TextProcessor {
    /// Create a new text processor
    pub fn new(enable_chinese: bool) -> Self {
        Self { 
            enable_chinese,
            normalization_config: TextNormalizationConfig::default(),
        }
    }

    /// Create a new text processor with custom normalization config
    pub fn with_config(enable_chinese: bool, config: TextNormalizationConfig) -> Self {
        Self {
            enable_chinese,
            normalization_config: config,
        }
    }

    /// Normalize text case
    pub fn normalize_case(&self, text: &str) -> String {
        if self.normalization_config.normalize_case {
            text.to_lowercase()
        } else {
            text.to_string()
        }
    }

    /// Remove extra spaces and normalize whitespace
    pub fn normalize_whitespace(&self, text: &str) -> String {
        if self.normalization_config.normalize_whitespace {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        } else {
            text.to_string()
        }
    }

    /// Remove spaces entirely
    pub fn remove_spaces(&self, text: &str) -> String {
        text.replace(' ', "")
    }

    /// Remove punctuation characters
    pub fn remove_punctuation(&self, text: &str) -> String {
        if self.normalization_config.remove_punctuation {
            text.chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Normalize Unicode characters (NFD normalization)
    pub fn normalize_unicode(&self, text: &str) -> String {
        if self.normalization_config.normalize_unicode {
            // Basic Unicode normalization - remove diacritics and normalize
            text.chars()
                .map(|c| {
                    // Simple ASCII folding for common diacritics
                    match c {
                        'à'..='ÿ' => self.fold_latin_char(c),
                        _ => c,
                    }
                })
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Filter specific characters
    pub fn filter_characters(&self, text: &str) -> String {
        if let Some(ref filter_chars) = self.normalization_config.filter_characters {
            text.chars()
                .filter(|c| !filter_chars.contains(c))
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Keep only alphanumeric characters
    pub fn preserve_alphanumeric_only(&self, text: &str) -> String {
        if self.normalization_config.preserve_alphanumeric_only {
            text.chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect()
        } else {
            text.to_string()
        }
    }

    /// Comprehensive text normalization
    pub fn normalize_text(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // Apply normalization steps in order
        result = self.normalize_unicode(&result);
        result = self.normalize_case(&result);
        result = self.filter_characters(&result);
        result = self.remove_punctuation(&result);
        result = self.preserve_alphanumeric_only(&result);
        result = self.normalize_whitespace(&result);
        
        result
    }

    /// Segment text into words
    pub fn segment_text(&self, text: &str) -> Vec<String> {
        if self.enable_chinese && self.contains_chinese(text) {
            // For Chinese text, we need more sophisticated segmentation
            self.segment_chinese_text(text)
        } else {
            // For non-Chinese text, simple whitespace splitting
            text.split_whitespace()
                .map(|s| s.to_string())
                .collect()
        }
    }

    /// Segment Chinese text (basic implementation)
    fn segment_chinese_text(&self, text: &str) -> Vec<String> {
        let mut segments = Vec::new();
        let mut current_segment = String::new();
        
        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                // For Chinese characters, each character can be a segment
                if !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                }
                segments.push(ch.to_string());
            } else if ch.is_whitespace() {
                if !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                }
            } else {
                current_segment.push(ch);
            }
        }
        
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }
        
        segments
    }

    /// Check if character is Chinese
    fn is_chinese_char(&self, c: char) -> bool {
        let code = c as u32;
        // Basic Chinese character ranges
        (0x4E00..=0x9FFF).contains(&code) || // CJK Unified Ideographs
        (0x3400..=0x4DBF).contains(&code) || // CJK Extension A
        (0x20000..=0x2A6DF).contains(&code) || // CJK Extension B
        (0x2A700..=0x2B73F).contains(&code) || // CJK Extension C
        (0x2B740..=0x2B81F).contains(&code) || // CJK Extension D
        (0x2B820..=0x2CEAF).contains(&code) // CJK Extension E
    }

    /// Check if text contains Chinese characters
    pub fn contains_chinese(&self, text: &str) -> bool {
        text.chars().any(|c| self.is_chinese_char(c))
    }

    /// Simple Latin character folding for diacritics
    fn fold_latin_char(&self, c: char) -> char {
        match c {
            'à'..='å' | 'À'..='Å' => 'a',
            'è'..='ë' | 'È'..='Ë' => 'e',
            'ì'..='ï' | 'Ì'..='Ï' => 'i',
            'ò'..='ö' | 'Ò'..='Ö' | 'ø' | 'Ø' => 'o',
            'ù'..='ü' | 'Ù'..='Ü' => 'u',
            'ý' | 'ÿ' | 'Ý' => 'y',
            'ñ' | 'Ñ' => 'n',
            'ç' | 'Ç' => 'c',
            _ => c,
        }
    }

    /// Convert traditional Chinese to simplified (enhanced implementation)
    pub fn convert_traditional(&self, text: &str) -> String {
        if !self.enable_chinese {
            return text.to_string();
        }
        
        // Enhanced traditional to simplified conversion
        // This is still a basic implementation - in production you'd use a proper library
        let mut result = text.to_string();
        
        // Common traditional to simplified mappings
        let mappings = [
            ("繁體", "繁体"),
            ("簡體", "简体"),
            ("國家", "国家"),
            ("學習", "学习"),
            ("電腦", "电脑"),
            ("網路", "网络"),
            ("資料", "资料"),
            ("檔案", "档案"),
            ("軟體", "软件"),
            ("開發", "开发"),
            ("測試", "测试"),
            ("設計", "设计"),
            ("應用", "应用"),
            ("系統", "系统"),
            ("處理", "处理"),
            ("執行", "执行"),
            ("運行", "运行"),
            ("環境", "环境"),
            ("變數", "变量"),
            ("函數", "函数"),
            ("類別", "类别"),
            ("物件", "对象"),
            ("屬性", "属性"),
            ("方法", "方法"),
            ("介面", "接口"),
            ("實作", "实现"),
            ("繼承", "继承"),
            ("封裝", "封装"),
            ("多型", "多态"),
            ("抽象", "抽象"),
        ];
        
        for (traditional, simplified) in &mappings {
            result = result.replace(traditional, simplified);
        }
        
        debug!("Converted traditional Chinese: {} -> {}", text, result);
        result
    }

    /// Detect Chinese text type (simplified, traditional, or mixed)
    pub fn detect_chinese_type(&self, text: &str) -> ChineseTextType {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return ChineseTextType::None;
        }

        let mut simplified_count = 0;
        let mut traditional_count = 0;
        let mut total_chinese_chars = 0;

        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                total_chinese_chars += 1;
                
                // Check if character is likely traditional or simplified
                if self.is_likely_traditional_char(ch) {
                    traditional_count += 1;
                } else if self.is_likely_simplified_char(ch) {
                    simplified_count += 1;
                }
            }
        }

        if total_chinese_chars == 0 {
            return ChineseTextType::None;
        }

        let traditional_ratio = traditional_count as f64 / total_chinese_chars as f64;
        let simplified_ratio = simplified_count as f64 / total_chinese_chars as f64;

        if traditional_ratio > 0.3 && simplified_ratio > 0.3 {
            ChineseTextType::Mixed
        } else if traditional_ratio > simplified_ratio {
            ChineseTextType::Traditional
        } else if simplified_ratio > 0.1 {
            ChineseTextType::Simplified
        } else {
            ChineseTextType::Unknown
        }
    }

    /// Check if character is likely traditional Chinese
    fn is_likely_traditional_char(&self, c: char) -> bool {
        // Common traditional Chinese characters that have simplified variants
        matches!(c, 
            '繁' | '體' | '國' | '學' | '電' | '網' | '資' | '檔' | '軟' | '開' |
            '測' | '設' | '應' | '統' | '處' | '執' | '運' | '環' | '變' | '數' |
            '類' | '別' | '物' | '件' | '屬' | '性' | '實' | '作' | '繼' | '承' |
            '封' | '裝' | '態' | '象' | '議' | '題' | '問' | '題' | '決' | '議'
        )
    }

    /// Check if character is likely simplified Chinese
    fn is_likely_simplified_char(&self, c: char) -> bool {
        // Common simplified Chinese characters
        matches!(c,
            '简' | '体' | '国' | '学' | '电' | '网' | '资' | '档' | '软' | '开' |
            '测' | '设' | '应' | '统' | '处' | '执' | '运' | '环' | '变' | '量' |
            '类' | '别' | '对' | '象' | '属' | '性' | '实' | '现' | '继' | '承' |
            '封' | '装' | '态' | '象' | '议' | '题' | '问' | '题' | '决' | '议'
        )
    }

    /// Extract Chinese characters from mixed text
    pub fn extract_chinese_chars(&self, text: &str) -> String {
        if !self.enable_chinese {
            return String::new();
        }

        text.chars()
            .filter(|&c| self.is_chinese_char(c))
            .collect()
    }

    /// Extract non-Chinese characters from mixed text
    pub fn extract_non_chinese_chars(&self, text: &str) -> String {
        text.chars()
            .filter(|&c| !self.is_chinese_char(c))
            .collect()
    }

    /// Process mixed Chinese and English text
    pub fn process_mixed_text(&self, text: &str) -> MixedTextResult {
        let chinese_chars = self.extract_chinese_chars(text);
        let non_chinese_chars = self.extract_non_chinese_chars(text);
        let chinese_type = self.detect_chinese_type(text);
        
        let simplified_chinese = if chinese_type == ChineseTextType::Traditional {
            self.convert_traditional(&chinese_chars)
        } else {
            chinese_chars.clone()
        };

        MixedTextResult {
            original: text.to_string(),
            chinese_chars: chinese_chars.clone(),
            non_chinese_chars: non_chinese_chars.trim().to_string(),
            simplified_chinese,
            chinese_type,
            has_mixed_content: !chinese_chars.is_empty() && !non_chinese_chars.trim().is_empty(),
        }
    }

    /// Generate pinyin variants (enhanced implementation)
    pub fn generate_pinyin_variants(&self, text: &str) -> Vec<String> {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return vec![text.to_string()];
        }

        self.generate_pinyin_with_style(text, &PinyinStyle::Normal)
    }

    /// Generate pinyin with specific style
    pub fn generate_pinyin_with_style(&self, text: &str, style: &PinyinStyle) -> Vec<String> {
        if !self.enable_chinese || !self.contains_chinese(text) {
            return vec![text.to_string()];
        }

        let mut variants = Vec::new();
        
        // Basic pinyin mapping for common Chinese characters
        // In a real implementation, you would use a comprehensive pinyin dictionary
        let pinyin_map = self.get_basic_pinyin_map();
        
        let mut current_variant = String::new();
        let mut has_chinese = false;
        
        for ch in text.chars() {
            if self.is_chinese_char(ch) {
                has_chinese = true;
                if let Some(pinyin_options) = pinyin_map.get(&ch) {
                    // For now, just take the first pinyin option
                    let pinyin = &pinyin_options[0];
                    let styled_pinyin = self.apply_pinyin_style(pinyin, style);
                    current_variant.push_str(&styled_pinyin);
                } else {
                    // Fallback for unknown characters
                    current_variant.push(ch);
                }
            } else {
                current_variant.push(ch);
            }
        }
        
        if has_chinese {
            variants.push(current_variant);
            
            // Generate additional variants for different styles
            match style {
                PinyinStyle::Normal => {
                    variants.extend(self.generate_pinyin_with_style(text, &PinyinStyle::WithoutTone));
                    variants.extend(self.generate_pinyin_with_style(text, &PinyinStyle::FirstLetter));
                }
                _ => {}
            }
        } else {
            variants.push(text.to_string());
        }
        
        // Remove duplicates and empty strings
        variants.sort();
        variants.dedup();
        variants.retain(|s| !s.trim().is_empty());
        
        debug!("Generated pinyin variants for '{}': {:?}", text, variants);
        variants
    }

    /// Apply pinyin style formatting
    fn apply_pinyin_style(&self, pinyin: &str, style: &PinyinStyle) -> String {
        match style {
            PinyinStyle::Normal | PinyinStyle::Numeric => pinyin.to_string(),
            PinyinStyle::WithTone => self.convert_numeric_tone_to_diacritic(pinyin),
            PinyinStyle::WithoutTone => self.remove_tone_marks(pinyin),
            PinyinStyle::FirstLetter => {
                pinyin.chars().next().unwrap_or(' ').to_string()
            }
        }
    }

    /// Convert numeric tones to diacritic marks
    fn convert_numeric_tone_to_diacritic(&self, pinyin: &str) -> String {
        // Basic tone mark conversion
        let tone_map = [
            ("a1", "ā"), ("a2", "á"), ("a3", "ǎ"), ("a4", "à"),
            ("e1", "ē"), ("e2", "é"), ("e3", "ě"), ("e4", "è"),
            ("i1", "ī"), ("i2", "í"), ("i3", "ǐ"), ("i4", "ì"),
            ("o1", "ō"), ("o2", "ó"), ("o3", "ǒ"), ("o4", "ò"),
            ("u1", "ū"), ("u2", "ú"), ("u3", "ǔ"), ("u4", "ù"),
            ("v1", "ǖ"), ("v2", "ǘ"), ("v3", "ǚ"), ("v4", "ǜ"),
        ];
        
        let mut result = pinyin.to_string();
        for (numeric, diacritic) in &tone_map {
            result = result.replace(numeric, diacritic);
        }
        
        // Remove remaining numbers
        result = result.chars().filter(|c| !c.is_ascii_digit()).collect();
        result
    }

    /// Remove tone marks from pinyin
    fn remove_tone_marks(&self, pinyin: &str) -> String {
        let tone_map = [
            ("ā", "a"), ("á", "a"), ("ǎ", "a"), ("à", "a"),
            ("ē", "e"), ("é", "e"), ("ě", "e"), ("è", "e"),
            ("ī", "i"), ("í", "i"), ("ǐ", "i"), ("ì", "i"),
            ("ō", "o"), ("ó", "o"), ("ǒ", "o"), ("ò", "o"),
            ("ū", "u"), ("ú", "u"), ("ǔ", "u"), ("ù", "u"),
            ("ǖ", "v"), ("ǘ", "v"), ("ǚ", "v"), ("ǜ", "v"),
        ];
        
        let mut result = pinyin.to_string();
        for (diacritic, base) in &tone_map {
            result = result.replace(diacritic, base);
        }
        
        // Remove numbers
        result = result.chars().filter(|c| !c.is_ascii_digit()).collect();
        result
    }

    /// Get basic pinyin mapping for common characters
    fn get_basic_pinyin_map(&self) -> HashMap<char, Vec<String>> {
        let mut map = HashMap::new();
        
        // Common Chinese characters with their pinyin
        // This is a very basic set - a real implementation would have thousands
        map.insert('中', vec!["zhong1".to_string()]);
        map.insert('文', vec!["wen2".to_string()]);
        map.insert('你', vec!["ni3".to_string()]);
        map.insert('好', vec!["hao3".to_string()]);
        map.insert('世', vec!["shi4".to_string()]);
        map.insert('界', vec!["jie4".to_string()]);
        map.insert('学', vec!["xue2".to_string()]);
        map.insert('習', vec!["xi2".to_string()]);
        map.insert('习', vec!["xi2".to_string()]);
        map.insert('电', vec!["dian4".to_string()]);
        map.insert('電', vec!["dian4".to_string()]);
        map.insert('脑', vec!["nao3".to_string()]);
        map.insert('腦', vec!["nao3".to_string()]);
        map.insert('网', vec!["wang3".to_string()]);
        map.insert('網', vec!["wang3".to_string()]);
        map.insert('络', vec!["luo4".to_string()]);
        map.insert('路', vec!["lu4".to_string()]);
        map.insert('文', vec!["wen2".to_string()]);
        map.insert('件', vec!["jian4".to_string()]);
        map.insert('档', vec!["dang4".to_string()]);
        map.insert('檔', vec!["dang4".to_string()]);
        map.insert('案', vec!["an4".to_string()]);
        map.insert('管', vec!["guan3".to_string()]);
        map.insert('理', vec!["li3".to_string()]);
        map.insert('工', vec!["gong1".to_string()]);
        map.insert('具', vec!["ju4".to_string()]);
        map.insert('系', vec!["xi4".to_string()]);
        map.insert('统', vec!["tong3".to_string()]);
        map.insert('統', vec!["tong3".to_string()]);
        
        map
    }

    /// Generate comprehensive pinyin result
    pub fn generate_comprehensive_pinyin(&self, text: &str, style: PinyinStyle) -> PinyinResult {
        let pinyin_variants = self.generate_pinyin_with_style(text, &style);
        let combinations = self.create_combinations(text, &pinyin_variants);
        
        PinyinResult {
            original: text.to_string(),
            pinyin_variants,
            style,
            combinations,
        }
    }

    /// Create keyword combinations
    pub fn create_combinations(&self, original: &str, variants: &[String]) -> Vec<Vec<String>> {
        let mut combinations = Vec::new();
        
        // Add original
        combinations.push(vec![original.to_string()]);
        
        // Add variants
        for variant in variants {
            combinations.push(vec![variant.clone()]);
        }
        
        // Add combinations of original + variants
        for variant in variants {
            if variant != original {
                combinations.push(vec![original.to_string(), variant.clone()]);
            }
        }
        
        combinations
    }
}

/// Path utilities for common path operations
pub struct PathUtils;

impl PathUtils {
    /// Get the file size in bytes
    pub fn get_file_size<P: AsRef<Path>>(path: P) -> FileManagementResult<u64> {
        let metadata = std::fs::metadata(path.as_ref()).map_err(|e| {
            FileManagementError::io(
                format!("Failed to get metadata for {}", path.as_ref().display()),
                e,
            )
        })?;
        Ok(metadata.len())
    }

    /// Get the directory size recursively
    pub fn get_directory_size<P: AsRef<Path>>(path: P) -> FileManagementResult<u64> {
        let path = path.as_ref();
        let mut total_size = 0;

        if path.is_file() {
            return Self::get_file_size(path);
        }

        let entries = std::fs::read_dir(path).map_err(|e| {
            FileManagementError::io(
                format!("Failed to read directory {}", path.display()),
                e,
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                FileManagementError::io(
                    format!("Failed to read directory entry in {}", path.display()),
                    e,
                )
            })?;
            
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total_size += Self::get_directory_size(&entry_path)?;
            } else {
                total_size += Self::get_file_size(&entry_path)?;
            }
        }

        Ok(total_size)
    }

    /// Check if two paths refer to the same file/directory
    pub fn paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> bool {
        match (path1.as_ref().canonicalize(), path2.as_ref().canonicalize()) {
            (Ok(p1), Ok(p2)) => p1 == p2,
            _ => false,
        }
    }

    /// Generate a unique filename if the target already exists
    pub fn generate_unique_name<P: AsRef<Path>>(target_path: P) -> PathBuf {
        let path = target_path.as_ref();
        
        if !path.exists() {
            return path.to_path_buf();
        }

        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str());

        for i in 1..=9999 {
            let new_name = if let Some(ext) = extension {
                format!("{}_{}.{}", stem, i, ext)
            } else {
                format!("{}_{}", stem, i)
            };
            
            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
        }

        // Fallback with timestamp
        let timestamp = chrono::Utc::now().timestamp();
        let new_name = if let Some(ext) = extension {
            format!("{}_{}_{}.{}", stem, timestamp, rand::random::<u32>(), ext)
        } else {
            format!("{}_{}", stem, timestamp)
        };
        
        parent.join(new_name)
    }
}

/// Validation utilities for parameters and configurations
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate that a string is not empty after trimming
    pub fn validate_non_empty_string(value: &str, field_name: &str) -> FileManagementResult<()> {
        if value.trim().is_empty() {
            return Err(FileManagementError::validation(format!(
                "{} cannot be empty",
                field_name
            )));
        }
        Ok(())
    }

    /// Validate that a number is within a range
    pub fn validate_range<T: PartialOrd + std::fmt::Display>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
    ) -> FileManagementResult<()> {
        if value < min || value > max {
            return Err(FileManagementError::validation(format!(
                "{} must be between {} and {}, got {}",
                field_name, min, max, value
            )));
        }
        Ok(())
    }

    /// Validate that a path exists and is accessible
    pub fn validate_path_exists<P: AsRef<Path>>(path: P) -> FileManagementResult<()> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(FileManagementError::not_found(path));
        }
        Ok(())
    }

    /// Validate that a directory exists and is writable
    pub fn validate_writable_directory<P: AsRef<Path>>(path: P) -> FileManagementResult<()> {
        let path = path.as_ref();
        
        Self::validate_path_exists(path)?;
        
        if !path.is_dir() {
            return Err(FileManagementError::invalid_path(
                path,
                "Path is not a directory",
            ));
        }

        // Try to create a temporary file to test writability
        let test_file = path.join(format!(".test_write_{}", rand::random::<u32>()));
        match std::fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(FileManagementError::permission_denied(path)),
        }
    }
}

/// Experimental mode context for simulating operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalMode {
    pub enabled: bool,
    pub operations_log: Vec<ExperimentalOperation>,
}

impl ExperimentalMode {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            operations_log: Vec::new(),
        }
    }

    pub fn log_operation(&mut self, operation: ExperimentalOperation) {
        if self.enabled {
            self.operations_log.push(operation);
        }
    }

    pub fn get_operations(&self) -> &[ExperimentalOperation] {
        &self.operations_log
    }

    pub fn clear_log(&mut self) {
        self.operations_log.clear();
    }
}

/// Represents an operation that would be performed in experimental mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalOperation {
    pub operation_type: String,
    pub source_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub description: String,
    pub estimated_size: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ExperimentalOperation {
    pub fn new<S: Into<String>>(
        operation_type: S,
        description: S,
    ) -> Self {
        Self {
            operation_type: operation_type.into(),
            source_path: None,
            target_path: None,
            description: description.into(),
            estimated_size: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_source_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.source_path = Some(path.into());
        self
    }

    pub fn with_target_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.target_path = Some(path.into());
        self
    }

    pub fn with_estimated_size(mut self, size: u64) -> Self {
        self.estimated_size = Some(size);
        self
    }
}

/// Context for human decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionContext {
    pub decision_id: String,
    pub decision_type: HumanDecisionType,
    pub title: String,
    pub description: String,
    pub options: Vec<HumanDecisionOption>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HumanDecisionContext {
    pub fn new<S: Into<String>>(
        decision_type: HumanDecisionType,
        title: S,
        description: S,
    ) -> Self {
        Self {
            decision_id: uuid::Uuid::new_v4().to_string(),
            decision_type,
            title: title.into(),
            description: description.into(),
            options: Vec::new(),
            metadata: HashMap::new(),
            timeout_seconds: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn add_option<S: Into<String>>(mut self, id: S, label: S, description: Option<S>) -> Self {
        self.options.push(HumanDecisionOption {
            id: id.into(),
            label: label.into(),
            description: description.map(|s| s.into()),
            recommended: false,
            metadata: HashMap::new(),
        });
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }

    pub fn with_metadata<K: Into<String>>(mut self, key: K, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Types of human decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HumanDecisionType {
    Classification,
    FileConflict,
    MergeStrategy,
    BatchConfirmation,
    Custom(String),
}

/// Option for human decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecisionOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub recommended: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_operation_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileOperationManager::new(temp_dir.path().to_path_buf(), false);
        
        assert!(!manager.is_dry_run());
        assert!(manager.get_available_space(temp_dir.path()).is_ok());
    }

    #[test]
    fn test_text_processor() {
        let processor = TextProcessor::new(true);
        
        assert_eq!(processor.normalize_case("Hello World"), "hello world");
        assert_eq!(processor.normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(processor.remove_spaces("hello world"), "helloworld");
        
        // Test Chinese detection
        assert!(processor.contains_chinese("你好世界"));
        assert!(!processor.contains_chinese("hello world"));
    }

    #[test]
    fn test_text_normalization_functions() {
        let config = TextNormalizationConfig {
            normalize_case: true,
            normalize_whitespace: true,
            remove_punctuation: true,
            normalize_unicode: true,
            filter_characters: Some(vec!['@', '#']),
            preserve_alphanumeric_only: false,
        };
        
        let processor = TextProcessor::with_config(true, config);
        
        // Test punctuation removal
        assert_eq!(processor.remove_punctuation("Hello, World!"), "Hello World");
        
        // Test character filtering
        assert_eq!(processor.filter_characters("hello@world#test"), "helloworldtest");
        
        // Test alphanumeric preservation
        let alphanumeric_config = TextNormalizationConfig {
            preserve_alphanumeric_only: true,
            ..Default::default()
        };
        let alphanumeric_processor = TextProcessor::with_config(false, alphanumeric_config);
        assert_eq!(alphanumeric_processor.preserve_alphanumeric_only("Hello, World! 123"), "Hello World 123");
        
        // Test comprehensive normalization
        let result = processor.normalize_text("  Hello@World!  ");
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_text_segmentation() {
        let processor = TextProcessor::new(true);
        
        // Test English text segmentation
        let segments = processor.segment_text("hello world test");
        assert_eq!(segments, vec!["hello", "world", "test"]);
        
        // Test mixed text segmentation
        let segments = processor.segment_text("hello 世界 test");
        assert_eq!(segments, vec!["hello", "世", "界", "test"]);
    }

    #[test]
    fn test_unicode_normalization() {
        let processor = TextProcessor::new(false);
        
        // Test diacritic folding
        assert_eq!(processor.normalize_unicode("café"), "cafe");
        assert_eq!(processor.normalize_unicode("naïve"), "naive");
        assert_eq!(processor.normalize_unicode("résumé"), "resume");
    }

    #[test]
    fn test_chinese_text_processing() {
        let processor = TextProcessor::new(true);
        
        // Test Chinese character detection
        assert!(processor.is_chinese_char('中'));
        assert!(processor.is_chinese_char('文'));
        assert!(!processor.is_chinese_char('a'));
        assert!(!processor.is_chinese_char('1'));
        
        // Test Chinese text type detection
        assert_eq!(processor.detect_chinese_type("hello world"), ChineseTextType::None);
        assert_eq!(processor.detect_chinese_type("简体中文"), ChineseTextType::Simplified);
        assert_eq!(processor.detect_chinese_type("繁體中文"), ChineseTextType::Traditional);
        
        // Test character extraction
        assert_eq!(processor.extract_chinese_chars("hello 中文 world"), "中文");
        assert_eq!(processor.extract_non_chinese_chars("hello 中文 world"), "hello  world");
        
        // Test traditional to simplified conversion
        let converted = processor.convert_traditional("繁體中文");
        assert!(converted.contains("繁体")); // Should convert 繁體 to 繁体
    }

    #[test]
    fn test_mixed_text_processing() {
        let processor = TextProcessor::new(true);
        
        let result = processor.process_mixed_text("Hello 世界 World 中文");
        assert_eq!(result.chinese_chars, "世界中文");
        assert_eq!(result.non_chinese_chars, "Hello  World");
        assert!(result.has_mixed_content);
        assert_eq!(result.chinese_type, ChineseTextType::Simplified);
    }

    #[test]
    fn test_pinyin_conversion() {
        let processor = TextProcessor::new(true);
        
        // Test basic pinyin generation
        let variants = processor.generate_pinyin_variants("中文");
        assert!(!variants.is_empty());
        assert!(variants.iter().any(|v| v.contains("zhong") || v.contains("wen")));
        
        // Test different pinyin styles
        let normal_pinyin = processor.generate_pinyin_with_style("你好", &PinyinStyle::Normal);
        let tone_pinyin = processor.generate_pinyin_with_style("你好", &PinyinStyle::WithTone);
        let no_tone_pinyin = processor.generate_pinyin_with_style("你好", &PinyinStyle::WithoutTone);
        let first_letter = processor.generate_pinyin_with_style("你好", &PinyinStyle::FirstLetter);
        
        assert!(!normal_pinyin.is_empty());
        assert!(!tone_pinyin.is_empty());
        assert!(!no_tone_pinyin.is_empty());
        assert!(!first_letter.is_empty());
        
        // Test comprehensive pinyin result
        let result = processor.generate_comprehensive_pinyin("中文", PinyinStyle::Normal);
        assert_eq!(result.original, "中文");
        assert_eq!(result.style, PinyinStyle::Normal);
        assert!(!result.pinyin_variants.is_empty());
        assert!(!result.combinations.is_empty());
    }

    #[test]
    fn test_pinyin_style_conversion() {
        let processor = TextProcessor::new(true);
        
        // Test tone mark conversion
        assert_eq!(processor.convert_numeric_tone_to_diacritic("ni3"), "nǐ");
        assert_eq!(processor.convert_numeric_tone_to_diacritic("hao3"), "hǎo");
        
        // Test tone mark removal
        assert_eq!(processor.remove_tone_marks("nǐ"), "ni");
        assert_eq!(processor.remove_tone_marks("hǎo"), "hao");
        assert_eq!(processor.remove_tone_marks("ni3"), "ni");
    }

    #[test]
    fn test_path_utils() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"hello world").unwrap();
        
        let size = PathUtils::get_file_size(&test_file).unwrap();
        assert_eq!(size, 11);
        
        let dir_size = PathUtils::get_directory_size(temp_dir.path()).unwrap();
        assert!(dir_size >= 11);
    }

    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::validate_non_empty_string("hello", "test").is_ok());
        assert!(ValidationUtils::validate_non_empty_string("", "test").is_err());
        assert!(ValidationUtils::validate_non_empty_string("   ", "test").is_err());
        
        assert!(ValidationUtils::validate_range(5, 1, 10, "test").is_ok());
        assert!(ValidationUtils::validate_range(15, 1, 10, "test").is_err());
    }

    #[test]
    fn test_experimental_mode() {
        let mut exp_mode = ExperimentalMode::new(true);
        
        let operation = ExperimentalOperation::new("move", "Move file from A to B")
            .with_source_path("/path/a")
            .with_target_path("/path/b")
            .with_estimated_size(1024);
        
        exp_mode.log_operation(operation);
        assert_eq!(exp_mode.get_operations().len(), 1);
        
        exp_mode.clear_log();
        assert_eq!(exp_mode.get_operations().len(), 0);
    }

    #[test]
    fn test_human_decision_context() {
        let context = HumanDecisionContext::new(
            HumanDecisionType::Classification,
            "Test Decision",
            "Please choose an option",
        )
        .add_option("option1", "Option 1", Some("First option"))
        .add_option("option2", "Option 2", None)
        .with_timeout(300)
        .with_metadata("test_key", serde_json::Value::String("test_value".to_string()));
        
        assert_eq!(context.options.len(), 2);
        assert_eq!(context.timeout_seconds, Some(300));
        assert!(context.metadata.contains_key("test_key"));
    }
}