use eframe::egui;
use serde_json::Value;

/// 布局类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LayoutType {
    /// 单栏布局：输入区域在上，输出区域在下
    SingleColumn,
    /// 双栏布局：左侧输入，右侧输出（默认）
    #[default]
    TwoColumns,
    /// 三栏布局：左侧输入，中间预览，右侧输出
    ThreeColumns,
}

impl LayoutType {
    /// 获取布局类型的显示名称
    pub fn display_name(self) -> &'static str {
        match self {
            LayoutType::SingleColumn => "单栏布局",
            LayoutType::TwoColumns => "双栏布局",
            LayoutType::ThreeColumns => "三栏布局",
        }
    }
    
    /// 获取所有布局类型
    pub fn all() -> [LayoutType; 3] {
        [
            LayoutType::SingleColumn,
            LayoutType::TwoColumns,
            LayoutType::ThreeColumns,
        ]
    }
}

/// 布局管理器
#[derive(Default)]
pub struct LayoutManager {
    /// 当前布局类型
    pub current_layout: LayoutType,
}

impl LayoutManager {
    /// 创建新的布局管理器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 切换布局类型
    #[allow(dead_code)]
    pub fn switch_layout(&mut self, layout_type: LayoutType) {
        self.current_layout = layout_type;
    }
    
    /// 渲染布局切换按钮
    pub fn render_layout_switcher(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("布局：");
            egui::ComboBox::from_id_salt("layout_switcher")
                .selected_text(self.current_layout.display_name())
                .show_ui(ui, |ui| {
                    for layout_type in LayoutType::all() {
                        ui.selectable_value(
                            &mut self.current_layout,
                            layout_type,
                            layout_type.display_name()
                        );
                    }
                });
        });
    }
    
    /// 渲染单栏布局
    pub fn render_single_column(
        &self,
        ui: &mut egui::Ui,
        input_widget: impl FnOnce(&mut egui::Ui),
        output_widget: impl FnOnce(&mut egui::Ui),
        _preview_widget: Option<impl FnOnce(&mut egui::Ui)>,
    ) {
        // 单栏布局：输入在上，输出在下
        ui.vertical(|ui| {
            // 输入区域
            input_widget(ui);
            
            ui.separator();
            
            // 预览区域（可选）
            if let Some(preview) = _preview_widget {
                preview(ui);
                ui.separator();
            }
            
            // 输出区域
            output_widget(ui);
        });
    }
    
    /// 渲染双栏布局
    pub fn render_two_columns(
        &self,
        ui: &mut egui::Ui,
        input_widget: impl FnOnce(&mut egui::Ui),
        output_widget: impl FnOnce(&mut egui::Ui),
        _preview_widget: Option<impl FnOnce(&mut egui::Ui)>,
    ) {
        // 双栏布局：左侧输入，右侧输出
        ui.columns(2, |columns| {
            // 左侧：输入区域
            columns[0].vertical(|ui| {
                input_widget(ui);
            });
            
            // 右侧：输出区域
            columns[1].vertical(|ui| {
                output_widget(ui);
            });
        });
    }
    
    /// 渲染三栏布局
    pub fn render_three_columns(
        &self,
        ui: &mut egui::Ui,
        input_widget: impl FnOnce(&mut egui::Ui),
        output_widget: impl FnOnce(&mut egui::Ui),
        _preview_widget: Option<impl FnOnce(&mut egui::Ui)>,
    ) {
        // 三栏布局：左侧输入，中间预览，右侧输出
        ui.columns(3, |columns| {
            // 左侧：输入区域
            columns[0].vertical(|ui| {
                input_widget(ui);
            });
            
            // 中间：预览区域
            columns[1].vertical(|ui| {
                if let Some(preview) = _preview_widget {
                    preview(ui);
                } else {
                    ui.label("预览区域");
                }
            });
            
            // 右侧：输出区域
            columns[2].vertical(|ui| {
                output_widget(ui);
            });
        });
    }
    
    /// 根据当前布局类型渲染内容
    pub fn render_content(
        &self,
        ui: &mut egui::Ui,
        input_widget: impl FnOnce(&mut egui::Ui),
        output_widget: impl FnOnce(&mut egui::Ui),
        preview_widget: Option<impl FnOnce(&mut egui::Ui)>,
    ) {
        match self.current_layout {
            LayoutType::SingleColumn => {
                self.render_single_column(ui, input_widget, output_widget, preview_widget);
            },
            LayoutType::TwoColumns => {
                self.render_two_columns(ui, input_widget, output_widget, preview_widget);
            },
            LayoutType::ThreeColumns => {
                self.render_three_columns(ui, input_widget, output_widget, preview_widget);
            },
        }
    }
}

/// 渲染JSON Schema表单的辅助函数
pub fn render_schema(ui: &mut egui::Ui, schema: &Value, data: &mut Value, read_only: bool, path: &str) {
    // 确保schema有type字段
    let obj_type = schema.get("type").and_then(|v| v.as_str()).unwrap_or("object");
    
    match obj_type {
        "object" => {
            // 确保数据是对象类型
            if !data.is_object() {
                *data = serde_json::json!({});
            }
            
            if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                for (key, prop_schema) in props {
                    // 使用垂直布局代替水平布局，避免表单内容被截断
                    ui.vertical(|ui| {
                        // 确保数据有这个键（仅在编辑模式下）
                        if !read_only && data.get(key).is_none() {
                            // 根据类型初始化安全默认值
                            let default = match prop_schema.get("type").and_then(|v| v.as_str()) {
                                Some("string") => serde_json::json!(""),
                                Some("boolean") => serde_json::json!(false),
                                Some("integer") | Some("number") => serde_json::json!(0),
                                Some("object") => serde_json::json!({}),
                                Some("array") => serde_json::json!([]),
                                _ => serde_json::json!(null),
                            };
                            data.as_object_mut().unwrap().insert(key.clone(), default);
                        }
                        
                        // 确保数据是对象类型，避免崩溃
                        if let Some(val) = data.get_mut(key) {
                            // 使用标题（如果有），否则使用key
                            let label_text = prop_schema.get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or(key);
                            
                            ui.label(label_text);
                            
                            let child_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                            // 使用缩进和分组来提升表单的可读性
                            ui.group(|ui| {
                                render_schema(ui, prop_schema, val, read_only, &child_path);
                            });
                        } else {
                            ui.weak("(null)");
                        }
                    });
                }
            }
        },
        "string" => {
            // 检查该字段是否有枚举约束
            if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_array()) {
                // 尝试获取标签
                let labels = schema.get("x-enum-labels").and_then(|v| v.as_object());

                // 对于枚举字段，渲染为组合框
                if let Some(s) = data.as_str() {
                    let original = s.to_string();  // 在闭包前克隆
                    let mut selected = original.clone();
                    
                    let current_label = labels
                        .and_then(|l| l.get(&selected))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| selected.clone());

                    if read_only {
                        ui.label(&current_label);
                    } else {
                        ui.push_id(path, |ui| {
                            egui::ComboBox::from_id_salt(path)
                                .selected_text(&current_label)
                                .show_ui(ui, |ui| {
                                    for enum_val in enum_values {
                                        if let Some(val_str) = enum_val.as_str() {
                                            let label = labels
                                                .and_then(|l| l.get(val_str))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(val_str);
                                            
                                            ui.selectable_value(&mut selected, val_str.to_string(), label);
                                        }
                                    }
                                });
                        });
                        if selected != original {
                            *data = serde_json::json!(selected);
                        }
                    }
                } else {
                    if !read_only { 
                        // 使用第一个枚举值初始化
                        if let Some(first) = enum_values.first().and_then(|v| v.as_str()) {
                            *data = serde_json::json!(first);
                        }
                    }
                }
            } else {
                // 对于非枚举字符串，使用常规文本输入
                if let Some(s) = data.as_str() {
                    let mut text = s.to_string();
                    if read_only {
                         ui.label(text);
                    } else {
                        ui.push_id(path, |ui| {
                            if ui.text_edit_multiline(&mut text).changed() {
                                *data = serde_json::json!(text);
                            }
                        });
                    }
                } else {
                    // 如果类型不匹配，强制重置
                    if !read_only {
                        *data = serde_json::json!("");
                    } else {
                        ui.label("Invalid Type");
                    }
                }
            }
        },
        "boolean" => {
            if let Some(b) = data.as_bool() {
                let mut val = b;
                if read_only {
                     ui.add_enabled(false, egui::Checkbox::new(&mut val, ""));
                } else {
                    ui.push_id(path, |ui| {
                        if ui.checkbox(&mut val, "").changed() {
                            *data = serde_json::json!(val);
                        }
                    });
                }
            } else if !read_only {
                *data = serde_json::json!(false);
            }
        },
         "integer" | "number" => {
             let mut num = data.as_f64().unwrap_or(0.0);
             if read_only {
                 ui.label(num.to_string());
             } else {
                 ui.push_id(path, |ui| {
                     if ui.add(egui::DragValue::new(&mut num)).changed() {
                         *data = serde_json::json!(num); 
                     }
                 });
             }
        },
        "array" => {
            if !data.is_array() && !read_only { *data = serde_json::json!([]); }
            
            if read_only {
                if let Some(arr) = data.as_array() {
                    ui.label(format!("[{} items]", arr.len()));
                } else {
                    ui.label("[]");
                }
            } else {
                // 先获取数组的长度，避免借用冲突
                let array_length = data.as_array().map(|arr| arr.len()).unwrap_or(0);
                
                // 显示数组长度和添加按钮
                ui.horizontal(|ui| {
                    ui.label(format!("[{} items]", array_length));
                    // 添加简单的添加按钮
                    if ui.button("+").clicked() {
                        // 添加一个默认值到数组
                        let item_default = serde_json::json!("");
                        data.as_array_mut().unwrap().push(item_default);
                    }
                });
                
                // 处理数组项的编辑和删除
                let mut remove_indices = Vec::new();
                
                // 使用索引访问数组项，避免可变借用冲突
                let mut index = 0;
                while index < data.as_array().map(|arr| arr.len()).unwrap_or(0) {
                    let mut item_changed = false;
                    let mut item_value = data.as_array_mut().unwrap()[index].clone();
                    
                    ui.horizontal(|ui| {
                        ui.label(format!("Item {}", index + 1));
                        
                        // 对于字符串类型的数组项，允许编辑
                        if let Some(s) = item_value.as_str() {
                            let mut text = s.to_string();
                            if ui.text_edit_singleline(&mut text).changed() {
                                item_value = serde_json::json!(text);
                                item_changed = true;
                            }
                        }
                        
                        // 添加删除按钮
                        if ui.button("✕").clicked() {
                            remove_indices.push(index);
                        }
                    });
                    
                    // 更新数组项
                    if item_changed {
                        data.as_array_mut().unwrap()[index] = item_value;
                    }
                    
                    index += 1;
                }
                
                // 在循环外执行删除操作，从后往前删除避免索引问题
                remove_indices.sort_by(|a, b| b.cmp(a));
                for i in remove_indices {
                    data.as_array_mut().unwrap().remove(i);
                }
            }
        }
        _ => {
            ui.label(format!("Unsupported type: {}", obj_type));
        }
    }
}
