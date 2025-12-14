use eframe::egui::{Ui, Response, Vec2, Layout, Direction, Align, Align2};use std::fmt::Display;

/// 弹性布局方向
enum FlexDirection {
    /// 水平方向
    Row,
    /// 垂直方向
    Column,
    /// 水平反向
    RowReverse,
    /// 垂直反向
    ColumnReverse,
}

/// 弹性布局对齐方式
enum FlexAlign {
    /// 左对齐
    Left,
    /// 居中对齐
    Center,
    /// 右对齐
    Right,
    /// 上对齐
    Top,
    /// 下对齐
    Bottom,
    /// 空间均匀分布
    SpaceBetween,
    /// 空间环绕分布
    SpaceAround,
    /// 空间平均分布
    SpaceEvenly,
    /// 拉伸对齐
    Stretch,
}

/// 弹性布局配置
struct FlexConfig {
    /// 布局方向
    direction: FlexDirection,
    /// 主轴对齐方式
    justify_content: FlexAlign,
    /// 交叉轴对齐方式
    align_items: FlexAlign,
    /// 子项换行方式
    wrap: bool,
    /// 水平间距
    spacing_x: f32,
    /// 垂直间距
    spacing_y: f32,
    /// 内边距
    padding: egui::Margin,
    /// 外边距
    margin: egui::Margin,
    /// 背景色
    background_color: Option<egui::Color32>,
    /// 宽度（可选，自动计算时为None）
    width: Option<f32>,
    /// 高度（可选，自动计算时为None）
    height: Option<f32>,
}

impl Default for FlexConfig {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            justify_content: FlexAlign::Left,
            align_items: FlexAlign::Top,
            wrap: false,
            spacing_x: 16.0,
            spacing_y: 16.0,
            padding: egui::Margin::symmetric(0.0, 0.0),
            margin: egui::Margin::symmetric(0.0, 0.0),
            background_color: None,
            width: None,
            height: None,
        }
    }
}

/// 弹性布局组件
pub struct Flex {
    /// 弹性布局配置
    config: FlexConfig,
    /// 子组件渲染函数
    children: Box<dyn FnMut(&mut Ui)>,
}

impl Flex {
    /// 创建水平弹性布局
    pub fn row<F>(children: F) -> Self
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Self {
            config: FlexConfig::default(),
            children: Box::new(children),
        }
    }
    
    /// 创建垂直弹性布局
    pub fn column<F>(children: F) -> Self
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Self {
            config: FlexConfig {
                direction: FlexDirection::Column,
                ..Default::default()
            },
            children: Box::new(children),
        }
    }
    
    /// 设置为水平反向布局
    pub fn row_reverse(mut self) -> Self {
        self.config.direction = FlexDirection::RowReverse;
        self
    }
    
    /// 设置为垂直反向布局
    pub fn column_reverse(mut self) -> Self {
        self.config.direction = FlexDirection::ColumnReverse;
        self
    }
    
    /// 设置主轴对齐方式
    pub fn justify(mut self, align: FlexAlign) -> Self {
        self.config.justify_content = align;
        self
    }
    
    /// 设置交叉轴对齐方式
    pub fn align_items(mut self, align: FlexAlign) -> Self {
        self.config.align_items = align;
        self
    }
    
    /// 设置为换行
    pub fn wrap(mut self) -> Self {
        self.config.wrap = true;
        self
    }
    
    /// 设置水平间距
    pub fn spacing_x(mut self, spacing: f32) -> Self {
        self.config.spacing_x = spacing;
        self
    }
    
    /// 设置垂直间距
    pub fn spacing_y(mut self, spacing: f32) -> Self {
        self.config.spacing_y = spacing;
        self
    }
    
    /// 设置内边距
    pub fn padding(mut self, padding: egui::Margin) -> Self {
        self.config.padding = padding;
        self
    }
    
    /// 设置对称内边距
    pub fn padding_sym(mut self, x: f32, y: f32) -> Self {
        self.config.padding = egui::Margin::symmetric(x, y);
        self
    }
    
    /// 设置外边距
    pub fn margin(mut self, margin: egui::Margin) -> Self {
        self.config.margin = margin;
        self
    }
    
    /// 设置对称外边距
    pub fn margin_sym(mut self, x: f32, y: f32) -> Self {
        self.config.margin = egui::Margin::symmetric(x, y);
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: egui::Color32) -> Self {
        self.config.background_color = Some(color);
        self
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self
    }
    
    /// 设置高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = Some(height);
        self
    }
    
    /// 设置为全屏
    pub fn fullscreen(mut self) -> Self {
        self.config.width = None;
        self.config.height = None;
        self
    }
    
    /// 渲染弹性布局
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let FlexConfig {
            direction,
            justify_content,
            align_items,
            wrap,
            spacing_x,
            spacing_y,
            padding,
            margin,
            background_color,
            width,
            height,
        } = &self.config;
        
        // 转换为egui布局方向
        let layout_direction = match direction {
            FlexDirection::Row => Direction::LeftToRight,
            FlexDirection::Column => Direction::TopDown,
            FlexDirection::RowReverse => Direction::RightToLeft,
            FlexDirection::ColumnReverse => Direction::BottomUp,
        };
        
        // 转换为主轴对齐方式
        let justify = match justify_content {
            FlexAlign::Left | FlexAlign::Top => egui::Align::Min,
            FlexAlign::Center => egui::Align::Center,
            FlexAlign::Right | FlexAlign::Bottom => egui::Align::Max,
            FlexAlign::SpaceBetween => egui::Align::Min,
            FlexAlign::SpaceAround => egui::Align::Center,
            FlexAlign::SpaceEvenly => egui::Align::Center,
            FlexAlign::Stretch => egui::Align::Min,
        };
        
        // 转换为交叉轴对齐方式
        let align = match align_items {
            FlexAlign::Left | FlexAlign::Right | FlexAlign::Center => egui::Align::Center,
            FlexAlign::Top | FlexAlign::Bottom => egui::Align::Min,
            FlexAlign::SpaceBetween | FlexAlign::SpaceAround | FlexAlign::SpaceEvenly => egui::Align::Center,
            FlexAlign::Stretch => egui::Align::Center,
        };
        
        // 创建布局
        let mut layout = Layout::from_direction(layout_direction)
            .with_main_align(justify)
            .with_cross_align(align)
            .with_main_wrap(*wrap);
        
        // 设置间距
        if *direction == FlexDirection::Row || *direction == FlexDirection::RowReverse {
            layout = layout.with_main_spacing(*spacing_x);
            layout = layout.with_cross_spacing(*spacing_y);
        } else {
            layout = layout.with_main_spacing(*spacing_y);
            layout = layout.with_cross_spacing(*spacing_x);
        }
        
        // 应用外边距
        ui.add_space(margin.top);
        let response = ui.horizontal(|ui| {
            ui.add_space(margin.left);
            
            // 设置宽度和高度
            let available_rect = if let Some(width) = width {
                if let Some(height) = height {
                    egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(*width, *height))
                } else {
                    egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(*width, ui.available_height()))
                }
            } else if let Some(height) = height {
                egui::Rect::from_min_size(ui.next_widget_position(), egui::vec2(ui.available_width(), *height))
            } else {
                ui.available_rect_before_wrap()
            };
            
            // 渲染背景
            if let Some(color) = background_color {
                ui.painter().rect_filled(available_rect, 0.0, *color);
            }
            
            // 应用内边距
            let inner_rect = available_rect.shrink2(*padding);
            
            // 渲染子组件
            let response = ui.with_layout(layout, |ui| {
                ui.allocate_ui_at_rect(inner_rect, |ui| {
                    (self.children)(ui);
                })
            });
            
            ui.add_space(margin.right);
            
            response.response
        });
        
        ui.add_space(margin.bottom);
        
        response.response
    }
}

/// 弹性布局构建器
pub struct FlexBuilder {
    /// 弹性布局配置
    config: FlexConfig,
}

impl FlexBuilder {
    /// 创建水平弹性布局构建器
    pub fn row() -> Self {
        Self {
            config: FlexConfig::default(),
        }
    }
    
    /// 创建垂直弹性布局构建器
    pub fn column() -> Self {
        Self {
            config: FlexConfig {
                direction: FlexDirection::Column,
                ..Default::default()
            },
        }
    }
    
    /// 设置为水平反向布局
    pub fn row_reverse(mut self) -> Self {
        self.config.direction = FlexDirection::RowReverse;
        self
    }
    
    /// 设置为垂直反向布局
    pub fn column_reverse(mut self) -> Self {
        self.config.direction = FlexDirection::ColumnReverse;
        self
    }
    
    /// 设置主轴对齐方式
    pub fn justify(mut self, align: FlexAlign) -> Self {
        self.config.justify_content = align;
        self
    }
    
    /// 设置交叉轴对齐方式
    pub fn align_items(mut self, align: FlexAlign) -> Self {
        self.config.align_items = align;
        self
    }
    
    /// 设置为换行
    pub fn wrap(mut self) -> Self {
        self.config.wrap = true;
        self
    }
    
    /// 设置水平间距
    pub fn spacing_x(mut self, spacing: f32) -> Self {
        self.config.spacing_x = spacing;
        self
    }
    
    /// 设置垂直间距
    pub fn spacing_y(mut self, spacing: f32) -> Self {
        self.config.spacing_y = spacing;
        self
    }
    
    /// 设置内边距
    pub fn padding(mut self, padding: egui::Margin) -> Self {
        self.config.padding = padding;
        self
    }
    
    /// 设置对称内边距
    pub fn padding_sym(mut self, x: f32, y: f32) -> Self {
        self.config.padding = egui::Margin::symmetric(x, y);
        self
    }
    
    /// 设置外边距
    pub fn margin(mut self, margin: egui::Margin) -> Self {
        self.config.margin = margin;
        self
    }
    
    /// 设置对称外边距
    pub fn margin_sym(mut self, x: f32, y: f32) -> Self {
        self.config.margin = egui::Margin::symmetric(x, y);
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: egui::Color32) -> Self {
        self.config.background_color = Some(color);
        self
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self
    }
    
    /// 设置高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = Some(height);
        self
    }
    
    /// 设置为全屏
    pub fn fullscreen(mut self) -> Self {
        self.config.width = None;
        self.config.height = None;
        self
    }
    
    /// 构建弹性布局
    pub fn build<F>(self, children: F) -> Flex
    where
        F: FnMut(&mut Ui) + 'static,
    {
        Flex {
            config: self.config,
            children: Box::new(children),
        }
    }
}
