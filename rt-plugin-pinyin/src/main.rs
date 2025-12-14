use clap::{Parser, Subcommand};
use pinyin::ToPinyin;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, Read};

use rt_core::plugin::{PluginMetadata, LocalizedString};
use rt_core::tool::Tool;
use rt_core::locale::Locale;
use async_trait::async_trait;

mod i18n;

/// Chinese to Pinyin Plugin
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Output the plugin specification
    Spec {
        /// Locale for the spec (e.g., "en", "zh-CN")
        #[arg(long, default_value = "en")]
        locale: String,
    },
    /// Run the plugin logic
    Run,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Input {
    /// Chinese text to convert
    text: String,
    /// Include tone marks (default: true)
    #[serde(default = "default_tone")]
    tone: bool,
}

fn default_tone() -> bool {
    true
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Output {
    /// Converted Pinyin text
    pinyin: String,
}

/// Chinese to Pinyin Plugin
struct PinyinTool;

#[async_trait]
impl Tool for PinyinTool {
    /// Returns the unique name of the tool.
    fn name(&self) -> &str {
        "text.pinyin"
    }

    /// Returns the display name of the tool based on the provided locale.
    fn display_name(&self, locale: Locale) -> String {
        i18n::display_name(locale)
    }

    /// Returns the description of the tool based on the provided locale.
    fn description(&self, locale: Locale) -> String {
        i18n::description(locale)
    }

    /// Returns the user guide for the tool based on the provided locale.
    fn user_guide(&self, locale: Locale) -> String {
        i18n::user_guide(locale)
    }

    /// Returns the JSON schema for the tool's input based on the provided locale.
    fn input_schema(&self, _locale: Locale) -> Value {
        serde_json::to_value(schema_for!(Input)).unwrap_or_else(|_| json!({}))
    }

    /// Returns the JSON schema for the tool's output based on the provided locale.
    fn output_schema(&self, _locale: Locale) -> Value {
        serde_json::to_value(schema_for!(Output)).unwrap_or_else(|_| json!({}))
    }

    /// Runs the Chinese to Pinyin conversion logic.
    /// It takes a JSON string as input, parses it into an `Input` struct, performs the conversion,
    /// and returns the result as a JSON string representing the `Output` struct.
    async fn run(&self, input: Value) -> rt_core::error::Result<Value> {
        let input: Input = serde_json::from_value(input).map_err(anyhow::Error::from)?;

        let mut result = String::new();
        for c in input.text.chars() {
            if let Some(p) = c.to_pinyin() {
                let py: pinyin::Pinyin = p;
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                if input.tone {
                    result.push_str(py.with_tone());
                } else {
                    result.push_str(py.plain());
                }
            } else {
                if !result.is_empty() && result.ends_with(' ') && c.is_alphanumeric() {
                    result.pop(); // Remove trailing space if next char is alphanumeric
                }
                result.push(c);
                if c.is_alphanumeric() && !result.ends_with(' ') {
                    result.push(' ');
                }
            }
        }
        let pinyin = result.trim().to_string();

        let output = Output { pinyin };
        Ok(serde_json::to_value(&output).map_err(anyhow::Error::from)?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let tool = PinyinTool;

    match cli.command {
        Commands::Spec { locale } => {
            let parsed_locale = match locale.as_str() {
                "en" => Locale::En,
                "zh-CN" => Locale::Zh,
                _ => anyhow::bail!("Unsupported locale: {}", locale),
            };

            let metadata = PluginMetadata {
                name: tool.name().to_string(),
                display_name: LocalizedString {
                    en: tool.display_name(Locale::En),
                    zh: Some(tool.display_name(Locale::Zh)),
                },
                description: LocalizedString {
                    en: tool.description(Locale::En),
                    zh: Some(tool.description(Locale::Zh)),
                },
                user_guide: LocalizedString {
                    en: tool.user_guide(Locale::En),
                    zh: Some(tool.user_guide(Locale::Zh)),
                },
                input_schema: tool.input_schema(parsed_locale),
                output_schema: Some(tool.output_schema(parsed_locale)),
                input_fields: Some(i18n::get_input_field_map()),
                output_fields: Some(i18n::get_output_field_map()),
                author: None,
                version: None,
                ..Default::default()
            };
            println!("{}", serde_json::to_string_pretty(&metadata)?);
        }
        Commands::Run => {
            let mut input_str = String::new();
            io::stdin().read_to_string(&mut input_str)?;
            let input_value: Value = serde_json::from_str(&input_str)?;
            let output_value = tool.run(input_value).await?;
            println!("{}", serde_json::to_string(&output_value)?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_pinyin_conversion_with_tone() -> Result<()> {
        let tool = PinyinTool;
        let input_json = json!({
            "text": "你好世界",
            "tone": true
        });
        let output_value = tool.run(input_json).await?;
        let output: Output = serde_json::from_value(output_value)?;
        assert_eq!(output.pinyin, "nǐ hǎo shì jiè");
        Ok(())
    }

    #[tokio::test]
    async fn test_pinyin_conversion_without_tone() -> Result<()> {
        let tool = PinyinTool;
        let input_json = json!({
            "text": "你好世界",
            "tone": false
        });
        let output_value = tool.run(input_json).await?;
        let output: Output = serde_json::from_value(output_value)?;
        assert_eq!(output.pinyin, "ni hao shi jie");
        Ok(())
    }

    #[tokio::test]
    async fn test_pinyin_conversion_mixed_text() -> Result<()> {
        let tool = PinyinTool;
        let input_json = json!({
            "text": "Hello世界",
            "tone": true
        });
        let output_value = tool.run(input_json).await?;
        let output: Output = serde_json::from_value(output_value)?;
        assert_eq!(output.pinyin, "Hello shì jiè");
        Ok(())
    }

    #[tokio::test]
    async fn test_pinyin_conversion_empty_text() -> Result<()> {
        let tool = PinyinTool;
        let input_json = json!({
            "text": "",
            "tone": true
        });
        let output_value = tool.run(input_json).await?;
        let output: Output = serde_json::from_value(output_value)?;
        assert_eq!(output.pinyin, "");
        Ok(())
    }
}
