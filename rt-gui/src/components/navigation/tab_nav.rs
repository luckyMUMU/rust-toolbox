use eframe::egui::{Ui, Response, Color32, RichText, Id, Layout, Direction, Align, Stroke, Rounding, Vec2};use std::fmt::Display;

/// 标签页状态
enum TabState {
    /// 激活状态
    Active,
    /// 非激活状态
    Inactive,
    /// 悬停状态
    Hovered,
    /// 关闭状态
    Closing,
}

/// 标签页配置
struct TabConfig {
    /// 标签页ID
    id: String,
    /// 标签页标题
    title: String,
    /// 标签页图标（可选）
    icon: Option<&'static str>,
    /// 标签页状态
    state: TabState,
    /// 是否显示关闭按钮
    show_close_button: bool,
    /// 是否有未读内容
    has_unread: bool,
    /// 未读计数（可选）
    unread_count: Option<usize>,
    /// 点击回调
    on_click: Option<Box<dyn FnMut()>>,
    /// 关闭回调
    on_close: Option<Box<dyn FnMut()>>,
}

impl Default for TabConfig {
    fn default() -> Self {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            Self {
                id: format!("tab-{}", COUNTER),
                title: "New Tab".to_string(),
                icon: None,
                state: TabState::Inactive,
                show_close_button: true,
                has_unread: false,
                unread_count: None,
                on_click: None,
                on_close: None,
            }
        }
    }
}

/// 标签页导航配置
struct TabNavConfig {
    /// 标签页列表
    tabs: Vec<TabConfig>,
    /// 激活的标签页ID
    active_tab_id: Option<String>,
    /// 显示新建标签页按钮
    show_new_button: bool,
    /// 新建标签页回调
    on_new_tab: Option<Box<dyn FnMut()>>,
    /// 水平间距
    spacing: f32,
    /// 标签页高度
    height: f32,
    /// 激活标签页样式
    active_tab_style: TabStyle,
    /// 非激活标签页样式
    inactive_tab_style: TabStyle,
    /// 悬停标签页样式
    hovered_tab_style: TabStyle,
    /// 显示标签页边框
    show_border: bool,
    /// 边框颜色
    border_color: Color32,
    /// 边框宽度
    border_width: f32,
    /// 标签页可拖拽
    draggable: bool,
}

impl Default for TabNavConfig {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            show_new_button: true,
            on_new_tab: None,
            spacing: 0.0,
            height: 36.0,
            active_tab_style: TabStyle::default_active(),
            inactive_tab_style: TabStyle::default_inactive(),
            hovered_tab_style: TabStyle::default_hovered(),
            show_border: true,
            border_color: Color32::from_rgb(200, 200, 200),
            border_width: 1.0,
            draggable: true,
        }
    }
}

/// 标签页样式
struct TabStyle {
    /// 背景色
    background_color: Color32,
    /// 文本颜色
    text_color: Color32,
    /// 边框颜色
    border_color: Color32,
    /// 边框宽度
    border_width: f32,
    /// 底部边框颜色
    bottom_border_color: Color32,
    /// 底部边框宽度
    bottom_border_width: f32,
    /// 圆角半径
    corner_radius: f32,
}

impl TabStyle {
    /// 默认激活标签页样式
    fn default_active() -> Self {
        Self {
            background_color: Color32::WHITE,
            text_color: Color32::from_rgb(74, 144, 226),
            border_color: Color32::from_rgb(200, 200, 200),
            border_width: 1.0,
            bottom_border_color: Color32::from_rgb(74, 144, 226),
            bottom_border_width: 2.0,
            corner_radius: 4.0,
        }
    }
    
    /// 默认非激活标签页样式
    fn default_inactive() -> Self {
        Self {
            background_color: Color32::from_rgb(245, 245, 245),
            text_color: Color32::BLACK,
            border_color: Color32::from_rgb(200, 200, 200),
            border_width: 1.0,
            bottom_border_color: Color32::from_rgb(200, 200, 200),
            bottom_border_width: 1.0,
            corner_radius: 4.0,
        }
    }
    
    /// 默认悬停标签页样式
    fn default_hovered() -> Self {
        Self {
            background_color: Color32::from_rgb(232, 240, 254),
            text_color: Color32::BLACK,
            border_color: Color32::from_rgb(150, 150, 150),
            border_width: 1.0,
            bottom_border_color: Color32::from_rgb(74, 144, 226),
            bottom_border_width: 2.0,
            corner_radius: 4.0,
        }
    }
}

/// 标签页组件
pub struct Tab {
    /// 标签页配置
    config: TabConfig,
}

impl Tab {
    /// 创建新标签页
    pub fn new<T: Display>(title: T) -> Self {
        Self {
            config: TabConfig {
                title: title.to_string(),
                ..Default::default()
            },
        }
    }
    
    /// 设置标签页ID
    pub fn id<T: Display>(mut self, id: T) -> Self {
        self.config.id = id.to_string();
        self
    }
    
    /// 设置标签页图标
    pub fn icon(mut self, icon: &'static str) -> Self {
        self.config.icon = Some(icon);
        self
    }
    
    /// 设置为激活状态
    pub fn active(mut self) -> Self {
        self.config.state = TabState::Active;
        self
    }
    
    /// 设置为非激活状态
    pub fn inactive(mut self) -> Self {
        self.config.state = TabState::Inactive;
        self
    }
    
    /// 设置是否显示关闭按钮
    pub fn show_close_button(mut self, show: bool) -> Self {
        self.config.show_close_button = show;
        self
    }
    
    /// 设置有未读内容
    pub fn has_unread(mut self) -> Self {
        self.config.has_unread = true;
        self
    }
    
    /// 设置未读计数
    pub fn unread_count(mut self, count: usize) -> Self {
        self.config.has_unread = true;
        self.config.unread_count = Some(count);
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
    
    /// 设置关闭回调
    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_close = Some(Box::new(callback));
        self
    }
    
    /// 获取标签页ID
    pub fn get_id(&self) -> &str {
        &self.config.id
    }
    
    /// 获取标签页标题
    pub fn get_title(&self) -> &str {
        &self.config.title
    }
    
    /// 设置标签页状态
    pub fn set_state(&mut self, state: TabState) {
        self.config.state = state;
    }
    
    /// 获取标签页状态
    pub fn get_state(&self) -> &TabState {
        &self.config.state
    }
}

/// 标签页导航组件
pub struct TabNav {
    /// 标签页导航配置
    config: TabNavConfig,
    /// 拖拽状态
    drag_state: DragState,
}

/// 拖拽状态
enum DragState {
    /// 未拖拽
    None,
    /// 拖拽中
    Dragging {
        /// 拖拽的标签页ID
        tab_id: String,
        /// 拖拽的起始位置
        start_x: f32,
    },
}

impl Default for DragState {
    fn default() -> Self {
        Self::None
    }
}

impl TabNav {
    /// 创建新的标签页导航
    pub fn new() -> Self {
        Self {
            config: TabNavConfig::default(),
            drag_state: DragState::None,
        }
    }
    
    /// 添加标签页
    pub fn add_tab(&mut self, tab: Tab) {
        // 如果是第一个标签页，设置为激活状态
        if self.config.tabs.is_empty() {
            let mut tab_config = tab.config;
            tab_config.state = TabState::Active;
            self.config.active_tab_id = Some(tab_config.id.clone());
            self.config.tabs.push(tab_config);
        } else {
            self.config.tabs.push(tab.config);
        }
    }
    
    /// 移除标签页
    pub fn remove_tab(&mut self, tab_id: &str) {
        let index = self.config.tabs.iter().position(|t| t.id == tab_id);
        if let Some(index) = index {
            self.config.tabs.remove(index);
            
            // 如果移除的是激活的标签页，激活前一个标签页
            if self.config.active_tab_id == Some(tab_id.to_string()) {
                if index > 0 {
                    self.config.active_tab_id = Some(self.config.tabs[index - 1].id.clone());
                } else if !self.config.tabs.is_empty() {
                    self.config.active_tab_id = Some(self.config.tabs[0].id.clone());
                } else {
                    self.config.active_tab_id = None;
                }
            }
        }
    }
    
    /// 激活标签页
    pub fn activate_tab(&mut self, tab_id: &str) {
        // 更新所有标签页状态
        for tab in &mut self.config.tabs {
            if tab.id == tab_id {
                tab.state = TabState::Active;
            } else {
                tab.state = TabState::Inactive;
            }
        }
        
        self.config.active_tab_id = Some(tab_id.to_string());
    }
    
    /// 设置是否显示新建标签页按钮
    pub fn show_new_button(mut self, show: bool) -> Self {
        self.config.show_new_button = show;
        self
    }
    
    /// 设置新建标签页回调
    pub fn on_new_tab<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_new_tab = Some(Box::new(callback));
        self
    }
    
    /// 设置水平间距
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.config.spacing = spacing;
        self
    }
    
    /// 设置标签页高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = height;
        self
    }
    
    /// 设置是否显示边框
    pub fn show_border(mut self, show: bool) -> Self {
        self.config.show_border = show;
        self
    }
    
    /// 设置边框颜色
    pub fn border_color(mut self, color: Color32) -> Self {
        self.config.border_color = color;
        self
    }
    
    /// 设置边框宽度
    pub fn border_width(mut self, width: f32) -> Self {
        self.config.border_width = width;
        self
    }
    
    /// 设置标签页可拖拽
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.config.draggable = draggable;
        self
    }
    
    /// 渲染标签页导航
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let TabNavConfig {
            ref mut tabs,
            ref active_tab_id,
            show_new_button,
            ref mut on_new_tab,
            spacing,
            height,
            active_tab_style,
            inactive_tab_style,
            hovered_tab_style,
            show_border,
            border_color,
            border_width,
            draggable,
        } = &mut self.config;
        
        // 渲染标签页导航背景
        let tab_nav_rect = ui.available_rect_before_wrap();
        
        // 渲染底部边框
        if *show_border {
            ui.painter().line_segment(
                [
                    tab_nav_rect.left_bottom(),
                    tab_nav_rect.right_bottom(),
                ],
                egui::Stroke::new(*border_width, *border_color),
            );
        }
        
        // 渲染标签页导航内容
        let response = ui.horizontal(|ui| {
            ui.set_min_height(*height);
            
            // 渲染标签页
            for (index, tab) in tabs.iter_mut().enumerate() {
                let is_active = active_tab_id.as_deref() == Some(&tab.id);
                
                // 获取标签页样式
                let style = match tab.state {
                    TabState::Active => active_tab_style,
                    TabState::Hovered => hovered_tab_style,
                    _ => inactive_tab_style,
                };
                
                // 渲染标签页按钮
                let tab_response = ui.push_id(&tab.id, |ui| {
                    let mut button = egui::Button::new(self.render_tab_content(tab))
                        .fill(style.background_color)
                        .text_color(style.text_color)
                        .frame(false)
                        .sense(egui::Sense::click_and_drag());
                    
                    let response = button.show(ui);
                    
                    // 处理标签页点击
                    if response.clicked() {
                        if let Some(callback) = tab.on_click.as_mut() {
                            callback();
                        }
                        // 激活标签页
                        self.activate_tab(&tab.id);
                    }
                    
                    // 处理关闭按钮点击
                    if tab.show_close_button {
                        let close_response = ui.button(egui::RichText::new("✕").size(12.0));
                        if close_response.clicked() {
                            if let Some(callback) = tab.on_close.as_mut() {
                                callback();
                            }
                            self.remove_tab(&tab.id);
                        }
                    }
                    
                    response.response
                });
                
                // 处理拖拽
                if *draggable {
                    self.handle_drag(&tab_response, &tab.id, index, ui);
                }
                
                // 应用水平间距
                if index < tabs.len() - 1 {
                    ui.add_space(*spacing);
                }
            }
            
            // 渲染新建标签页按钮
            if *show_new_button {
                ui.add_space(*spacing);
                if ui.button(egui::RichText::new("+").size(16.0)).clicked() {
                    if let Some(callback) = on_new_tab.as_mut() {
                        callback();
                    }
                }
            }
        });
        
        response.response
    }
    
    /// 渲染标签页内容
    fn render_tab_content(&self, tab: &TabConfig) -> egui::WidgetText {
        let mut text = if let Some(icon) = tab.icon {
            RichText::new(format!("{} {}", icon, tab.title))
        } else {
            RichText::new(&tab.title)
        };
        
        // 渲染未读提示
        if tab.has_unread {
            if let Some(count) = tab.unread_count {
                text = text.append(" ")
                    .append(RichText::new(format!("({})
", count))
                        .color(Color32::RED)
                        .small());
            } else {
                text = text.append(" ")
                    .append(RichText::new("●")
                        .color(Color32::RED)
                        .small());
            }
        }
        
        text.into()
    }
    
    /// 处理拖拽
    fn handle_drag(&mut self, tab_response: &Response, tab_id: &str, tab_index: usize, ui: &mut Ui) {
        // 处理拖拽开始
        if tab_response.drag_started() {
            self.drag_state = DragState::Dragging {
                tab_id: tab_id.to_string(),
                start_x: ui.input().pointer.hover_pos().unwrap_or_default().x,
            };
        }
        
        // 处理拖拽中
        if let DragState::Dragging { ref tab_id, ref start_x } = self.drag_state {
            if tab_id == tab_id {
                let current_x = ui.input().pointer.hover_pos().unwrap_or_default().x;
                let delta_x = current_x - *start_x;
                
                // 处理标签页拖拽位置更新
                // TODO: 实现标签页拖拽重排逻辑
            }
        }
        
        // 处理拖拽结束
        if tab_response.drag_released() {
            self.drag_state = DragState::None;
        }
    }
    
    /// 获取激活的标签页ID
    pub fn get_active_tab_id(&self) -> Option<&str> {
        self.config.active_tab_id.as_deref()
    }
    
    /// 设置激活的标签页ID
    pub fn set_active_tab_id(&mut self, tab_id: &str) {
        self.activate_tab(tab_id);
    }
    
    /// 获取标签页数量
    pub fn get_tab_count(&self) -> usize {
        self.config.tabs.len()
    }
    
    /// 获取所有标签页ID
    pub fn get_all_tab_ids(&self) -> Vec<&str> {
        self.config.tabs.iter().map(|t| t.id.as_str()).collect()
    }
}

/// 标签页导航构建器
pub struct TabNavBuilder {
    /// 标签页导航配置
    config: TabNavConfig,
}

impl TabNavBuilder {
    /// 创建新的标签页导航构建器
    pub fn new() -> Self {
        Self {
            config: TabNavConfig::default(),
        }
    }
    
    /// 添加标签页
    pub fn add_tab(mut self, tab: Tab) -> Self {
        self.config.tabs.push(tab.config);
        self
    }
    
    /// 设置激活的标签页ID
    pub fn active_tab_id(mut self, tab_id: &str) -> Self {
        self.config.active_tab_id = Some(tab_id.to_string());
        self
    }
    
    /// 设置是否显示新建标签页按钮
    pub fn show_new_button(mut self, show: bool) -> Self {
        self.config.show_new_button = show;
        self
    }
    
    /// 设置新建标签页回调
    pub fn on_new_tab<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_new_tab = Some(Box::new(callback));
        self
    }
    
    /// 设置水平间距
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.config.spacing = spacing;
        self
    }
    
    /// 设置标签页高度
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = height;
        self
    }
    
    /// 设置是否显示边框
    pub fn show_border(mut self, show: bool) -> Self {
        self.config.show_border = show;
        self
    }
    
    /// 设置边框颜色
    pub fn border_color(mut self, color: Color32) -> Self {
        self.config.border_color = color;
        self
    }
    
    /// 设置边框宽度
    pub fn border_width(mut self, width: f32) -> Self {
        self.config.border_width = width;
        self
    }
    
    /// 设置标签页可拖拽
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.config.draggable = draggable;
        self
    }
    
    /// 构建标签页导航
    pub fn build(self) -> TabNav {
        TabNav {
            config: self.config,
            drag_state: DragState::None,
        }
    }
}
