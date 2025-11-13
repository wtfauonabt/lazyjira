pub mod client;
pub mod jira_client;
pub mod jira_cli_adapter;

pub use client::ApiClient;
// Re-export for convenience (will be used when API integration is complete)
#[allow(unused_imports)]
pub use jira_client::JiraApiClient;
