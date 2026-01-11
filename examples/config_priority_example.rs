//! Example demonstrating configuration priority and hot reload functionality

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use workflow_toolkit::config::{
    CliConfigOverrides, Config, ConfigManager, ConfigPriority, ConfigSource,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration Priority and Hot Reload Example ===\n");

    // 1. Demonstrate configuration priority
    println!("1. Configuration Priority Demonstration");
    println!("--------------------------------------");

    let config = Config::default();
    let manager = ConfigManager::new(config);

    println!(
        "Initial HTTP port: {}",
        manager.get_config().server.http_port
    );

    // Apply environment variable override (priority 2)
    std::env::set_var("WORKFLOW_TOOLKIT_SERVER_HTTP_PORT", "9090");
    manager.apply_environment_overrides()?;
    println!(
        "After environment override: {}",
        manager.get_config().server.http_port
    );

    // Apply command line override (priority 3 - highest)
    let mut cli_overrides = CliConfigOverrides::default();
    cli_overrides.http_port = Some(8888);
    manager.apply_command_line_overrides(&cli_overrides)?;
    println!(
        "After CLI override (highest priority): {}",
        manager.get_config().server.http_port
    );

    // Clean up environment variable
    std::env::remove_var("WORKFLOW_TOOLKIT_SERVER_HTTP_PORT");

    println!("\n2. Configuration Hot Reload Demonstration");
    println!("----------------------------------------");

    // Create a temporary config file
    let temp_dir = TempDir::new()?;
    let config_file = temp_dir.path().join("test_config.toml");

    // Write initial config
    let initial_config = r#"
[server]
http_port = 8080
ws_port = 8081

[logging]
level = "info"
"#;
    std::fs::write(&config_file, initial_config)?;

    // Load config with hot reload capability
    let hot_reload_manager = Config::load_from_path_with_priority(&config_file)?;
    println!(
        "Initial config loaded - HTTP port: {}",
        hot_reload_manager.get_config().server.http_port
    );

    // Start hot reload monitoring
    hot_reload_manager.start_hot_reload().await?;
    println!("Hot reload monitoring started...");

    // Wait a bit
    sleep(Duration::from_millis(100)).await;

    // Update the config file
    let updated_config = r#"
[server]
http_port = 9999
ws_port = 8081

[logging]
level = "debug"
"#;
    std::fs::write(&config_file, updated_config)?;
    println!("Config file updated with new HTTP port: 9999");

    // Manually trigger reload for demonstration
    hot_reload_manager.reload_from_file().await?;
    println!(
        "Config reloaded - HTTP port: {}",
        hot_reload_manager.get_config().server.http_port
    );

    println!("\n3. Configuration Sources Tracking");
    println!("--------------------------------");

    let sources = hot_reload_manager.get_sources();
    for source in sources {
        println!(
            "Source: {} (Priority: {:?})",
            source.source, source.priority
        );
    }

    println!("\n4. Watch Receiver Demonstration");
    println!("-------------------------------");

    let mut watch_receiver = hot_reload_manager.get_watch_receiver();

    // Update config to trigger watch notification
    let mut new_config = hot_reload_manager.get_config();
    new_config.server.http_port = 7777;

    let source = ConfigSource {
        priority: ConfigPriority::CommandLine,
        source: "manual_update".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    hot_reload_manager.update_config(new_config, source)?;

    // Check if watch receiver gets the update
    if watch_receiver.changed().await.is_ok() {
        let updated_config = watch_receiver.borrow().clone();
        println!(
            "Watch receiver detected config change - HTTP port: {}",
            updated_config.server.http_port
        );
    }

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
