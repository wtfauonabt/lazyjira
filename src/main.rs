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
    // Initialize logger
    logger::init_logger(log::LevelFilter::Info);

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
                Ok((_client, status)) => {
                    match status {
                        ConnectionStatus::Connected => {
                            println!("✓ Successfully connected to Jira!\n");
                            // TODO: Initialize UI and start application with client
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
