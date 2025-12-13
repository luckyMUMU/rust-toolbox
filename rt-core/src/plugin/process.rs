use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use async_trait::async_trait;
use serde_json::Value;
use crate::{Tool, Locale, Result, CoreError};
use super::manifest::PluginMetadata;

#[derive(Debug)]
pub struct ProcessPlugin {
    path: PathBuf,
    metadata: PluginMetadata,
}

impl ProcessPlugin {
    pub async fn new(path: PathBuf) -> Result<Self> {
        // Run `spec` command to get metadata
        let output = Command::new(&path)
            .arg("spec")
            .output()
            .await
            .map_err(|e| CoreError::ToolFailure(format!("Failed to execute plugin spec for {:?}: {}", path, e)))?;

        if !output.status.success() {
             return Err(CoreError::ToolFailure(format!(
                "Plugin spec failed for {:?} with status: {}", 
                path, output.status
            )));
        }

        let metadata: PluginMetadata = serde_json::from_slice(&output.stdout)
            .map_err(|e| CoreError::ToolFailure(format!("Failed to parse plugin metadata: {}", e)))?;

        Ok(Self {
            path,
            metadata,
        })
    }
}

#[async_trait]
impl Tool for ProcessPlugin {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn display_name(&self, locale: Locale) -> String {
        self.metadata.display_name.get(locale).to_string()
    }

    fn description(&self, locale: Locale) -> String {
        self.metadata.description.get(locale).to_string()
    }

    fn user_guide(&self, locale: Locale) -> String {
        self.metadata.user_guide.get(locale).to_string()
    }

    fn input_schema(&self, locale: Locale) -> Value {
        let mut schema = self.metadata.input_schema.clone();
        
        if let Some(fields) = &self.metadata.input_fields {
             if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
                 for (field_name, localized_title) in fields {
                     if let Some(field_schema) = props.get_mut(field_name) {
                         if let Some(field_obj) = field_schema.as_object_mut() {
                             field_obj.insert("title".to_string(), Value::String(localized_title.get(locale).to_string()));
                         }
                     }
                 }
             }
        }
        
        schema
    }

    fn output_schema(&self, locale: Locale) -> Value {
        let mut schema = self.metadata.output_schema.clone().unwrap_or_else(|| serde_json::json!({ "type": "object" }));

        if let Some(fields) = &self.metadata.output_fields {
             if let Some(props) = schema.get_mut("properties").and_then(|p| p.as_object_mut()) {
                 for (field_name, localized_title) in fields {
                     if let Some(field_schema) = props.get_mut(field_name) {
                         if let Some(field_obj) = field_schema.as_object_mut() {
                             field_obj.insert("title".to_string(), Value::String(localized_title.get(locale).to_string()));
                         }
                     }
                 }
             }
        }
        
        schema
    }

    async fn run(&self, input: Value) -> Result<Value> {
        let mut child = Command::new(&self.path)
            .arg("run")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| CoreError::ToolFailure(format!("Failed to spawn plugin: {}", e)))?;

        let input_str = serde_json::to_string(&input)
            .map_err(|e| CoreError::ToolFailure(format!("Failed to serialize input: {}", e)))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input_str.as_bytes()).await
                .map_err(|e| CoreError::ToolFailure(format!("Failed to write to plugin stdin: {}", e)))?;
        }

        let output = child.wait_with_output().await
            .map_err(|e| CoreError::ToolFailure(format!("Failed to wait for plugin: {}", e)))?;

        if !output.status.success() {
             return Err(CoreError::ToolFailure(format!(
                "Plugin execution failed with status: {}", 
                output.status
            )));
        }

        let result: Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| CoreError::ToolFailure(format!("Failed to parse plugin output: {}", e)))?;

        Ok(result)
    }
}
