use eframe::egui::{Response, Ui, Widget};

/// 数字输入框类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberInputType {
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
}

/// 数字输入框样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberInputStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 数字输入框组件配置
#[derive(Debug, Clone)]
pub struct NumberInputConfig {
    /// 数字输入框标签
    pub label: Option<String>,
    /// 数字输入框类型
    pub input_type: NumberInputType,
    /// 当前值（字符串表示）
    pub value: String,
    /// 最小值
    pub min: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
    /// 步长
    pub step: f64,
    /// 是否禁用
    pub disabled: bool,
    /// 数字输入框样式
    pub style: NumberInputStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 输入变化回调
    pub on_change: Option<Box<dyn Fn(f64) + Send + Sync>>,
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

impl Default for NumberInputConfig {
    fn default() -> Self {
        Self {
            label: None,
            input_type: NumberInputType::Float,
            value: "0".to_string(),
            min: None,
            max: None,
            step: 1.0,
            disabled: false,
            style: NumberInputStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            on_change: None,
        }
    }
}

/// 数字输入框组件构建器
pub struct NumberInputBuilder {
    config: NumberInputConfig,
}

impl NumberInputBuilder {
    /// 创建新的数字输入框构建器
    pub fn new() -> Self {
        Self {
            config: NumberInputConfig::default(),
        }
    }

    /// 设置数字输入框标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置数字输入框类型
    pub fn input_type(mut self, input_type: NumberInputType) -> Self {
        self.config.input_type = input_type;
        self
    }

    /// 设置初始值
    pub fn value(mut self, value: f64) -> Self {
        self.config.value = match self.config.input_type {
            NumberInputType::Integer => value.round().to_string(),
            NumberInputType::Float => value.to_string(),
        };
        self
    }

    /// 设置最小值
    pub fn min(mut self, min: f64) -> Self {
        self.config.min = Some(min);
        self
    }

    /// 设置最大值
    pub fn max(mut self, max: f64) -> Self {
        self.config.max = Some(max);
        self
    }

    /// 设置步长
    pub fn step(mut self, step: f64) -> Self {
        self.config.step = step;
        self
    }

    /// 设置是否禁用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }

    /// 设置数字输入框样式
    pub fn style(mut self, style: NumberInputStyle) -> Self {
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

    /// 设置输入变化回调函数
    pub fn on_change(mut self, callback: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.config.on_change = Some(Box::new(callback));
        self
    }

    /// 构建数字输入框组件
    pub fn build(self, ui: &mut Ui, value: &mut f64) -> Response {
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
            
            // 创建文本输入框
            let mut text_value = match config.input_type {
                NumberInputType::Integer => value.round().to_string(),
                NumberInputType::Float => value.to_string(),
            };
            
            let text_response = match config.style {
                NumberInputStyle::Default => {
                    ui.text_edit_singleline(&mut text_value)
                },
                NumberInputStyle::Bordered => {
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                    ui.text_edit_singleline(&mut text_value)
                },
                NumberInputStyle::Compact => {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.text_edit_singleline(&mut text_value)
                },
            };
            
            // 验证输入并转换为数字
            if let Ok(parsed_value) = text_value.parse::<f64>() {
                let mut new_value = parsed_value;
                
                // 应用整数限制
                if let NumberInputType::Integer = config.input_type {
                    new_value = new_value.round();
                }
                
                // 应用范围限制
                if let Some(min) = config.min {
                    new_value = new_value.max(min);
                }
                if let Some(max) = config.max {
                    new_value = new_value.min(max);
                }
                
                // 更新值并触发回调
                if *value != new_value {
                    *value = new_value;
                    if let Some(callback) = &config.on_change {
                        callback(new_value);
                    }
                }
            }
            
            text_response
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

/// 数字输入框组件
pub struct NumberInput {
    config: NumberInputConfig,
}

impl NumberInput {
    /// 创建新的数字输入框构建器
    pub fn builder() -> NumberInputBuilder {
        NumberInputBuilder::new()
    }
}

impl Widget for NumberInput {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut value = self.config.value.parse::<f64>().unwrap_or(0.0);
        self.builder()
            .label(self.config.label.unwrap_or_default())
            .input_type(self.config.input_type)
            .value(value)
            .min(self.config.min.unwrap_or(-f64::MAX))
            .max(self.config.max.unwrap_or(f64::MAX))
            .step(self.config.step)
            .disabled(self.config.disabled)
            .style(self.config.style)
            .helper_text(self.config.helper_text.unwrap_or_default())
            .required(self.config.required)
            .validation_state(self.config.validation_state.unwrap_or_default())
            .on_change(move |new_value| value = new_value)
            .build(ui, &mut value)
    }
}
