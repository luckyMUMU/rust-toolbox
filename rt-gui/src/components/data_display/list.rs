use eframe::egui::{Response, Ui, Widget, ScrollArea, Stroke, vec2, RichText, Layout, Align, Checkbox}; use std::fmt::Display;

/// 列表项配置
#[derive(Debug, Clone)]
pub struct ListItem {
    /// 列表项ID
    pub id: String,
    /// 列表项标题
    pub title: String,
    /// 列表项描述
    pub description: Option<String>,
    /// 列表项图标
    pub icon: Option<String>,
    /// 列表项右侧内容
    pub right_content: Option<String>,
    /// 是否选中
    pub selected: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 列表项元数据
    pub metadata: Option<Vec<(String, String)>>,
}

impl ListItem {
    /// 创建新的列表项
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            icon: None,
            right_content: None,
            selected: false,
            disabled: false,
            metadata: None,
        }
    }
    
    /// 设置列表项描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// 设置列表项图标
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// 设置列表项右侧内容
    pub fn right_content(mut self, right_content: impl Into<String>) -> Self {
        self.right_content = Some(right_content.into());
        self
    }
    
    /// 设置列表项是否选中
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    
    /// 设置列表项是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    
    /// 设置列表项元数据
    pub fn metadata(mut self, metadata: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        self.metadata = Some(metadata.into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect());
        self
    }
}

/// 列表样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    /// 默认样式
    Default,
    /// 紧凑样式
    Compact,
    /// 带边框的样式
    Bordered,
    /// 卡片式样式
    Card,
    /// 带图标的样式
    Icon,
}

/// 列表组件配置
#[derive(Debug)]
pub struct ListConfig {
    /// 列表项
    pub items: Vec<ListItem>,
    /// 列表样式
    pub style: ListStyle,
    /// 是否可选择
    pub selectable: bool,
    /// 是否支持多选
    pub multi_select: bool,
    /// 当前选中的列表项ID
    pub selected_items: Vec<String>,
    /// 是否显示搜索框
    pub show_search: bool,
    /// 搜索关键词
    pub search_query: String,
    /// 列表项点击回调
    pub on_item_click: Option<Box<dyn Fn(String) + Send + Sync>>,
    /// 列表项选择回调
    pub on_item_select: Option<Box<dyn Fn(Vec<String>) + Send + Sync>>,
    /// 搜索回调
    pub on_search: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            style: ListStyle::Default,
            selectable: false,
            multi_select: false,
            selected_items: Vec::new(),
            show_search: false,
            search_query: String::new(),
            on_item_click: None,
            on_item_select: None,
            on_search: None,
        }
    }
}

/// 列表组件构建器
pub struct ListBuilder {
    config: ListConfig,
}

impl ListBuilder {
    /// 创建新的列表构建器
    pub fn new() -> Self {
        Self {
            config: ListConfig::default(),
        }
    }

    /// 设置列表项
    pub fn items(mut self, items: Vec<ListItem>) -> Self {
        self.config.items = items;
        self
    }

    /// 添加单个列表项
    pub fn add_item(mut self, item: ListItem) -> Self {
        self.config.items.push(item);
        self
    }

    /// 设置列表样式
    pub fn style(mut self, style: ListStyle) -> Self {
        self.config.style = style;
        self
    }

    /// 设置是否可选择
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.config.selectable = selectable;
        self
    }

    /// 设置是否支持多选
    pub fn multi_select(mut self, multi_select: bool) -> Self {
        self.config.multi_select = multi_select;
        self
    }

    /// 设置当前选中的列表项ID
    pub fn selected_items(mut self, selected_items: Vec<String>) -> Self {
        self.config.selected_items = selected_items;
        self
    }

    /// 设置是否显示搜索框
    pub fn show_search(mut self, show_search: bool) -> Self {
        self.config.show_search = show_search;
        self
    }

    /// 设置搜索关键词
    pub fn search_query(mut self, search_query: impl Into<String>) -> Self {
        self.config.search_query = search_query.into();
        self
    }

    /// 设置列表项点击回调
    pub fn on_item_click(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.config.on_item_click = Some(Box::new(callback));
        self
    }

    /// 设置列表项选择回调
    pub fn on_item_select(mut self, callback: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.config.on_item_select = Some(Box::new(callback));
        self
    }

    /// 设置搜索回调
    pub fn on_search(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.config.on_search = Some(Box::new(callback));
        self
    }

    /// 构建列表组件
    pub fn build(self, ui: &mut Ui, selected_items: &mut Vec<String>) -> Response {
        let mut config = self.config;
        
        // 开始布局
        let response = ui.vertical(|ui| {
            // 显示搜索框
            if config.show_search {
                let mut search_query = config.search_query.clone();
                let search_response = ui.text_edit_singleline(&mut search_query);
                
                if search_response.changed() {
                    config.search_query = search_query.clone();
                    if let Some(callback) = config.on_search {
                        callback(search_query);
                    }
                }
                
                ui.separator();
            }
            
            // 过滤列表项
            let filtered_items: Vec<&ListItem> = config.items.iter()
                .filter(|item| {
                    if config.search_query.is_empty() {
                        true
                    } else {
                        item.title.contains(&config.search_query) || 
                        item.description.as_ref().map(|desc| desc.contains(&config.search_query)).unwrap_or(false)
                    }
                })
                .collect();
            
            // 创建滚动区域
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // 设置列表样式
                    match config.style {
                        ListStyle::Bordered => {
                            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                        },
                        ListStyle::Compact => {
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 2.0);
                        },
                        _ => {},
                    }
                    
                    // 显示列表项
                    for item in filtered_items {
                        let mut is_selected = selected_items.contains(&item.id);
                        
                        // 创建列表项容器
                        let item_response = match config.style {
                            ListStyle::Card => {
                                ui.group(|ui| {
                                    render_list_item(ui, item, config.style, config.selectable, &mut is_selected)
                                }).response
                            },
                            _ => {
                                render_list_item(ui, item, config.style, config.selectable, &mut is_selected)
                            },
                        };
                        
                        // 处理选择状态变化
                        if is_selected != selected_items.contains(&item.id) {
                            if is_selected {
                                if !config.multi_select {
                                    selected_items.clear();
                                }
                                selected_items.push(item.id.clone());
                            } else {
                                selected_items.retain(|id| id != &item.id);
                            }
                            
                            if let Some(callback) = &config.on_item_select {
                                callback(selected_items.clone());
                            }
                        }
                        
                        // 处理点击事件
                        if item_response.clicked() && !item.disabled {
                            if config.selectable {
                                if !config.multi_select {
                                    selected_items.clear();
                                    selected_items.push(item.id.clone());
                                } else if !selected_items.contains(&item.id) {
                                    selected_items.push(item.id.clone());
                                } else {
                                    selected_items.retain(|id| id != &item.id);
                                }
                                
                                if let Some(callback) = &config.on_item_select {
                                    callback(selected_items.clone());
                                }
                            }
                            
                            if let Some(callback) = &config.on_item_click {
                                callback(item.id.clone());
                            }
                        }
                        
                        // 添加分隔线
                        if config.style != ListStyle::Card {
                            ui.separator();
                        }
                    }
                });
        });
        
        response.response
    }
}

/// 渲染单个列表项
fn render_list_item(ui: &mut Ui, item: &ListItem, style: ListStyle, selectable: bool, is_selected: &mut bool) -> Response {
    let response = ui.horizontal(|ui| {
        // 显示选择框
        if selectable {
            ui.checkbox(is_selected, "");
        }
        
        // 显示图标
        if let Some(icon) = &item.icon {
            match style {
                ListStyle::Icon => {
                    ui.label(icon);
                },
                _ => {
                    ui.label(icon);
                },
            }
        }
        
        // 显示主要内容
        let content_response = ui.vertical(|ui| {
            // 显示标题
            let text_color = if item.disabled {
                // 使用灰色作为禁用颜色
                egui::Color32::GRAY
            } else {
                ui.visuals().text_color()
            };
            
            ui.label(egui::RichText::new(&item.title)
                .color(text_color)
                .strong());
            
            // 显示描述
            if let Some(description) = &item.description {
                let desc_color = if item.disabled {
                    // 使用浅灰色作为禁用描述颜色
                    egui::Color32::LIGHT_GRAY
                } else {
                    ui.visuals().weak_text_color()
                };
                
                ui.label(egui::RichText::new(description)
                    .color(desc_color)
                    .small());
            }
            
            // 显示元数据
            if let Some(metadata) = &item.metadata {
                ui.horizontal(|ui| {
                    for (key, value) in metadata {
                        let meta_color = if item.disabled {
                            // 使用浅灰色作为禁用元数据颜色
                            egui::Color32::LIGHT_GRAY
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        
                        ui.label(egui::RichText::new(format!("{}: {}", key, value))
                            .color(meta_color)
                            .small());
                        ui.add_space(8.0);
                    }
                });
            }
        }).response;
        
        // 显示右侧内容
        if let Some(right_content) = &item.right_content {
            let right_color = if item.disabled {
                // 使用灰色作为禁用右侧内容颜色
                egui::Color32::GRAY
            } else {
                ui.visuals().text_color()
            };
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(right_content)
                    .color(right_color));
            });
        }
        
        content_response
    });
    
    response.response
}

/// 列表组件
pub struct List {
    config: ListConfig,
}

impl List {
    /// 创建新的列表构建器
    pub fn builder() -> ListBuilder {
        ListBuilder::new()
    }
}

impl Widget for List {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut selected_items = self.config.selected_items.clone();
        List::builder()
            .items(self.config.items.clone())
            .style(self.config.style)
            .selectable(self.config.selectable)
            .multi_select(self.config.multi_select)
            .selected_items(selected_items.clone())
            .show_search(self.config.show_search)
            .search_query(self.config.search_query.clone())
            .on_item_click(move |item_id| println!("Item clicked: {}", item_id))
            .on_item_select(move |items| selected_items = items)
            .on_search(move |query| println!("Search: {}", query))
            .build(ui, &mut selected_items)
    }
}
