use eframe::egui::{Response, Ui, Widget};

/// 文件输入框类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInputType {
    /// 文件选择类型
    File,
    /// 目录选择类型
    Directory,
    /// 多文件选择类型
    MultipleFiles,
}

/// 文件输入框样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileInputStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 文件输入框组件配置
#[derive(Debug, Clone)]
pub struct FileInputConfig {
    /// 文件输入框标签
    pub label: Option<String>,
    /// 文件输入框类型
    pub input_type: FileInputType,
    /// 当前选中的文件路径
    pub selected_path: String,
    /// 当前选中的多个文件路径
    pub selected_paths: Vec<String>,
    /// 文件过滤器（例如：".txt,.md"）
    pub filters: Option<String>,
    /// 是否禁用
    pub disabled: bool,
    /// 文件输入框样式
    pub style: FileInputStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 选择文件回调
    pub on_select: Option<Box<dyn Fn(Vec<String>) + Send + Sync>>,
}

/// 验证状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    /// 成功状态
    Success,
    /// 警告状态
    Warning,
    /// 错误状态
    Error,
}

impl Default for FileInputConfig {
    fn default() -> Self {
        Self {
            label: None,
            input_type: FileInputType::File,
            selected_path: String::new(),
            selected_paths: Vec::new(),
            filters: None,
            disabled: false,
            style: FileInputStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            on_select: None,
        }
    }
}

/// 文件输入框组件构建器
pub struct FileInputBuilder {
    config: FileInputConfig,
}

impl FileInputBuilder {
    /// 创建新的文件输入框构建器
    pub fn new() -> Self {
        Self {
            config: FileInputConfig::default(),
        }
    }

    /// 设置文件输入框标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置文件输入框类型
    pub fn input_type(mut self, input_type: FileInputType) -> Self {
        self.config.input_type = input_type;
        self
    }

    /// 设置当前选中的文件路径
    pub fn selected_path(mut self, path: impl Into<String>) -> Self {
        self.config.selected_path = path.into();
        self
    }

    /// 设置当前选中的多个文件路径
    pub fn selected_paths(mut self, paths: Vec<String>) -> Self {
        self.config.selected_paths = paths;
        self
    }

    /// 设置文件过滤器
    pub fn filters(mut self, filters: impl Into<String>) -> Self {
        self.config.filters = Some(filters.into());
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置文件输入框样式
    pub fn style(mut self, style: FileInputStyle) -> Self {
        self.config.style = style;
        self
    }

    /// 设置帮助文本
    pub fn helper_text(mut self, helper_text: impl Into<String>) -> Self {
        self.config.helper_text = Some(helper_text.into());
        self
    }

    /// 设置是否为必填项
    pub fn required(mut self, required: bool) -> Self {
        self.config.required = required;
        self
    }

    /// 设置验证状态
    pub fn validation_state(mut self, state: ValidationState) -> Self {
        self.config.validation_state = Some(state);
        self
    }

    /// 设置选择文件回调函数
    pub fn on_select(mut self, callback: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        self.config.on_select = Some(Box::new(callback));
        self
    }

    /// 构建文件输入框组件
    pub fn build(self, ui: &mut Ui, selected_paths: &mut Vec<String>) -> Response {
        let config = self.config;
        
        // 开始布局
        let response = ui.vertical(|ui| {
            // 添加标签
            if let Some(label) = &config.label {
                ui.label(egui::RichText::new(label)
                    .color(if config.disabled {
                        ui.visuals().disabled_text_color()
                    } else {
                        ui.visuals().text_color()
                    })
                );
            }
            
            // 创建水平布局
            let horizontal_response = ui.horizontal(|ui| {
                // 创建文本显示区域
                let display_text = match config.input_type {
                    FileInputType::File => selected_paths.first().cloned().unwrap_or_default(),
                    FileInputType::Directory => selected_paths.first().cloned().unwrap_or_default(),
                    FileInputType::MultipleFiles => {
                        if selected_paths.is_empty() {
                            "未选择任何文件".to_string()
                        } else {
                            format!("已选择 {} 个文件", selected_paths.len())
                        }
                    },
                };
                
                let display_response = match config.style {
                    FileInputStyle::Default => {
                        ui.add_enabled(false, egui::TextEdit::singleline(&mut display_text.clone()))
                    },
                    FileInputStyle::Bordered => {
                        ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                        ui.add_enabled(false, egui::TextEdit::singleline(&mut display_text.clone()))
                    },
                    FileInputStyle::Compact => {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.add_enabled(false, egui::TextEdit::singleline(&mut display_text.clone()))
                    },
                };
                
                // 创建选择按钮
                let button_text = match config.input_type {
                    FileInputType::File => "选择文件",
                    FileInputType::Directory => "选择目录",
                    FileInputType::MultipleFiles => "选择多个文件",
                };
                
                let button_response = ui.button(button_text);
                
                // 处理按钮点击事件
                if button_response.clicked() && !config.disabled {
                    // 这里需要调用系统文件选择器，目前egui不直接支持，需要使用egui_file或其他库
                    // 暂时只做模拟实现
                    let mock_path = "C:\\example\\file.txt";
                    *selected_paths = vec![mock_path.to_string()];
                    
                    if let Some(callback) = &config.on_select {
                        callback(selected_paths.clone());
                    }
                }
                
                display_response
            });
            
            horizontal_response.outer
        });
        
        // 添加帮助文本
        if let Some(helper_text) = &config.helper_text {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(helper_text)
                    .size(ui.visuals().small_font_size)
                    .color(match config.validation_state {
                        Some(ValidationState::Success) => ui.visuals().widgets.active.fg_stroke.color,
                        Some(ValidationState::Warning) => egui::Color32::from_rgb(255, 165, 0),
                        Some(ValidationState::Error) => egui::Color32::from_rgb(255, 82, 82),
                        None => ui.visuals().text_color(),
                    })
                );
            });
        }
        
        response.outer
    }
}

/// 文件输入框组件
pub struct FileInput {
    config: FileInputConfig,
}

impl FileInput {
    /// 创建新的文件输入框构建器
    pub fn builder() -> FileInputBuilder {
        FileInputBuilder::new()
    }
}

impl Widget for FileInput {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut selected_paths = self.config.selected_paths.clone();
        self.builder()
            .label(self.config.label.unwrap_or_default())
            .input_type(self.config.input_type)
            .selected_path(self.config.selected_path.clone())
            .selected_paths(selected_paths.clone())
            .filters(self.config.filters.unwrap_or_default())
            .disabled(self.config.disabled)
            .style(self.config.style)
            .helper_text(self.config.helper_text.unwrap_or_default())
            .required(self.config.required)
            .validation_state(self.config.validation_state.unwrap_or_default())
            .on_select(move |new_paths| selected_paths = new_paths)
            .build(ui, &mut selected_paths)
    }
}
