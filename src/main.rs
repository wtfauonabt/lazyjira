mod app;
mod domain;
mod infrastructure;
mod ui;
mod utils;

use infrastructure::api::{ConnectionStatus, ConnectionValidator};
use infrastructure::config::Config;
use utils::logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger - use Info level by default, can be overridden with RUST_LOG env var
    // Set to Debug for troubleshooting: RUST_LOG=debug cargo run
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(log::LevelFilter::Info);
    logger::init_logger(log_level);
    
    // Set up panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        let message = panic_info.payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| {
                panic_info.payload()
                    .downcast_ref::<String>()
                    .map(|s| s.clone())
            })
            .unwrap_or_else(|| "unknown panic".to_string());
        
        eprintln!("\n=== PANIC OCCURRED ===");
        eprintln!("Location: {}", location);
        eprintln!("Message: {}", message);
        eprintln!("=====================\n");
        
        log::error!("PANIC at {}: {}", location, message);
    }));

    println!("LazyJira starting...\n");

    // Load application configuration
    let config = Config::load()?;
    
    // Try to load jira-cli config
    match Config::load_jira_cli_config()? {
        Some(jira_cli_config) => {
            println!("✓ Found jira-cli config for instance: {}", jira_cli_config.instance);
            
            // Validate configuration
            if let Err(e) = ConnectionValidator::validate_config(&jira_cli_config) {
                eprintln!("\n✗ Configuration validation failed:");
                eprintln!("  {}", e);
                eprintln!("\nPlease check your jira-cli configuration at:");
                eprintln!("  ~/.config/jira-cli/config.yaml");
                std::process::exit(1);
            }
            
            // Test connection
            println!("Testing connection to Jira instance...");
            match ConnectionValidator::connect_with_config(&jira_cli_config).await {
                Ok((client, status)) => {
                    match status {
                        ConnectionStatus::Connected => {
                            println!("✓ Successfully connected to Jira!\n");
                            
                            // Initialize UI and start application
                            let client: std::sync::Arc<dyn infrastructure::api::ApiClient> = 
                                std::sync::Arc::new(client);
                            let instance_url = jira_cli_config.instance.clone();
                            let mut app = ui::App::new("Connected".to_string(), client, instance_url)?;
                            app.run().await?;
                        }
                        _ => {
                            eprintln!("\n✗ Connection failed:");
                            if let Some(msg) = status.error_message() {
                                eprintln!("  {}", msg);
                            }
                            eprintln!("\nPlease check:");
                            eprintln!("  - Your internet connection");
                            eprintln!("  - Your jira-cli credentials");
                            eprintln!("  - The Jira instance URL");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\n✗ Failed to create API client:");
                    eprintln!("  {}", e);
                    eprintln!("\nPlease check your jira-cli configuration.");
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("✗ No jira-cli config found.");
            eprintln!("\nPlease configure jira-cli first:");
            eprintln!("  1. Install jira-cli: https://github.com/go-jira/jira");
            eprintln!("  2. Configure it: jira-cli configure");
            eprintln!("  3. Or create config manually at:");
            eprintln!("     ~/.config/jira-cli/config.yaml");
            eprintln!("\nExample config:");
            eprintln!("  instance: yourcompany.atlassian.net");
            eprintln!("  auth:");
            eprintln!("    type: api-token");
            eprintln!("    username: your.email@example.com");
            eprintln!("    token: YOUR_API_TOKEN");
            std::process::exit(1);
        }
    }

    println!("Configuration loaded: {:?}", config);
    println!("\nTODO: Initialize UI and start application");

    // TODO: Initialize UI and start application
    // For now, just exit successfully
    Ok(())
}
