use eframe::egui::{Ui, Response, Vec2};use std::fmt::Display;

/// 网格配置
struct GridConfig {
    /// 列数
    columns: usize,
    /// 列宽（可选，自动计算时为None）
    column_widths: Option<Vec<f32>>,
    /// 行高（可选，自动计算时为None）
    row_heights: Option<Vec<f32>>,
    /// 水平间距
    spacing_x: f32,
    /// 垂直间距
    spacing_y: f32,
    /// 对齐方式
    alignment: egui::Align,
    /// 垂直对齐方式
    vertical_alignment: egui::Align,
    /// 响应式
    responsive: bool,
    /// 响应式断点（可选）
    breakpoints: Option<Vec<f32>>,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            columns: 2,
            column_widths: None,
            row_heights: None,
            spacing_x: 16.0,
            spacing_y: 16.0,
            alignment: egui::Align::LEFT,
            vertical_alignment: egui::Align::TOP,
            responsive: false,
            breakpoints: None,
        }
    }
}

/// 网格项配置
struct GridItemConfig {
    /// 列索引
    column: usize,
    /// 行索引
    row: usize,
    /// 列跨度
    column_span: usize,
    /// 行跨度
    row_span: usize,
    /// 水平对齐方式
    alignment: egui::Align,
    /// 垂直对齐方式
    vertical_alignment: egui::Align,
}

impl Default for GridItemConfig {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            column_span: 1,
            row_span: 1,
            alignment: egui::Align::LEFT,
            vertical_alignment: egui::Align::TOP,
        }
    }
}

/// 网格项
struct GridItem {
    /// 配置
    config: GridItemConfig,
    /// 渲染函数
    render_fn: Box<dyn FnMut(&mut Ui)>,
}

/// 网格组件
pub struct Grid {
    /// 网格配置
    config: GridConfig,
    /// 网格项列表
    items: Vec<GridItem>,
    /// 当前行索引（用于流式布局）
    current_row: usize,
    /// 当前列索引（用于流式布局）
    current_col: usize,
}

impl Grid {
    /// 创建新网格
    pub fn new(columns: usize) -> Self {
        Self {
            config: GridConfig {
                columns,
                ..Default::default()
            },
            items: Vec::new(),
            current_row: 0,
            current_col: 0,
        }
    }
    
    /// 设置列宽
    pub fn column_widths(mut self, widths: &[f32]) -> Self {
        self.config.column_widths = Some(widths.to_vec());
        self
    }
    
    /// 设置行高
    pub fn row_heights(mut self, heights: &[f32]) -> Self {
        self.config.row_heights = Some(heights.to_vec());
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
    
    /// 设置对齐方式
    pub fn align(mut self, alignment: egui::Align) -> Self {
        self.config.alignment = alignment;
        self
    }
    
    /// 设置垂直对齐方式
    pub fn vertical_align(mut self, alignment: egui::Align) -> Self {
        self.config.vertical_alignment = alignment;
        self
    }
    
    /// 设置为响应式
    pub fn responsive(mut self) -> Self {
        self.config.responsive = true;
        self
    }
    
    /// 设置响应式断点
    pub fn breakpoints(mut self, breakpoints: &[f32]) -> Self {
        self.config.breakpoints = Some(breakpoints.to_vec());
        self.responsive()
    }
    
    /// 添加网格项
    pub fn item<F>(&mut self, render_fn: F)
    where
        F: FnMut(&mut Ui) + 'static,
    {
        // 流式布局：自动计算行列索引
        let item_config = GridItemConfig {
            column: self.current_col,
            row: self.current_row,
            ..Default::default()
        };
        
        self.items.push(GridItem {
            config: item_config,
            render_fn: Box::new(render_fn),
        });
        
        // 更新当前行列索引
        self.current_col += 1;
        if self.current_col >= self.config.columns {
            self.current_col = 0;
            self.current_row += 1;
        }
    }
    
    /// 添加指定位置的网格项
    pub fn item_at<F>(&mut self, column: usize, row: usize, render_fn: F)
    where
        F: FnMut(&mut Ui) + 'static,
    {
        let item_config = GridItemConfig {
            column,
            row,
            ..Default::default()
        };
        
        self.items.push(GridItem {
            config: item_config,
            render_fn: Box::new(render_fn),
        });
    }
    
    /// 添加跨列网格项
    pub fn item_span<F>(&mut self, column_span: usize, render_fn: F)
    where
        F: FnMut(&mut Ui) + 'static,
    {
        let item_config = GridItemConfig {
            column: self.current_col,
            row: self.current_row,
            column_span,
            ..Default::default()
        };
        
        self.items.push(GridItem {
            config: item_config,
            render_fn: Box::new(render_fn),
        });
        
        // 更新当前行列索引
        self.current_col += column_span;
        if self.current_col >= self.config.columns {
            self.current_col = 0;
            self.current_row += 1;
        }
    }
    
    /// 渲染网格
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let GridConfig {
            columns,
            column_widths,
            row_heights,
            spacing_x,
            spacing_y,
            alignment,
            vertical_alignment,
            responsive,
            breakpoints,
        } = &self.config;
        
        // 响应式处理
        let mut current_columns = *columns;
        if *responsive {
            let available_width = ui.available_width();
            if let Some(breakpoints) = breakpoints {
                // 根据可用宽度调整列数
                for (i, &breakpoint) in breakpoints.iter().enumerate() {
                    if available_width < breakpoint {
                        current_columns = i + 1;
                        break;
                    }
                }
            } else {
                // 简单响应式：根据宽度自动调整列数
                if available_width < 600.0 {
                    current_columns = 1;
                } else if available_width < 1200.0 {
                    current_columns = 2;
                }
            }
        }
        
        // 创建网格布局
        let mut grid = egui::Grid::new(egui::Id::new("grid"))
            .min_col_width(0.0)
            .spacing(egui::vec2(*spacing_x, *spacing_y))
            .num_columns(current_columns)
            .spacing(egui::vec2(*spacing_x, *spacing_y));
        
        // 设置对齐方式
        match *alignment {
            egui::Align::LEFT => grid = grid.align_left(),
            egui::Align::CENTER => grid = grid.align_center(),
            egui::Align::RIGHT => grid = grid.align_right(),
        }
        
        // 渲染网格
        let response = grid.show(ui, |ui| {
            // 按行渲染
            let mut max_row = self.current_row;
            for item in &mut self.items {
                if item.config.row > max_row {
                    max_row = item.config.row;
                }
            }
            
            for row in 0..=max_row {
                for col in 0..current_columns {
                    // 查找当前单元格的项
                    if let Some(item) = self.items.iter_mut().find(|item| {
                        item.config.row == row && item.config.column == col
                    }) {
                        // 设置列跨度
                        if item.config.column_span > 1 {
                            ui.set_col_span(item.config.column_span);
                        }
                        
                        // 渲染项
                        (item.render_fn)(ui);
                        
                        // 重置列跨度
                        if item.config.column_span > 1 {
                            ui.set_col_span(1);
                            // 跳过已跨越的列
                            for _ in 1..item.config.column_span {
                                ui.end_row();
                            }
                        }
                    } else {
                        // 空单元格
                        ui.label("");
                    }
                }
                // 结束行
                ui.end_row();
            }
        });
        
        response.response
    }
    
    /// 清除所有网格项
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_row = 0;
        self.current_col = 0;
    }
}

/// 网格构建器
pub struct GridBuilder {
    /// 网格配置
    config: GridConfig,
}

impl GridBuilder {
    /// 创建新的网格构建器
    pub fn new(columns: usize) -> Self {
        Self {
            config: GridConfig {
                columns,
                ..Default::default()
            },
        }
    }
    
    /// 设置列宽
    pub fn column_widths(mut self, widths: &[f32]) -> Self {
        self.config.column_widths = Some(widths.to_vec());
        self
    }
    
    /// 设置行高
    pub fn row_heights(mut self, heights: &[f32]) -> Self {
        self.config.row_heights = Some(heights.to_vec());
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
    
    /// 设置对齐方式
    pub fn align(mut self, alignment: egui::Align) -> Self {
        self.config.alignment = alignment;
        self
    }
    
    /// 设置垂直对齐方式
    pub fn vertical_align(mut self, alignment: egui::Align) -> Self {
        self.config.vertical_alignment = alignment;
        self
    }
    
    /// 设置为响应式
    pub fn responsive(mut self) -> Self {
        self.config.responsive = true;
        self
    }
    
    /// 设置响应式断点
    pub fn breakpoints(mut self, breakpoints: &[f32]) -> Self {
        self.config.breakpoints = Some(breakpoints.to_vec());
        self.responsive()
    }
    
    /// 构建网格
    pub fn build(self) -> Grid {
        Grid {
            config: self.config,
            items: Vec::new(),
            current_row: 0,
            current_col: 0,
        }
    }
}
