use eframe::egui::{Ui, Response, Color32, RichText, Id, TextEdit, InputState};use std::fmt::Display;

/// 文本输入框类型
enum TextInputType {
    /// 单行文本输入
    SingleLine,
    /// 多行文本输入
    MultiLine,
    /// 密码输入
    Password,
}

/// 文本输入框配置
struct TextInputConfig {
    /// 输入框类型
    input_type: TextInputType,
    /// 输入值
    value: String,
    /// 占位符文本
    placeholder: Option<String>,
    /// 标签文本
    label: Option<String>,
    /// 辅助文本
    helper_text: Option<String>,
    /// 验证状态
    validation_state: ValidationState,
    /// 错误信息
    error_message: Option<String>,
    /// 输入长度限制
    max_length: Option<usize>,
    /// 自动完成
    autocomplete: bool,
    /// 只读状态
    read_only: bool,
    /// 禁用状态
    disabled: bool,
    /// 前缀图标
    prefix_icon: Option<&'static str>,
    /// 后缀图标
    suffix_icon: Option<&'static str>,
    /// 宽度
    width: Option<f32>,
    /// 高度（仅用于多行输入）
    height: Option<f32>,
    /// 输入回调
    on_input: Option<Box<dyn FnMut(&str)>>,
    /// 失焦回调
    on_blur: Option<Box<dyn FnMut(&str)>>,
    /// 聚焦回调
    on_focus: Option<Box<dyn FnMut()>>,
    /// 回车键回调
    on_enter: Option<Box<dyn FnMut(&str)>>,
}

impl Default for TextInputConfig {
    fn default() -> Self {
        Self {
            input_type: TextInputType::SingleLine,
            value: String::new(),
            placeholder: None,
            label: None,
            helper_text: None,
            validation_state: ValidationState::None,
            error_message: None,
            max_length: None,
            autocomplete: true,
            read_only: false,
            disabled: false,
            prefix_icon: None,
            suffix_icon: None,
            width: None,
            height: None,
            on_input: None,
            on_blur: None,
            on_focus: None,
            on_enter: None,
        }
    }
}

/// 验证状态
enum ValidationState {
    /// 无状态
    None,
    /// 成功状态
    Success,
    /// 警告状态
    Warning,
    /// 错误状态
    Error,
}

/// 文本输入框组件
pub struct TextInput {
    /// 文本输入框配置
    config: TextInputConfig,
}

impl TextInput {
    /// 创建新的文本输入框
    pub fn new() -> Self {
        Self {
            config: TextInputConfig::default(),
        }
    }
    
    /// 设置为单行文本输入
    pub fn single_line(mut self) -> Self {
        self.config.input_type = TextInputType::SingleLine;
        self
    }
    
    /// 设置为多行文本输入
    pub fn multi_line(mut self) -> Self {
        self.config.input_type = TextInputType::MultiLine;
        self
    }
    
    /// 设置为密码输入
    pub fn password(mut self) -> Self {
        self.config.input_type = TextInputType::Password;
        self
    }
    
    /// 设置初始值
    pub fn value<T: Display>(mut self, value: T) -> Self {
        self.config.value = value.to_string();
        self
    }
    
    /// 设置占位符文本
    pub fn placeholder<T: Display>(mut self, placeholder: T) -> Self {
        self.config.placeholder = Some(placeholder.to_string());
        self
    }
    
    /// 设置标签文本
    pub fn label<T: Display>(mut self, label: T) -> Self {
        self.config.label = Some(label.to_string());
        self
    }
    
    /// 设置辅助文本
    pub fn helper_text<T: Display>(mut self, helper_text: T) -> Self {
        self.config.helper_text = Some(helper_text.to_string());
        self
    }
    
    /// 设置为成功状态
    pub fn success(mut self) -> Self {
        self.config.validation_state = ValidationState::Success;
        self.config.error_message = None;
        self
    }
    
    /// 设置为警告状态
    pub fn warning<T: Display>(mut self, message: T) -> Self {
        self.config.validation_state = ValidationState::Warning;
        self.config.error_message = Some(message.to_string());
        self
    }
    
    /// 设置为错误状态
    pub fn error<T: Display>(mut self, message: T) -> Self {
        self.config.validation_state = ValidationState::Error;
        self.config.error_message = Some(message.to_string());
        self
    }
    
    /// 设置输入长度限制
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.config.max_length = Some(max_length);
        self
    }
    
    /// 设置是否自动完成
    pub fn autocomplete(mut self, autocomplete: bool) -> Self {
        self.config.autocomplete = autocomplete;
        self
    }
    
    /// 设置为只读状态
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.config.read_only = read_only;
        self
    }
    
    /// 设置为禁用状态
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }
    
    /// 设置前缀图标
    pub fn prefix_icon(mut self, icon: &'static str) -> Self {
        self.config.prefix_icon = Some(icon);
        self
    }
    
    /// 设置后缀图标
    pub fn suffix_icon(mut self, icon: &'static str) -> Self {
        self.config.suffix_icon = Some(icon);
        self
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self
    }
    
    /// 设置高度（仅用于多行输入）
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = Some(height);
        self
    }
    
    /// 设置输入回调
    pub fn on_input<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_input = Some(Box::new(callback));
        self
    }
    
    /// 设置失焦回调
    pub fn on_blur<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_blur = Some(Box::new(callback));
        self
    }
    
    /// 设置聚焦回调
    pub fn on_focus<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_focus = Some(Box::new(callback));
        self
    }
    /// 设置回车键回调
    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_enter = Some(Box::new(callback));
        self
    }
    
    /// 渲染文本输入框
    pub fn render(&mut self, ui: &mut Ui) -> Response {
        let TextInputConfig {
            input_type,
            ref mut value,
            placeholder,
            label,
            helper_text,
            validation_state,
            error_message,
            max_length,
            autocomplete,
            read_only,
            disabled,
            prefix_icon,
            suffix_icon,
            width,
            height,
            ref mut on_input,
            ref mut on_blur,
            ref mut on_focus,
            ref mut on_enter,
        } = &mut self.config;
        
        let response = ui.vertical(|ui| {
            // 渲染标签
            if let Some(label) = label {
                ui.label(label);
            }
            
            // 渲染输入框
            let input_response = ui.horizontal(|ui| {
                // 渲染前缀图标
                if let Some(icon) = prefix_icon {
                    ui.label(icon);
                }
                
                // 创建输入框
                let mut text_edit = match input_type {
                    TextInputType::SingleLine => TextEdit::singleline(value),
                    TextInputType::MultiLine => {
                        let mut edit = TextEdit::multiline(value);
                        if let Some(height) = height {
                            edit = edit.desired_rows((height / ui.spacing().text_edit_line_height).ceil() as usize);
                        }
                        edit
                    },
                    TextInputType::Password => {
                        let mut edit = TextEdit::singleline(value);
                        edit = edit.password();
                        edit
                    },
                };
                
                // 设置占位符
                if let Some(placeholder) = placeholder {
                    text_edit = text_edit.placeholder(placeholder);
                }
                
                // 设置只读和禁用状态
                text_edit = text_edit.read_only(*read_only);
                text_edit = text_edit.enabled(!*disabled);
                
                // 设置宽度
                let input_response = if let Some(width) = width {
                    ui.add_sized([*width, ui.spacing().interact_size.y], text_edit)
                } else {
                    ui.add(text_edit)
                };
                
                // 渲染后缀图标
                if let Some(icon) = suffix_icon {
                    ui.label(icon);
                }
                
                input_response
            });
            
            // 渲染辅助文本和错误信息
            ui.horizontal(|ui| {
                // 渲染输入长度
                if let Some(max_length) = max_length {
                    let length_text = format!("{}/{}", value.len(), max_length);
                    let length_color = if value.len() > *max_length {
                        Color32::RED
                    } else {
                        Color32::GRAY
                    };
                    ui.label(RichText::new(length_text).small().color(length_color));
                }
                
                // 渲染辅助文本
                if let Some(helper_text) = helper_text {
                    ui.label(RichText::new(helper_text).small().color(Color32::GRAY));
                }
                
                // 渲染错误信息
                if let Some(error_message) = error_message {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(error_message).small().color(Color32::RED));
                    });
                }
            });
            
            // 处理输入事件
            if input_response.changed() {
                // 限制输入长度
                if let Some(max_length) = max_length {
                    if value.len() > *max_length {
                        value.truncate(*max_length);
                    }
                }
                
                // 调用输入回调
                if let Some(callback) = on_input.as_mut() {
                    callback(value);
                }
            }
            
            // 处理失焦事件
            if input_response.lost_focus() {
                if let Some(callback) = on_blur.as_mut() {
                    callback(value);
                }
            }
            
            // 处理聚焦事件
            if input_response.gained_focus() {
                if let Some(callback) = on_focus.as_mut() {
                    callback();
                }
            }
            
            // 处理回车键事件
            if input_response.ctx.input_mut(|i| i.key_pressed(egui::Key::Enter)) {
                if let Some(callback) = on_enter.as_mut() {
                    callback(value);
                }
            }
            
            input_response
        });
        
        response.response
    }
    
    /// 获取当前输入值
    pub fn get_value(&self) -> &str {
        &self.config.value
    }
    
    /// 设置输入值
    pub fn set_value<T: Display>(&mut self, value: T) {
        self.config.value = value.to_string();
    }
    
    /// 更新输入值
    pub fn update_value(&mut self, value: &str) {
        self.config.value = value.to_string();
    }
    
    /// 清除输入值
    pub fn clear(&mut self) {
        self.config.value.clear();
    }
    
    /// 获取验证状态
    pub fn get_validation_state(&self) -> &ValidationState {
        &self.config.validation_state
    }
}

/// 文本输入框构建器
pub struct TextInputBuilder {
    /// 文本输入框配置
    config: TextInputConfig,
}

impl TextInputBuilder {
    /// 创建新的文本输入框构建器
    pub fn new() -> Self {
        Self {
            config: TextInputConfig::default(),
        }
    }
    
    /// 设置为单行文本输入
    pub fn single_line(mut self) -> Self {
        self.config.input_type = TextInputType::SingleLine;
        self
    }
    
    /// 设置为多行文本输入
    pub fn multi_line(mut self) -> Self {
        self.config.input_type = TextInputType::MultiLine;
        self
    }
    
    /// 设置为密码输入
    pub fn password(mut self) -> Self {
        self.config.input_type = TextInputType::Password;
        self
    }
    
    /// 设置初始值
    pub fn value<T: Display>(mut self, value: T) -> Self {
        self.config.value = value.to_string();
        self
    }
    
    /// 设置占位符文本
    pub fn placeholder<T: Display>(mut self, placeholder: T) -> Self {
        self.config.placeholder = Some(placeholder.to_string());
        self
    }
    
    /// 设置标签文本
    pub fn label<T: Display>(mut self, label: T) -> Self {
        self.config.label = Some(label.to_string());
        self
    }
    
    /// 设置辅助文本
    pub fn helper_text<T: Display>(mut self, helper_text: T) -> Self {
        self.config.helper_text = Some(helper_text.to_string());
        self
    }
    
    /// 设置为成功状态
    pub fn success(mut self) -> Self {
        self.config.validation_state = ValidationState::Success;
        self.config.error_message = None;
        self
    }
    
    /// 设置为警告状态
    pub fn warning<T: Display>(mut self, message: T) -> Self {
        self.config.validation_state = ValidationState::Warning;
        self.config.error_message = Some(message.to_string());
        self
    }
    
    /// 设置为错误状态
    pub fn error<T: Display>(mut self, message: T) -> Self {
        self.config.validation_state = ValidationState::Error;
        self.config.error_message = Some(message.to_string());
        self
    }
    
    /// 设置输入长度限制
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.config.max_length = Some(max_length);
        self
    }
    
    /// 设置是否自动完成
    pub fn autocomplete(mut self, autocomplete: bool) -> Self {
        self.config.autocomplete = autocomplete;
        self
    }
    
    /// 设置为只读状态
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.config.read_only = read_only;
        self
    }
    
    /// 设置为禁用状态
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.config.disabled = disabled;
        self
    }
    
    /// 设置前缀图标
    pub fn prefix_icon(mut self, icon: &'static str) -> Self {
        self.config.prefix_icon = Some(icon);
        self
    }
    
    /// 设置后缀图标
    pub fn suffix_icon(mut self, icon: &'static str) -> Self {
        self.config.suffix_icon = Some(icon);
        self
    }
    
    /// 设置宽度
    pub fn width(mut self, width: f32) -> Self {
        self.config.width = Some(width);
        self
    }
    
    /// 设置高度（仅用于多行输入）
    pub fn height(mut self, height: f32) -> Self {
        self.config.height = Some(height);
        self
    }
    
    /// 设置输入回调
    pub fn on_input<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_input = Some(Box::new(callback));
        self
    }
    
    /// 设置失焦回调
    pub fn on_blur<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_blur = Some(Box::new(callback));
        self
    }
    
    /// 设置聚焦回调
    pub fn on_focus<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.config.on_focus = Some(Box::new(callback));
        self
    }
    
    /// 设置回车键回调
    pub fn on_enter<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.config.on_enter = Some(Box::new(callback));
        self
    }
    
    /// 构建文本输入框
    pub fn build(self) -> TextInput {
        TextInput {
            config: self.config,
        }
    }
}
