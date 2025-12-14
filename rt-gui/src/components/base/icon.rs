use eframe::egui::{Ui, Response, RichText, Color32, TextStyle, Widget};use std::fmt::Display;

/// 图标类型
enum IconType {
    /// 内置图标（使用字符表示）
    BuiltIn(&'static str),
    /// 自定义图标（使用路径表示）
    Custom(String),
}

/// 图标配置
struct IconConfig {
    /// 图标类型
    icon_type: IconType,
    /// 图标大小
    size: f32,
    /// 图标颜色
    color: Color32,
    /// 是否可点击
    clickable: bool,
    /// 点击回调
    on_click: Option<Box<dyn FnMut()>>,
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            icon_type: IconType::BuiltIn("📌"),
            size: 16.0,
            color: Color32::BLACK,
            clickable: false,
            on_click: None,
        }
    }
}

/// 图标组件
pub struct Icon {
    /// 图标配置
    config: IconConfig,
}

impl Icon {
    /// 创建内置图标
    pub fn new(icon: &'static str) -> Self {
        Self {
            config: IconConfig {
                icon_type: IconType::BuiltIn(icon),
                ..Default::default()
            },
        }
    }
    
    /// 创建自定义图标
    pub fn custom(path: &str) -> Self {
        Self {
            config: IconConfig {
                icon_type: IconType::Custom(path.to_string()),
                ..Default::default()
            },
        }
    }
    
    /// 设置图标大小
    pub fn size(mut self, size: f32) -> Self {
        self.config.size = size;
        self
    }
    
    /// 设置图标颜色
    pub fn color(mut self, color: Color32) -> Self {
        self.config.color = color;
        self
    }
    
    /// 设置为可点击图标
    pub fn clickable(mut self) -> Self {
        self.config.clickable = true;
        self
    }
    
    /// 设置点击回调
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.clickable = true;
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 渲染图标
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let IconConfig {
            icon_type,
            size,
            color,
            clickable,
            ref mut on_click,
        } = &mut self.config;
        
        // 渲染内置图标
        match icon_type {
            IconType::BuiltIn(icon_str) => {
                let rich_text = RichText::new(icon_str)
                    .size(*size)
                    .color(*color);
                
                if *clickable {
                    // 可点击图标
                    let response = ui.add(egui::Button::new(rich_text)
                        .fill(egui::Color32::TRANSPARENT)
                        .hover_fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 20))
                        .frame(false));
                    
                    if response.clicked() {
                        if let Some(callback) = on_click.as_mut() {
                            callback();
                        }
                    }
                    
                    response
                } else {
                    // 普通图标
                    ui.label(rich_text)
                }
            },
            IconType::Custom(path) => {
                // 自定义图标（暂时使用占位符）
                let rich_text = RichText::new("📁")
                    .size(*size)
                    .color(*color);
                
                ui.label(rich_text)
            },
        }
    }
}

/// 图标构建器
pub struct IconBuilder {
    /// 图标配置
    config: IconConfig,
}

impl IconBuilder {
    /// 创建内置图标构建器
    pub fn new(icon: &'static str) -> Self {
        Self {
            config: IconConfig {
                icon_type: IconType::BuiltIn(icon),
                ..Default::default()
            },
        }
    }
    
    /// 创建自定义图标构建器
    pub fn custom(path: &str) -> Self {
        Self {
            config: IconConfig {
                icon_type: IconType::Custom(path.to_string()),
                ..Default::default()
            },
        }
    }
    
    /// 设置图标大小
    pub fn size(mut self, size: f32) -> Self {
        self.config.size = size;
        self
    }
    
    /// 设置图标颜色
    pub fn color(mut self, color: Color32) -> Self {
        self.config.color = color;
        self
    }
    
    /// 设置为可点击图标
    pub fn clickable(mut self) -> Self {
        self.config.clickable = true;
        self
    }
    
    /// 设置点击回调
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.clickable = true;
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 构建图标
    pub fn build(self) -> Icon {
        Icon {
            config: self.config,
        }
    }
}

/// 常用图标常量
pub mod icons {
    /// 运行图标
    pub const RUN: &str = "▶️";
    /// 停止图标
    pub const STOP: &str = "⏹️";
    /// 刷新图标
    pub const REFRESH: &str = "🔄";
    /// 设置图标
    pub const SETTINGS: &str = "⚙️";
    /// 帮助图标
    pub const HELP: &str = "❓";
    /// 关闭图标
    pub const CLOSE: &str = "✕";
    /// 添加图标
    pub const ADD: &str = "➕";
    /// 删除图标
    pub const DELETE: &str = "🗑️";
    /// 搜索图标
    pub const SEARCH: &str = "🔍";
    /// 文件图标
    pub const FILE: &str = "📄";
    /// 文件夹图标
    pub const FOLDER: &str = "📁";
    /// 成功图标
    pub const SUCCESS: &str = "✅";
    /// 错误图标
    pub const ERROR: &str = "❌";
    /// 警告图标
    pub const WARNING: &str = "⚠️";
    /// 信息图标
    pub const INFO: &str = "ℹ️";
    /// 代码图标
    pub const CODE: &str = "💻";
    /// 文本图标
    pub const TEXT: &str = "📝";
    /// 图片图标
    pub const IMAGE: &str = "🖼️";
    /// 音频图标
    pub const AUDIO: &str = "🔊";
    /// 视频图标
    pub const VIDEO: &str = "🎬";
    /// 下载图标
    pub const DOWNLOAD: &str = "⬇️";
    /// 上传图标
    pub const UPLOAD: &str = "⬆️";
    /// 复制图标
    pub const COPY: &str = "📋";
    /// 剪切图标
    pub const CUT: &str = "✂️";
    /// 粘贴图标
    pub const PASTE: &str = "📋";
    /// 撤销图标
    pub const UNDO: &str = "↩️";
    /// 重做图标
    pub const REDO: &str = "↪️";
    /// 放大图标
    pub const ZOOM_IN: &str = "🔍➕";
    /// 缩小图标
    pub const ZOOM_OUT: &str = "🔍➖";
    /// 全屏图标
    pub const FULLSCREEN: &str = "⛶";
    /// 退出全屏图标
    pub const EXIT_FULLSCREEN: &str = "⛶";
    /// 锁图标
    pub const LOCK: &str = "🔒";
    /// 解锁图标
    pub const UNLOCK: &str = "🔓";
    /// 用户图标
    pub const USER: &str = "👤";
    /// 团队图标
    pub const TEAM: &str = "👥";
    /// 星标图标
    pub const STAR: &str = "⭐";
    /// 收藏图标
    pub const FAVORITE: &str = "❤️";
    /// 分享图标
    pub const SHARE: &str = "🔗";
    /// 评论图标
    pub const COMMENT: &str = "💬";
    /// 通知图标
    pub const NOTIFICATION: &str = "🔔";
    /// 时钟图标
    pub const CLOCK: &str = "🕒";
    /// 日历图标
    pub const CALENDAR: &str = "📅";
    /// 地理位置图标
    pub const LOCATION: &str = "📍";
    /// 导航图标
    pub const NAVIGATION: &str = "🧭";
    /// 过滤图标
    pub const FILTER: &str = "🔍";
    /// 排序图标
    pub const SORT: &str = "↕️";
    /// 升序图标
    pub const SORT_ASC: &str = "⬆️";
    /// 降序图标
    pub const SORT_DESC: &str = "⬇️";
    /// 网格视图图标
    pub const GRID_VIEW: &str = "⬜";
    /// 列表视图图标
    pub const LIST_VIEW: &str = "📋";
    /// 详情视图图标
    pub const DETAILS_VIEW: &str = "📊";
    /// 隐藏图标
    pub const HIDE: &str = "🙈";
    /// 显示图标
    pub const SHOW: &str = "👁️";
    /// 展开图标
    pub const EXPAND: &str = "▼";
    /// 折叠图标
    pub const COLLAPSE: &str = "▶️";
}
