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



pub struct PluginManager {
    plugin_dir: PathBuf,
    plugins: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

// 手动实现 Debug trait，避免 dyn Tool 没有 Debug 实现的问题
impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 对于异步 RwLock，我们不能直接在 Debug 中获取读锁，所以使用占位值
        f.debug_struct("PluginManager")
            .field("plugin_dir", &self.plugin_dir)
            .field("plugins_count", &"<async>")
            .finish()
    }
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
    
    /// 获取支持 MCP 的工具列表
    pub async fn list_mcp_tools(&self) -> Vec<Arc<dyn Tool>> {
        let plugins = self.plugins.read().await;
        plugins.values()
            .filter(|tool| tool.mcp_supported())
            .cloned()
            .collect()
    }
    
    /// 获取工具作为 MCP 工具
    pub async fn get_mcp_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let plugins = self.plugins.read().await;
        if let Some(tool) = plugins.get(name) {
            if tool.mcp_supported() {
                Some(tool.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

async fn load_plugins_internal(plugin_dir: &Path) -> Vec<Box<dyn Tool>> {
    let mut plugins: Vec<Box<dyn Tool>> = Vec::new();
    
    if let Ok(mut entries) = tokio::fs::read_dir(plugin_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".wasm") {
                    // 处理Wasm插件，支持多工具
                    match WasmPlugin::extract_all_metadata(&path).await {
                        Ok(metadata_list) => {
                            // 为每个元数据创建一个插件实例
                            for metadata in metadata_list {
                                let plugin = WasmPlugin::from_metadata(metadata, path.clone());
                                info!("Loaded plugin: {}, MCP supported: {}", plugin.name(), plugin.mcp_supported());
                                plugins.push(Box::new(plugin));
                            }
                        },
                        Err(e) => {
                            warn!("Failed to load Wasm plugin {:?}: {}", path, e);
                        }
                    }
                } else if file_name.starts_with("rt-plugin-") {
                        // 处理进程插件，支持多工具
                        match ProcessPlugin::extract_all_metadata(&path).await {
                            Ok(metadata_list) => {
                                // 为每个元数据创建一个插件实例
                                for metadata in metadata_list {
                                    let plugin = ProcessPlugin::from_metadata(metadata, path.clone());
                                    info!("Loaded plugin: {}, MCP supported: {}", plugin.name(), plugin.mcp_supported());
                                    plugins.push(Box::new(plugin));
                                }
                            },
                            Err(e) => {
                                warn!("Failed to load Process plugin {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
    }
    plugins
}


