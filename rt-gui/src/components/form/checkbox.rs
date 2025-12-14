use eframe::egui::{Response, Ui, Widget};

/// 复选框样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 复选框组件配置
#[derive(Debug, Clone)]
pub struct CheckboxConfig {
    /// 复选框标签
    pub label: Option<String>,
    /// 复选框状态
    pub checked: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 复选框样式
    pub style: CheckboxStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 点击回调
    pub on_click: Option<Box<dyn Fn(bool) + Send + Sync>>,
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

impl Default for CheckboxConfig {
    fn default() -> Self {
        Self {
            label: None,
            checked: false,
            disabled: false,
            style: CheckboxStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            on_click: None,
        }
    }
}

/// 复选框组件构建器
pub struct CheckboxBuilder {
    config: CheckboxConfig,
}

impl CheckboxBuilder {
    /// 创建新的复选框构建器
    pub fn new() -> Self {
        Self {
            config: CheckboxConfig::default(),
        }
    }

    /// 设置复选框标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置复选框初始状态
    pub fn checked(mut self, checked: bool) -> Self {
        self.config.checked = checked;
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置复选框样式
    pub fn style(mut self, style: CheckboxStyle) -> Self {
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

    /// 设置点击回调函数
    pub fn on_click(mut self, callback: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.config.on_click = Some(Box::new(callback));
        self
    }

    /// 构建复选框组件
    pub fn build(self, ui: &mut Ui, checked: &mut bool) -> Response {
        let config = self.config;
        
        // 开始布局
        let response = ui.horizontal(|ui| {
            // 创建egui复选框
            let checkbox = egui::Checkbox::new(checked, "")
                .sense(egui::Sense::click());
            
            let checkbox_response = match config.style {
                CheckboxStyle::Default => checkbox.ui(ui),
                CheckboxStyle::Bordered => {
                    ui.add(checkbox)
                },
                CheckboxStyle::Compact => {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.add(checkbox)
                },
            };
            
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
            
            checkbox_response
        });
        
        // 处理点击事件
        if response.inner.clicked() && !config.disabled {
            if let Some(callback) = config.on_click {
                callback(*checked);
            }
        }
        
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

/// 复选框组件
pub struct Checkbox {
    config: CheckboxConfig,
}

impl Checkbox {
    /// 创建新的复选框构建器
    pub fn builder() -> CheckboxBuilder {
        CheckboxBuilder::new()
    }
}

impl Widget for Checkbox {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut checked = self.config.checked;
        self.builder()
            .label(self.config.label.unwrap_or_default())
            .checked(checked)
            .disabled(self.config.disabled)
            .style(self.config.style)
            .helper_text(self.config.helper_text.unwrap_or_default())
            .required(self.config.required)
            .validation_state(self.config.validation_state.unwrap_or_default())
            .on_click(move |new_state| checked = new_state)
            .build(ui, &mut checked)
    }
}
