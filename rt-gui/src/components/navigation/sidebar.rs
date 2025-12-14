use egui::{Ui, Response, Color32, RichText, Id, Layout, Direction, Align, ScrollArea};use std::fmt::Display;

/// 菜单项类型
enum MenuItemType {
    /// 菜单项
    Item,
    /// 子菜单
    SubMenu,
    /// 分隔线
    Separator,
}

/// 菜单项配置
struct MenuItemConfig {
    /// 菜单项ID
    id: String,
    /// 菜单项类型
    item_type: MenuItemType,
    /// 菜单项标题
    title: String,
    /// 菜单项图标（可选）
    icon: Option<&'static str>,
    /// 是否激活
    active: bool,
    /// 是否展开（仅用于子菜单）
    expanded: bool,
    /// 子菜单项
    children: Vec<MenuItemConfig>,
    /// 点击回调
    on_click: Option<Box<dyn FnMut()>>,
    /// 显示数量（可选）
    badge_count: Option<usize>,
    /// 是否可见
    visible: bool,
}

impl Default for MenuItemConfig {
    fn default() -> Self {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            Self {
                id: format!("menu-item-{}", COUNTER),
                item_type: MenuItemType::Item,
                title: "New Menu Item".to_string(),
                icon: None,
                active: false,
                expanded: false,
                children: Vec::new(),
                on_click: None,
                badge_count: None,
                visible: true,
            }
        }
    }
}

/// 侧边栏配置
struct SidebarConfig {
    /// 菜单项列表
    menu_items: Vec<MenuItemConfig>,
    /// 宽度
    width: f32,
    /// 背景色
    background_color: Color32,
    /// 边框颜色
    border_color: Color32,
    /// 边框宽度
    border_width: f32,
    /// 显示搜索框
    show_search: bool,
    /// 搜索回调
    on_search: Option<Box<dyn FnMut(&str)>>,
    /// 搜索文本
    search_text: String,
    /// 激活菜单项ID
    active_item_id: Option<String>,
    /// 菜单项高度
    item_height: f32,
    /// 菜单项间距
    item_spacing: f32,
    /// 激活菜单项样式
    active_item_style: MenuItemStyle,
    /// 普通菜单项样式
    normal_item_style: MenuItemStyle,
    /// 悬停菜单项样式
    hover_item_style: MenuItemStyle,
    /// 显示分隔线
    show_separator: bool,
    /// 分隔线颜色
    separator_color: Color32,
    /// 分隔线高度
    separator_height: f32,
    /// 可折叠
    collapsible: bool,
    /// 是否折叠
    collapsed: bool,
    /// 折叠按钮图标
    collapse_icon: &'static str,
    /// 展开按钮图标
    expand_icon: &'static str,
}

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            menu_items: Vec::new(),
            width: 240.0,
            background_color: Color32::from_rgb(245, 245, 245),
            border_color: Color32::from_rgb(200, 200, 200),
            border_width: 1.0,
            show_search: true,
            on_search: None,
            search_text: String::new(),
            active_item_id: None,
            item_height: 32.0,
            item_spacing: 4.0,
            active_item_style: MenuItemStyle::default_active(),
            normal_item_style: MenuItemStyle::default_normal(),
            hover_item_style: MenuItemStyle::default_hover(),
            show_separator: true,
            separator_color: Color32::from_rgb(200, 200, 200),
            separator_height: 1.0,
            collapsible: true,
            collapsed: false,
            collapse_icon: "▶️",
            expand_icon: "▼",
        }
    }
}

/// 菜单项样式
struct MenuItemStyle {
    /// 背景色
    background_color: Color32,
    /// 文本颜色
    text_color: Color32,
    /// 边框颜色
    border_color: Color32,
    /// 边框宽度
    border_width: f32,
    /// 左侧边框颜色
    left_border_color: Color32,
    /// 左侧边框宽度
    left_border_width: f32,
}

impl MenuItemStyle {
    /// 默认激活菜单项样式
    fn default_active() -> Self {
        Self {
            background_color: Color32::from_rgb(232, 240, 254),
            text_color: Color32::from_rgb(74, 144, 226),
            border_color: Color32::TRANSPARENT,
            border_width: 0.0,
            left_border_color: Color32::from_rgb(74, 144, 226),
            left_border_width: 3.0,
        }
    }
    
    /// 默认普通菜单项样式
    fn default_normal() -> Self {
        Self {
            background_color: Color32::TRANSPARENT,
            text_color: Color32::BLACK,
            border_color: Color32::TRANSPARENT,
            border_width: 0.0,
            left_border_color: Color32::TRANSPARENT,
            left_border_width: 0.0,
        }
    }
    
    /// 默认悬停菜单项样式
    fn default_hover() -> Self {
        Self {
            background_color: Color32::from_rgb(224, 224, 224),
            text_color: Color32::BLACK,
            border_color: Color32::TRANSPARENT,
            border_width: 0.0,
            left_border_color: Color32::TRANSPARENT,
            left_border_width: 0.0,
        }
    }
}

/// 菜单项组件
pub struct MenuItem {
    /// 菜单项配置
    config: MenuItemConfig,
}

impl MenuItem {
    /// 创建新菜单项
    pub fn new<T: Display>(title: T) -> Self {
        Self {
            config: MenuItemConfig {
                title: title.to_string(),
                ..Default::default()
            },
        }
    }
    
    /// 创建子菜单
    pub fn submenu<T: Display>(title: T) -> Self {
        Self {
            config: MenuItemConfig {
                title: title.to_string(),
                item_type: MenuItemType::SubMenu,
                expanded: false,
                ..Default::default()
            },
        }
    }
    
    /// 创建分隔线
    pub fn separator() -> Self {
        Self {
            config: MenuItemConfig {
                item_type: MenuItemType::Separator,
                ..Default::default()
            },
        }
    }
    
    /// 设置菜单项ID
    pub fn id<T: Display>(mut self, id: T) -> Self {
        self.config.id = id.to_string();
        self
    }
    
    /// 设置菜单项图标
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.config.icon = Some(icon);
        self
    }
    
    /// 设置为激活状态
    pub fn active(mut self) -> Self {
        self.config.active = true;
        self
    }
    
    /// 设置为展开状态
    pub fn expanded(mut self) -> Self {
        self.config.expanded = true;
        self
    }
    
    /// 添加子菜单项
    pub fn add_child(&mut self, child: MenuItem) {
        self.config.children.push(child.config);
    }
    
    /// 设置点击回调
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_click = Some(Box::new(callback));
        self
    }
    
    /// 设置徽章数量
    pub fn badge_count(mut self, count: usize) -> Self {
        self.config.badge_count = Some(count);
        self
    }
    
    /// 设置是否可见
    pub fn visible(mut self, visible: bool) -> Self {
        self.config.visible = visible;
        self
    }
    
    /// 获取菜单项ID
    pub fn get_id(&self) -> &str {
        &self.config.id
    }
    
    /// 获取菜单项标题
    pub fn get_title(&self) -> &str {
        &self.config.title
    }
}

/// 侧边栏组件
pub struct Sidebar {
    /// 侧边栏配置
    config: SidebarConfig,
}

impl Sidebar {
    /// 创建新的侧边栏
    pub fn new() -> Self {
        Self {
            config: SidebarConfig::default(),
        }
    }
    
    /// 添加菜单项
    pub fn add_menu_item(&mut self, menu_item: MenuItem) {
        self.config.menu_items.push(menu_item.config);
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = width;
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: Color32) -> Self {
        self.config.background_color = color;
        self
    }
    
    /// 设置边框
    pub fn border(mut self, color: Color32, width: f32) -> Self {
        self.config.border_color = color;
        self.config.border_width = width;
        self
    }
    
    /// 设置是否显示搜索框
    pub fn show_search(mut self, show: bool) -> Self {
        self.config.show_search = show;
        self
    }
    
    /// 设置搜索回调
    pub fn on_search<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_search = Some(Box::new(callback));
        self.config.show_search = true;
        self
    }
    
    /// 设置激活菜单项ID
    pub fn active_item_id(mut self, item_id: &str) -> Self {
        self.config.active_item_id = Some(item_id.to_string());
        self
    }
    
    /// 设置菜单项高度
    pub fn item_height(mut self, height: f32) -> Self {
        self.config.item_height = height;
        self
    }
    
    /// 设置菜单项间距
    pub fn item_spacing(mut self, spacing: f32) -> Self {
        self.config.item_spacing = spacing;
        self
    }
    
    /// 设置是否可折叠
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.config.collapsible = collapsible;
        self
    }
    
    /// 设置是否折叠
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.config.collapsed = collapsed;
        self
    }
    
    /// 渲染侧边栏
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let SidebarConfig {
            ref mut menu_items,
            width,
            background_color,
            border_color,
            border_width,
            show_search,
            ref mut on_search,
            ref mut search_text,
            ref active_item_id,
            item_height,
            item_spacing,
            active_item_style,
            normal_item_style,
            hover_item_style,
            show_separator,
            separator_color,
            separator_height,
            collapsible,
            collapsed,
            collapse_icon,
            expand_icon,
        } = &mut self.config;
        
        // 渲染侧边栏背景
        let sidebar_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(sidebar_rect, 0.0, *background_color);
        
        // 渲染右侧边框
        ui.painter().line_segment(
            [
                sidebar_rect.right_top(),
                sidebar_rect.right_bottom(),
            ],
            egui::Stroke::new(*border_width, *border_color),
        );
        
        // 渲染折叠/展开按钮
        let response = ui.vertical(|ui| {
            if *collapsible {
                ui.horizontal(|ui| {
                    let collapse_button = ui.button(if *collapsed { *expand_icon } else { *collapse_icon });
                    if collapse_button.clicked() {
                        *collapsed = !*collapsed;
                    }
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label("菜单");
                    });
                });
            }
            
            if !*collapsed {
                // 渲染搜索框
                if *show_search {
                    ui.add_space(*item_spacing);
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        let search_response = ui.text_edit_singleline(search_text);
                        if search_response.changed() {
                            if let Some(callback) = on_search.as_mut() {
                                callback(search_text);
                            }
                        }
                    });
                    ui.separator();
                }
                
                // 渲染菜单项
                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_menu_items(ui, menu_items, active_item_id, item_height, item_spacing, active_item_style, normal_item_style, hover_item_style, show_separator, separator_color, separator_height);
                    });
            }
        });
        
        response.response
    }
    
    /// 渲染菜单项
    fn render_menu_items(
        &mut self,
        ui: &mut Ui,
        menu_items: &mut [MenuItemConfig],
        active_item_id: &Option<String>,
        item_height: &f32,
        item_spacing: &f32,
        active_item_style: &MenuItemStyle,
        normal_item_style: &MenuItemStyle,
        hover_item_style: &MenuItemStyle,
        show_separator: &bool,
        separator_color: &Color32,
        separator_height: &f32,
    ) {
        for item in menu_items {
            if !item.visible {
                continue;
            }
            
            match item.item_type {
                MenuItemType::Item => {
                    // 渲染菜单项
                    let is_active = active_item_id.as_deref() == Some(&item.id);
                    let style = if is_active {
                        active_item_style
                    } else {
                        normal_item_style
                    };
                    
                    ui.horizontal(|ui| {
                        // 左侧边框
                        if is_active {
                            ui.painter().line_segment(
                                [
                                    ui.next_widget_position(),
                                    ui.next_widget_position() + egui::vec2(0.0, *item_height),
                                ],
                                egui::Stroke::new(style.left_border_width, style.left_border_color),
                            );
                        }
                        
                        // 菜单项按钮
                        let mut button = egui::Button::new(self.render_menu_item_content(item))
                            .fill(style.background_color)
                            .text_color(style.text_color)
                            .frame(false)
                            .min_size(egui::vec2(ui.available_width(), *item_height));
                        
                        let response = button.show(ui);
                        
                        // 处理点击
                        if response.clicked() {
                            if let Some(callback) = item.on_click.as_mut() {
                                callback();
                            }
                        }
                    });
                },
                MenuItemType::SubMenu => {
                    // 渲染子菜单
                    let is_active = item.children.iter().any(|child| active_item_id.as_deref() == Some(&child.id));
                    let style = if is_active {
                        active_item_style
                    } else {
                        normal_item_style
                    };
                    
                    ui.horizontal(|ui| {
                        // 左侧边框
                        if is_active {
                            ui.painter().line_segment(
                                [
                                    ui.next_widget_position(),
                                    ui.next_widget_position() + egui::vec2(0.0, *item_height),
                                ],
                                egui::Stroke::new(style.left_border_width, style.left_border_color),
                            );
                        }
                        
                        // 子菜单折叠/展开按钮
                        let expand_button = ui.button(if item.expanded { *expand_icon } else { *collapse_icon });
                        if expand_button.clicked() {
                            item.expanded = !item.expanded;
                        }
                        
                        // 子菜单标题
                        let mut button = egui::Button::new(self.render_menu_item_content(item))
                            .fill(style.background_color)
                            .text_color(style.text_color)
                            .frame(false)
                            .min_size(egui::vec2(ui.available_width() - 20.0, *item_height));
                        
                        let response = button.show(ui);
                        
                        // 处理点击
                        if response.clicked() {
                            item.expanded = !item.expanded;
                        }
                    });
                    
                    // 渲染子菜单项
                    if item.expanded {
                        ui.indent(16.0, |ui| {
                            self.render_menu_items(ui, &mut item.children, active_item_id, item_height, item_spacing, active_item_style, normal_item_style, hover_item_style, show_separator, separator_color, separator_height);
                        });
                    }
                },
                MenuItemType::Separator => {
                    // 渲染分隔线
                    if *show_separator {
                        ui.add_space(*item_spacing);
                        ui.painter().line_segment(
                            [
                                ui.next_widget_position(),
                                ui.next_widget_position() + egui::vec2(ui.available_width(), *separator_height),
                            ],
                            egui::Stroke::new(*separator_height, *separator_color),
                        );
                        ui.add_space(*item_spacing);
                    }
                },
            }
            
            // 应用菜单项间距
            ui.add_space(*item_spacing);
        }
    }
    
    /// 渲染菜单项内容
    fn render_menu_item_content(&self, item: &MenuItemConfig) -> egui::WidgetText {
        let mut text = if let Some(icon) = item.icon {
            RichText::new(format!("{} {}", icon, item.title))
        } else {
            RichText::new(&item.title)
        };
        
        // 渲染徽章
        if let Some(count) = item.badge_count {
            text = text.append(" ")
                .append(RichText::new(format!("({})
", count))
                    .color(Color32::RED)
                    .small());
        }
        
        text.into()
    }
    
    /// 获取激活的菜单项ID
    pub fn get_active_item_id(&self) -> Option<&str> {
        self.config.active_item_id.as_deref()
    }
    
    /// 设置激活的菜单项ID
    pub fn set_active_item_id(&mut self, item_id: &str) {
        self.config.active_item_id = Some(item_id.to_string());
    }
    
    /// 获取菜单项数量
    pub fn get_item_count(&self) -> usize {
        self.config.menu_items.len()
    }
    
    /// 搜索菜单项
    pub fn search_items(&mut self, query: &str) {
        for item in &mut self.config.menu_items {
            item.visible = item.title.to_lowercase().contains(&query.to_lowercase());
            
            // 递归搜索子菜单项
            search_subitems(&mut item.children, query);
        }
    }
}

/// 递归搜索子菜单项的辅助函数
fn search_subitems(items: &mut [MenuItemConfig], query: &str) {
    for item in items {
        item.visible = item.title.to_lowercase().contains(&query.to_lowercase());
        search_subitems(&mut item.children, query);
    }
}

/// 侧边栏构建器
pub struct SidebarBuilder {
    /// 侧边栏配置
    config: SidebarConfig,
}

impl SidebarBuilder {
    /// 创建新的侧边栏构建器
    pub fn new() -> Self {
        Self {
            config: SidebarConfig::default(),
        }
    }
    
    /// 添加菜单项
    pub fn add_menu_item(mut self, menu_item: MenuItem) -> Self {
        self.config.menu_items.push(menu_item.config);
        self
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = width;
        self
    }
    
    /// 设置背景色
    pub fn background(mut self, color: Color32) -> Self {
        self.config.background_color = color;
        self
    }
    
    /// 设置边框
    pub fn border(mut self, color: Color32, width: f32) -> Self {
        self.config.border_color = color;
        self.config.border_width = width;
        self
    }
    
    /// 设置是否显示搜索框
    pub fn show_search(mut self, show: bool) -> Self {
        self.config.show_search = show;
        self
    }
    
    /// 设置搜索回调
    pub fn on_search<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_search = Some(Box::new(callback));
        self.config.show_search = true;
        self
    }
    
    /// 设置激活菜单项ID
    pub fn active_item_id(mut self, item_id: &str) -> Self {
        self.config.active_item_id = Some(item_id.to_string());
        self
    }
    
    /// 设置菜单项高度
    pub fn item_height(mut self, height: f32) -> Self {
        self.config.item_height = height;
        self
    }
    
    /// 设置菜单项间距
    pub fn item_spacing(mut self, spacing: f32) -> Self {
        self.config.item_spacing = spacing;
        self
    }
    
    /// 设置是否可折叠
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.config.collapsible = collapsible;
        self
    }
    
    /// 设置是否折叠
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.config.collapsed = collapsed;
        self
    }
    
    /// 构建侧边栏
    pub fn build(self) -> Sidebar {
        Sidebar {
            config: self.config,
        }
    }
}
