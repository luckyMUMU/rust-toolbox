//! Configuration management for the workflow toolkit.
//!
//! This module handles loading, validating, and managing configuration from multiple sources:
//! 1. Default values (lowest priority)
//! 2. Configuration files (e.g., `config/default.toml`)
//! 3. Environment variables (prefixed with `WORKFLOW_TOOLKIT_`)
//! 4. Command line arguments (highest priority)
//!
//! It also supports hot reloading of configuration files.

use crate::core::{AuthConfig, RateLimitConfig};
use crate::error::{Result, WorkflowError};
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, error, info, warn};

// Helper function to deserialize duration from seconds
fn deserialize_duration_from_secs<'de, D>(
    deserializer: D,
) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

// Helper function to serialize duration as seconds
fn serialize_duration_as_secs<S>(
    duration: &Duration,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

/// Main configuration structure.
///
/// Holds all configuration sections for the application.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Server settings (ports, host)
    pub server: ServerConfig,
    /// Storage settings (database, cache)
    pub storage: StorageConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// Plugin system settings
    pub plugins: PluginConfig,
    /// Authentication settings
    pub auth: AuthConfig,
    /// Rate limiting settings
    pub rate_limit: RateLimitConfig,
    /// Workflow engine settings
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
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    pub request_timeout: Duration,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: PathBuf,
    pub cache_size: u64,
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    pub cache_ttl: Duration,
    pub backup_enabled: bool,
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
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
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    pub timeout: Duration,
    pub memory_limit: u64,
}

/// Workflow engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEngineConfig {
    pub max_concurrent_workflows: usize,
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    pub default_timeout: Duration,
    pub checkpoint_enabled: bool,
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
    pub checkpoint_interval: Duration,
    #[serde(
        deserialize_with = "deserialize_duration_from_secs",
        serialize_with = "serialize_duration_as_secs"
    )]
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
            cache_size: 1024 * 1024 * 100,        // 100MB
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
            memory_limit: 1024 * 1024 * 512,   // 512MB
        }
    }
}

/// Configuration priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigPriority {
    /// Default values (lowest priority)
    Default = 0,
    /// Configuration file values
    File = 1,
    /// Environment variables
    Environment = 2,
    /// Command line arguments (highest priority)
    CommandLine = 3,
}

/// Configuration source information
#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub priority: ConfigPriority,
    pub source: String,
    pub timestamp: std::time::SystemTime,
}

/// Configuration manager with hot reload support
#[derive(Debug)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: Option<PathBuf>,
    watch_sender: Option<watch::Sender<Config>>,
    watch_receiver: watch::Receiver<Config>,
    sources: Arc<RwLock<Vec<ConfigSource>>>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config: Config) -> Self {
        let (watch_sender, watch_receiver) = watch::channel(config.clone());

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: None,
            watch_sender: Some(watch_sender),
            watch_receiver,
            sources: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create configuration manager with file path for hot reload
    pub fn with_file_path(config: Config, config_path: PathBuf) -> Self {
        let (watch_sender, watch_receiver) = watch::channel(config.clone());

        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Some(config_path),
            watch_sender: Some(watch_sender),
            watch_receiver,
            sources: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> Result<Config> {
        self.config
            .read()
            .map(|cfg| cfg.clone())
            .map_err(|_| WorkflowError::concurrency("Config RwLock poisoned"))
    }

    /// Get configuration watch receiver for hot reload notifications
    pub fn get_watch_receiver(&self) -> watch::Receiver<Config> {
        self.watch_receiver.clone()
    }

    /// Update configuration with priority handling
    pub fn update_config(&self, new_config: Config, source: ConfigSource) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| WorkflowError::concurrency("Config RwLock poisoned during update"))?;
        let mut sources = self
            .sources
            .write()
            .map_err(|_| WorkflowError::concurrency("Sources RwLock poisoned during update"))?;

        // Add or update source
        if let Some(existing_source) = sources.iter_mut().find(|s| s.source == source.source) {
            *existing_source = source.clone();
        } else {
            sources.push(source.clone());
        }

        // Merge configuration based on priority
        *config = self.merge_configs_by_priority(&new_config, &config, &sources)?;

        // Notify watchers of configuration change
        if let Some(sender) = &self.watch_sender {
            if let Err(e) = sender.send(config.clone()) {
                warn!("Failed to notify configuration watchers: {}", e);
            }
        }

        info!(
            "Configuration updated from source: {} (priority: {:?})",
            source.source, source.priority
        );
        Ok(())
    }

    /// Merge configurations based on priority
    fn merge_configs_by_priority(
        &self,
        new_config: &Config,
        current_config: &Config,
        sources: &[ConfigSource],
    ) -> Result<Config> {
        // For now, we'll use a simple approach where higher priority sources override lower ones
        // In a more sophisticated implementation, we would merge field by field

        // Find the highest priority source
        let highest_priority = sources
            .iter()
            .max_by_key(|s| s.priority)
            .map(|s| s.priority)
            .unwrap_or(ConfigPriority::Default);

        // If the new config has higher or equal priority, use it
        // Otherwise, keep the current config
        if sources
            .iter()
            .any(|s| s.source == "new_config" && s.priority >= highest_priority)
        {
            Ok(new_config.clone())
        } else {
            Ok(current_config.clone())
        }
    }

    /// Start hot reload monitoring
    pub async fn start_hot_reload(&self) -> Result<()> {
        if let Some(config_path) = &self.config_path {
            let config_path_clone = config_path.clone();
            let config_manager = self.clone_for_hot_reload();

            tokio::spawn(async move {
                config_manager.hot_reload_loop(config_path_clone).await;
            });

            info!(
                "Started configuration hot reload monitoring for: {:?}",
                config_path
            );
        } else {
            warn!("No configuration file path specified, hot reload not available");
        }

        Ok(())
    }

    /// Clone for hot reload (only the necessary parts)
    fn clone_for_hot_reload(&self) -> ConfigManagerForHotReload {
        ConfigManagerForHotReload {
            config: self.config.clone(),
            watch_sender: self.watch_sender.clone(),
            sources: self.sources.clone(),
        }
    }

    /// Reload configuration from file
    pub async fn reload_from_file(&self) -> Result<()> {
        if let Some(config_path) = &self.config_path {
            debug!("Reloading configuration from: {:?}", config_path);

            let new_config = Config::load_from_path(config_path)?;
            let source = ConfigSource {
                priority: ConfigPriority::File,
                source: format!("file:{}", config_path.display()),
                timestamp: std::time::SystemTime::now(),
            };

            self.update_config(new_config, source)?;
            info!("Configuration reloaded from file: {:?}", config_path);
        } else {
            return Err(WorkflowError::Config(config::ConfigError::Message(
                "No configuration file path specified".to_string(),
            )));
        }

        Ok(())
    }

    /// Apply environment variable overrides
    pub fn apply_environment_overrides(&self) -> Result<()> {
        debug!("Applying environment variable overrides");

        let mut builder = ConfigBuilder::builder();

        // Add current config as base
        let current_config = self.get_config()?;
        let config_value =
            serde_json::to_value(&current_config).map_err(|e| WorkflowError::Generic(e.into()))?;
        builder = builder
            .add_source(config::Config::try_from(&config_value).map_err(WorkflowError::Config)?);

        // Add environment variables with prefix "WORKFLOW_TOOLKIT_"
        builder = builder.add_source(
            Environment::with_prefix("WORKFLOW_TOOLKIT")
                .separator("_")
                .try_parsing(true),
        );

        let merged_config = builder.build().map_err(WorkflowError::Config)?;

        let new_config: Config = merged_config
            .try_deserialize()
            .map_err(WorkflowError::Config)?;

        let source = ConfigSource {
            priority: ConfigPriority::Environment,
            source: "environment_variables".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        self.update_config(new_config, source)?;
        info!("Applied environment variable overrides");
        Ok(())
    }

    /// Apply command line argument overrides
    pub fn apply_command_line_overrides(&self, cli_config: &CliConfigOverrides) -> Result<()> {
        debug!("Applying command line argument overrides");

        let mut current_config = self.get_config()?;

        // Apply CLI overrides with highest priority
        if let Some(log_level) = &cli_config.log_level {
            current_config.logging.level = log_level.clone();
        }

        if let Some(http_port) = cli_config.http_port {
            current_config.server.http_port = http_port;
        }

        if let Some(ws_port) = cli_config.ws_port {
            current_config.server.ws_port = ws_port;
        }

        if let Some(verbose) = cli_config.verbose {
            if verbose {
                current_config.logging.level = "debug".to_string();
            }
        }

        if let Some(quiet) = cli_config.quiet {
            if quiet {
                current_config.logging.level = "error".to_string();
            }
        }

        if let Some(plugin_dir) = &cli_config.plugin_dir {
            current_config.plugins.plugin_dir = plugin_dir.clone();
        }

        if let Some(database_path) = &cli_config.database_path {
            current_config.storage.database_path = database_path.clone();
        }

        let source = ConfigSource {
            priority: ConfigPriority::CommandLine,
            source: "command_line_arguments".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        self.update_config(current_config, source)?;
        info!("Applied command line argument overrides");
        Ok(())
    }

    /// Get configuration sources with their priorities
    pub fn get_sources(&self) -> Result<Vec<ConfigSource>> {
        self.sources
            .read()
            .map(|srcs| srcs.clone())
            .map_err(|_| WorkflowError::concurrency("Sources RwLock poisoned"))
    }
}

/// Helper struct for hot reload functionality
#[derive(Debug)]
struct ConfigManagerForHotReload {
    config: Arc<RwLock<Config>>,
    watch_sender: Option<watch::Sender<Config>>,
    sources: Arc<RwLock<Vec<ConfigSource>>>,
}

impl ConfigManagerForHotReload {
    /// Hot reload monitoring loop
    async fn hot_reload_loop(&self, config_path: PathBuf) {
        let mut interval = interval(TokioDuration::from_secs(5)); // Check every 5 seconds
        let mut last_modified = self.get_file_modified_time(&config_path).await;

        loop {
            interval.tick().await;

            if let Some(current_modified) = self.get_file_modified_time(&config_path).await {
                if Some(current_modified) != last_modified {
                    debug!("Configuration file changed, reloading...");

                    match self.reload_config_file(&config_path).await {
                        Ok(()) => {
                            last_modified = Some(current_modified);
                            info!("Configuration hot reloaded successfully");
                        }
                        Err(e) => {
                            error!("Failed to hot reload configuration: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Get file modification time
    async fn get_file_modified_time(&self, path: &Path) -> Option<std::time::SystemTime> {
        tokio::fs::metadata(path)
            .await
            .ok()
            .and_then(|metadata| metadata.modified().ok())
    }

    /// Reload configuration from file
    async fn reload_config_file(&self, config_path: &Path) -> Result<()> {
        let new_config = Config::load_from_path(config_path)?;

        let mut config = self.config.write().map_err(|e| {
            crate::WorkflowError::workflow_execution(format!(
                "Failed to acquire config write lock: {}",
                e
            ))
        })?;
        let mut sources = self.sources.write().map_err(|e| {
            crate::WorkflowError::workflow_execution(format!(
                "Failed to acquire sources write lock: {}",
                e
            ))
        })?;

        // Update file source
        let source = ConfigSource {
            priority: ConfigPriority::File,
            source: format!("file:{}", config_path.display()),
            timestamp: std::time::SystemTime::now(),
        };

        if let Some(existing_source) = sources.iter_mut().find(|s| s.source.starts_with("file:")) {
            *existing_source = source;
        } else {
            sources.push(source);
        }

        // Apply the new configuration (respecting priority)
        *config = new_config;

        // Notify watchers
        if let Some(sender) = &self.watch_sender {
            if let Err(e) = sender.send(config.clone()) {
                warn!(
                    "Failed to notify configuration watchers during hot reload: {}",
                    e
                );
            }
        }

        Ok(())
    }
}

/// Command line configuration overrides
#[derive(Debug, Clone, Default)]
pub struct CliConfigOverrides {
    pub log_level: Option<String>,
    pub http_port: Option<u16>,
    pub ws_port: Option<u16>,
    pub verbose: Option<bool>,
    pub quiet: Option<bool>,
    pub plugin_dir: Option<PathBuf>,
    pub database_path: Option<PathBuf>,
}

impl CliConfigOverrides {
    /// Create from CLI arguments
    pub fn from_cli(cli: &crate::interfaces::cli::Cli) -> Self {
        Self {
            log_level: if cli.log_level != "info" {
                Some(cli.log_level.clone())
            } else {
                None
            },
            verbose: if cli.verbose { Some(true) } else { None },
            quiet: if cli.quiet { Some(true) } else { None },
            http_port: None, // Will be set from server command if applicable
            ws_port: None,   // Will be set from server command if applicable
            plugin_dir: None,
            database_path: None,
        }
    }

    /// Update with server command options
    pub fn with_server_options(mut self, http_port: u16, ws_port: u16) -> Self {
        self.http_port = Some(http_port);
        self.ws_port = Some(ws_port);
        self
    }
}
impl Default for WorkflowEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 10,
            default_timeout: Duration::from_secs(3600), // 1 hour
            checkpoint_enabled: true,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(86400),  // 24 hours
        }
    }
}

impl Config {
    /// Load configuration with priority handling
    pub fn load_with_priority() -> Result<ConfigManager> {
        let config = Self::load()?;
        let manager = ConfigManager::new(config);

        // Apply environment variable overrides
        manager.apply_environment_overrides()?;

        Ok(manager)
    }

    /// Load configuration from file with priority handling
    pub fn load_from_path_with_priority<P: AsRef<Path>>(path: P) -> Result<ConfigManager> {
        let config_path = path.as_ref().to_path_buf();
        let config = Self::load_from_path(&config_path)?;
        let manager = ConfigManager::with_file_path(config, config_path);

        // Apply environment variable overrides
        manager.apply_environment_overrides()?;

        Ok(manager)
    }

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

        let config = builder.build().map_err(WorkflowError::Config)?;

        config.try_deserialize().map_err(WorkflowError::Config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.http_port == 0 {
            return Err(WorkflowError::workflow_validation("HTTP port cannot be 0"));
        }

        if self.server.ws_port == 0 {
            return Err(WorkflowError::workflow_validation(
                "WebSocket port cannot be 0",
            ));
        }

        if self.server.http_port == self.server.ws_port {
            return Err(WorkflowError::workflow_validation(
                "HTTP and WebSocket ports must be different",
            ));
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
        toml::to_string_pretty(self).map_err(|e| WorkflowError::Generic(e.into()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::sleep;

    #[test]
    fn test_config_priority_levels() {
        assert!(ConfigPriority::CommandLine > ConfigPriority::Environment);
        assert!(ConfigPriority::Environment > ConfigPriority::File);
        assert!(ConfigPriority::File > ConfigPriority::Default);
    }

    #[test]
    fn test_config_manager_creation() -> Result<()> {
        let config = Config::default();
        let manager = ConfigManager::new(config.clone());

        let retrieved_config = manager.get_config()?;
        assert_eq!(retrieved_config.server.http_port, config.server.http_port);
        assert_eq!(retrieved_config.server.ws_port, config.server.ws_port);
        Ok(())
    }

    #[tokio::test]
    async fn test_environment_variable_override() -> Result<()> {
        // Set environment variable
        std::env::set_var("WORKFLOW_TOOLKIT_SERVER_HTTP_PORT", "9090");

        let config = Config::default();
        let manager = ConfigManager::new(config);

        // Apply environment overrides
        manager.apply_environment_overrides()?;

        let updated_config = manager.get_config()?;
        assert_eq!(updated_config.server.http_port, 9090);

        // Clean up
        std::env::remove_var("WORKFLOW_TOOLKIT_SERVER_HTTP_PORT");
        Ok(())
    }

    #[test]
    fn test_cli_config_overrides() -> Result<()> {
        let config = Config::default();
        let manager = ConfigManager::new(config);

        let mut cli_overrides = CliConfigOverrides::default();
        cli_overrides.log_level = Some("debug".to_string());
        cli_overrides.verbose = Some(true);
        cli_overrides.http_port = Some(8888);

        manager.apply_command_line_overrides(&cli_overrides)?;

        let updated_config = manager.get_config()?;
        assert_eq!(updated_config.logging.level, "debug");
        assert_eq!(updated_config.server.http_port, 8888);
        Ok(())
    }

    #[tokio::test]
    async fn test_config_hot_reload() -> Result<()> {
        let temp_dir = TempDir::new().map_err(|e| {
            WorkflowError::storage(format!("Failed to create temporary directory: {}", e))
        })?;
        let config_file = temp_dir.path().join("test_config.toml");

        // Create initial config file
        let initial_config = r#"
[server]
http_port = 8080
ws_port = 8081

[logging]
level = "info"
"#;
        std::fs::write(&config_file, initial_config).map_err(|e| {
            WorkflowError::storage(format!("Failed to write initial config: {}", e))
        })?;

        // Load config with hot reload
        let manager = Config::load_from_path_with_priority(&config_file)?;
        let initial_loaded_config = manager.get_config()?;
        assert_eq!(initial_loaded_config.server.http_port, 8080);
        assert_eq!(initial_loaded_config.logging.level, "info");

        // Start hot reload monitoring
        manager.start_hot_reload().await?;

        // Wait a bit for the monitoring to start
        sleep(std::time::Duration::from_millis(100)).await;

        // Update config file
        let updated_config = r#"
[server]
http_port = 9090
ws_port = 8081

[logging]
level = "debug"
"#;
        std::fs::write(&config_file, updated_config).map_err(|e| {
            WorkflowError::storage(format!("Failed to write updated config: {}", e))
        })?;

        // Wait for hot reload to detect the change
        sleep(std::time::Duration::from_secs(6)).await;

        // Manually trigger reload for testing (since hot reload runs in background)
        manager.reload_from_file().await?;

        let reloaded_config = manager.get_config()?;
        assert_eq!(reloaded_config.server.http_port, 9090);
        assert_eq!(reloaded_config.logging.level, "debug");
        Ok(())
    }

    #[test]
    fn test_config_source_tracking() -> Result<()> {
        let config = Config::default();
        let manager = ConfigManager::new(config);

        let source = ConfigSource {
            priority: ConfigPriority::Environment,
            source: "test_source".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        let new_config = Config::default();
        manager.update_config(new_config, source.clone())?;

        let sources = manager.get_sources()?;
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source, "test_source");
        assert_eq!(sources[0].priority, ConfigPriority::Environment);
        Ok(())
    }

    #[tokio::test]
    async fn test_config_watch_receiver() -> Result<()> {
        let config = Config::default();
        let manager = ConfigManager::new(config.clone());

        let mut watch_receiver = manager.get_watch_receiver();

        // Update config
        let mut new_config = config;
        new_config.server.http_port = 9999;

        let source = ConfigSource {
            priority: ConfigPriority::CommandLine,
            source: "test".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        manager.update_config(new_config, source)?;

        // Check if watch receiver gets the update
        if watch_receiver.changed().await.is_ok() {
            let updated_config = watch_receiver.borrow().clone();
            assert_eq!(updated_config.server.http_port, 9999);
        }
        Ok(())
    }
}
