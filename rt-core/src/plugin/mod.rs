pub mod manifest;
pub mod process;
pub mod wasm;

use crate::{Tool, Result};
use std::path::{Path, PathBuf};
use process::ProcessPlugin;
use wasm::WasmPlugin;
use tracing::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use manifest::{PluginMetadata, LocalizedString};

// Backward compatibility
pub type PluginTool = ProcessPlugin;

pub struct PluginManager {
    plugin_dir: PathBuf,
    plugins: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub async fn load_all(&self) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.clear();
        
        let loaded = load_plugins_internal(&self.plugin_dir).await;
        for tool in loaded {
            plugins.insert(tool.name().to_string(), Arc::from(tool));
        }
        Ok(())
    }
    
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.plugins.read().await.get(name).cloned()
    }
    
    pub async fn list_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.plugins.read().await.values().cloned().collect()
    }
}

async fn load_plugins_internal(plugin_dir: &Path) -> Vec<Box<dyn Tool>> {
    let mut plugins = Vec::new();
    
    if let Ok(mut entries) = tokio::fs::read_dir(plugin_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let tool: Option<Box<dyn Tool>> = if file_name.ends_with(".wasm") {
                        match WasmPlugin::new(path.clone()).await {
                            Ok(t) => Some(Box::new(t)),
                            Err(e) => {
                                warn!("Failed to load Wasm plugin {:?}: {}", path, e);
                                None
                            }
                        }
                    } else if file_name.starts_with("rt-plugin-") {
                         match ProcessPlugin::new(path.clone()).await {
                            Ok(t) => Some(Box::new(t)),
                            Err(e) => {
                                warn!("Failed to load Process plugin {:?}: {}", path, e);
                                None
                            }
                        }
                    } else {
                        None
                    };

                    if let Some(tool) = tool {
                        info!("Loaded plugin: {}", tool.name());
                        plugins.push(tool);
                    }
                }
            }
        }
    }
    plugins
}

pub async fn load_plugins(plugin_dir: &Path) -> Vec<Box<dyn Tool>> {
    load_plugins_internal(plugin_dir).await
}
