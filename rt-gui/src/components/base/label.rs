use eframe::egui::{Ui, Response, Label as EguiLabel, RichText, Color32, TextStyle};
use std::fmt::Display;

/// 标签类型
enum LabelType {
    /// 标题标签 - 用于页面或区域标题
    Title,
    /// 副标题标签 - 用于子标题
    Subtitle,
    /// 正文标签 - 用于普通文本
    Body,
    /// 辅助文本标签 - 用于说明或提示文本
    Helper,
    /// 链接标签 - 用于可点击的链接文本
    Link,
}

/// 标签配置
struct LabelConfig {
    /// 标签类型
    label_type: LabelType,
    /// 文本颜色（可选）
    color: Option<Color32>,
    /// 是否加粗
    bold: bool,
    /// 是否斜体
    italic: bool,
    /// 是否带下划线
    underline: bool,
    /// 文本对齐方式
    alignment: egui::Align,
    /// 点击回调（仅用于链接标签）
    on_click: Option<Box<dyn FnMut()>>,
}

impl Default for LabelConfig {
    fn default() -> Self {
        Self {
            label_type: LabelType::Body,
            color: None,
            bold: false,
            italic: false,
            underline: false,
            alignment: egui::Align::LEFT,
            on_click: None,
        }
    }
}

/// 标签组件
pub struct Label {
    /// 标签文本
    text: String,
    /// 标签配置
    config: LabelConfig,
}

impl Label {
    /// 创建新标签
    pub fn new<T: Display>(text: T) -> Self {
        Self {
            text: text.to_string(),
            config: LabelConfig::default(),
        }
    }
    
    /// 设置为标题标签
    pub fn title(mut self) -> Self {
        self.config.label_type = LabelType::Title;
        self
    }
    
    /// 设置为副标题标签
    pub fn subtitle(mut self) -> Self {
        self.config.label_type = LabelType::Subtitle;
        self
    }
    
    /// 设置为正文标签
    pub fn body(mut self) -> Self {
        self.config.label_type = LabelType::Body;
        self
    }
    
    /// 设置为辅助文本标签
    pub fn helper(mut self) -> Self {
        self.config.label_type = LabelType::Helper;
        self
    }
    
    /// 设置为链接标签
    pub fn link(mut self) -> Self {
        self.config.label_type = LabelType::Link;
        self.config.underline = true;
        self
    }
    
    /// 设置文本颜色
    pub fn color(mut self, color: Color32) -> Self {
        self.config.color = Some(color);
        self
    }
    
    /// 设置为加粗文本
    pub fn bold(mut self) -> Self {
        self.config.bold = true;
        self
    }
    
    /// 设置为斜体文本
    pub fn italic(mut self) -> Self {
        self.config.italic = true;
        self
    }
    
    /// 设置带下划线
    pub fn underline(mut self) -> Self {
        self.config.underline = true;
        self
    }
    
    /// 设置文本对齐方式
    pub fn align(mut self, alignment: egui::Align) -> Self {
        self.config.alignment = alignment;
        self
    }
    
    /// 设置点击回调（仅用于链接标签）
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 渲染标签
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let LabelConfig {
            label_type,
            color,
            bold,
            italic,
            underline,
            alignment,
            ref mut on_click,
        } = &mut self.config;
        
        // 根据标签类型获取文本样式
        let text_style = match label_type {
            LabelType::Title => TextStyle::Heading,
            LabelType::Subtitle => TextStyle::Subheading,
            LabelType::Body => TextStyle::Body,
            LabelType::Helper => TextStyle::Small,
            LabelType::Link => TextStyle::Body,
        };
        
        // 根据标签类型获取默认颜色
        let default_color = match label_type {
            LabelType::Title => Color32::BLACK,
            LabelType::Subtitle => Color32::BLACK,
            LabelType::Body => Color32::BLACK,
            LabelType::Helper => Color32::GRAY,
            LabelType::Link => Color32::from_rgb(74, 144, 226),
        };
        
        // 构建富文本
        let mut rich_text = RichText::new(&self.text)
            .text_style(text_style)
            .color(color.unwrap_or(default_color));
        
        // 应用样式
        if *bold {
            rich_text = rich_text.strong();
        }
        if *italic {
            rich_text = rich_text.italics();
        }
        if *underline {
            rich_text = rich_text.underline();
        }
        
        // 根据对齐方式渲染
        match *alignment {
            egui::Align::LEFT => {
                self.render_label(ui, rich_text, on_click.as_mut())
            },
            egui::Align::Center => {
                ui.horizontal_centered(|ui| {
                    self.render_label(ui, rich_text, on_click.as_mut())
                })
            },
            egui::Align::RIGHT => {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_label(ui, rich_text, on_click.as_mut())
                })
            },
        }
    }
    
    /// 渲染标签核心逻辑
    fn render_label(&mut self, ui: &mut Ui, rich_text: RichText, on_click: Option<&mut Box<dyn FnMut()>>) -> Response {
        match on_click {
            Some(_) => {
                // 可点击标签（链接）
                let response = ui.link(rich_text);
                if response.clicked() {
                    if let Some(callback) = on_click {
                        callback();
                    }
                }
                response
            },
            None => {
                // 普通标签
                ui.add(EguiLabel::new(rich_text))
            },
        }
    }
}

/// 标签构建器
pub struct LabelBuilder {
    text: String,
    config: LabelConfig,
}

impl LabelBuilder {
    /// 创建新的标签构建器
    pub fn new<T: Display>(text: T) -> Self {
        Self {
            text: text.to_string(),
            config: LabelConfig::default(),
        }
    }
    
    /// 设置为标题标签
    pub fn title(mut self) -> Self {
        self.config.label_type = LabelType::Title;
        self
    }
    
    /// 设置为副标题标签
    pub fn subtitle(mut self) -> Self {
        self.config.label_type = LabelType::Subtitle;
        self
    }
    
    /// 设置为正文标签
    pub fn body(mut self) -> Self {
        self.config.label_type = LabelType::Body;
        self
    }
    
    /// 设置为辅助文本标签
    pub fn helper(mut self) -> Self {
        self.config.label_type = LabelType::Helper;
        self
    }
    
    /// 设置为链接标签
    pub fn link(mut self) -> Self {
        self.config.label_type = LabelType::Link;
        self.config.underline = true;
        self
    }
    
    /// 设置文本颜色
    pub fn color(mut self, color: Color32) -> Self {
        self.config.color = Some(color);
        self
    }
    
    /// 设置为加粗文本
    pub fn bold(mut self) -> Self {
        self.config.bold = true;
        self
    }
    
    /// 设置为斜体文本
    pub fn italic(mut self) -> Self {
        self.config.italic = true;
        self
    }
    
    /// 设置带下划线
    pub fn underline(mut self) -> Self {
        self.config.underline = true;
        self
    }
    
    /// 设置文本对齐方式
    pub fn align(mut self, alignment: egui::Align) -> Self {
        self.config.alignment = alignment;
        self
    }
    
    /// 设置点击回调（仅用于链接标签）
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 构建标签
    pub fn build(self) -> Label {
        Label {
            text: self.text,
            config: self.config,
        }
    }
}
