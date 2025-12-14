use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use crate::{error::Result, logger::port::LogSinkPort}; // 删除未使用的 CoreError 导入

/// 控制台日志输出适配器，将日志输出到控制台
pub struct ConsoleSink {
    /// 是否启用彩色输出
    color_enabled: bool,
    
    /// 输出目标名称
    name: String,
    
    /// 输出流，使用 Mutex 确保线程安全
    stdout: Mutex<BufWriter<std::io::Stdout>>,
}

impl ConsoleSink {
    /// 创建新的控制台日志输出适配器
    /// 
    /// # 参数
    /// * `color_enabled` - 是否启用彩色输出
    /// * `name` - 输出目标名称
    /// 
    /// # 返回值
    /// * `Ok(ConsoleSink)` - 成功创建控制台日志输出适配器
    pub fn new(color_enabled: Option<bool>, name: Option<String>) -> Result<Self> {
        Ok(Self {
            color_enabled: color_enabled.unwrap_or(true), // 默认启用彩色输出
            name: name.unwrap_or_else(|| "console".to_string()),
            stdout: Mutex::new(BufWriter::new(std::io::stdout())),
        })
    }
    
    /// 根据日志级别添加颜色
    /// 
    /// # 参数
    /// * `formatted_log` - 格式化后的日志字符串
    /// 
    /// # 返回值
    /// * `String` - 添加颜色后的日志字符串
    fn add_color(&self, formatted_log: &str) -> String {
        if !self.color_enabled {
            return formatted_log.to_string();
        }
        
        // 检查日志级别并添加颜色
        let colored = if formatted_log.contains("DEBUG") {
            // 调试级别：蓝色
            format!("\x1b[34m{}\x1b[0m", formatted_log)
        } else if formatted_log.contains("INFO") {
            // 信息级别：绿色
            format!("\x1b[32m{}\x1b[0m", formatted_log)
        } else if formatted_log.contains("WARN") {
            // 警告级别：黄色
            format!("\x1b[33m{}\x1b[0m", formatted_log)
        } else if formatted_log.contains("ERROR") {
            // 错误级别：红色
            format!("\x1b[31m{}\x1b[0m", formatted_log)
        } else {
            formatted_log.to_string()
        };
        
        colored
    }
}

impl LogSinkPort for ConsoleSink {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn write(&self, formatted_log: &str) -> Result<()> {
        let mut stdout = self.stdout.lock().unwrap();
        
        // 添加颜色
        let colored_log = self.add_color(formatted_log);
        
        // 写入控制台并刷新
        writeln!(stdout, "{}", colored_log)?;
        stdout.flush()?;
        
        Ok(())
    }
}

/// 文件日志输出适配器，将日志输出到文件
pub struct FileSink {
    /// 输出目标名称
    name: String,
    
    /// 日志文件路径
    #[allow(dead_code)]
    path: PathBuf,
    
    /// 输出流，使用 Mutex 确保线程安全
    file: Mutex<BufWriter<std::fs::File>>,
}

impl FileSink {
    /// 创建新的文件日志输出适配器
    /// 
    /// # 参数
    /// * `path` - 日志文件路径
    /// * `name` - 输出目标名称
    /// 
    /// # 返回值
    /// * `Ok(FileSink)` - 成功创建文件日志输出适配器
    pub fn new(path: PathBuf, name: Option<String>) -> Result<Self> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // 打开文件，支持追加写入
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&path)?;
        
        Ok(Self {
            name: name.unwrap_or_else(|| "file".to_string()),
            path,
            file: Mutex::new(BufWriter::new(file)),
        })
    }
}

impl LogSinkPort for FileSink {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn write(&self, formatted_log: &str) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        
        // 写入文件并刷新
        writeln!(file, "{}", formatted_log)?;
        file.flush()?;
        
        Ok(())
    }
}