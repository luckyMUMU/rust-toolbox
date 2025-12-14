use eframe::egui::{Ui, Response, ProgressBar as EguiProgressBar, Color32, RichText};use std::fmt::Display;

/// 进度条样式
enum ProgressStyle {
    /// 标准进度条
    Standard,
    /// 成功进度条
    Success,
    /// 警告进度条
    Warning,
    /// 危险进度条
    Danger,
    /// 信息进度条
    Info,
}

/// 进度条配置
struct ProgressConfig {
    /// 进度条样式
    style: ProgressStyle,
    /// 进度值 (0.0 - 1.0)
    progress: f32,
    /// 是否显示百分比
    show_percentage: bool,
    /// 是否显示文本
    show_text: bool,
    /// 自定义文本（可选）
    text: Option<String>,
    /// 进度条高度
    height: f32,
    /// 是否动画
    animated: bool,
    /// 动画速度
    animation_speed: f32,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            style: ProgressStyle::Standard,
            progress: 0.0,
            show_percentage: true,
            show_text: true,
            text: None,
            height: 8.0,
            animated: false,
            animation_speed: 1.0,
        }
    }
}

/// 进度条组件
pub struct ProgressBar {
    /// 进度条配置
    config: ProgressConfig,
}

impl ProgressBar {
    /// 创建新进度条
    pub fn new() -> Self {
        Self {
            config: ProgressConfig::default(),
        }
    }
    
    /// 设置进度值 (0.0 - 1.0)
    pub fn progress(mut self, progress: f32) -> Self {
        self.config.progress = progress.clamp(0.0, 1.0);
        self
    }
    
    /// 设置为成功样式
    pub fn success(mut self) -> Self {
        self.config.style = ProgressStyle::Success;
        self
    }
    
    /// 设置为警告样式
    pub fn warning(mut self) -> Self {
        self.config.style = ProgressStyle::Warning;
        self
    }
    
    /// 设置为危险样式
    pub fn danger(mut self) -> Self {
        self.config.style = ProgressStyle::Danger;
        self
    }
    
    /// 设置为信息样式
    pub fn info(mut self) -> Self {
        self.config.style = ProgressStyle::Info;
        self
    }
    
    /// 设置是否显示百分比
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.config.show_percentage = show;
        self
    }
    
    /// 设置是否显示文本
    pub fn show_text(mut self, show: bool) -> Self {
        self.config.show_text = show;
        self
    }
    
    /// 设置自定义文本
    pub fn text<T: Display>(mut self, text: T) -> Self {
        self.config.text = Some(text.to_string());
        self
    }
    
    /// 设置进度条高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = height;
        self
    }
    
    /// 设置为动画进度条
    pub fn animated(mut self) -> Self {
        self.config.animated = true;
        self
    }
    
    /// 设置动画速度
    pub fn animation_speed(mut self, speed: f32) -> Self {
        self.config.animation_speed = speed;
        self
    }
    
    /// 渲染进度条
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let ProgressConfig {
            style,
            progress,
            show_percentage,
            show_text,
            text,
            height,
            animated,
            animation_speed,
        } = &self.config;
        
        // 根据样式获取颜色
        let fill_color = match style {
            ProgressStyle::Standard => Color32::from_rgb(74, 144, 226),
            ProgressStyle::Success => Color32::from_rgb(126, 211, 33),
            ProgressStyle::Warning => Color32::from_rgb(245, 166, 35),
            ProgressStyle::Danger => Color32::from_rgb(208, 2, 27),
            ProgressStyle::Info => Color32::from_rgb(33, 150, 243),
        };
        
        // 构建进度条
        let mut progress_bar = EguiProgressBar::new(*progress)
            .fill(fill_color)
            .height(*height);
        
        // 设置进度条文本
        if *show_text {
            let display_text = if let Some(text) = text {
                text.clone()
            } else if *show_percentage {
                format!("{:.0}%", *progress * 100.0)
            } else {
                String::new()
            };
            
            progress_bar = progress_bar.text(RichText::new(display_text));
        }
        
        // 渲染进度条
        ui.add(progress_bar)
    }
    
    /// 获取当前进度值
    pub fn get_progress(&self) -> f32 {
        self.config.progress
    }
    
    /// 更新进度值
    pub fn update_progress(&mut self, progress: f32) {
        self.config.progress = progress.clamp(0.0, 1.0);
    }
}

/// 进度条构建器
pub struct ProgressBarBuilder {
    /// 进度条配置
    config: ProgressConfig,
}

impl ProgressBarBuilder {
    /// 创建新的进度条构建器
    pub fn new() -> Self {
        Self {
            config: ProgressConfig::default(),
        }
    }
    
    /// 设置进度值 (0.0 - 1.0)
    pub fn progress(mut self, progress: f32) -> Self {
        self.config.progress = progress.clamp(0.0, 1.0);
        self
    }
    
    /// 设置为成功样式
    pub fn success(mut self) -> Self {
        self.config.style = ProgressStyle::Success;
        self
    }
    
    /// 设置为警告样式
    pub fn warning(mut self) -> Self {
        self.config.style = ProgressStyle::Warning;
        self
    }
    
    /// 设置为危险样式
    pub fn danger(mut self) -> Self {
        self.config.style = ProgressStyle::Danger;
        self
    }
    
    /// 设置为信息样式
    pub fn info(mut self) -> Self {
        self.config.style = ProgressStyle::Info;
        self
    }
    
    /// 设置是否显示百分比
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.config.show_percentage = show;
        self
    }
    
    /// 设置是否显示文本
    pub fn show_text(mut self, show: bool) -> Self {
        self.config.show_text = show;
        self
    }
    
    /// 设置自定义文本
    pub fn text<T: Display>(mut self, text: T) -> Self {
        self.config.text = Some(text.to_string());
        self
    }
    
    /// 设置进度条高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = height;
        self
    }
    
    /// 设置为动画进度条
    pub fn animated(mut self) -> Self {
        self.config.animated = true;
        self
    }
    
    /// 设置动画速度
    pub fn animation_speed(mut self, speed: f32) -> Self {
        self.config.animation_speed = speed;
        self
    }
    
    /// 构建进度条
    pub fn build(self) -> ProgressBar {
        ProgressBar {
            config: self.config,
        }
    }
}
