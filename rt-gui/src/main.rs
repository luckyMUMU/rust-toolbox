use eframe::egui;
use rt_core::{Tool, Locale};
use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use serde_json::{Value, json};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::time::SystemTime;

// 导入布局管理器
mod layout_manager;
use layout_manager::{LayoutType, LayoutManager, render_schema};

// 导入设置模块
mod settings;
use settings::{Settings, Theme};

enum GuiMessage {
    Output(Value),
    Error(String),
}

/// Tab页数据结构
#[derive(Clone)]
struct Tab {
    /// Tab页ID
    #[allow(dead_code)]
    id: String,
    /// 工具名称
    tool_name: String,
    /// Tab页标题
    title: String,
    /// 输入值
    input_value: Value,
    /// 当前输入 schema
    current_schema: Option<Value>,
    /// 输出值
    output_value: Option<Value>,
    /// 输出错误
    output_error: Option<String>,
    /// 当前输出 schema
    output_schema: Option<Value>,
    /// 是否显示帮助
    show_help: bool,
    /// 当前布局类型
    layout_type: LayoutType,
    /// 是否未读
    unread: bool,
}

/// 工具分类
#[derive(Clone)]
struct ToolCategory {
    /// 分类名称
    #[allow(dead_code)]
    name: String,
    /// 分类显示名称
    display_name: String,
    /// 子分类
    subcategories: Vec<ToolCategory>,
    /// 工具列表 - 存储工具名称，在渲染时再从tools哈希表中获取
    tools: Vec<String>,
}

struct ToolkitApp {
    /// 所有工具
    tools: Arc<HashMap<String, Arc<dyn Tool>>>,
    /// 工具分类
    tool_categories: Vec<ToolCategory>,
    
    // Tab页管理
    tabs: Vec<Tab>,
    active_tab_index: Option<usize>,
    
    // Help UI state
    markdown_cache: CommonMarkCache,

    // I18n
    locale: Locale,

    // Communication channel
    tx: mpsc::Sender<GuiMessage>,
    rx: mpsc::Receiver<GuiMessage>,
    
    // Async runtime
    runtime: tokio::runtime::Runtime,
    
    // UI state
    #[allow(dead_code)]
    show_sidebar: bool,
    sidebar_width: f32,
    
    // Settings
    /// 应用配置
    settings: Settings,
    /// 是否显示设置页面
    show_settings: bool,
    /// 设置操作状态信息
    settings_status: Option<(bool, String)>,
}

impl ToolCategory {
    /// 创建新的工具分类
    fn new(name: &str, display_name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            subcategories: Vec::new(),
            tools: Vec::new(),
        }
    }
    
    /// 添加子分类
    #[allow(dead_code)]
    fn add_subcategory(&mut self, category: ToolCategory) {
        self.subcategories.push(category);
    }
    
    /// 添加工具
    fn add_tool(&mut self, tool_name: String) {
        self.tools.push(tool_name);
    }
}

/// 工具分类辅助函数
fn categorize_tools(tools: Arc<HashMap<String, Arc<dyn Tool>>>, _locale: Locale) -> Vec<ToolCategory> {
    let mut categories: HashMap<String, ToolCategory> = HashMap::new();
    
    // 创建默认分类
    categories.insert("file".to_string(), ToolCategory::new("file", "文件操作"));
    categories.insert("text".to_string(), ToolCategory::new("text", "文本操作"));
    categories.insert("media".to_string(), ToolCategory::new("media", "媒体操作"));
    categories.insert("other".to_string(), ToolCategory::new("other", "其他工具"));
    
    // 按工具名称前缀分类
    for (name, _) in tools.iter() {
        let category_name = name.split('.').next().unwrap_or("other");
        let category = categories.entry(category_name.to_string()).or_insert_with(|| {
            ToolCategory::new(category_name, category_name)
        });
        category.add_tool(name.clone());
    }
    
    // 转换为Vec并排序
    let mut result: Vec<ToolCategory> = categories.into_values().collect();
    result.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    
    result
}

impl ToolkitApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut tools: HashMap<String, Arc<dyn Tool>> = HashMap::new();
        
        // Built-in tools
        for tool in rt_tools::get_all_tools() {
            tools.insert(tool.name().to_string(), Arc::from(tool));
        }

        // Load plugins calling rt_core (blocking on UI thread for startup)
        runtime.block_on(async {
            let plugin_dir = std::path::Path::new("plugins");
            if plugin_dir.exists() {
                 let plugin_manager = rt_core::plugin::PluginManager::new(plugin_dir.to_path_buf());
                 if let Ok(()) = plugin_manager.load_all().await {
                     for tool in plugin_manager.list_tools().await {
                         tools.insert(tool.name().to_string(), tool.clone());
                     }
                 }
            }
        });
        
        let tools_arc = Arc::new(tools);
        
        // 加载设置
        let settings = match Settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("Failed to load settings: {}", e);
                Settings::default()
            }
        };
        
        let locale = settings.display.locale;
        let tool_categories = categorize_tools(tools_arc.clone(), locale);
        
        let (tx, rx) = mpsc::channel();

        Self {
            tools: tools_arc,
            tool_categories,
            tabs: Vec::new(),
            active_tab_index: None,
            markdown_cache: CommonMarkCache::default(),
            locale,
            tx,
            rx,
            runtime,
            show_sidebar: true,
            sidebar_width: settings.display.sidebar_width,
            settings,
            show_settings: false,
            settings_status: None,
        }
    }

    fn tr(&self, key: &str) -> String {
        match (self.locale, key) {
            // Common
            (Locale::En, "Run") => "Run".to_string(),
            (Locale::Zh, "Run") => "运行".to_string(),
            (Locale::En, "Output") => "Output".to_string(),
            (Locale::Zh, "Output") => "输出".to_string(),
            (Locale::En, "Tools") => "Tools".to_string(),
            (Locale::Zh, "Tools") => "工具箱".to_string(),
            (Locale::En, "Help") => "Help".to_string(),
            (Locale::Zh, "Help") => "帮助".to_string(),
            (Locale::En, "Show Help") => "Show Help".to_string(),
            (Locale::Zh, "Show Help") => "显示帮助".to_string(),
            (Locale::En, "Hide Help") => "Hide Help".to_string(),
            (Locale::Zh, "Hide Help") => "隐藏帮助".to_string(),
            (Locale::En, "Running") => "Running".to_string(),
            (Locale::Zh, "Running") => "正在运行".to_string(),
            (Locale::En, "Select a tool") => "Select a tool to begin".to_string(),
            (Locale::Zh, "Select a tool") => "请选择一个工具开始".to_string(),
            (Locale::En, "Select a tool help") => "Select a tool to view help.".to_string(),
            (Locale::Zh, "Select a tool help") => "选择工具查看帮助文档".to_string(),
            (Locale::En, "Raw JSON") => "Raw JSON (Debug)".to_string(),
            (Locale::Zh, "Raw JSON") => "原始 JSON (调试)".to_string(),
            (Locale::En, "File Operations") => "File Operations".to_string(),
            (Locale::Zh, "File Operations") => "文件操作".to_string(),
            (Locale::En, "Text Operations") => "Text Operations".to_string(),
            (Locale::Zh, "Text Operations") => "文本操作".to_string(),
            (Locale::En, "Media Operations") => "Media Operations".to_string(),
            (Locale::Zh, "Media Operations") => "媒体操作".to_string(),
            (Locale::En, "Other") => "Other".to_string(),
            (Locale::Zh, "Other") => "其他工具".to_string(),
            (Locale::En, "New Tab") => "New Tab".to_string(),
            (Locale::Zh, "New Tab") => "新建标签页".to_string(),
            (Locale::En, "Close Tab") => "Close".to_string(),
            (Locale::Zh, "Close Tab") => "关闭".to_string(),
            (Locale::En, "Home") => "Home".to_string(),
            (Locale::Zh, "Home") => "首页".to_string(),
            (Locale::En, "Settings") => "Settings".to_string(),
            (Locale::Zh, "Settings") => "设置".to_string(),
            (Locale::En, "About") => "About".to_string(),
            (Locale::Zh, "About") => "关于".to_string(),
            _ => key.to_string(),
        }
    }
    
    /// 打开新的工具标签页
    fn open_tool_tab(&mut self, tool_name: &str) {
        if let Some(tool) = self.tools.get(tool_name) {
            // 使用SystemTime生成唯一的tab_id
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let tab_id = format!("{}-{}", tool_name, timestamp);
            let display_name = tool.display_name(self.locale);
            
            let tab = Tab {
                id: tab_id,
                tool_name: tool_name.to_string(),
                title: display_name,
                input_value: json!({}),
                current_schema: Some(tool.input_schema(self.locale)),
                output_value: None,
                output_error: None,
                output_schema: None,
                show_help: false,
                layout_type: LayoutType::default(),
                unread: false,
            };
            
            self.tabs.push(tab);
            self.active_tab_index = Some(self.tabs.len() - 1);
        }
    }
    
    /// 关闭标签页
    fn close_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            
            // 更新活动标签页索引
            if self.active_tab_index == Some(index) {
                self.active_tab_index = if self.tabs.is_empty() {
                    None
                } else {
                    Some(index.min(self.tabs.len() - 1))
                };
            } else if self.active_tab_index.is_some() && self.active_tab_index.unwrap() > index {
                self.active_tab_index = self.active_tab_index.map(|i| i - 1);
            }
        }
    }
    
    /// 获取当前活动标签页
    #[allow(dead_code)]
    fn active_tab(&mut self) -> Option<&mut Tab> {
        if let Some(index) = self.active_tab_index {
            self.tabs.get_mut(index)
        } else {
            None
        }
    }
}



impl ToolkitApp {
    /// 渲染多级工具分类菜单
    #[allow(dead_code)]
    fn render_tool_categories(&self, ui: &mut egui::Ui, mut open_tool: impl FnMut(&str)) {
        for category in &self.tool_categories {
            self.render_category(ui, category, &self.tools, &self.locale, &mut open_tool);
        }
    }
    
    /// 递归渲染分类和子分类
    #[allow(dead_code)]
    fn render_category(
        &self, 
        ui: &mut egui::Ui, 
        category: &ToolCategory, 
        tools: &Arc<HashMap<String, Arc<dyn Tool>>>, 
        locale: &Locale,
        open_tool: &mut impl FnMut(&str)
    ) {
        // 使用CollapsingHeader实现折叠菜单
        egui::CollapsingHeader::new(&category.display_name)
            .default_open(false)
            .show(ui, |ui| {
                // 渲染子分类
                for subcategory in &category.subcategories {
                    self.render_category(ui, subcategory, tools, locale, open_tool);
                }
                
                // 渲染工具列表
                for tool_name in &category.tools {
                    if let Some(tool) = tools.get(tool_name) {
                        let display_name = tool.display_name(*locale);
                        if ui.selectable_label(false, display_name).clicked() {
                            // 打开新的工具标签页
                            open_tool(tool_name);
                        }
                        ui.label(egui::RichText::new(tool.description(*locale)).small().weak());
                    }
                }
            });
    }
    
    /// 渲染标签页栏
    #[allow(dead_code)]
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 新建标签页按钮
            if ui.button(self.tr("New Tab")).clicked() {
                // 这里可以添加默认打开的工具，或者显示工具选择界面
            }
            
            // 标签页列表
            // 使用迭代器索引而不是iter_mut来避免可变借用冲突
            for index in 0..self.tabs.len() {
                let tab_title = self.tabs[index].title.clone();
                let is_active = self.active_tab_index == Some(index);
                
                ui.horizontal(|ui| {
                    // 标签页标题
                    if ui.selectable_label(is_active, &tab_title).clicked() {
                        self.active_tab_index = Some(index);
                    }
                    
                    // 关闭按钮
                    if ui.button("✕").clicked() {
                        // 复制索引以避免闭包中的可变借用冲突
                        let close_index = index;
                        self.close_tab(close_index);
                    }
                });
            }
        });
        ui.separator();
    }
    
    /// 渲染标签页内容，使用依赖注入避免借用规则冲突
    #[allow(dead_code)]
    fn render_tab_content_with_deps(
        &mut self, 
        ui: &mut egui::Ui, 
        tab: &mut Tab, 
        ctx: &egui::Context,
        tools: Arc<HashMap<String, Box<dyn Tool>>>,
        locale: Locale,
        tx: mpsc::Sender<GuiMessage>,
        runtime: &tokio::runtime::Runtime
    ) {
        // 工具名称和帮助按钮
        ui.horizontal(|ui| {
            ui.heading(format!("{}", tab.title));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let help_label = if tab.show_help { self.tr("Hide Help") } else { self.tr("Show Help") };
                if ui.button(help_label).clicked() {
                    tab.show_help = !tab.show_help;
                }
            });
        });
        ui.separator();
        
        // 帮助面板（可折叠）
        if tab.show_help {
            ui.collapsing(self.tr("Help"), |ui| {
                if let Some(tool) = tools.get(&tab.tool_name) {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let guide = tool.user_guide(locale);
                        CommonMarkViewer::new()
                            .show(ui, &mut self.markdown_cache, &guide);
                    });
                }
            });
        }
        
        // 主内容区域
        egui::Grid::new("tab_content_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                // 左侧：输入表单
                ui.vertical(|ui| {
                    ui.heading("输入");
                    if let Some(schema) = &tab.current_schema {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            render_schema(ui, schema, &mut tab.input_value, false, "");
                        });
                    }
                    
                    // Run button
                    if ui.button(self.tr("Run")).clicked() {
                        let tools_ref = tools.clone();
                        let tool_name = tab.tool_name.clone();
                        let input_val = tab.input_value.clone();
                        let tx_clone = tx.clone();
                        let ctx_clone = ctx.clone();
                        
                        tab.output_value = None;
                        tab.output_error = None;
                        
                        runtime.spawn(async move {
                            let result_msg = if let Some(t) = tools_ref.get(&tool_name) {
                                 match t.run(input_val).await {
                                     Ok(res) => GuiMessage::Output(res),
                                     Err(e) => GuiMessage::Error(e.to_string()),
                                 }
                            } else {
                                GuiMessage::Error("Tool not found internal error".to_string())
                            };

                            let _ = tx_clone.send(result_msg);
                            ctx_clone.request_repaint(); 
                        });
                    }
                });
                
                ui.separator();
                
                // 右侧：输出结果
                ui.vertical(|ui| {
                    ui.heading(self.tr("Output"));
                    
                    if let Some(error) = &tab.output_error {
                        ui.colored_label(egui::Color32::RED, error);
                    } else if let Some(val) = &mut tab.output_value {
                        if let Some(schema) = &tab.output_schema {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                render_schema(ui, schema, val, true, "");
                            });
                        } else {
                            // Fallback to raw json if no schema
                            ui.add(egui::TextEdit::multiline(&mut serde_json::to_string_pretty(val).unwrap()).interactive(false));
                        }
                    } else {
                        ui.label("Ready");
                    }
                });
                
                ui.end_row();
            });
            
            // Debug View
            ui.collapsing(self.tr("Raw JSON"), |ui| {
                ui.label(serde_json::to_string_pretty(&tab.input_value).unwrap_or_default());
            });
    }
}

impl eframe::App for ToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 处理消息
        self.handle_messages();
        
        // 2. 渲染顶部面板
        self.render_top_panel(ctx);
        
        // 3. 渲染左侧面板（工具分类）
        self.render_left_panel(ctx);
        
        // 4. 渲染中央面板（标签页管理和内容）
        self.render_central_panel(ctx);
    }
}

impl ToolkitApp {
    /// 处理来自工具的消息
    fn handle_messages(&mut self) {
        let tools_clone = self.tools.clone();
        let locale = self.locale;
        
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(index) = self.active_tab_index {
                if index < self.tabs.len() {
                    match msg {
                        GuiMessage::Output(val) => {
                            let tab = &mut self.tabs[index];
                            tab.output_value = Some(val);
                            tab.output_error = None;
                            
                            // 同时获取输出 schema
                            if let Some(tool) = tools_clone.get(&tab.tool_name) {
                                tab.output_schema = Some(tool.output_schema(locale));
                            }
                            
                            // 如果当前标签页不是活动标签页，将其标记为未读
                            tab.unread = true;
                        },
                        GuiMessage::Error(err) => {
                            let tab = &mut self.tabs[index];
                            tab.output_error = Some(err);
                            tab.output_value = None;
                            
                            // 如果当前标签页不是活动标签页，将其标记为未读
                            tab.unread = true;
                        }
                    }
                }
            }
        }
    }
    
    /// 渲染顶部面板：标题和语言选择器
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Rust Toolbox");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 设置按钮
                    if ui.button("⚙️").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    
                    egui::ComboBox::from_id_salt("locale_combo")
                        .selected_text(format!("{:?}", self.locale))
                        .show_ui(ui, |ui| {
                            let mut new_locale = self.locale;
                            let mut changed = false;
                            if ui.selectable_value(&mut new_locale, Locale::En, "English").clicked() { changed = true; }
                            if ui.selectable_value(&mut new_locale, Locale::Zh, "中文").clicked() { changed = true; }
                            
                            // 如果语言改变，更新所有标签页的 schema
                            if changed {
                                self.locale = new_locale;
                                self.settings.display.locale = new_locale;
                                for tab in &mut self.tabs {
                                    if let Some(tool) = self.tools.get(&tab.tool_name) {
                                        tab.current_schema = Some(tool.input_schema(self.locale));
                                        if tab.output_value.is_some() {
                                            tab.output_schema = Some(tool.output_schema(self.locale));
                                        }
                                    }
                                }
                            }
                        });
                });
            });
        });
        
        // 渲染设置页面
        self.render_settings(ctx);
    }
    
    /// 渲染左侧面板：多级导航菜单
    fn render_left_panel(&mut self, ctx: &egui::Context) {
        // 保存当前状态的副本，避免借用冲突
        let tool_categories = self.tool_categories.clone();
        let locale = self.locale;
        
        egui::SidePanel::left("left_panel")
            .default_width(self.sidebar_width)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Tools");
                ui.separator();
                
                // 渲染工具分类并收集选中的工具
                let mut selected_tools = Vec::new();
                for category in &tool_categories {
                    let mut category_selected = self.render_category_simple(ui, category, locale);
                    selected_tools.append(&mut category_selected);
                }
                
                // 处理选中的工具 - 在渲染完成后
                for tool_name in selected_tools {
                    self.open_tool_tab(&tool_name);
                }
            });
    }
    
    /// 简化版的分类渲染，使用不可变引用和返回选中工具列表来避免借用冲突
    fn render_category_simple(&self, ui: &mut egui::Ui, category: &ToolCategory, locale: Locale) -> Vec<String> {
        let mut selected_tools = Vec::new();
        
        // 使用CollapsingHeader实现折叠菜单
        egui::CollapsingHeader::new(&category.display_name)
            .default_open(false)
            .show(ui, |ui| {
                // 渲染子分类
                for subcategory in &category.subcategories {
                    let mut sub_selected = self.render_category_simple(ui, subcategory, locale);
                    selected_tools.append(&mut sub_selected);
                }
                
                // 渲染工具列表
                for tool_name in &category.tools {
                    if let Some(tool) = self.tools.get(tool_name) {
                        let display_name = tool.display_name(locale);
                        if ui.selectable_label(false, display_name).clicked() {
                            // 记录选中的工具，而不是直接修改状态
                            selected_tools.push(tool_name.clone());
                        }
                        ui.label(egui::RichText::new(tool.description(locale)).small().weak());
                    }
                }
            });
            
        selected_tools
    }
    
    /// 渲染中央面板：标签页管理和内容
    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 渲染标签页栏
            self.render_tab_bar_simple(ui);
            
            // 渲染标签页内容
            if let Some(index) = self.active_tab_index {
                if index < self.tabs.len() {
                    self.render_active_tab_content(ui, index, ctx);
                }
            } else {
                // 没有标签页时的空状态
                ui.vertical_centered(|ui| {
                    ui.heading(self.tr("Select a tool"));
                    ui.label("请从左侧选择一个工具开始使用");
                });
            }
        });
    }
    
    /// 简化版的标签页栏渲染
    fn render_tab_bar_simple(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // 新建标签页按钮
            if ui.button(self.tr("New Tab")).clicked() {
                // 这里可以添加默认打开的工具，或者显示工具选择界面
            }
            
            // 标签页列表
            let mut tabs_to_close = Vec::new();
            let mut new_active_index = self.active_tab_index;
            let mut tabs_to_mark_read = Vec::new();
            
            for (index, tab) in self.tabs.iter().enumerate() {
                ui.horizontal(|ui| {
                    // 标签页标题
                    let is_active = self.active_tab_index == Some(index);
                    
                    // 如果是未读状态，添加未读指示
                    let mut label_text = tab.title.clone();
                    if tab.unread && !is_active {
                        label_text = format!("{} ⭕", label_text);
                    }
                    
                    if ui.selectable_label(is_active, &label_text).clicked() {
                        new_active_index = Some(index);
                        // 记录需要标记为已读的标签页索引
                        tabs_to_mark_read.push(index);
                    }
                    
                    // 关闭按钮
                    if ui.button("✕").clicked() {
                        tabs_to_close.push(index);
                    }
                });
            }
            
            // 更新活动标签页索引
            if let Some(index) = new_active_index {
                self.active_tab_index = Some(index);
            }
            
            // 标记需要标记为已读的标签页
            for index in tabs_to_mark_read {
                if index < self.tabs.len() {
                    self.tabs[index].unread = false;
                }
            }
            
            // 关闭需要关闭的标签页
            for index in tabs_to_close.iter().rev() {
                self.close_tab(*index);
            }
        });
        ui.separator();
    }
    
    /// 渲染当前活动标签页的内容
    fn render_active_tab_content(&mut self, ui: &mut egui::Ui, tab_index: usize, ctx: &egui::Context) {
        // 获取当前标签页的副本，避免可变借用冲突
        let mut tab = self.tabs[tab_index].clone();
        let locale = self.locale;
        let tool_name = tab.tool_name.clone();
        let tool_guide = self.tools.get(&tool_name).map(|t| t.user_guide(locale));
        
        // 1. 渲染工具名称、帮助按钮和布局切换按钮
        ui.horizontal(|ui| {
            ui.heading(format!("{}", tab.title));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // 布局切换按钮
                let mut layout_manager = LayoutManager::new();
                layout_manager.current_layout = tab.layout_type;
                layout_manager.render_layout_switcher(ui);
                
                // 更新标签页的布局类型
                if layout_manager.current_layout != tab.layout_type {
                    tab.layout_type = layout_manager.current_layout;
                }
                
                // 帮助按钮
                let help_label = if tab.show_help { self.tr("Hide Help") } else { self.tr("Show Help") };
                if ui.button(help_label).clicked() {
                    tab.show_help = !tab.show_help;
                }
            });
        });
        ui.separator();
        
        // 2. 渲染帮助面板（可折叠）
        if tab.show_help {
            ui.collapsing(self.tr("Help"), |ui| {
                if let Some(guide) = tool_guide {
                    egui::ScrollArea::vertical()
                        .id_salt(format!("help_scroll_{}", tab_index))
                        .show(ui, |ui| {
                            CommonMarkViewer::new()
                                .show(ui, &mut self.markdown_cache, &guide);
                        });
                }
            });
        }
        
        // 3. 渲染主内容区域
        let mut run_button_clicked = false;
        let mut input_value = tab.input_value.clone();
        let current_schema = tab.current_schema.clone();
        
        // 使用卡片组件包装内容区域
        ui.group(|ui| {
            // 创建布局管理器
            let mut layout_manager = LayoutManager::new();
            layout_manager.current_layout = tab.layout_type;
            
            // 渲染输入区域的闭包
            let render_input = |ui: &mut egui::Ui| {
                ui.heading(self.tr("Input"));
                if let Some(schema) = &current_schema {
                    egui::ScrollArea::vertical()
                        .id_salt(format!("input_scroll_{}", tab_index))
                        .show(ui, |ui| {
                            render_schema(ui, schema, &mut input_value, false, "");
                        });
                }
                
                // Run button
                if ui.button(self.tr("Run")).clicked() {
                    run_button_clicked = true;
                }
            };
            
            // 渲染输出区域的闭包
            let render_output = |ui: &mut egui::Ui| {
                ui.heading(self.tr("Output"));
                
                if let Some(error) = &tab.output_error {
                    ui.colored_label(egui::Color32::RED, error);
                } else if let Some(val) = &tab.output_value {
                        if let Some(schema) = &tab.output_schema {
                            egui::ScrollArea::vertical()
                                .id_salt(format!("output_scroll_{}", tab_index))
                                .show(ui, |ui| {
                                    // 明确类型为Value
                                    let mut output_val: Value = val.clone();
                                    render_schema(ui, schema, &mut output_val, true, "");
                                });
                        } else {
                            // 如果没有 schema，显示原始 JSON
                            ui.add(egui::TextEdit::multiline(&mut serde_json::to_string_pretty(val).unwrap()).interactive(false));
                        }
                    } else {
                        ui.label(self.tr("Ready"));
                    }
            };
            
            // 渲染预览区域的闭包（可选）
            let render_preview = Some(|ui: &mut egui::Ui| {
                ui.heading(self.tr("Preview"));
                ui.label("预览功能开发中...");
            });
            
            // 使用布局管理器渲染内容
            layout_manager.render_content(ui, render_input, render_output, render_preview);
        });
            
        // 4. 渲染 Debug View - 使用卡片组件包装
        ui.collapsing(self.tr("Raw JSON"), |ui| {
            ui.group(|ui| {
                ui.label(serde_json::to_string_pretty(&input_value).unwrap_or_default());
            });
        });
        
        // 5. 更新状态
        // 更新帮助显示状态
        self.tabs[tab_index].show_help = tab.show_help;
        // 更新输入值
        self.tabs[tab_index].input_value = input_value;
        // 更新布局类型
        self.tabs[tab_index].layout_type = tab.layout_type;
        
        // 6. 如果 Run 按钮被点击，执行工具
        if run_button_clicked {
            let tools_ref = self.tools.clone();
            let tool_name = tab.tool_name.clone();
            let input_val = self.tabs[tab_index].input_value.clone();
            let tx = self.tx.clone();
            let ctx_clone = ctx.clone();
            
            // 清空之前的输出
            self.tabs[tab_index].output_value = None;
            self.tabs[tab_index].output_error = None;
            
            // 异步执行工具
            self.runtime.spawn(async move {
                let result_msg = if let Some(t) = tools_ref.get(&tool_name) {
                     match t.run(input_val).await {
                         Ok(res) => GuiMessage::Output(res),
                         Err(e) => GuiMessage::Error(e.to_string()),
                     }
                } else {
                    GuiMessage::Error("Tool not found internal error".to_string())
                };

                let _ = tx.send(result_msg);
                ctx_clone.request_repaint(); 
            });
        }
    }
    
    /// 渲染设置页面
    fn render_settings(&mut self, ctx: &egui::Context) {
        if self.show_settings {
            // 创建一个模态窗口
            egui::Window::new("设置")
                .resizable(true)
                .default_width(600.0)
                .default_height(500.0)
                .show(ctx, |ui| {
                    // 设置一个滚动区域
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // 显示状态信息
                        if let Some((success, message)) = &self.settings_status {
                            let color = if *success {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            };
                            ui.colored_label(color, message);
                        }
                        
                        // 显示偏好设置
                        ui.heading("显示偏好");
                        ui.group(|ui| {
                            // 主题选择
                            ui.horizontal(|ui| {
                                ui.label("主题：");
                                egui::ComboBox::from_id_salt("theme_combo")
                                    .selected_text(self.settings.display.theme.display_name())
                                    .show_ui(ui, |ui| {
                                        for theme in Theme::all() {
                                            ui.selectable_value(
                                                &mut self.settings.display.theme,
                                                theme,
                                                theme.display_name()
                                            );
                                        }
                                    });
                            });
                            
                            // 字体大小
                            ui.horizontal(|ui| {
                                ui.label("字体大小：");
                                let font_size = self.settings.display.font_size;
                                ui.add(egui::Slider::new(&mut self.settings.display.font_size, 8.0..=32.0)
                                    .text(format!("{:.1} px", font_size)));
                            });
                            
                            // 侧边栏宽度
                            ui.horizontal(|ui| {
                                ui.label("侧边栏宽度：");
                                let sidebar_width = self.settings.display.sidebar_width;
                                ui.add(egui::Slider::new(&mut self.settings.display.sidebar_width, 100.0..=500.0)
                                    .text(format!("{:.0} px", sidebar_width)));
                            });
                        });
                        
                        ui.separator();
                        
                        // 通知设置
                        ui.heading("通知设置");
                        ui.group(|ui| {
                            ui.checkbox(&mut self.settings.notifications.task_completed, "工具运行完成通知");
                            ui.checkbox(&mut self.settings.notifications.errors, "错误通知");
                            ui.checkbox(&mut self.settings.notifications.warnings, "警告通知");
                        });
                        
                        ui.separator();
                        
                        // 数据存储设置
                        ui.heading("数据存储");
                        ui.group(|ui| {
                            // 插件目录
                            ui.horizontal(|ui| {
                                ui.label("插件目录：");
                                ui.text_edit_singleline(&mut format!("{}", self.settings.data.plugins_dir.display()));
                                if ui.button("选择").clicked() {
                                    // 这里可以添加文件选择对话框
                                }
                            });
                            
                            // 日志目录
                            ui.horizontal(|ui| {
                                ui.label("日志目录：");
                                ui.text_edit_singleline(&mut format!("{}", self.settings.data.logs_dir.display()));
                                if ui.button("选择").clicked() {
                                    // 这里可以添加文件选择对话框
                                }
                            });
                            
                            // 临时文件目录
                            ui.horizontal(|ui| {
                                ui.label("临时文件目录：");
                                ui.text_edit_singleline(&mut format!("{}", self.settings.data.temp_dir.display()));
                                if ui.button("选择").clicked() {
                                    // 这里可以添加文件选择对话框
                                }
                            });
                        });
                        
                        ui.separator();
                        
                        // 快捷键配置
                        ui.heading("快捷键配置");
                        ui.group(|ui| {
                            ui.label("提示：重启应用后生效");
                            ui.horizontal(|ui| {
                                ui.label("新建标签页：");
                                ui.text_edit_singleline(&mut self.settings.shortcuts.new_tab);
                            });
                            ui.horizontal(|ui| {
                                ui.label("关闭标签页：");
                                ui.text_edit_singleline(&mut self.settings.shortcuts.close_tab);
                            });
                            ui.horizontal(|ui| {
                                ui.label("运行工具：");
                                ui.text_edit_singleline(&mut self.settings.shortcuts.run_tool);
                            });
                            ui.horizontal(|ui| {
                                ui.label("显示/隐藏帮助：");
                                ui.text_edit_singleline(&mut self.settings.shortcuts.toggle_help);
                            });
                            ui.horizontal(|ui| {
                                ui.label("显示/隐藏设置：");
                                ui.text_edit_singleline(&mut self.settings.shortcuts.toggle_settings);
                            });
                        });
                        
                        ui.separator();
                        
                        // 操作按钮
                        ui.horizontal(|ui| {
                            // 恢复默认值按钮
                            if ui.button("恢复默认值").clicked() {
                                self.settings.reset_to_default();
                                self.settings_status = Some((true, "已恢复默认设置".to_string()));
                            }
                            
                            ui.add_space(10.0);
                            
                            // 保存设置按钮
                            if ui.button(egui::RichText::new("保存设置").strong()).clicked() {
                                match self.settings.validate() {
                                    Ok(_) => {
                                        if let Err(e) = self.settings.save() {
                                            self.settings_status = Some((false, format!("保存失败：{}", e)));
                                        } else {
                                            self.settings_status = Some((true, "设置已保存".to_string()));
                                            // 更新应用状态
                                            self.locale = self.settings.display.locale;
                                            self.sidebar_width = self.settings.display.sidebar_width;
                                        }
                                    }
                                    Err(e) => {
                                        self.settings_status = Some((false, format!("验证失败：{}", e)));
                                    }
                                }
                            }
                            
                            ui.add_space(10.0);
                            
                            // 关闭按钮
                            if ui.button("关闭").clicked() {
                                self.show_settings = false;
                            }
                        });
                    });
                });
        }
    }
}

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();
    
    // Load a font that supports Chinese?
    // Egui's default font doesn't support Chinese characters well by default usually?
    // Actually eframe's default font on Windows might be okay, but ideally we should load a font.
    // For MVP, lets assume system font fallback works or just try.
    // If we need custom fonts, we need to configure ctx.set_fonts.
    // Let's first implementation simply and check if we need font config via task.

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust Toolbox",
        options,
        Box::new(|cc| {
            // Setup fonts
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(ToolkitApp::new(cc)))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Embed font from assets
    // Note: include_bytes! path is relative to the file it's in (src/main.rs), so we need to go up to root
    const FONT_DATA: &[u8] = include_bytes!("../../assets/SimHei.ttf");
    let font_name = "EmbeddedSimHei";

    fonts.font_data.insert(
        font_name.to_owned(),
        egui::FontData::from_static(FONT_DATA).tweak(
            egui::FontTweak {
                scale: 1.2,
                ..Default::default()
            }
        ),
    );

    // Set as first priority for Proportional
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        family.insert(0, font_name.to_owned());
    }
    
    // Also add to Monospace as fallback
    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        family.push(font_name.to_owned());
    }
    
    ctx.set_fonts(fonts);
    println!("Loaded embedded font: {}", font_name);
}
