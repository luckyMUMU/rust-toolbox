use eframe::egui::{Response, Ui, Widget};

/// 下拉列表样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 下拉列表选项
#[derive(Debug, Clone)]
pub struct DropdownOption {
    /// 选项值
    pub value: String,
    /// 选项显示文本
    pub label: String,
    /// 是否禁用
    pub disabled: bool,
}

impl DropdownOption {
    /// 创建新的下拉列表选项
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
    
    /// 设置选项是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 下拉列表组件配置
#[derive(Debug, Clone)]
pub struct DropdownConfig {
    /// 下拉列表标签
    pub label: Option<String>,
    /// 下拉列表选项
    pub options: Vec<DropdownOption>,
    /// 当前选中值
    pub selected_value: String,
    /// 是否禁用
    pub disabled: bool,
    /// 下拉列表样式
    pub style: DropdownStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 是否支持搜索
    pub searchable: bool,
    /// 占位文本
    pub placeholder: Option<String>,
    /// 选择回调
    pub on_select: Option<Box<dyn Fn(String) + Send + Sync>>,
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

impl Default for DropdownConfig {
    fn default() -> Self {
        Self {
            label: None,
            options: Vec::new(),
            selected_value: String::new(),
            disabled: false,
            style: DropdownStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            searchable: false,
            placeholder: None,
            on_select: None,
        }
    }
}

/// 下拉列表组件构建器
pub struct DropdownBuilder {
    config: DropdownConfig,
}

impl DropdownBuilder {
    /// 创建新的下拉列表构建器
    pub fn new() -> Self {
        Self {
            config: DropdownConfig::default(),
        }
    }

    /// 设置下拉列表标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置下拉列表选项
    pub fn options(mut self, options: Vec<DropdownOption>) -> Self {
        self.config.options = options;
        self
    }

    /// 添加单个选项
    pub fn add_option(mut self, option: DropdownOption) -> Self {
        self.config.options.push(option);
        self
    }

    /// 设置当前选中值
    pub fn selected_value(mut self, selected_value: impl Into<String>) -> Self {
        self.config.selected_value = selected_value.into();
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置下拉列表样式
    pub fn style(mut self, style: DropdownStyle) -> Self {
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

    /// 设置是否支持搜索
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.config.searchable = searchable;
        self
    }

    /// 设置占位文本
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.config.placeholder = Some(placeholder.into());
        self
    }

    /// 设置选择回调函数
    pub fn on_select(mut self, callback: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.config.on_select = Some(Box::new(callback));
        self
    }

    /// 构建下拉列表组件
    pub fn build(self, ui: &mut Ui, selected_value: &mut String) -> Response {
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
            
            // 创建下拉列表
            let selected_label = config.options.iter()
                .find(|option| option.value == *selected_value)
                .map(|option| option.label.clone())
                .unwrap_or_default();
            
            let combo_box_response = match config.style {
                DropdownStyle::Default => {
                    ui.add(egui::ComboBox::from_id_source("dropdown")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for option in &config.options {
                                if option.disabled {
                                    ui.add_enabled(false, egui::SelectableLabel::new(
                                        option.value == *selected_value,
                                        &option.label
                                    ));
                                } else {
                                    if ui.selectable_label(
                                        option.value == *selected_value,
                                        &option.label
                                    ).clicked() {
                                        *selected_value = option.value.clone();
                                        if let Some(callback) = &config.on_select {
                                            callback(option.value.clone());
                                        }
                                    }
                                }
                            }
                        })
                        .response
                    )
                },
                DropdownStyle::Bordered => {
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                    ui.add(egui::ComboBox::from_id_source("dropdown")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for option in &config.options {
                                if option.disabled {
                                    ui.add_enabled(false, egui::SelectableLabel::new(
                                        option.value == *selected_value,
                                        &option.label
                                    ));
                                } else {
                                    if ui.selectable_label(
                                        option.value == *selected_value,
                                        &option.label
                                    ).clicked() {
                                        *selected_value = option.value.clone();
                                        if let Some(callback) = &config.on_select {
                                            callback(option.value.clone());
                                        }
                                    }
                                }
                            }
                        })
                        .response
                    )
                },
                DropdownStyle::Compact => {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.add(egui::ComboBox::from_id_source("dropdown")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for option in &config.options {
                                if option.disabled {
                                    ui.add_enabled(false, egui::SelectableLabel::new(
                                        option.value == *selected_value,
                                        &option.label
                                    ));
                                } else {
                                    if ui.selectable_label(
                                        option.value == *selected_value,
                                        &option.label
                                    ).clicked() {
                                        *selected_value = option.value.clone();
                                        if let Some(callback) = &config.on_select {
                                            callback(option.value.clone());
                                        }
                                    }
                                }
                            }
                        })
                        .response
                    )
                },
            };
            
            combo_box_response
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

/// 下拉列表组件
pub struct Dropdown {
    config: DropdownConfig,
}

impl Dropdown {
    /// 创建新的下拉列表构建器
    pub fn builder() -> DropdownBuilder {
        DropdownBuilder::new()
    }
}

impl Widget for Dropdown {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut selected_value = self.config.selected_value.clone();
        self.builder()
            .label(self.config.label.unwrap_or_default())
            .options(self.config.options.clone())
            .selected_value(selected_value.clone())
            .disabled(self.config.disabled)
            .style(self.config.style)
            .helper_text(self.config.helper_text.unwrap_or_default())
            .required(self.config.required)
            .validation_state(self.config.validation_state.unwrap_or_default())
            .searchable(self.config.searchable)
            .placeholder(self.config.placeholder.unwrap_or_default())
            .on_select(move |new_value| selected_value = new_value)
            .build(ui, &mut selected_value)
    }
}
