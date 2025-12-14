use eframe::egui::{Ui, Response, Color32, Margin, Frame, TextStyle};use std::fmt::Display;

/// 卡片配置
struct CardConfig {
    /// 标题（可选）
    title: Option<String>,
    /// 标题样式
    title_style: TextStyle,
    /// 标题颜色
    title_color: Color32,
    /// 内边距
    padding: Margin,
    /// 外边距
    margin: Margin,
    /// 背景色
    background_color: Option<Color32>,
    /// 边框颜色
    border_color: Option<Color32>,
    /// 边框宽度
    border_width: f32,
    /// 圆角半径
    corner_radius: f32,
    /// 是否有阴影
    shadow: bool,
    /// 阴影强度
    shadow_intensity: f32,
    /// 可折叠
    collapsible: bool,
    /// 是否展开
    expanded: bool,
    /// 显示关闭按钮
    show_close_button: bool,
    /// 关闭按钮回调（可选）
    on_close: Option<Box<dyn FnMut()>>,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            title: None,
            title_style: egui::TextStyle::Subheading,
            title_color: Color32::BLACK,
            padding: Margin::symmetric(16.0, 16.0),
            margin: Margin::symmetric(8.0, 8.0),
            background_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 8.0,
            shadow: true,
            shadow_intensity: 0.3,
            collapsible: false,
            expanded: true,
            show_close_button: false,
            on_close: None,
        }
    }
}

/// 卡片组件
pub struct Card {
    /// 卡片配置
    config: CardConfig,
    /// 子组件渲染函数
    children: Box<dyn FnMut(&mut Ui)>,
}

impl Card {
    /// 创建新卡片
    pub fn new<F>(children: F) -> Self
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Self {
            config: CardConfig::default(),
            children: Box::new(children),
        }
    }
    
    /// 设置卡片标题
    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.config.title = Some(title.to_string());
        self
    }
    
    /// 设置标题样式
    pub fn title_style(mut self, style: egui::TextStyle) -> Self {
        self.config.title_style = style;
        self
    }
    
    /// 设置标题颜色
    pub fn title_color(mut self, color: Color32) -> Self {
        self.config.title_color = color;
        self
    }
    
    /// 设置内边距
    pub fn padding(mut self, padding: Margin) -> Self {
        self.config.padding = padding;
        self
    }
    
    /// 设置对称内边距
    pub fn padding_sym(mut self, x: f32, y: f32) -> Self {
        self.config.padding = Margin::symmetric(x, y);
        self
    }
    
    /// 设置外边距
    pub fn margin(mut self, margin: Margin) -> Self {
        self.config.margin = margin;
        self
    }
    
    /// 设置对称外边距
    pub fn margin_sym(mut self, x: f32, y: f32) -> Self {
        self.config.margin = Margin::symmetric(x, y);
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: Color32) -> Self {
        self.config.background_color = Some(color);
        self
    }
    
    /// 设置边框
    pub fn border(mut self, color: Color32, width: f32) -> Self {
        self.config.border_color = Some(color);
        self.config.border_width = width;
        self
    }
    
    /// 设置圆角
    pub fn rounded(mut self, radius: f32) -> Self {
        self.config.corner_radius = radius;
        self
    }
    
    /// 添加阴影
    pub fn shadow(mut self, intensity: f32) -> Self {
        self.config.shadow = true;
        self.config.shadow_intensity = intensity;
        self
    }
    
    /// 移除阴影
    pub fn no_shadow(mut self) -> Self {
        self.config.shadow = false;
        self
    }
    
    /// 设置为可折叠
    pub fn collapsible(mut self) -> Self {
        self.config.collapsible = true;
        self
    }
    
    /// 设置是否展开
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.config.expanded = expanded;
        self
    }
    
    /// 显示关闭按钮
    pub fn show_close_button(mut self) -> Self {
        self.config.show_close_button = true;
        self
    }
    
    /// 设置关闭按钮回调
    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.show_close_button = true;
        self.config.on_close = Some(Box::new(callback));
        self
    }
    
    /// 渲染卡片
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let CardConfig {
            title,
            title_style,
            title_color,
            padding,
            margin,
            background_color,
            border_color,
            border_width,
            corner_radius,
            shadow,
            shadow_intensity,
            collapsible,
            mut expanded,
            show_close_button,
            ref mut on_close,
        } = &mut self.config;
        
        // 应用外边距
        ui.add_space(margin.top);
        let response = ui.horizontal(|ui| {
            ui.add_space(margin.left);
            
            // 创建帧配置
            let mut frame = Frame::none();
            
            // 设置背景色
            let bg_color = background_color.unwrap_or(ui.style().visuals.panel_fill);
            frame = frame.fill(bg_color);
            
            // 设置边框
            if let Some(color) = border_color {
                frame = frame.stroke(egui::Stroke::new(*border_width, *color));
            }
            
            // 设置圆角
            frame = frame.rounding(*corner_radius);
            
            // 设置阴影
            if *shadow {
                frame = frame.shadow(egui::epaint::Shadow {
                    offset: egui::Vec2::new(0.0, 2.0),
                    blur: 8.0,
                    color: Color32::from_rgba_premultiplied(0, 0, 0, (*shadow_intensity * 255.0) as u8),
                });
            }
            
            // 渲染卡片
            let card_response = if *collapsible {
                // 可折叠卡片
                let title_str = title.as_deref().unwrap_or("Card");
                ui.collapsing(title_str, *expanded, |ui| {
                    self.render_card_content(ui, &frame, padding, show_close_button, on_close)
                })
            } else {
                // 普通卡片
                frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        // 渲染标题
                        if let Some(title_str) = title {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(title_str)
                                        .text_style(title_style)
                                        .color(*title_color)
                                        .strong()
                                );
                                
                                // 渲染关闭按钮
                                if *show_close_button {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(egui::RichText::new("✕").size(14.0)).clicked() {
                                            if let Some(callback) = on_close.as_mut() {
                                                callback();
                                            }
                                        }
                                    });
                                }
                            });
                            ui.separator();
                        }
                        
                        // 渲染内容
                        ui.scope(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::vec2(padding.left, padding.top);
                            (self.children)(ui);
                        });
                    })
                })
            };
            
            ui.add_space(margin.right);
            
            card_response.response
        });
        
        ui.add_space(margin.bottom);
        
        response.response
    }
    
    /// 渲染卡片内容
    fn render_card_content(
        &mut self,
        ui: &mut Ui,
        frame: &Frame,
        padding: &Margin,
        show_close_button: bool,
        on_close: &mut Option<Box<dyn FnMut()>>
    ) {
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // 渲染内容
                ui.scope(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(padding.left, padding.top);
                    (self.children)(ui);
                });
            })
        });
    }
}

/// 卡片构建器
pub struct CardBuilder {
    /// 卡片配置
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
    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.config.title = Some(title.to_string());
        self
    }
    
    /// 设置标题样式
    pub fn title_style(mut self, style: egui::TextStyle) -> Self {
        self.config.title_style = style;
        self
    }
    
    /// 设置标题颜色
    pub fn title_color(mut self, color: Color32) -> Self {
        self.config.title_color = color;
        self
    }
    
    /// 设置内边距
    pub fn padding(mut self, padding: Margin) -> Self {
        self.config.padding = padding;
        self
    }
    
    /// 设置对称内边距
    pub fn padding_sym(mut self, x: f32, y: f32) -> Self {
        self.config.padding = Margin::symmetric(x, y);
        self
    }
    
    /// 设置外边距
    pub fn margin(mut self, margin: Margin) -> Self {
        self.config.margin = margin;
        self
    }
    
    /// 设置对称外边距
    pub fn margin_sym(mut self, x: f32, y: f32) -> Self {
        self.config.margin = Margin::symmetric(x, y);
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: Color32) -> Self {
        self.config.background_color = Some(color);
        self
    }
    
    /// 设置边框
    pub fn border(mut self, color: Color32, width: f32) -> Self {
        self.config.border_color = Some(color);
        self.config.border_width = width;
        self
    }
    
    /// 设置圆角
    pub fn rounded(mut self, radius: f32) -> Self {
        self.config.corner_radius = radius;
        self
    }
    
    /// 添加阴影
    pub fn shadow(mut self, intensity: f32) -> Self {
        self.config.shadow = true;
        self.config.shadow_intensity = intensity;
        self
    }
    
    /// 移除阴影
    pub fn no_shadow(mut self) -> Self {
        self.config.shadow = false;
        self
    }
    
    /// 设置为可折叠
    pub fn collapsible(mut self) -> Self {
        self.config.collapsible = true;
        self
    }
    
    /// 设置是否展开
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.config.expanded = expanded;
        self
    }
    
    /// 显示关闭按钮
    pub fn show_close_button(mut self) -> Self {
        self.config.show_close_button = true;
        self
    }
    
    /// 设置关闭按钮回调
    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.show_close_button = true;
        self.config.on_close = Some(Box::new(callback));
        self
    }
    
    /// 构建卡片
    pub fn build<F>(self, children: F) -> Card
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Card {
            config: self.config,
            children: Box::new(children),
        }
    }
}
