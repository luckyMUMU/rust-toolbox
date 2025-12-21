use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::Result;
use std::fs;
use std::io::Write;
use rt_core::Locale;

/// 主题选项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    /// 浅色主题
    Light,
    /// 深色主题
    Dark,
    /// 跟随系统主题
    #[default]
    System,
}

impl Theme {
    /// 获取主题的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Light => "浅色",
            Theme::Dark => "深色",
            Theme::System => "跟随系统",
        }
    }
    
    /// 获取所有主题选项
    pub fn all() -> [Theme; 3] {
        [Theme::Light, Theme::Dark, Theme::System]
    }
}

/// 通知设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationSettings {
    /// 工具运行完成通知
    pub task_completed: bool,
    /// 错误通知
    pub errors: bool,
    /// 警告通知
    pub warnings: bool,
}

/// 快捷键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    /// 新建标签页快捷键
    pub new_tab: String,
    /// 关闭标签页快捷键
    pub close_tab: String,
    /// 运行工具快捷键
    pub run_tool: String,
    /// 显示/隐藏帮助快捷键
    pub toggle_help: String,
    /// 显示/隐藏设置快捷键
    pub toggle_settings: String,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// 显示偏好设置
    pub display: DisplaySettings,
    /// 通知设置
    pub notifications: NotificationSettings,
    /// 数据存储设置
    pub data: DataSettings,
    /// 快捷键配置
    pub shortcuts: ShortcutSettings,
}

/// 显示偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    /// 主题
    pub theme: Theme,
    /// 语言
    pub locale: Locale,
    /// 字体大小
    pub font_size: f32,
    /// 侧边栏宽度
    pub sidebar_width: f32,
}

/// 数据存储设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSettings {
    /// 插件目录
    pub plugins_dir: PathBuf,
    /// 日志目录
    pub logs_dir: PathBuf,
    /// 临时文件目录
    pub temp_dir: PathBuf,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            locale: Locale::En,
            font_size: 14.0,
            sidebar_width: 250.0,
        }
    }
}

impl Default for DataSettings {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "rusttoolbox", "RustToolbox").unwrap();
        
        Self {
            plugins_dir: project_dirs.data_dir().join("plugins"),
            logs_dir: project_dirs.cache_dir().join("logs"),
            temp_dir: std::env::temp_dir().join("rusttoolbox"),
        }
    }
}



impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            new_tab: "Ctrl+T".to_string(),
            close_tab: "Ctrl+W".to_string(),
            run_tool: "Ctrl+Enter".to_string(),
            toggle_help: "F1".to_string(),
            toggle_settings: "Ctrl+, ".to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display: DisplaySettings::default(),
            notifications: NotificationSettings::default(),
            data: DataSettings::default(),
            shortcuts: ShortcutSettings::default(),
        }
    }
}

impl Settings {
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        if let Some(config_path) = Self::get_config_path() {
            if config_path.exists() {
                let content = fs::read_to_string(config_path)?;
                let settings: Self = serde_json::from_str(&content)?;
                return Ok(settings);
            }
        }
        Ok(Self::default())
    }
    
    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        if let Some(config_path) = Self::get_config_path() {
            // 确保配置目录存在
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            let content = serde_json::to_string_pretty(self)?;
            let mut file = fs::File::create(config_path)?;
            file.write_all(content.as_bytes())?;
        }
        Ok(())
    }
    
    /// 获取配置文件路径
    fn get_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "rusttoolbox", "RustToolbox")
            .map(|dirs| dirs.config_dir().join("settings.json"))
    }
    
    /// 验证配置是否有效
    pub fn validate(&self) -> Result<()> {
        // 验证插件目录
        if !self.data.plugins_dir.exists() {
            fs::create_dir_all(&self.data.plugins_dir)?;
        }
        
        // 验证日志目录
        if !self.data.logs_dir.exists() {
            fs::create_dir_all(&self.data.logs_dir)?;
        }
        
        // 验证临时文件目录
        if !self.data.temp_dir.exists() {
            fs::create_dir_all(&self.data.temp_dir)?;
        }
        
        // 验证字体大小
        if self.display.font_size < 8.0 || self.display.font_size > 32.0 {
            anyhow::bail!("字体大小必须在8到32之间");
        }
        
        // 验证侧边栏宽度
        if self.display.sidebar_width < 100.0 || self.display.sidebar_width > 500.0 {
            anyhow::bail!("侧边栏宽度必须在100到500之间");
        }
        
        Ok(())
    }
    
    /// 恢复默认设置
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}
