use crate::domain::models::ticket::Ticket;
use crate::utils::Result;

/// Trait for API client implementations
#[async_trait::async_trait]
pub trait ApiClient: Send + Sync {
    /// Get a single issue by key
    async fn get_issue(&self, key: &str) -> Result<Ticket>;
    
    /// Search issues using JQL
    async fn search_issues(
        &self,
        jql: &str,
        start_at: usize,
        max_results: usize,
    ) -> Result<SearchResult>;
    
    /// Create a new issue
    #[allow(dead_code)] // Will be used when ticket creation UI is implemented
    async fn create_issue(&self, data: CreateIssueData) -> Result<Ticket>;
    
    /// Update an existing issue
    #[allow(dead_code)] // Will be used when update feature is implemented
    async fn update_issue(&self, key: &str, data: UpdateIssueData) -> Result<()>;
    
    /// Transition an issue to a new status
    #[allow(dead_code)] // Will be used when transitions are implemented
    async fn transition_issue(
        &self,
        key: &str,
        transition_id: &str,
        comment: Option<String>,
    ) -> Result<()>;
    
    /// Get available transitions for an issue
    #[allow(dead_code)] // Will be used when transitions are implemented
    async fn get_transitions(&self, key: &str) -> Result<Vec<Transition>>;
    
    /// Add a comment to an issue
    #[allow(dead_code)] // Will be used when commenting is implemented
    async fn add_comment(&self, key: &str, comment: String) -> Result<()>;
}

/// Search result with pagination
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub start_at: usize,
    #[allow(dead_code)] // Field available for future use
    pub max_results: usize,
    pub total: usize,
    pub issues: Vec<Ticket>,
}

impl SearchResult {
    #[allow(dead_code)] // Will be used when pagination is implemented
    pub fn has_more(&self) -> bool {
        self.start_at + self.issues.len() < self.total
    }
    
    #[allow(dead_code)] // Will be used when pagination is implemented
    pub fn next_start_at(&self) -> usize {
        self.start_at + self.issues.len()
    }
}

/// Data for creating a new issue
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when ticket creation UI is implemented
pub struct CreateIssueData {
    pub project_key: String,
    #[allow(dead_code)] // Will be used when create issue is fully implemented
    pub issue_type: String,
    pub summary: String,
    #[allow(dead_code)] // Will be used when create issue is fully implemented
    pub description: Option<String>,
    #[allow(dead_code)] // Will be used when create issue is fully implemented
    pub assignee: Option<String>,
    #[allow(dead_code)] // Will be used when create issue is fully implemented
    pub priority: Option<String>,
}

/// Data for updating an issue
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when update feature is implemented
pub struct UpdateIssueData {
    pub fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Available transition for an issue
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when transitions are implemented
pub struct Transition {
    pub id: String,
    pub name: String,
    pub to_status: String,
}
