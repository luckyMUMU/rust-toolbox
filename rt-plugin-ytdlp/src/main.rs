use clap::{Parser, Subcommand};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use tokio::process::Command;
use log::{debug, info, error};
use log4rs;
use anyhow::Result;

use rt_core::{Locale, plugin::{PluginMetadata, LocalizedString}};
use rt_core::{Tool, CoreError};
use async_trait::async_trait;

mod i18n;

/// 初始化日志系统
fn init_logger() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap_or_else(|_| {
        // 如果找不到配置文件，使用默认配置
        let config = log4rs::config::Config::builder()
            .appender(
                log4rs::config::Appender::builder()
                    .build("stdout", Box::new(log4rs::append::console::ConsoleAppender::builder().build()))
            )
            .logger(
                log4rs::config::Logger::builder()
                    .appender("stdout")
                    .build("rt-plugin-ytdlp", log::LevelFilter::Info)
            )
            .build(log4rs::config::Root::builder().build(log::LevelFilter::Off))
            .unwrap();
        
        log4rs::init_config(config).unwrap();
    });
}

/// YouTube Downloader Plugin
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 输出插件规范
    Spec {
        /// 规范的语言环境（例如："en"，"zh-CN"）
        #[arg(long, default_value = "en")]
        locale: String,
    },
    /// 运行插件逻辑
    Run,
}

/// 输入参数结构体
#[derive(Serialize, Deserialize, JsonSchema)]
struct Input {
    /// 要下载的视频或播放列表的URL
    url: String,
    /// 要下载的格式（例如：'best', 'bestvideo+bestaudio', 'bestaudio'）
    #[serde(default = "default_format")]
    format: String,
    /// 是否下载整个播放列表
    #[serde(default = "default_playlist")]
    playlist: bool,
    /// 是否下载字幕
    #[serde(default = "default_subtitles")]
    subtitles: bool,
    /// 保存下载文件的目录
    #[serde(default = "default_output_dir")]
    output_dir: String,
    /// 输出文件名的模板
    #[serde(default = "default_filename_template")]
    filename_template: String,
}

fn default_format() -> String { "best".to_string() }
fn default_playlist() -> bool { false }
fn default_subtitles() -> bool { false }
fn default_output_dir() -> String { ".".to_string() }
fn default_filename_template() -> String { "%(title)s.%(ext)s".to_string() }

/// 下载的文件信息
#[derive(Serialize, Deserialize, JsonSchema)]
struct DownloadedFile {
    /// 文件路径
    path: String,
    /// 文件大小（字节）
    size: u64,
}

/// 输出结果结构体
#[derive(Serialize, Deserialize, JsonSchema)]
struct Output {
    /// 下载是否成功
    success: bool,
    /// 下载的文件列表
    files: Vec<DownloadedFile>,
    /// 描述结果的消息
    message: String,
}

/// YouTube Downloader Tool
struct YtdlpTool;

#[async_trait]
impl Tool for YtdlpTool {
    /// 返回工具的唯一名称
    fn name(&self) -> &str {
        "media.ytdlp"
    }

    /// 返回工具的本地化显示名称
    fn display_name(&self, locale: Locale) -> String {
        i18n::display_name(locale)
    }

    /// 返回工具的本地化描述
    fn description(&self, locale: Locale) -> String {
        i18n::description(locale)
    }

    /// 返回工具的本地化用户指南
    fn user_guide(&self, locale: Locale) -> String {
        i18n::user_guide(locale)
    }

    /// 返回工具的输入JSON Schema
    fn input_schema(&self, _locale: Locale) -> serde_json::Value {
        serde_json::to_value(schema_for!(Input)).unwrap_or_else(|_| serde_json::json!({}))
    }

    /// 返回工具的输出JSON Schema
    fn output_schema(&self, _locale: Locale) -> serde_json::Value {
        serde_json::to_value(schema_for!(Output)).unwrap_or_else(|_| serde_json::json!({}))
    }

    /// 运行YouTube下载器逻辑
    async fn run(&self, input: serde_json::Value) -> std::result::Result<serde_json::Value, CoreError> {
        let input: Input = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(e) => {
                return Err(CoreError::ToolFailure(format!("解析输入失败: {}", e)));
            }
        };
        
        info!("开始下载: URL={}, Format={}, Playlist={}, Subtitles={}", 
              input.url, input.format, input.playlist, input.subtitles);

        let mut cmd = Command::new("yt-dlp");
        cmd.arg(&input.url)
            .arg("--format")
            .arg(&input.format)
            .arg("--output")
            .arg(&format!("{}/{}", input.output_dir, input.filename_template));

        // 添加播放列表选项
        if input.playlist {
            cmd.arg("--yes-playlist");
        } else {
            cmd.arg("--no-playlist");
        }

        // 添加字幕选项
        if input.subtitles {
            cmd.arg("--write-auto-sub")
                .arg("--sub-lang")
                .arg("all")
                .arg("--convert-subs")
                .arg("srt");
        }

        // 执行命令
        debug!("执行命令: {:?}", cmd);
        let output = match cmd.output().await {
            Ok(output) => output,
            Err(e) => {
                return Err(CoreError::ToolFailure(format!("执行yt-dlp命令失败: {}", e)));
            }
        };

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("yt-dlp stdout: {}", stdout);
            debug!("yt-dlp stderr: {}", stderr);

            // 解析输出以获取下载的文件信息（简化版本）
            let files = Vec::new(); // 实际实现中可以解析yt-dlp输出获取文件列表
            let message = "下载成功".to_string();

            let output = Output {
                success: true,
                files,
                message,
            };

            match serde_json::to_value(output) {
                Ok(output_value) => Ok(output_value),
                Err(e) => Err(CoreError::ToolFailure(format!("序列化输出失败: {}", e))),
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("yt-dlp 下载失败: {}", stderr);
            Err(CoreError::ToolFailure(format!("下载失败: {}", stderr)))
        }
    }
}

/// 生成并打印插件规范
fn print_spec(locale: &str) {
    let tool = YtdlpTool;
    let plugin_locale = match locale {
        "zh" | "zh-CN" => Locale::Zh,
        _ => Locale::En,
    };
    
    let metadata = PluginMetadata {
        name: tool.name().to_string(),
        display_name: LocalizedString {
            en: i18n::display_name(Locale::En),
            zh: Some(i18n::display_name(Locale::Zh)),
        },
        description: LocalizedString {
            en: i18n::description(Locale::En),
            zh: Some(i18n::description(Locale::Zh)),
        },
        user_guide: LocalizedString {
            en: i18n::user_guide(Locale::En),
            zh: Some(i18n::user_guide(Locale::Zh)),
        },
        input_schema: tool.input_schema(plugin_locale),
        output_schema: Some(tool.output_schema(plugin_locale)),
        input_fields: Some(i18n::get_input_field_map()),
        output_fields: Some(i18n::get_output_field_map()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        author: Some(env!("CARGO_PKG_AUTHORS").to_string()),
    };

    let spec_json = serde_json::to_string_pretty(&metadata).unwrap();
    println!("{}", spec_json);
}

/// 从标准输入读取JSON，运行工具，并将结果输出到标准输出
async fn run_tool() -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    
    let input_value: serde_json::Value = serde_json::from_str(&input)?;
    let tool = YtdlpTool;
    let output_value = tool.run(input_value).await?;
    
    println!("{}", serde_json::to_string_pretty(&output_value)?);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    info!("YouTube Downloader Plugin 启动");
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Spec { locale } => {
            print_spec(&locale);
        }
        Commands::Run => {
            run_tool().await?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let input = Input {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            format: default_format(),
            playlist: default_playlist(),
            subtitles: default_subtitles(),
            output_dir: default_output_dir(),
            filename_template: default_filename_template(),
        };
        
        assert_eq!(input.format, "best");
        assert_eq!(input.playlist, false);
        assert_eq!(input.subtitles, false);
        assert_eq!(input.output_dir, ".");
        assert_eq!(input.filename_template, "%(title)s.%(ext)s");
    }

    #[tokio::test]
    async fn test_run_invalid_url() {
        let tool = YtdlpTool;
        let input = serde_json::json!({
            "url": "invalid_url",
            "format": "best",
            "playlist": false,
            "subtitles": false,
            "output_dir": ".",
            "filename_template": "%(title)s.%(ext)s"
        });
        
        let result = tool.run(input).await;
        assert!(result.is_err());
    }
}
