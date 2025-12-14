use eframe::egui::{Response, Ui, Widget}; 

/// 开关样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleSwitchStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 开关组件配置
#[derive(Debug, Clone)]
pub struct ToggleSwitchConfig {
    /// 开关标签
    pub label: Option<String>,
    /// 当前状态
    pub checked: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 开关样式
    pub style: ToggleSwitchStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 状态变化回调
    pub on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
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

impl Default for ToggleSwitchConfig {
    fn default() -> Self {
        Self {
            label: None,
            checked: false,
            disabled: false,
            style: ToggleSwitchStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            on_change: None,
        }
    }
}

/// 开关组件构建器
pub struct ToggleSwitchBuilder {
    config: ToggleSwitchConfig,
}

impl ToggleSwitchBuilder {
    /// 创建新的开关构建器
    pub fn new() -> Self {
        Self {
            config: ToggleSwitchConfig::default(),
        }
    }

    /// 设置开关标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置初始状态
    pub fn checked(mut self, checked: bool) -> Self {
        self.config.checked = checked;
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置开关样式
    pub fn style(mut self, style: ToggleSwitchStyle) -> Self {
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

    /// 设置状态变化回调函数
    pub fn on_change(mut self, callback: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.config.on_change = Some(Box::new(callback));
        self
    }

    /// 构建开关组件
    pub fn build(self, ui: &mut Ui, checked: &mut bool) -> Response {
        let config = self.config;
        
        // 开始布局
        let response = ui.vertical(|ui| {
            // 创建水平布局
            let horizontal_response = ui.horizontal(|ui| {
                // 创建开关
                let toggle_response = match config.style {
                    ToggleSwitchStyle::Default => {
                        ui.add(egui::Toggle::new(checked, ""))
                    },
                    ToggleSwitchStyle::Bordered => {
                        ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                        ui.add(egui::Toggle::new(checked, ""))
                    },
                    ToggleSwitchStyle::Compact => {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        ui.add(egui::Toggle::new(checked, ""))
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
                
                toggle_response
            });
            
            horizontal_response.outer
        });
        
        // 处理状态变化事件
        if response.inner.changed() && !config.disabled {
            if let Some(callback) = config.on_change {
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

/// 开关组件
pub struct ToggleSwitch {
    config: ToggleSwitchConfig,
}

impl ToggleSwitch {
    /// 创建新的开关构建器
    pub fn builder() -> ToggleSwitchBuilder {
        ToggleSwitchBuilder::new()
    }
}

impl Widget for ToggleSwitch {
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
            .on_change(move |new_state| checked = new_state)
            .build(ui, &mut checked)
    }
}
