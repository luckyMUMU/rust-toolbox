use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::locale::Locale;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalizedString {
    pub en: String,
    #[serde(alias = "zh-CN")]
    pub zh: Option<String>,
}

impl LocalizedString {
    pub fn get(&self, locale: Locale) -> &str {
        match locale {
            Locale::En => &self.en,
            Locale::Zh => self.zh.as_deref().unwrap_or(&self.en),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginMetadata {
    pub name: String,
    pub display_name: LocalizedString,
    pub description: LocalizedString,
    pub user_guide: LocalizedString,
    pub input_schema: Value,
    #[serde(default)]
    pub output_schema: Option<Value>,
    #[serde(default)]
    pub input_fields: Option<HashMap<String, LocalizedString>>,
    #[serde(default)]
    pub output_fields: Option<HashMap<String, LocalizedString>>,
    
    // New fields for manifest
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}
