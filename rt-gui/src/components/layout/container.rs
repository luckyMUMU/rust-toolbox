use eframe::egui::{Ui, Response, Color32, Margin, Frame};use std::fmt::Display;

/// 容器配置
struct ContainerConfig {
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
    /// 显示标题（可选）
    title: Option<String>,
    /// 可折叠
    collapsible: bool,
    /// 是否展开
    expanded: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            padding: Margin::symmetric(16.0, 16.0),
            margin: Margin::symmetric(0.0, 0.0),
            background_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 4.0,
            shadow: false,
            shadow_intensity: 0.5,
            title: None,
            collapsible: false,
            expanded: true,
        }
    }
}

/// 容器组件
pub struct Container {
    /// 容器配置
    config: ContainerConfig,
    /// 子组件渲染函数
    children: Box<dyn FnMut(&mut Ui)>,
}

impl Container {
    /// 创建新容器
    pub fn new<F>(children: F) -> Self
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Self {
            config: ContainerConfig::default(),
            children: Box::new(children),
        }
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
    
    /// 设置标题
    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.config.title = Some(title.to_string());
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
    
    /// 渲染容器
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let ContainerConfig {
            padding,
            margin,
            background_color,
            border_color,
            border_width,
            corner_radius,
            shadow,
            shadow_intensity,
            title,
            collapsible,
            mut expanded,
        } = &mut self.config;
        
        // 应用外边距
        ui.add_space(margin.top);
        ui.horizontal(|ui| {
            ui.add_space(margin.left);
            
            // 创建帧配置
            let mut frame = Frame::none();
            
            // 设置背景色
            if let Some(color) = background_color {
                frame = frame.fill(*color);
            }
            
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
            
            // 渲染可折叠容器
            if *collapsible {
                match title {
                    Some(title_str) => {
                        ui.collapsing(title_str, *expanded, |ui| {
                            frame.show(ui, |ui| {
                                ui.scope(|ui| {
                                    ui.style_mut().spacing.item_spacing = egui::vec2(padding.left, padding.top);
                                    (self.children)(ui);
                                });
                            });
                        });
                    },
                    None => {
                        ui.collapsing("Container", *expanded, |ui| {
                            frame.show(ui, |ui| {
                                ui.scope(|ui| {
                                    ui.style_mut().spacing.item_spacing = egui::vec2(padding.left, padding.top);
                                    (self.children)(ui);
                                });
                            });
                        });
                    },
                }
            } else {
                // 渲染普通容器
                let response = frame.show(ui, |ui| {
                    ui.scope(|ui| {
                        ui.style_mut().spacing.item_spacing = egui::vec2(padding.left, padding.top);
                        (self.children)(ui);
                    });
                });
                
                ui.add_space(margin.right);
                return response.response;
            }
            
            ui.add_space(margin.right);
        });
        ui.add_space(margin.bottom);
        
        ui.interact(ui.max_rect(), egui::Id::new("container"), egui::Sense::hover())
    }
}

/// 容器构建器
pub struct ContainerBuilder {
    /// 容器配置
    config: ContainerConfig,
}

impl ContainerBuilder {
    /// 创建新容器构建器
    pub fn new() -> Self {
        Self {
            config: ContainerConfig::default(),
        }
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
    
    /// 设置标题
    pub fn title<T: Display>(mut self, title: T) -> Self {
        self.config.title = Some(title.to_string());
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
    
    /// 构建容器
    pub fn build<F>(self, children: F) -> Container
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Container {
            config: self.config,
            children: Box::new(children),
        }
    }
}
