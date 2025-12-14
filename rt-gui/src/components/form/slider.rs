use eframe::egui::{Response, Ui, Widget};

/// 滑块类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderType {
    /// 整数类型
    Integer,
    /// 浮点数类型
    Float,
}

/// 滑块样式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderStyle {
    /// 默认样式
    Default,
    /// 带边框的样式
    Bordered,
    /// 紧凑样式
    Compact,
}

/// 滑块组件配置
#[derive(Debug, Clone)]
pub struct SliderConfig {
    /// 滑块标签
    pub label: Option<String>,
    /// 滑块类型
    pub slider_type: SliderType,
    /// 当前值
    pub value: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 步长
    pub step: f64,
    /// 是否禁用
    pub disabled: bool,
    /// 滑块样式
    pub style: SliderStyle,
    /// 帮助文本
    pub helper_text: Option<String>,
    /// 是否为必填项
    pub required: bool,
    /// 验证状态
    pub validation_state: Option<ValidationState>,
    /// 值变化回调
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

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            label: None,
            slider_type: SliderType::Float,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            disabled: false,
            style: SliderStyle::Default,
            helper_text: None,
            required: false,
            validation_state: None,
            on_change: None,
        }
    }
}

/// 滑块组件构建器
pub struct SliderBuilder {
    config: SliderConfig,
}

impl SliderBuilder {
    /// 创建新的滑块构建器
    pub fn new() -> Self {
        Self {
            config: SliderConfig::default(),
        }
    }

    /// 设置滑块标签
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.config.label = Some(label.into());
        self
    }

    /// 设置滑块类型
    pub fn slider_type(mut self, slider_type: SliderType) -> Self {
        self.config.slider_type = slider_type;
        self
    }

    /// 设置初始值
    pub fn value(mut self, value: f64) -> Self {
        self.config.value = value;
        self
    }

    /// 设置最小值
    pub fn min(mut self, min: f64) -> Self {
        self.config.min = min;
        self
    }

    /// 设置最大值
    pub fn max(mut self, max: f64) -> Self {
        self.config.max = max;
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

    /// 设置滑块样式
    pub fn style(mut self, style: SliderStyle) -> Self {
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

    /// 设置值变化回调函数
    pub fn on_change(mut self, callback: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.config.on_change = Some(Box::new(callback));
        self
    }

    /// 构建滑块组件
    pub fn build(self, ui: &mut Ui, value: &mut f64) -> Response {
        let config = self.config;
        
        // 开始布局
        let response = ui.vertical(|ui| {
            // 添加标签和当前值显示
            if let Some(label) = &config.label {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label)
                        .color(if config.disabled {
                            ui.visuals().disabled_text_color()
                        } else {
                            ui.visuals().text_color()
                        })
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let display_value = match config.slider_type {
                            SliderType::Integer => value.round().to_string(),
                            SliderType::Float => format!("{:.2}", value),
                        };
                        ui.label(egui::RichText::new(display_value)
                            .color(if config.disabled {
                                ui.visuals().disabled_text_color()
                            } else {
                                ui.visuals().text_color()
                            })
                        );
                    });
                });
            }
            
            // 创建滑块
            let slider_response = match config.style {
                SliderStyle::Default => {
                    match config.slider_type {
                        SliderType::Integer => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .integer()
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                        SliderType::Float => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                    }
                },
                SliderStyle::Bordered => {
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color);
                    match config.slider_type {
                        SliderType::Integer => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .integer()
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                        SliderType::Float => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                    }
                },
                SliderStyle::Compact => {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    match config.slider_type {
                        SliderType::Integer => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .integer()
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                        SliderType::Float => {
                            ui.add(egui::Slider::new(value, config.min..=config.max)
                                .step_by(config.step)
                                .sense(egui::Sense::click_and_drag())
                            )
                        },
                    }
                },
            };
            
            // 处理值变化事件
            if slider_response.changed() && !config.disabled {
                if let Some(callback) = config.on_change {
                    callback(*value);
                }
            }
            
            slider_response
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

/// 滑块组件
pub struct Slider {
    config: SliderConfig,
}

impl Slider {
    /// 创建新的滑块构建器
    pub fn builder() -> SliderBuilder {
        SliderBuilder::new()
    }
}

impl Widget for Slider {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut value = self.config.value;
        self.builder()
            .label(self.config.label.unwrap_or_default())
            .slider_type(self.config.slider_type)
            .value(value)
            .min(self.config.min)
            .max(self.config.max)
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
