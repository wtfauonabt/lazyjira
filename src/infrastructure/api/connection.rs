use crate::infrastructure::api::{ApiClient, JiraApiClient};
use crate::infrastructure::config::JiraCliConfig;
use crate::utils::{LazyJiraError, Result};
use log::{debug, info, warn};

/// Connection validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    AuthenticationFailed,
    NetworkError,
    ConfigurationError,
    UnknownError(String),
}

impl ConnectionStatus {
    #[allow(dead_code)] // Will be used for connection status checks
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    pub fn error_message(&self) -> Option<String> {
        match self {
            ConnectionStatus::Connected => None,
            ConnectionStatus::AuthenticationFailed => {
                Some("Authentication failed. Please check your credentials.".to_string())
            }
            ConnectionStatus::NetworkError => {
                Some("Network error. Please check your internet connection.".to_string())
            }
            ConnectionStatus::ConfigurationError => {
                Some("Configuration error. Please check your jira-cli config.".to_string())
            }
            ConnectionStatus::UnknownError(msg) => Some(msg.clone()),
        }
    }
}

/// Connection validator for testing Jira API connectivity
pub struct ConnectionValidator;

impl ConnectionValidator {
    /// Test connection to Jira instance
    /// Uses a simple search query as a lightweight connectivity test
    pub async fn test_connection(client: &dyn ApiClient) -> ConnectionStatus {
        info!("Testing connection to Jira instance...");
        
        // Try a simple search query to validate auth and connection
        // Using a bounded JQL query (the /search/jql endpoint requires bounded queries)
        // Search for issues assigned to current user, which is a valid bounded query
        match client.search_issues("assignee = currentUser() ORDER BY updated DESC", 0, 1).await {
            Ok(_) => {
                info!("Connection test successful");
                ConnectionStatus::Connected
            }
            Err(e) => {
                warn!("Connection test failed: {:?}", e);
                match &e {
                    LazyJiraError::Authentication(_) => ConnectionStatus::AuthenticationFailed,
                    LazyJiraError::Network(_) => ConnectionStatus::NetworkError,
                    LazyJiraError::Config(_) => ConnectionStatus::ConfigurationError,
                    _ => ConnectionStatus::UnknownError(format!("{}", e)),
                }
            }
        }
    }

    /// Create API client from jira-cli config and test connection
    pub async fn connect_with_config(config: &JiraCliConfig) -> Result<(JiraApiClient, ConnectionStatus)> {
        debug!("Creating API client from jira-cli config");
        let client = JiraApiClient::from_jira_cli_config(config)?;
        
        debug!("Testing connection...");
        let status = Self::test_connection(&client).await;
        
        Ok((client, status))
    }

    /// Validate jira-cli configuration
    pub fn validate_config(config: &JiraCliConfig) -> Result<()> {
        if config.instance.is_empty() {
            return Err(LazyJiraError::Config(
                "Jira instance URL is empty".to_string()
            ));
        }

        if config.auth.username.is_empty() {
            return Err(LazyJiraError::Config(
                "Username is empty".to_string()
            ));
        }

        if config.auth.auth_type == "api-token" {
            if config.auth.token.is_none() {
                return Err(LazyJiraError::Config(
                    "API token is required for api-token authentication".to_string()
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::api::client::SearchResult;
    use crate::domain::models::ticket::Ticket;
    use async_trait::async_trait;

    // Mock API client for testing
    struct MockApiClient {
        should_fail: bool,
        error_type: Option<String>, // Store error type as string to avoid Clone issues
    }

    #[async_trait]
    impl ApiClient for MockApiClient {
        async fn get_issue(&self, _key: &str) -> Result<Ticket> {
            if self.should_fail {
                let error_msg = self.error_type.as_deref().unwrap_or("Network error");
                Err(match error_msg {
                    "auth" => LazyJiraError::Authentication("Invalid credentials".to_string()),
                    "config" => LazyJiraError::Config("Config error".to_string()),
                    _ => LazyJiraError::Api("Network error".to_string()),
                })
            } else {
                Err(LazyJiraError::Internal("Not implemented in mock".to_string()))
            }
        }

        async fn search_issues(
            &self,
            _jql: &str,
            _start_at: usize,
            _max_results: usize,
        ) -> Result<SearchResult> {
            if self.should_fail {
                let error_msg = self.error_type.as_deref().unwrap_or("Network error");
                Err(match error_msg {
                    "auth" => LazyJiraError::Authentication("Invalid credentials".to_string()),
                    "config" => LazyJiraError::Config("Config error".to_string()),
                    _ => LazyJiraError::Api("Network error".to_string()),
                })
            } else {
                Ok(SearchResult {
                    start_at: 0,
                    max_results: 1,
                    total: 0,
                    issues: vec![],
                })
            }
        }

        async fn create_issue(&self, _data: crate::infrastructure::api::client::CreateIssueData) -> Result<Ticket> {
            Err(LazyJiraError::Internal("Not implemented".to_string()))
        }

        async fn update_issue(
            &self,
            _key: &str,
            _data: crate::infrastructure::api::client::UpdateIssueData,
        ) -> Result<()> {
            Err(LazyJiraError::Internal("Not implemented".to_string()))
        }

        async fn transition_issue(
            &self,
            _key: &str,
            _transition_id: &str,
            _comment: Option<String>,
        ) -> Result<()> {
            Err(LazyJiraError::Internal("Not implemented".to_string()))
        }

        async fn get_transitions(&self, _key: &str) -> Result<Vec<crate::infrastructure::api::client::Transition>> {
            Err(LazyJiraError::Internal("Not implemented".to_string()))
        }

        async fn add_comment(&self, _key: &str, _comment: String) -> Result<()> {
            Err(LazyJiraError::Internal("Not implemented".to_string()))
        }

        async fn get_comments(&self, _key: &str) -> Result<Vec<crate::domain::models::comment::Comment>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_connection_success() {
        let client = MockApiClient {
            should_fail: false,
            error_type: None,
        };
        
        let status = ConnectionValidator::test_connection(&client).await;
        assert_eq!(status, ConnectionStatus::Connected);
        assert!(status.is_connected());
        assert!(status.error_message().is_none());
    }

    #[tokio::test]
    async fn test_connection_authentication_failed() {
        let client = MockApiClient {
            should_fail: true,
            error_type: Some("auth".to_string()),
        };
        
        let status = ConnectionValidator::test_connection(&client).await;
        assert_eq!(status, ConnectionStatus::AuthenticationFailed);
        assert!(!status.is_connected());
        assert!(status.error_message().is_some());
        assert!(status.error_message().unwrap().contains("Authentication"));
    }

    #[tokio::test]
    async fn test_connection_network_error() {
        let client = MockApiClient {
            should_fail: true,
            error_type: Some("network".to_string()),
        };
        
        let status = ConnectionValidator::test_connection(&client).await;
        // Network errors map to UnknownError since we're using Api error
        assert!(!status.is_connected());
        assert!(status.error_message().is_some());
    }

    #[tokio::test]
    async fn test_connection_config_error() {
        let client = MockApiClient {
            should_fail: true,
            error_type: Some("config".to_string()),
        };
        
        let status = ConnectionValidator::test_connection(&client).await;
        assert_eq!(status, ConnectionStatus::ConfigurationError);
        assert!(!status.is_connected());
    }

    #[test]
    fn test_validate_config_success() {
        let config = JiraCliConfig {
            instance: "test.atlassian.net".to_string(),
            auth: crate::infrastructure::config::JiraCliAuth {
                auth_type: "api-token".to_string(),
                username: "test@example.com".to_string(),
                token: Some("token123".to_string()),
            },
        };

        assert!(ConnectionValidator::validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_instance() {
        let config = JiraCliConfig {
            instance: String::new(),
            auth: crate::infrastructure::config::JiraCliAuth {
                auth_type: "api-token".to_string(),
                username: "test@example.com".to_string(),
                token: Some("token123".to_string()),
            },
        };

        assert!(ConnectionValidator::validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_empty_username() {
        let config = JiraCliConfig {
            instance: "test.atlassian.net".to_string(),
            auth: crate::infrastructure::config::JiraCliAuth {
                auth_type: "api-token".to_string(),
                username: String::new(),
                token: Some("token123".to_string()),
            },
        };

        assert!(ConnectionValidator::validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_missing_token() {
        let config = JiraCliConfig {
            instance: "test.atlassian.net".to_string(),
            auth: crate::infrastructure::config::JiraCliAuth {
                auth_type: "api-token".to_string(),
                username: "test@example.com".to_string(),
                token: None,
            },
        };

        assert!(ConnectionValidator::validate_config(&config).is_err());
    }
}
