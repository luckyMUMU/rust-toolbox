use async_trait::async_trait;
use rt_core::{Tool, CoreError, Result, Locale};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use schemars::JsonSchema;
use ferrous_opencc::{OpenCC, config::BuiltinConfig};
use crate::utils::ToolI18n;

#[derive(Debug, Deserialize, JsonSchema)]
struct ConvertChineseInput {
    text: String,
    mode: String, // s2t, t2s, s2tw, tw2s, s2hk, hk2s, s2twp, tw2sp
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ConvertChineseOutput {
    converted: String,
}

pub struct ConvertChinese {
    i18n: ToolI18n,
}

// 注册工具
crate::register_tool!(ConvertChinese);

impl ConvertChinese {
    pub fn new() -> Self {
        Self {
            i18n: ToolI18n::new(
                include_str!("locales/tool.en.json"),
                include_str!("locales/tool.zh-CN.json"),
            ).expect("Failed to load i18n resources for convert_chinese tool"),
        }
    }
}

#[async_trait]
impl Tool for ConvertChinese {
    fn name(&self) -> &str {
        "text.convert_chinese"
    }

    fn display_name(&self, locale: Locale) -> String {
        self.i18n.display_name(locale).to_string()
    }

    fn description(&self, locale: Locale) -> String {
        self.i18n.description(locale).to_string()
    }

    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(ConvertChineseInput)).unwrap();
        
        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.input_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
            
            // Add enum constraint for mode field
            if let Some(mode_schema) = props.get_mut("mode") {
                let modes = vec![
                    "s2t", "t2s", "s2tw", "tw2s", 
                    "s2hk", "hk2s", "s2twp", "tw2sp"
                ];
                mode_schema["enum"] = json!(modes);

                let mut labels = serde_json::Map::new();
                for mode in &modes {
                    if let Some(label) = self.i18n.extra(mode, locale) {
                         labels.insert(mode.to_string(), json!(label));
                    }
                }
                mode_schema["x-enum-labels"] = Value::Object(labels);
            }
        }
        
        schema
    }

    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = serde_json::to_value(schemars::schema_for!(ConvertChineseOutput)).unwrap();

        if let Some(props) = schema.get_mut("properties").and_then(|v| v.as_object_mut()) {
            for (key, val) in props.iter_mut() {
                if let Some(title) = self.i18n.output_title(key, locale) {
                    val["title"] = serde_json::json!(title);
                }
            }
        }
        schema
    }

    fn user_guide(&self, locale: Locale) -> String {
        self.i18n.user_guide(locale).to_string()
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let args: ConvertChineseInput = serde_json::from_value(input)
            .map_err(|e| CoreError::InvalidInput(format!("Failed to parse input: {}", e)))?;

        let config = match args.mode.as_str() {
            "s2t" => BuiltinConfig::S2t,
            "t2s" => BuiltinConfig::T2s,
            "s2tw" => BuiltinConfig::S2tw,
            "tw2s" => BuiltinConfig::Tw2s,
            "s2hk" => BuiltinConfig::S2hk,
            "hk2s" => BuiltinConfig::Hk2s,
            "s2twp" => BuiltinConfig::S2twp,
            "tw2sp" => BuiltinConfig::Tw2sp,
            _ => return Err(CoreError::InvalidInput(format!("Invalid mode: {}", args.mode))),
        };

        let converter = OpenCC::from_config(config)
            .map_err(|e| CoreError::ToolFailure(format!("Failed to initialize converter: {}", e)))?;

        let converted = converter.convert(&args.text);

        let output = ConvertChineseOutput { converted };
        Ok(serde_json::to_value(output)
            .map_err(|e| CoreError::ToolFailure(e.to_string()))?)
    }
}

#[cfg(test)]
mod tests;
