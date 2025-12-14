use rt_core::Locale;
use serde::Deserialize;
use std::collections::HashMap;

/// 工具的本地化数据结构
#[derive(Debug, Deserialize)]
pub struct ToolLocale {
    pub display_name: String,
    pub description: String,
    pub user_guide: String,
    #[serde(default)]
    pub input_schema: HashMap<String, SchemaField>,
    #[serde(default)]
    pub output_schema: HashMap<String, SchemaField>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct SchemaField {
    pub title: String,
}

/// 管理多语言数据的辅助结构体
pub struct ToolI18n {
    en: ToolLocale,
    zh: ToolLocale,
}

impl ToolI18n {
    /// 从 JSON 字符串加载多语言数据
    pub fn new(en_json: &str, zh_json: &str) -> Result<Self, serde_json::Error> {
        let en: ToolLocale = serde_json::from_str(en_json)?;
        let zh: ToolLocale = serde_json::from_str(zh_json)?;
        Ok(Self { en, zh })
    }

    pub fn get(&self, locale: Locale) -> &ToolLocale {
        match locale {
            Locale::En => &self.en,
            Locale::Zh => &self.zh,
        }
    }

    pub fn display_name(&self, locale: Locale) -> &str {
        &self.get(locale).display_name
    }

    pub fn description(&self, locale: Locale) -> &str {
        &self.get(locale).description
    }

    pub fn user_guide(&self, locale: Locale) -> &str {
        &self.get(locale).user_guide
    }

    pub fn input_title(&self, field: &str, locale: Locale) -> Option<&str> {
        self.get(locale).input_schema.get(field).map(|f| f.title.as_str())
    }

    pub fn output_title(&self, field: &str, locale: Locale) -> Option<&str> {
        self.get(locale).output_schema.get(field).map(|f| f.title.as_str())
    }

    pub fn extra(&self, key: &str, locale: Locale) -> Option<&str> {
        self.get(locale).extra.get(key).map(|s| s.as_str())
    }
}
