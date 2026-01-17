# 文件管理工具 API 参考

本文档为所有文件管理工具提供全面的 API 参考文档，包括详细的参数规范、返回值、错误处理和集成示例。

## 目录

1. [插件架构](#插件架构)
2. [核心工具 API](#核心工具-api)
3. [实用工具 API](#实用工具-api)
4. [数据结构](#数据结构)
5. [错误处理](#错误处理)
6. [集成示例](#集成示例)
7. [性能考虑](#性能考虑)

## 插件架构

### FileManagementPlugin

主插件类，负责在 workflow-toolkit 中注册所有文件管理工具。

```rust
pub struct FileManagementPlugin {
    tools: Vec<Box<dyn ToolNode>>,
    config: FileManagementConfig,
}

impl Plugin for FileManagementPlugin {
    fn name(&self) -> &str { "file-management" }
    fn version(&self) -> &str { "1.0.0" }
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn get_tools(&self) -> Vec<Box<dyn ToolNode>>;
    fn shutdown(&mut self) -> Result<()>;
}
```

### FileManagementConfig

插件配置结构：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManagementConfig {
    pub max_threads: usize,
    pub temp_directory: PathBuf,
    pub default_encoding: String,
    pub enable_chinese_processing: bool,
    pub memory_limit: Option<String>,
    pub streaming_mode: bool,
}
```
