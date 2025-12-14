use clap::{Parser, Subcommand}; use schemars::schema_for; use serde_json::{json, Value}; use std::io::{self, Read}; use rt_core::plugin::{PluginMetadata, LocalizedString}; use rt_core::locale::Locale; use crate::tools::Tool;

mod i18n; mod tools;

/// Czkawka Plugin for Rust Toolbox
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

/// 工具元数据，包含工具的名称、显示名称、描述等信息
struct ToolMetadata {
    name: &'static str,
    display_name_en: &'static str,
    display_name_zh: &'static str,
    description_en: &'static str,
    description_zh: &'static str,
}

/// 所有支持的工具列表
const ALL_TOOLS: &[ToolMetadata] = &[
    ToolMetadata {
        name: "file.duplicates",
        display_name_en: "Duplicate Files",
        display_name_zh: "重复文件",
        description_en: "Find duplicate files in specified directories",
        description_zh: "在指定目录中查找重复文件",
    },
    ToolMetadata {
        name: "file.similar_images",
        display_name_en: "Similar Images",
        display_name_zh: "相似图片",
        description_en: "Find visually similar images",
        description_zh: "查找视觉相似的图片",
    },
    ToolMetadata {
        name: "file.empty_directories",
        display_name_en: "Empty Directories",
        display_name_zh: "空目录",
        description_en: "Find empty directories",
        description_zh: "查找空目录",
    },
    ToolMetadata {
        name: "file.temporary_files",
        display_name_en: "Temporary Files",
        display_name_zh: "临时文件",
        description_en: "Find temporary files",
        description_zh: "查找临时文件",
    },
    ToolMetadata {
        name: "file.broken_symlinks",
        display_name_en: "Broken Symbolic Links",
        display_name_zh: "损坏的符号链接",
        description_en: "Find broken symbolic links",
        description_zh: "查找损坏的符号链接",
    },
];

/// 获取工具的输入 Schema
///
/// # 参数
/// * `tool_name` - 工具名称
///
/// # 返回值
/// 返回工具的输入 Schema
fn get_input_schema(tool_name: &str) -> Value {
    match tool_name {
        "file.duplicates" => serde_json::to_value(schema_for!(tools::duplicates::Input)).unwrap_or_else(|_| json!({})),
        "file.similar_images" => serde_json::to_value(schema_for!(tools::similar_images::Input)).unwrap_or_else(|_| json!({})),
        "file.empty_directories" => serde_json::to_value(schema_for!(tools::empty_dirs::Input)).unwrap_or_else(|_| json!({})),
        "file.temporary_files" => serde_json::to_value(schema_for!(tools::temp_files::Input)).unwrap_or_else(|_| json!({})),
        "file.broken_symlinks" => serde_json::to_value(schema_for!(tools::broken_symlinks::Input)).unwrap_or_else(|_| json!({})),
        _ => json!({}),
    }
}

/// 获取工具的输出 Schema
///
/// # 参数
/// * `tool_name` - 工具名称
///
/// # 返回值
/// 返回工具的输出 Schema
fn get_output_schema(tool_name: &str) -> Value {
    match tool_name {
        "file.duplicates" => serde_json::to_value(schema_for!(tools::duplicates::Output)).unwrap_or_else(|_| json!({})),
        "file.similar_images" => serde_json::to_value(schema_for!(tools::similar_images::Output)).unwrap_or_else(|_| json!({})),
        "file.empty_directories" => serde_json::to_value(schema_for!(tools::empty_dirs::Output)).unwrap_or_else(|_| json!({})),
        "file.temporary_files" => serde_json::to_value(schema_for!(tools::temp_files::Output)).unwrap_or_else(|_| json!({})),
        "file.broken_symlinks" => serde_json::to_value(schema_for!(tools::broken_symlinks::Output)).unwrap_or_else(|_| json!({})),
        _ => json!({}),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Spec { locale } => {
            // 解析语言环境，但目前未使用
            let _ = match locale.as_str() {
                "en" => Locale::En,
                "zh" | "zh-CN" => Locale::Zh,
                _ => anyhow::bail!("Unsupported locale: {}", locale),
            };

            // 收集所有工具的元数据，输出为JSON数组
            let mut all_metadata = Vec::new();
            for tool in ALL_TOOLS {
                let metadata = PluginMetadata {
                    name: tool.name.to_string(),
                    display_name: LocalizedString {
                        en: tool.display_name_en.to_string(),
                        zh: Some(tool.display_name_zh.to_string()),
                    },
                    description: LocalizedString {
                        en: tool.description_en.to_string(),
                        zh: Some(tool.description_zh.to_string()),
                    },
                    user_guide: LocalizedString {
                        en: i18n::get_tool_user_guide(tool.name, Locale::En),
                        zh: Some(i18n::get_tool_user_guide(tool.name, Locale::Zh)),
                    },
                    input_schema: get_input_schema(tool.name),
                    output_schema: Some(get_output_schema(tool.name)),
                    input_fields: Some(i18n::get_input_field_map()),
                    output_fields: Some(i18n::get_output_field_map()),
                    author: None,
                    version: None,
                    // 添加缺少的字段
                    mcp_supported: false,
                    mcp_capabilities: json!({}),
                    requires_full_context: false,
                    context_validation_rules: json!({}),
                };
                all_metadata.push(metadata);
            }
            
            // 输出JSON数组，支持多工具插件
            println!("{}", serde_json::to_string_pretty(&all_metadata)?);
        }
        Commands::Run => {
            // 从标准输入读取输入数据
            let mut input_str = String::new();
            io::stdin().read_to_string(&mut input_str)?;
            let input_value: Value = serde_json::from_str(&input_str)?;

            // 检查输入数据是否包含工具名称
            let tool_name = if let Some(tool_name) = input_value.get("tool_name").and_then(|v| v.as_str()) {
                tool_name
            } else {
                // 如果输入数据中没有工具名称，则尝试从字段判断
                // 简单实现：检查是否有 threshold 字段（相似图片工具特有）
                if input_value.get("threshold").is_some() {
                    "file.similar_images"
                } else if input_value.get("min_size").is_some() {
                    "file.duplicates"
                } else {
                    // 其他工具都只有 directories 字段，无法区分，默认使用重复文件工具
                    "file.duplicates"
                }
            };

            // 根据工具名称直接匹配对应的工具实现并执行
            let output_value = match tool_name {
                name if name == tools::duplicates::DuplicateFilesTool.name() => {
                    let tool = tools::duplicates::DuplicateFilesTool;
                    tool.run(input_value).await?
                },
                name if name == tools::similar_images::SimilarImagesTool.name() => {
                    let tool = tools::similar_images::SimilarImagesTool;
                    tool.run(input_value).await?
                },
                name if name == tools::empty_dirs::EmptyDirectoriesTool.name() => {
                    let tool = tools::empty_dirs::EmptyDirectoriesTool;
                    tool.run(input_value).await?
                },
                name if name == tools::temp_files::TemporaryFilesTool.name() => {
                    let tool = tools::temp_files::TemporaryFilesTool;
                    tool.run(input_value).await?
                },
                name if name == tools::broken_symlinks::BrokenSymlinksTool.name() => {
                    let tool = tools::broken_symlinks::BrokenSymlinksTool;
                    tool.run(input_value).await?
                },
                _ => {
                    anyhow::bail!("Unknown tool: {}", tool_name)
                }
            };
            println!("{}", serde_json::to_string(&output_value)?);
        }
    }

    Ok(())
}
