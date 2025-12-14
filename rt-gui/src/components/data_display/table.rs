use eframe::egui::{Response, Ui, Widget, ScrollArea, Label, ComboBox, Stroke, vec2, RichText, Layout, Align}; use std::fmt::Display; 

/// 表格列配置
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// 列标题
    pub title: String,
    /// 列宽度
    pub width: Option<f32>,
    /// 是否可排序
    pub sortable: bool,
    /// 是否隐藏
    pub hidden: bool,
    /// 列对齐方式
    pub alignment: egui::Align,
}

impl TableColumn {
    /// 创建新的表格列
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: None,
            sortable: false,
            hidden: false,
            alignment: egui::Align::Left,
        }
    }
    
    /// 设置列宽度
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    
    /// 设置是否可排序
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
    
    /// 设置是否隐藏
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
    
    /// 设置列对齐方式
    pub fn alignment(mut self, alignment: egui::Align) -> Self {
        self.alignment = alignment;
        self
    }
}

/// 表格行数据
#[derive(Debug, Clone)]
pub struct TableRow {
    /// 行ID
    pub id: String,
    /// 行数据
    pub data: Vec<String>,
    /// 是否选中
    pub selected: bool,
    /// 是否禁用
    pub disabled: bool,
}

impl TableRow {
    /// 创建新的表格行
    pub fn new(id: impl Into<String>, data: Vec<String>) -> Self {
        Self {
            id: id.into(),
            data,
            selected: false,
            disabled: false,
        }
    }
    
    /// 设置行是否选中
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    
    /// 设置行是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 表格排序配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSortConfig {
    /// 排序列索引
    pub column_index: usize,
    /// 是否升序排序
    pub ascending: bool,
}

/// 表格分页配置
#[derive(Debug, Clone)]
pub struct TablePaginationConfig {
    /// 当前页码
    pub current_page: usize,
    /// 每页行数
    pub page_size: usize,
    /// 总页数
    pub total_pages: usize,
    /// 总记录数
    pub total_records: usize,
    /// 是否显示分页控件
    pub show_pagination: bool,
}

impl Default for TablePaginationConfig {
    fn default() -> Self {
        Self {
            current_page: 1,
            page_size: 10,
            total_pages: 1,
            total_records: 0,
            show_pagination: true,
        }
    }
}

/// 表格样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
    /// 斑马纹样式
    Striped,
}

/// 表格组件配置
#[derive(Debug, Clone)]
pub struct TableConfig {
    /// 表格列
    pub columns: Vec<TableColumn>,
    /// 表格行数据
    pub rows: Vec<TableRow>,
    /// 表格样式
    pub style: TableStyle,
    /// 是否显示表头
    pub show_header: bool,
    /// 是否可选择行
    pub selectable: bool,
    /// 是否显示复选框列
    pub show_checkbox_column: bool,
    /// 是否支持多选
    pub multi_select: bool,
    /// 当前选中的行ID
    pub selected_rows: Vec<String>,
    /// 排序配置
    pub sort_config: Option<TableSortConfig>,
    /// 分页配置
    pub pagination_config: TablePaginationConfig,
    /// 行点击回调
    pub on_row_click: Option<Box<dyn Fn(String) + Send + Sync>>,
    /// 行选择回调
    pub on_row_select: Option<Box<dyn Fn(Vec<String>) + Send + Sync>>,
    /// 列排序回调
    pub on_column_sort: Option<Box<dyn Fn(TableSortConfig) + Send + Sync>>,
    /// 页码变化回调
    pub on_page_change: Option<Box<dyn Fn(usize) + Send + Sync>>,
    /// 页大小变化回调
    pub on_page_size_change: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            style: TableStyle::Default,
            show_header: true,
            selectable: false,
            show_checkbox_column: false,
            multi_select: false,
            selected_rows: Vec::new(),
            sort_config: None,
            pagination_config: TablePaginationConfig::default(),
            on_row_click: None,
            on_row_select: None,
            on_column_sort: None,
            on_page_change: None,
            on_page_size_change: None,
        }
    }
}

/// 表格组件构建器
pub struct TableBuilder {
    config: TableConfig,
}

impl TableBuilder {
    /// 创建新的表格构建器
    pub fn new() -> Self {
        Self {
            config: TableConfig::default(),
        }
    }

    /// 设置表格列
    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.config.columns = columns;
        self
    }

    /// 添加单个列
    pub fn add_column(mut self, column: TableColumn) -> Self {
        self.config.columns.push(column);
        self
    }

    /// 设置表格行数据
    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.config.rows = rows;
        self
    }

    /// 添加单行数据
    pub fn add_row(mut self, row: TableRow) -> Self {
        self.config.rows.push(row);
        self
    }

    /// 设置表格样式
    pub fn style(mut self, style: TableStyle) -> Self {
        self.config.style = style;
        self
    }

    /// 设置是否显示表头
    pub fn show_header(mut self, show_header: bool) -> Self {
        self.config.show_header = show_header;
        self
    }

    /// 设置是否可选择行
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.config.selectable = selectable;
        self
    }

    /// 设置是否显示复选框列
    pub fn show_checkbox_column(mut self, show_checkbox_column: bool) -> Self {
        self.config.show_checkbox_column = show_checkbox_column;
        self
    }

    /// 设置是否支持多选
    pub fn multi_select(mut self, multi_select: bool) -> Self {
        self.config.multi_select = multi_select;
        self
    }

    /// 设置当前选中的行ID
    pub fn selected_rows(mut self, selected_rows: Vec<String>) -> Self {
        self.config.selected_rows = selected_rows;
        self
    }

    /// 设置排序配置
    pub fn sort_config(mut self, sort_config: TableSortConfig) -> Self {
        self.config.sort_config = Some(sort_config);
        self
    }

    /// 设置分页配置
    pub fn pagination_config(mut self, pagination_config: TablePaginationConfig) -> Self {
        self.config.pagination_config = pagination_config;
        self
    }

    /// 设置行点击回调
    pub fn on_row_click(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.config.on_row_click = Some(Box::new(callback));
        self
    }

    /// 设置行选择回调
    pub fn on_row_select(mut self, callback: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.config.on_row_select = Some(Box::new(callback));
        self
    }

    /// 设置列排序回调
    pub fn on_column_sort(mut self, callback: impl Fn(TableSortConfig) + Send + Sync + 'static) -> Self {
        self.config.on_column_sort = Some(Box::new(callback));
        self
    }

    /// 设置页码变化回调
    pub fn on_page_change(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.config.on_page_change = Some(Box::new(callback));
        self
    }

    /// 设置页大小变化回调
    pub fn on_page_size_change(mut self, callback: impl Fn(usize) + Send + Sync + 'static) -> Self {
        self.config.on_page_size_change = Some(Box::new(callback));
        self
    }

    /// 构建表格组件
    pub fn build(self, ui: &mut Ui, selected_rows: &mut Vec<String>) -> Response {
        let config = self.config;
        
        // 过滤显示的列
        let visible_columns: Vec<&TableColumn> = config.columns.iter()
            .filter(|col| !col.hidden)
            .collect();
        
        // 开始布局
        let response = ui.vertical(|ui| {
            // 创建表格容器
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // 设置表格样式
                    match config.style {
                        TableStyle::Bordered => {
                            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                        },
                        TableStyle::Compact => {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 2.0);
                        },
                        _ => {},
                    }
                    
                    // 显示表头
                    if config.show_header {
                        ui.horizontal(|ui| {
                            // 显示复选框列
                            if config.show_checkbox_column && config.selectable {
                                ui.checkbox(&mut false, "");
                            }
                            
                            // 显示列标题
                            for (col_index, column) in visible_columns.iter().enumerate() {
                                let mut header_response = ui.with_layout(egui::Layout::centered_and_justified(column.alignment), |ui| {
                                    let header_text = match &config.sort_config {
                                        Some(sort_config) if sort_config.column_index == col_index => {
                                            let sort_indicator = if sort_config.ascending { " ↑" } else { " ↓" };
                                            format!("{}{}", column.title, sort_indicator)
                                        },
                                        _ => column.title.clone(),
                                    };
                                    ui.label(header_text)
                                }).response;
                                
                                // 设置列宽度
                                if let Some(width) = column.width {
                                    header_response = ui.add_sized([width, 0.0], egui::Label::new(column.title.clone()));
                                }
                                
                                // 处理列排序
                                if column.sortable && header_response.clicked() {
                                    let new_sort_config = TableSortConfig {
                                        column_index: col_index,
                                        ascending: match &config.sort_config {
                                            Some(sort_config) if sort_config.column_index == col_index => {
                                                !sort_config.ascending
                                            },
                                            _ => true,
                                        },
                                    };
                                    if let Some(callback) = &config.on_column_sort {
                                        callback(new_sort_config);
                                    }
                                }
                            }
                        });
                        
                        // 添加表头分隔线
                        ui.separator();
                    }
                    
                    // 显示表格行
                    for (row_index, row) in config.rows.iter().enumerate() {
                        // 设置斑马纹样式
                        if config.style == TableStyle::Striped && row_index % 2 == 1 {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = ui.visuals().extreme_bg_color;
                        }
                        
                        let row_response = ui.horizontal(|ui| {
                            // 显示行复选框
                            if config.show_checkbox_column && config.selectable {
                                let mut is_selected = selected_rows.contains(&row.id);
                                let checkbox_response = ui.checkbox(&mut is_selected, "");
                                
                                if checkbox_response.changed() {
                                    if is_selected {
                                        if !config.multi_select {
                                            selected_rows.clear();
                                        }
                                        selected_rows.push(row.id.clone());
                                    } else {
                                        selected_rows.retain(|id| id != &row.id);
                                    }
                                    
                                    if let Some(callback) = &config.on_row_select {
                                        callback(selected_rows.clone());
                                    }
                                }
                            }
                            
                            // 显示行数据
                            for (col_index, cell_value) in row.data.iter().enumerate() {
                                // 跳过隐藏列
                                if config.columns[col_index].hidden {
                                    continue;
                                }
                                
                                let mut cell_response = ui.with_layout(egui::Layout::centered_and_justified(visible_columns[col_index].alignment), |ui| {
                                    ui.label(cell_value.clone())
                                }).response;
                                
                                // 设置列宽度
                                if let Some(width) = visible_columns[col_index].width {
                                    cell_response = ui.add_sized([width, 0.0], egui::Label::new(cell_value.clone()));
                                }
                            }
                        });
                        
                        // 处理行点击事件
                        if config.selectable && row_response.clicked() && !row.disabled {
                            if !config.multi_select {
                                selected_rows.clear();
                                selected_rows.push(row.id.clone());
                            } else if !selected_rows.contains(&row.id) {
                                selected_rows.push(row.id.clone());
                            }
                            
                            if let Some(callback) = &config.on_row_click {
                                callback(row.id.clone());
                            }
                            if let Some(callback) = &config.on_row_select {
                                callback(selected_rows.clone());
                            }
                        }
                    }
                });
            
            // 显示分页控件
            if config.pagination_config.show_pagination {
                ui.separator();
                ui.horizontal(|ui| {
                    // 跳转到第一页
                    if ui.button("首页").clicked() {
                        if let Some(callback) = &config.on_page_change {
                            callback(1);
                        }
                    }
                    
                    // 上一页
                    if ui.button("上一页").clicked() && config.pagination_config.current_page > 1 {
                        if let Some(callback) = &config.on_page_change {
                            callback(config.pagination_config.current_page - 1);
                        }
                    }
                    
                    // 显示页码信息
                    ui.label(format!("第 {} / {} 页，共 {} 条记录", 
                        config.pagination_config.current_page,
                        config.pagination_config.total_pages,
                        config.pagination_config.total_records));
                    
                    // 下一页
                    if ui.button("下一页").clicked() && config.pagination_config.current_page < config.pagination_config.total_pages {
                        if let Some(callback) = &config.on_page_change {
                            callback(config.pagination_config.current_page + 1);
                        }
                    }
                    
                    // 跳转到最后一页
                    if ui.button("末页").clicked() {
                        if let Some(callback) = &config.on_page_change {
                            callback(config.pagination_config.total_pages);
                        }
                    }
                    
                    // 页大小选择
                    ui.label("每页条数：");
                    let page_sizes = vec![10, 20, 50, 100];
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{}", config.pagination_config.page_size))
                        .show_ui(ui, |ui| {
                            for &size in &page_sizes {
                                if ui.selectable_label(
                                    config.pagination_config.page_size == size,
                                    format!("{}", size)
                                ).clicked() {
                                    if let Some(callback) = &config.on_page_size_change {
                                        callback(size);
                                    }
                                }
                            }
                        });
                });
            }
        });
        
        response.outer
    }
}

/// 表格组件
pub struct Table {
    config: TableConfig,
}

impl Table {
    /// 创建新的表格构建器
    pub fn builder() -> TableBuilder {
        TableBuilder::new()
    }
}

impl Widget for Table {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut selected_rows = self.config.selected_rows.clone();
        self.builder()
            .columns(self.config.columns.clone())
            .rows(self.config.rows.clone())
            .style(self.config.style)
            .show_header(self.config.show_header)
            .selectable(self.config.selectable)
            .show_checkbox_column(self.config.show_checkbox_column)
            .multi_select(self.config.multi_select)
            .selected_rows(selected_rows.clone())
            .sort_config(self.config.sort_config.unwrap_or_default())
            .pagination_config(self.config.pagination_config.clone())
            .on_row_click(move |row_id| println!("Row clicked: {}", row_id))
            .on_row_select(move |rows| selected_rows = rows)
            .on_column_sort(move |sort_config| println!("Sorting column {} {}", sort_config.column_index, if sort_config.ascending { "asc" } else { "desc" }))
            .on_page_change(move |page| println!("Page changed to {}", page))
            .on_page_size_change(move |size| println!("Page size changed to {}", size))
            .build(ui, &mut selected_rows)
    }
}
