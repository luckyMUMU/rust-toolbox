use eframe::egui::{Response, Ui, Widget, CollapsingHeader, Stroke, vec2, RichText, Layout, Align, Color32, Rounding}; use std::fmt::Display;

/// 卡片样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
    /// 阴影样式
    Shadow,
    /// 圆角样式
    Rounded,
}

/// 卡片组件配置
#[derive(Debug)]
pub struct CardConfig {
    /// 卡片标题
    pub title: Option<String>,
    /// 卡片内容
    pub content: String,
    /// 卡片样式
    pub style: CardStyle,
    /// 是否可折叠
    pub collapsible: bool,
    /// 是否默认折叠
    pub default_collapsed: bool,
    /// 是否可点击
    pub clickable: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 卡片图标
    pub icon: Option<String>,
    /// 卡片操作按钮
    pub actions: Vec<CardAction>,
    /// 卡片点击回调
    pub on_click: Option<Box<dyn Fn() + Send + Sync>>,
    /// 折叠状态变化回调
    pub on_collapse_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

/// 卡片操作按钮
#[derive(Debug)]
pub struct CardAction {
    /// 操作按钮文本
    pub text: String,
    /// 操作按钮图标
    pub icon: Option<String>,
    /// 操作按钮点击回调
    pub on_click: Box<dyn Fn() + Send + Sync>,
    /// 是否禁用
    pub disabled: bool,
}

impl CardAction {
    /// 创建新的卡片操作按钮
    pub fn new(text: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            text: text.into(),
            icon: None,
            on_click: Box::new(on_click),
            disabled: false,
        }
    }
    
    /// 设置操作按钮图标
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// 设置操作按钮是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            title: None,
            content: String::new(),
            style: CardStyle::Default,
            collapsible: false,
            default_collapsed: false,
            clickable: false,
            disabled: false,
            icon: None,
            actions: Vec::new(),
            on_click: None,
            on_collapse_change: None,
        }
    }
}

/// 卡片组件构建器
pub struct CardBuilder {
    config: CardConfig,
}

impl CardBuilder {
    /// 创建新的卡片构建器
    pub fn new() -> Self {
        Self {
            config: CardConfig::default(),
        }
    }

    /// 设置卡片标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = Some(title.into());
        self
    }

    /// 设置卡片内容
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.config.content = content.into();
        self
    }

    /// 设置卡片样式
    pub fn style(mut self, style: CardStyle) -> Self {
        self.config.style = style;
        self
    }

    /// 设置是否可折叠
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.config.collapsible = collapsible;
        self
    }

    /// 设置是否默认折叠
    pub fn default_collapsed(mut self, default_collapsed: bool) -> Self {
        self.config.default_collapsed = default_collapsed;
        self
    }

    /// 设置是否可点击
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.config.clickable = clickable;
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置卡片图标
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.config.icon = Some(icon.into());
        self
    }

    /// 添加操作按钮
    pub fn add_action(mut self, action: CardAction) -> Self {
        self.config.actions.push(action);
        self
    }

    /// 设置卡片点击回调
    pub fn on_click(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.config.on_click = Some(Box::new(callback));
        self
    }

    /// 设置折叠状态变化回调
    pub fn on_collapse_change(mut self, callback: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.config.on_collapse_change = Some(Box::new(callback));
        self
    }

    /// 构建卡片组件
    pub fn build(self, ui: &mut Ui, collapsed: &mut bool) -> Response {
        let config = self.config;
        
        // 开始布局
        let response = if config.collapsible {
            // 创建可折叠卡片
            let header_title = config.title.clone().unwrap_or_else(|| "".to_string());
            let collapsing_response = CollapsingHeader::new(header_title)
                .default_open(!config.default_collapsed)
                .show(ui, |ui| {
                    render_card_content(ui, &config);
                });
            
            // 处理折叠状态变化
            if collapsing_response.openness.changed() {
                let new_state = collapsing_response.openness.is_open();
                if let Some(callback) = config.on_collapse_change {
                    callback(new_state);
                }
            }
            
            collapsing_response.header_response
        } else {
            // 创建普通卡片
            let group_response = ui.group(|ui| {
                render_card_header(ui, &config);
                render_card_content(ui, &config);
            });
            group_response.response
        };
        
        // 处理点击事件
        if response.clicked() && config.clickable && !config.disabled {
            if let Some(callback) = config.on_click {
                callback();
            }
        }
        
        response
    }
}

/// 渲染卡片头部
fn render_card_header(ui: &mut Ui, config: &CardConfig) {
    ui.horizontal(|ui| {
        // 显示图标
        if let Some(icon) = &config.icon {
            ui.label(icon);
        }
        
        // 显示标题
        if let Some(title) = &config.title {
            ui.label(egui::RichText::new(title)
                .color(if config.disabled {
                    ui.visuals().disabled_text_color()
                } else {
                    ui.visuals().text_color()
                })
                .strong());
        }
        
        // 显示操作按钮
        if !config.actions.is_empty() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for action in &config.actions {
                    let button_response = match &action.icon {
                        Some(icon) => {
                            ui.button(format!("{} {}", icon, action.text))
                        },
                        None => {
                            ui.button(&action.text)
                        },
                    };
                    
                    if button_response.clicked() && !action.disabled && !config.disabled {
                        (action.on_click)();
                    }
                }
            });
        }
    });
}

/// 渲染卡片内容
fn render_card_content(ui: &mut Ui, config: &CardConfig) {
    // 设置卡片样式
    match config.style {
        CardStyle::Bordered => {
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
        },
        CardStyle::Compact => {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 2.0);
        },
        CardStyle::Shadow => {
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);
            ui.style_mut().visuals.widgets.inactive.bg_fill = ui.visuals().panel_fill;
            ui.style_mut().visuals.widgets.inactive.fg_stroke = egui::Stroke::new(0.0, egui::Color32::TRANSPARENT);
        },
        CardStyle::Rounded => {
            ui.style_mut().visuals.widgets.inactive.rounding = egui::Rounding::new(8.0);
        },
        _ => {},
    }
    
    // 显示内容
    ui.label(egui::RichText::new(&config.content)
        .color(if config.disabled {
            ui.visuals().disabled_text_color()
        } else {
            ui.visuals().text_color()
        }));
}

/// 卡片组件
pub struct Card {
    config: CardConfig,
}

impl Card {
    /// 创建新的卡片构建器
    pub fn builder() -> CardBuilder {
        CardBuilder::new()
    }
}

impl Widget for Card {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut collapsed = self.config.default_collapsed;
        self.builder()
            .title(self.config.title.unwrap_or_default())
            .content(self.config.content.clone())
            .style(self.config.style)
            .collapsible(self.config.collapsible)
            .default_collapsed(collapsed)
            .clickable(self.config.clickable)
            .disabled(self.config.disabled)
            .icon(self.config.icon.unwrap_or_default())
            .on_click(move || println!("Card clicked"))
            .on_collapse_change(move |new_collapsed| collapsed = new_collapsed)
            .build(ui, &mut collapsed)
    }
}
