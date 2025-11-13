mod app;
mod domain;
mod infrastructure;
mod ui;
mod utils;

use infrastructure::config::Config;
use utils::logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    logger::init_logger(log::LevelFilter::Info);

    // Load configuration
    let config = Config::load()?;
    
    // Try to load jira-cli config
    if let Some(jira_cli_config) = Config::load_jira_cli_config()? {
        println!("Found jira-cli config for instance: {}", jira_cli_config.instance);
    } else {
        println!("No jira-cli config found. Please configure jira-cli first.");
    }

    println!("LazyJira starting...");
    println!("Configuration loaded: {:?}", config);

    // TODO: Initialize UI and start application
    // For now, just exit successfully
    Ok(())
}
