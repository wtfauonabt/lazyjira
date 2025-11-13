use super::client::{ApiClient, CreateIssueData, SearchResult, Transition, UpdateIssueData};
use super::parser::{parse_issue, parse_search_results};
use super::rate_limiter::RateLimiter;
use super::retry::{retry_with_backoff, RetryConfig};
use crate::domain::models::ticket::Ticket;
use crate::infrastructure::config::JiraCliConfig;
use crate::utils::{LazyJiraError, Result};
use base64::Engine;
use reqwest::Client;
use std::sync::Arc;

/// Jira REST API client implementation
#[allow(dead_code)] // Will be used when API integration is complete
pub struct JiraApiClient {
    client: Client,
    base_url: String,
    auth_header: String,
    rate_limiter: Arc<RateLimiter>,
    retry_config: RetryConfig,
}

impl JiraApiClient {
    /// Create a new Jira API client from jira-cli config
    #[allow(dead_code)] // Will be used when API integration is complete
    pub fn from_jira_cli_config(config: &JiraCliConfig) -> Result<Self> {
        Self::new(config, None)
    }

    /// Create a new Jira API client with optional custom base URL (for testing)
    pub fn new(config: &JiraCliConfig, base_url_override: Option<String>) -> Result<Self> {
        let base_url = if let Some(override_url) = base_url_override {
            override_url
        } else {
            format!("https://{}/rest/api/3", config.instance)
        };
        
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
            rate_limiter: Arc::new(RateLimiter::jira_cloud()),
            retry_config: RetryConfig::default(),
        })
    }

    /// Make an authenticated GET request with rate limiting and retry
    async fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        // Wait for rate limiter token
        self.rate_limiter.wait_for_token().await?;

        // Retry with exponential backoff
        let url = format!("{}/{}", self.base_url, endpoint);
        let auth_header = self.auth_header.clone();
        let client = self.client.clone();
        
        retry_with_backoff(&self.retry_config, move || {
            let url = url.clone();
            let auth_header = auth_header.clone();
            let client = client.clone();
            async move {
                let response = client
                    .get(&url)
                    .header("Authorization", &auth_header)
                    .header("Accept", "application/json")
                    .send()
                    .await
                    .map_err(|e| LazyJiraError::Network(e))?;

                // Handle 429 (Too Many Requests) specifically
                if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    // Wait a bit longer for rate limit
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    return Err(LazyJiraError::Api("429 Too Many Requests".to_string()));
                }

                // Handle response
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
        })
        .await
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
    /// (Kept for POST/PUT requests that don't use retry yet)
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
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    LazyJiraError::Api("429 Too Many Requests".to_string())
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
        let json = self.get(&endpoint).await?;
        
        // Parse ticket from JSON response
        parse_issue(&json)
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
        let json = self.get(&endpoint).await?;
        
        // Parse search result
        let (parsed_start_at, parsed_max_results, total, issues) =
            parse_search_results(&json)?;
        
        Ok(SearchResult {
            start_at: parsed_start_at,
            max_results: parsed_max_results,
            total,
            issues,
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
