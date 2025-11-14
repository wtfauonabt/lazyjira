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
        // For Atlassian Cloud, API token auth uses Basic auth with username:token
        // So we support both "api-token" and "basic" (when token is present)
        let auth_header = if config.auth.auth_type == "api-token" || 
            (config.auth.auth_type == "basic" && config.auth.token.is_some()) {
            if let Some(token) = &config.auth.token {
                let credentials = format!("{}:{}", config.auth.username, token);
                let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                format!("Basic {}", encoded)
            } else {
                return Err(LazyJiraError::Authentication(
                    "API token not found in config".to_string(),
                ));
            }
        } else if config.auth.auth_type == "basic" {
            // Basic auth with username/password (not API token)
            if let Some(password) = &config.auth.token {
                let credentials = format!("{}:{}", config.auth.username, password);
                let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                format!("Basic {}", encoded)
            } else {
                return Err(LazyJiraError::Authentication(
                    "Password not found for basic auth".to_string(),
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
        // Use the standard /search endpoint (still supported, /search/jql is for POST requests)
        // The /search endpoint with JQL query parameter is still the recommended approach
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

    async fn create_issue(&self, data: CreateIssueData) -> Result<Ticket> {
        let endpoint = "issue";
        
        // Build request body according to Jira API format
        let mut body = serde_json::json!({
            "fields": {
                "project": {
                    "key": data.project_key
                },
                "summary": data.summary,
                "issuetype": {
                    "name": data.issue_type
                }
            }
        });

        // Add optional fields
        if let Some(description) = data.description {
            body["fields"]["description"] = serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": description
                    }]
                }]
            });
        }

        if let Some(assignee) = data.assignee {
            body["fields"]["assignee"] = serde_json::json!({
                "accountId": assignee
            });
        }

        if let Some(priority) = data.priority {
            body["fields"]["priority"] = serde_json::json!({
                "name": priority
            });
        }

        let json = self.post(&endpoint, &body).await?;
        parse_issue(&json)
    }

    async fn update_issue(&self, _key: &str, _data: UpdateIssueData) -> Result<()> {
        // Placeholder implementation
        Err(LazyJiraError::Internal("Not yet implemented".to_string()))
    }

    async fn transition_issue(
        &self,
        key: &str,
        transition_id: &str,
        comment: Option<String>,
    ) -> Result<()> {
        let endpoint = format!("issue/{}/transitions", key);
        
        let mut body = serde_json::json!({
            "transition": {
                "id": transition_id
            }
        });

        // Add comment if provided
        if let Some(comment_text) = comment {
            body["update"]["comment"] = serde_json::json!([{
                "add": {
                    "body": {
                        "type": "doc",
                        "version": 1,
                        "content": [{
                            "type": "paragraph",
                            "content": [{
                                "type": "text",
                                "text": comment_text
                            }]
                        }]
                    }
                }
            }]);
        }

        self.post(&endpoint, &body).await?;
        Ok(())
    }

    async fn get_transitions(&self, key: &str) -> Result<Vec<Transition>> {
        let endpoint = format!("issue/{}/transitions", key);
        let json = self.get(&endpoint).await?;
        
        let transitions_array = json
            .get("transitions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| LazyJiraError::Parse("Missing 'transitions' array".to_string()))?;

        let mut transitions = Vec::new();
        for transition_json in transitions_array {
            let id = transition_json
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LazyJiraError::Parse("Missing transition 'id'".to_string()))?
                .to_string();

            let name = transition_json
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LazyJiraError::Parse("Missing transition 'name'".to_string()))?
                .to_string();

            let to_status = transition_json
                .get("to")
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            transitions.push(Transition {
                id,
                name,
                to_status,
            });
        }

        Ok(transitions)
    }

    async fn add_comment(&self, key: &str, comment: String) -> Result<()> {
        let endpoint = format!("issue/{}/comment", key);
        
        let body = serde_json::json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": comment
                    }]
                }]
            }
        });

        self.post(&endpoint, &body).await?;
        Ok(())
    }
}
