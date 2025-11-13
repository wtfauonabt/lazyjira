use super::client::{ApiClient, CreateIssueData, SearchResult, Transition, UpdateIssueData};
use crate::domain::models::ticket::Ticket;
use crate::infrastructure::config::JiraCliConfig;
use crate::utils::{LazyJiraError, Result};
use base64::Engine;
use reqwest::Client;

/// Jira REST API client implementation
#[allow(dead_code)] // Will be used when API integration is complete
pub struct JiraApiClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl JiraApiClient {
    /// Create a new Jira API client from jira-cli config
    #[allow(dead_code)] // Will be used when API integration is complete
    pub fn from_jira_cli_config(config: &JiraCliConfig) -> Result<Self> {
        let base_url = format!("https://{}/rest/api/3", config.instance);
        
        // Build authentication header
        let auth_header = if config.auth.auth_type == "api-token" {
            if let Some(token) = &config.auth.token {
                let credentials = format!("{}:{}", config.auth.username, token);
                let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                format!("Basic {}", encoded)
            } else {
                return Err(LazyJiraError::Authentication(
                    "API token not found in jira-cli config".to_string(),
                ));
            }
        } else {
            return Err(LazyJiraError::Authentication(format!(
                "Unsupported auth type: {}",
                config.auth.auth_type
            )));
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| LazyJiraError::Network(e))?;

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    /// Make an authenticated GET request
    async fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| LazyJiraError::Network(e))?;

        self.handle_response(response).await
    }

    /// Make an authenticated POST request
    async fn post(&self, endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| LazyJiraError::Network(e))?;

        self.handle_response(response).await
    }

    /// Make an authenticated PUT request
    async fn put(&self, endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .put(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| LazyJiraError::Network(e))?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and convert to Result
    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<serde_json::Value> {
        let status = response.status();
        
        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| LazyJiraError::Network(e))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(match status {
                reqwest::StatusCode::UNAUTHORIZED => {
                    LazyJiraError::Authentication("Unauthorized".to_string())
                }
                reqwest::StatusCode::FORBIDDEN => {
                    LazyJiraError::Authentication("Forbidden".to_string())
                }
                _ => LazyJiraError::Api(format!("API error ({}): {}", status, error_text)),
            })
        }
    }
}

#[async_trait::async_trait]
impl ApiClient for JiraApiClient {
    async fn get_issue(&self, key: &str) -> Result<Ticket> {
        let endpoint = format!("issue/{}", key);
        let _json = self.get(&endpoint).await?;
        
        // Parse ticket from JSON response
        // This is a placeholder - we'll implement proper parsing when we have the Ticket model
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }

    async fn search_issues(
        &self,
        jql: &str,
        start_at: usize,
        max_results: usize,
    ) -> Result<SearchResult> {
        let endpoint = format!(
            "search?jql={}&startAt={}&maxResults={}",
            urlencoding::encode(jql),
            start_at,
            max_results
        );
        let _json = self.get(&endpoint).await?;
        
        // Parse search result
        // Placeholder implementation
        Ok(SearchResult {
            start_at,
            max_results,
            total: 0,
            issues: vec![],
        })
    }

    async fn create_issue(&self, _data: CreateIssueData) -> Result<Ticket> {
        // Placeholder implementation
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }

    async fn update_issue(&self, _key: &str, _data: UpdateIssueData) -> Result<()> {
        // Placeholder implementation
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }

    async fn transition_issue(
        &self,
        _key: &str,
        _transition_id: &str,
        _comment: Option<String>,
    ) -> Result<()> {
        // Placeholder implementation
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }

    async fn get_transitions(&self, _key: &str) -> Result<Vec<Transition>> {
        // Placeholder implementation
        Ok(vec![])
    }

    async fn add_comment(&self, _key: &str, _comment: String) -> Result<()> {
        // Placeholder implementation
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }
}
