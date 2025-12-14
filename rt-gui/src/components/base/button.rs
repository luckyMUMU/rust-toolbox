use eframe::egui::{Ui, Response, Button as EguiButton, RichText, Color32};use std::fmt::Display;

/// 按钮样式
enum ButtonStyle {
    /// 主要按钮 - 用于重要操作
    Primary,
    /// 次要按钮 - 用于次要操作
    Secondary,
    /// 危险按钮 - 用于危险操作
    Danger,
    /// 文本按钮 - 仅显示文本，无背景
    Text,
}

/// 按钮大小
enum ButtonSize {
    /// 小按钮
    Small,
    /// 中按钮
    Medium,
    /// 大按钮
    Large,
}

/// 按钮配置
struct ButtonConfig {
    /// 按钮样式
    style: ButtonStyle,
    /// 按钮大小
    size: ButtonSize,
    /// 是否禁用
    disabled: bool,
    /// 图标（可选）
    icon: Option<&'static str>,
    /// 点击回调
    on_click: Option<Box<dyn FnMut()>>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            style: ButtonStyle::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            icon: None,
            on_click: None,
        }
    }
}

/// 按钮组件
pub struct Button {
    /// 按钮文本
    text: String,
    /// 按钮配置
    config: ButtonConfig,
}

impl Button {
    /// 创建新按钮
    pub fn new<T: Display>(text: T) -> Self {
        Self {
            text: text.to_string(),
            config: ButtonConfig::default(),
        }
    }
    
    /// 设置按钮样式
    pub fn primary(mut self) -> Self {
        self.config.style = ButtonStyle::Primary;
        self
    }
    
    /// 设置为次要按钮
    pub fn secondary(mut self) -> Self {
        self.config.style = ButtonStyle::Secondary;
        self
    }
    
    /// 设置为危险按钮
    pub fn danger(mut self) -> Self {
        self.config.style = ButtonStyle::Danger;
        self
    }
    
    /// 设置为文本按钮
    pub fn text(mut self) -> Self {
        self.config.style = ButtonStyle::Text;
        self
    }
    
    /// 设置按钮大小为小
    pub fn small(mut self) -> Self {
        self.config.size = ButtonSize::Small;
        self
    }
    
    /// 设置按钮大小为中
    pub fn medium(mut self) -> Self {
        self.config.size = ButtonSize::Medium;
        self
    }
    
    /// 设置按钮大小为大
    pub fn large(mut self) -> Self {
        self.config.size = ButtonSize::Large;
        self
    }
    
    /// 设置按钮图标
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.config.icon = Some(icon);
        self
    }
    
    /// 设置按钮为禁用状态
    pub fn disabled(mut self) -> Self {
        self.config.disabled = true;
        self
    }
    
    /// 设置按钮点击回调
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 渲染按钮
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let ButtonConfig {
            style,
            size,
            disabled,
            icon,
            ref mut on_click,
        } = &mut self.config;
        
        // 根据样式获取颜色
        let (bg_color, text_color, hover_color) = match style {
            ButtonStyle::Primary => {
                if *disabled {
                    (Color32::from_rgb(173, 216, 230), Color32::WHITE, Color32::from_rgb(135, 206, 235))
                } else {
                    (Color32::from_rgb(74, 144, 226), Color32::WHITE, Color32::from_rgb(56, 118, 191))
                }
            },
            ButtonStyle::Secondary => {
                if *disabled {
                    (Color32::from_rgb(224, 224, 224), Color32::GRAY, Color32::from_rgb(200, 200, 200))
                } else {
                    (Color32::from_rgb(245, 245, 245), Color32::BLACK, Color32::from_rgb(224, 224, 224))
                }
            },
            ButtonStyle::Danger => {
                if *disabled {
                    (Color32::from_rgb(255, 192, 203), Color32::WHITE, Color32::from_rgb(255, 182, 193))
                } else {
                    (Color32::from_rgb(208, 2, 27), Color32::WHITE, Color32::from_rgb(178, 1, 23))
                }
            },
            ButtonStyle::Text => {
                if *disabled {
                    (Color32::TRANSPARENT, Color32::GRAY, Color32::TRANSPARENT)
                } else {
                    (Color32::TRANSPARENT, Color32::from_rgb(74, 144, 226), Color32::from_rgb(232, 240, 254))
                }
            },
        };
        
        // 根据大小获取内边距
        let padding = match size {
            ButtonSize::Small => (4.0, 8.0),
            ButtonSize::Medium => (8.0, 16.0),
            ButtonSize::Large => (12.0, 24.0),
        };
        
        // 构建按钮文本
        let mut button_text = if let Some(icon) = icon {
            RichText::new(format!("{} {}", icon, self.text))
        } else {
            RichText::new(&self.text)
        };
        
        button_text = button_text.color(text_color);
        
        // 创建按钮
        let mut button = EguiButton::new(button_text)
            .fill(bg_color)
            .text_color(text_color)
            .small()
            .enabled(!*disabled);
        
        // 根据大小调整按钮
        match size {
            ButtonSize::Small => button = button.small(),
            ButtonSize::Medium => button = button.normal(),
            ButtonSize::Large => button = button.large(),
        }
        
        // 渲染按钮
        let response = ui.add_sized(
            (ui.spacing().interact_size.x * 1.5, ui.spacing().interact_size.y),
            button,
        );
        
        // 处理点击事件
        if response.clicked() && !*disabled {
            if let Some(callback) = on_click.as_mut() {
                callback();
            }
        }
        
        response
    }
}

/// 按钮构建器
pub struct ButtonBuilder {
    text: String,
    config: ButtonConfig,
}

impl ButtonBuilder {
    /// 创建新的按钮构建器
    pub fn new<T: Display>(text: T) -> Self {
        Self {
            text: text.to_string(),
            config: ButtonConfig::default(),
        }
    }
    
    /// 设置为主要按钮
    pub fn primary(mut self) -> Self {
        self.config.style = ButtonStyle::Primary;
        self
    }
    
    /// 设置为次要按钮
    pub fn secondary(mut self) -> Self {
        self.config.style = ButtonStyle::Secondary;
        self
    }
    
    /// 设置为危险按钮
    pub fn danger(mut self) -> Self {
        self.config.style = ButtonStyle::Danger;
        self
    }
    
    /// 设置为文本按钮
    pub fn text(mut self) -> Self {
        self.config.style = ButtonStyle::Text;
        self
    }
    
    /// 设置为小按钮
    pub fn small(mut self) -> Self {
        self.config.size = ButtonSize::Small;
        self
    }
    
    /// 设置为中按钮
    pub fn medium(mut self) -> Self {
        self.config.size = ButtonSize::Medium;
        self
    }
    
    /// 设置为大按钮
    pub fn large(mut self) -> Self {
        self.config.size = ButtonSize::Large;
        self
    }
    
    /// 设置按钮图标
    pub fn with_icon(mut self, icon: &'static str) -> Self {
        self.config.icon = Some(icon);
        self
    }
    
    /// 设置按钮为禁用状态
    pub fn disabled(mut self) -> Self {
        self.config.disabled = true;
        self
    }
    
    /// 设置点击回调
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 构建按钮
    pub fn build(self) -> Button {
        Button {
            text: self.text,
            config: self.config,
        }
    }
}
