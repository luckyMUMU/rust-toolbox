//! Configuration management for the workflow toolkit

use crate::core::{AuthConfig, RateLimitConfig};
use crate::error::{Result, WorkflowError};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
    pub plugins: PluginConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub workflow: WorkflowEngineConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub http_port: u16,
    pub ws_port: u16,
    pub bind_address: String,
    pub cors_origins: Vec<String>,
    pub max_connections: usize,
    pub request_timeout: Duration,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: PathBuf,
    pub cache_size: u64,
    pub cache_ttl: Duration,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
    pub retention_days: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<PathBuf>,
    pub max_file_size: u64,
    pub max_files: u32,
}

/// Plugin system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_dir: PathBuf,
    pub auto_load: bool,
    pub sandbox_enabled: bool,
    pub timeout: Duration,
    pub memory_limit: u64,
}

/// Workflow engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEngineConfig {
    pub max_concurrent_workflows: usize,
    pub default_timeout: Duration,
    pub checkpoint_enabled: bool,
    pub checkpoint_interval: Duration,
    pub cleanup_interval: Duration,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

/// Log output options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File,
    Both,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
            plugins: PluginConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            workflow: WorkflowEngineConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 8080,
            ws_port: 8081,
            bind_address: "127.0.0.1".to_string(),
            cors_origins: vec!["*".to_string()],
            max_connections: 1000,
            request_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("./data/workflow.db"),
            cache_size: 1024 * 1024 * 100, // 100MB
            cache_ttl: Duration::from_secs(3600), // 1 hour
            backup_enabled: true,
            backup_interval: Duration::from_secs(86400), // 24 hours
            retention_days: 30,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Stdout,
            file_path: Some(PathBuf::from("./logs/workflow-toolkit.log")),
            max_file_size: 1024 * 1024 * 10, // 10MB
            max_files: 5,
        }
    }
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_dir: PathBuf::from("./plugins"),
            auto_load: true,
            sandbox_enabled: true,
            timeout: Duration::from_secs(300), // 5 minutes
            memory_limit: 1024 * 1024 * 512, // 512MB
        }
    }
}

impl Default for WorkflowEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 10,
            default_timeout: Duration::from_secs(3600), // 1 hour
            checkpoint_enabled: true,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(86400), // 24 hours
        }
    }
}

impl Config {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self> {
        Self::load_from_path("config/default.toml")
    }
    
    /// Load configuration from a specific path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut builder = ConfigBuilder::builder();
        
        // Add configuration file if it exists
        let config_path = path.as_ref();
        if config_path.exists() {
            builder = builder.add_source(File::from(config_path));
        }
        
        // Add environment variables with prefix "WORKFLOW_TOOLKIT_"
        builder = builder.add_source(
            Environment::with_prefix("WORKFLOW_TOOLKIT")
                .separator("_")
                .try_parsing(true),
        );
        
        let config = builder.build()
            .map_err(|e| WorkflowError::Config(e))?;
            
        config.try_deserialize()
            .map_err(|e| WorkflowError::Config(e))
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.http_port == 0 {
            return Err(WorkflowError::workflow_validation("HTTP port cannot be 0"));
        }
        
        if self.server.ws_port == 0 {
            return Err(WorkflowError::workflow_validation("WebSocket port cannot be 0"));
        }
        
        if self.server.http_port == self.server.ws_port {
            return Err(WorkflowError::workflow_validation("HTTP and WebSocket ports must be different"));
        }
        
        // Validate storage configuration
        if let Some(parent) = self.storage.database_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Validate plugin configuration
        if !self.plugins.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugins.plugin_dir)?;
        }
        
        // Validate logging configuration
        if let Some(log_path) = &self.logging.file_path {
            if let Some(parent) = log_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the configuration as a TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| WorkflowError::Generic(e.into()))
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let toml_content = self.to_toml()?;
        std::fs::write(path, toml_content)?;
        Ok(())
    }
}

// Implement Serialize for Config using toml
impl Config {
    /// Create a sample configuration file
    pub fn create_sample_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let config = Config::default();
        config.save_to_file(path)
    }
}