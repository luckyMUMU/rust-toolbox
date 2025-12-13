use clap::{Parser, Subcommand};
use pinyin::ToPinyin;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, Read};

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
    Spec,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Spec => {
            let spec = json!({
                "name": "text.pinyin",
                "display_name": {
                    "en": "Chinese to Pinyin",
                    "zh-CN": "汉字转拼音"
                },
                "description": {
                    "en": "Convert Chinese characters to Pinyin.",
                    "zh-CN": "将中文字符转换为拼音。"
                },
                "user_guide": {
                    "en": "Convert Chinese characters to Pinyin.\n\n### Input\n- `text`: Chinese text to convert.\n- `tone`: Include tone marks (default: true).",
                    "zh-CN": "将中文字符转换为拼音。\n\n### 输入\n- `text`: 要转换的中文文本。\n- `tone`: 是否包含声调（默认为 true）。"
                },
                "input_schema": schema_for!(Input),
                "output_schema": schema_for!(Output),
            });
            println!("{}", serde_json::to_string_pretty(&spec)?);
        }
        Commands::Run => {
            // Read input from stdin
            let mut input_str = String::new();
            io::stdin().read_to_string(&mut input_str)?;
            let input: Input = serde_json::from_str(&input_str)?;

            // Convert logic
            let mut result = String::new();
            for c in input.text.chars() {
                if let Some(p) = c.to_pinyin() {
                    // p is already Pinyin struct in newer version or Option<Pinyin>?
                    // Error says: expected `Pinyin`, found `Option<_>`
                    // wait, c.to_pinyin() returns Option<Pinyin> usually.
                    // Ah, the error says: if let Some(py) = p
                    // "expected `Pinyin`, found `Option<_>`" -> Wait.
                    // Let's check docs or infer.
                    // If c.to_pinyin() returns Option<Pinyin>, then `p` is Pinyin.
                    // So `if let Some(py) = p` is wrong because p is not Option.
                    // Let's fix it.
                    let py = p;
                    if input.tone {
                        result.push_str(py.with_tone());
                    } else {
                        result.push_str(py.plain());
                    }
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
            // Trim trailing space and collapse multiple spaces if any? 
            // The simple logic adds space after every pinyin char.
            // Let's just trim the end.
            let pinyin = result.trim().to_string();

            let output = Output { pinyin };
            println!("{}", serde_json::to_string(&output)?);
        }
    }

    Ok(())
}
