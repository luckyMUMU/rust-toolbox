use eframe::egui;
use rt_core::{Tool, Locale};
use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use serde_json::{Value, json};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

enum GuiMessage {
    Output(Value),
    Error(String),
}

struct ToolkitApp {
    tools: Arc<HashMap<String, Box<dyn Tool>>>,
    selected_tool_name: Option<String>,
    
    // Form state
    input_value: Value,
    current_schema: Option<Value>,

    output_value: Option<Value>,
    output_error: Option<String>,
    output_schema: Option<Value>,
    
    // Help UI state
    show_help: bool,
    markdown_cache: CommonMarkCache,

    // I18n
    locale: Locale,

    // Communication channel
    tx: mpsc::Sender<GuiMessage>,
    rx: mpsc::Receiver<GuiMessage>,
    
    // Async runtime
    runtime: tokio::runtime::Runtime,
}

impl ToolkitApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
        tools.insert(rt_tools::MoveFolder.name().to_string(), Box::new(rt_tools::MoveFolder));
        
        let (tx, rx) = mpsc::channel();

        Self {
            tools: Arc::new(tools),
            selected_tool_name: None,
            input_value: json!({}),
            current_schema: None,
            output_value: None,
            output_error: None,
            output_schema: None,
            show_help: false,
            markdown_cache: CommonMarkCache::default(),
            locale: Locale::En, // Default En
            tx,
            rx,
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    fn tr(&self, key: &str) -> String {
        match (self.locale, key) {
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
            _ => key.to_string(),
        }
    }
}

// Recursive Schema Renderer
fn render_schema(ui: &mut egui::Ui, schema: &Value, data: &mut Value, read_only: bool) {
    if let Some(obj_type) = schema.get("type").and_then(|v| v.as_str()) {
        match obj_type {
            "object" => {
                if !data.is_object() && !read_only { *data = json!({}); }
                
                if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
                    for (key, prop_schema) in props {
                        ui.horizontal(|ui| {
                            // Use title if available, otherwise key
                            let label_text = prop_schema.get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or(key);
                                
                            ui.label(label_text);
                            
                            // Ensure data has this key (only in edit mode)
                            if !read_only && data.get(key).is_none() {
                                // Initialize with safe default based on type
                                let default = match prop_schema.get("type").and_then(|v| v.as_str()) {
                                    Some("string") => json!(""),
                                    Some("boolean") => json!(false),
                                    Some("integer") | Some("number") => json!(0),
                                    _ => json!(null),
                                };
                                data.as_object_mut().unwrap().insert(key.clone(), default);
                            }
                            
                            if let Some(val) = data.get_mut(key) {
                                render_schema(ui, prop_schema, val, read_only);
                            } else if read_only {
                                ui.weak("(null)");
                            }
                        });
                    }
                }
            },
            "string" => {
                if let Some(s) = data.as_str() {
                    let mut text = s.to_string();
                    if read_only {
                         ui.label(text);
                    } else if ui.text_edit_singleline(&mut text).changed() {
                        *data = json!(text);
                    }
                } else {
                    // Force reset if type mismatch
                    if !read_only { *data = json!(""); }
                    else { ui.label("Invalid Type"); }
                }
            },
            "boolean" => {
                if let Some(b) = data.as_bool() {
                    let mut val = b;
                    if read_only {
                         ui.add_enabled(false, egui::Checkbox::new(&mut val, ""));
                    } else if ui.checkbox(&mut val, "").changed() {
                        *data = json!(val);
                    }
                } else {
                    if !read_only { *data = json!(false); }
                }
            },
             "integer" | "number" => {
                 let mut num = data.as_f64().unwrap_or(0.0);
                 if read_only {
                     ui.label(num.to_string());
                 } else if ui.add(egui::DragValue::new(&mut num)).changed() {
                     *data = json!(num); 
                 }
            }
            _ => {
                ui.label(format!("Unsupported type: {}", obj_type));
            }
        }
    } else {
        ui.label("Invalid Schema");
    }
}

impl eframe::App for ToolkitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                GuiMessage::Output(val) => {
                    self.output_value = Some(val);
                    self.output_error = None;
                    
                    // Also fetch schema for output
                     if let Some(name) = &self.selected_tool_name {
                        if let Some(tool) = self.tools.get(name) {
                            self.output_schema = Some(tool.output_schema(self.locale));
                        }
                    }
                },
                GuiMessage::Error(err) => {
                     self.output_error = Some(err);
                     self.output_value = None;
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Rust Toolbox");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("locale_combo")
                        .selected_text(format!("{:?}", self.locale))
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            if ui.selectable_value(&mut self.locale, Locale::En, "English").clicked() { changed = true; }
                            if ui.selectable_value(&mut self.locale, Locale::Zh, "中文").clicked() { changed = true; }
                            
                            // If locale changed, we need to re-fetch the schema for current tool to update localized titles
                            if changed {
                                if let Some(name) = &self.selected_tool_name {
                                    if let Some(tool) = self.tools.get(name) {
                                        self.current_schema = Some(tool.input_schema(self.locale));
                                        // Also update output schema if we have output
                                        if self.output_value.is_some() {
                                             self.output_schema = Some(tool.output_schema(self.locale));
                                        }
                                    }
                                }
                            }
                        });
                });
            });
        });
        // Left Panel: Tool Selection
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading(self.tr("Tools"));
            ui.separator();
            
            for (name, tool) in self.tools.iter() {
                // Use display name for the list item
                let display = tool.display_name(self.locale);
                if ui.selectable_label(self.selected_tool_name.as_deref() == Some(name), display).clicked() {
                    self.selected_tool_name = Some(name.clone());
                    self.current_schema = Some(tool.input_schema(self.locale));
                    self.input_value = json!({});
                    self.output_value = None;
                    self.output_error = None;
                    self.output_schema = None;
                }
                ui.label(egui::RichText::new(tool.description(self.locale)).small().weak());
                ui.separator();
            }
        });
        // Right Panel: Help (Collapsible)
        if self.show_help {
            egui::SidePanel::right("help_panel").min_width(300.0).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(self.tr("Help"));
                    if ui.button("X").clicked() {
                        self.show_help = false;
                    }
                });
                ui.separator();
                if let Some(name) = &self.selected_tool_name {
                    if let Some(tool) = self.tools.get(name) {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let guide = tool.user_guide(self.locale);
                            CommonMarkViewer::new()
                                .show(ui, &mut self.markdown_cache, &guide);
                        });
                    }
                } else {
                     ui.label(self.tr("Select a tool help"));
                }
            });
        }

        // Main Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            let name_opt = self.selected_tool_name.clone();
            if let Some(name) = name_opt {
                 ui.horizontal(|ui| {
                     let run_label = self.tr("Running");
                     ui.heading(format!("{}: {}", run_label, name));
                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                         let help_label = if self.show_help { self.tr("Hide Help") } else { self.tr("Show Help") };
                         if ui.button(help_label).clicked() {
                             self.show_help = !self.show_help;
                         }
                     });
                 });
                ui.separator();
                // Render Dynamic Form
                if let Some(schema) = &self.current_schema {
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                         render_schema(ui, schema, &mut self.input_value, false);
                    });
                } else {
                    ui.label("No schema available.");
                }

                ui.separator();
                
                // Debug View
                ui.collapsing(self.tr("Raw JSON"), |ui| {
                    ui.label(serde_json::to_string_pretty(&self.input_value).unwrap_or_default());
                });

                if ui.button(self.tr("Run")).clicked() {
                    let _tool = self.tools.get(&name).unwrap(); 
                    
                    let tools_ref = self.tools.clone();
                    let tool_name = name.clone();
                    let input_val = self.input_value.clone();
                    let tx = self.tx.clone();
                    let ctx_clone = ctx.clone();
                    
                    self.output_value = None;
                    self.output_error = None;
                    
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

                ui.separator();
                ui.heading(self.tr("Output"));
                
                if let Some(error) = &self.output_error {
                    ui.colored_label(egui::Color32::RED, error);
                } else if let Some(val) = &mut self.output_value {
                    if let Some(schema) = &self.output_schema {
                         egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                             render_schema(ui, schema, val, true);
                         });
                    } else {
                         // Fallback to raw json if no schema
                         ui.add(egui::TextEdit::multiline(&mut serde_json::to_string_pretty(val).unwrap()).interactive(false));
                    }
                } else {
                     ui.label("Ready");
                }
            } else {
                ui.heading(self.tr("Select a tool"));
            }
        });
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

    // Attempt to load "Microsoft YaHei" (msyh.ttc)
    // Note: msyh.ttc is a collection. setup_font_data usually takes raw bytes.
    // If it fails, fallback to SimHei? Or just try specific paths.
    
    let font_path = "C:\\Windows\\Fonts\\msyh.ttc";
    let font_name = "Microsoft YaHei";

    // Read font file
    match std::fs::read(font_path) {
        Ok(font_data) => {
             fonts.font_data.insert(
                font_name.to_owned(),
                egui::FontData::from_owned(font_data).tweak(
                    egui::FontTweak {
                        scale: 1.2, // Slightly larger for readability
                        ..Default::default()
                    }
                ),
            );

            // Prioritize it for Proportional and Monospace
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, font_name.to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.push(font_name.to_owned());
            }
            
            ctx.set_fonts(fonts);
            println!("Loaded font: {}", font_path);
        },
        Err(e) => {
            eprintln!("Failed to load font {}: {}", font_path, e);
            // Fallback to SimHei if YaHei fails
             let font_path_alt = "C:\\Windows\\Fonts\\simhei.ttf";
             if let Ok(font_data) = std::fs::read(font_path_alt) {
                  fonts.font_data.insert(
                    "SimHei".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                 if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.insert(0, "SimHei".to_owned());
                }
                 if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    family.push("SimHei".to_owned());
                }
                ctx.set_fonts(fonts);
                println!("Loaded fallback font: {}", font_path_alt);
             } else {
                 eprintln!("Failed to load fallback font SimHei");
             }
        }
    }
}
