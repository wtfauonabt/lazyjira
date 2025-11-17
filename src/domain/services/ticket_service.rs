use crate::domain::models::ticket::Ticket;
use crate::infrastructure::api::ApiClient;
use crate::infrastructure::api::client::CreateIssueData;
use crate::utils::Result;
use std::sync::Arc;

/// Service for ticket operations
#[allow(dead_code)] // Will be used when service layer is integrated
pub struct TicketService {
    api_client: Arc<dyn ApiClient>,
}

impl TicketService {
    /// Create a new ticket service
    #[allow(dead_code)] // Will be used when service layer is integrated
    pub fn new(api_client: Arc<dyn ApiClient>) -> Self {
        Self { api_client }
    }

    /// Get a ticket by key
    #[allow(dead_code)] // Will be used when service layer is integrated
    pub async fn get_ticket(&self, key: &str) -> Result<Ticket> {
        self.api_client.get_issue(key).await
    }

    /// Create a new ticket
    #[allow(dead_code)] // Will be used when service layer is integrated
    pub async fn create_ticket(&self, data: CreateIssueData) -> Result<Ticket> {
        // Validate data before creating
        if data.summary.trim().is_empty() {
            return Err(crate::utils::LazyJiraError::Validation(
                "Summary cannot be empty".to_string(),
            ));
        }

        if data.project_key.trim().is_empty() {
            return Err(crate::utils::LazyJiraError::Validation(
                "Project key cannot be empty".to_string(),
            ));
        }

        self.api_client.create_issue(data).await
    }

    /// Search tickets using JQL
    #[allow(dead_code)] // Will be used when service layer is integrated
    pub async fn search_tickets(
        &self,
        jql: &str,
        start_at: usize,
        max_results: usize,
    ) -> Result<Vec<Ticket>> {
        let result = self
            .api_client
            .search_issues(jql, start_at, max_results)
            .await?;
        Ok(result.issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ticket::{Status, StatusCategory};
    use crate::infrastructure::api::client::ApiClient;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockApiClient;

    #[async_trait]
    impl ApiClient for MockApiClient {
        async fn get_issue(&self, key: &str) -> Result<Ticket> {
            Ok(Ticket::new(
                key.to_string(),
                "Test ticket".to_string(),
                Status {
                    id: "1".to_string(),
                    name: "To Do".to_string(),
                    category: StatusCategory::ToDo,
                },
            ))
        }

        async fn search_issues(
            &self,
            _jql: &str,
            _start_at: usize,
            _max_results: usize,
        ) -> Result<crate::infrastructure::api::client::SearchResult> {
            Ok(crate::infrastructure::api::client::SearchResult {
                start_at: 0,
                max_results: 50,
                total: 1,
                issues: vec![Ticket::new(
                    "PROJ-123".to_string(),
                    "Test".to_string(),
                    Status {
                        id: "1".to_string(),
                        name: "To Do".to_string(),
                        category: StatusCategory::ToDo,
                    },
                )],
            })
        }

        async fn create_issue(&self, data: CreateIssueData) -> Result<Ticket> {
            Ok(Ticket::new(
                format!("PROJ-{}", uuid::Uuid::new_v4()),
                data.summary,
                Status {
                    id: "1".to_string(),
                    name: "To Do".to_string(),
                    category: StatusCategory::ToDo,
                },
            ))
        }

        async fn update_issue(
            &self,
            _key: &str,
            _data: crate::infrastructure::api::client::UpdateIssueData,
        ) -> Result<()> {
            Ok(())
        }

        async fn transition_issue(
            &self,
            _key: &str,
            _transition_id: &str,
            _comment: Option<String>,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_transitions(
            &self,
            _key: &str,
        ) -> Result<Vec<crate::infrastructure::api::client::Transition>> {
            Ok(vec![])
        }

        async fn add_comment(&self, _key: &str, _comment: String) -> Result<()> {
            Ok(())
        }

        async fn get_comments(&self, _key: &str) -> Result<Vec<crate::domain::models::comment::Comment>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_get_ticket() {
        let service = TicketService::new(Arc::new(MockApiClient));
        let ticket = service.get_ticket("PROJ-123").await.unwrap();
        assert_eq!(ticket.key, "PROJ-123");
    }

    #[tokio::test]
    async fn test_create_ticket_success() {
        let service = TicketService::new(Arc::new(MockApiClient));
        let data = CreateIssueData {
            project_key: "PROJ".to_string(),
            issue_type: "Task".to_string(),
            summary: "New ticket".to_string(),
            description: None,
            assignee: None,
            priority: None,
        };

        let ticket = service.create_ticket(data).await.unwrap();
        assert_eq!(ticket.summary, "New ticket");
    }

    #[tokio::test]
    async fn test_create_ticket_empty_summary() {
        let service = TicketService::new(Arc::new(MockApiClient));
        let data = CreateIssueData {
            project_key: "PROJ".to_string(),
            issue_type: "Task".to_string(),
            summary: "   ".to_string(), // Empty after trim
            description: None,
            assignee: None,
            priority: None,
        };

        let result = service.create_ticket(data).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::utils::LazyJiraError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn test_search_tickets() {
        let service = TicketService::new(Arc::new(MockApiClient));
        let tickets = service
            .search_tickets("assignee = currentUser()", 0, 50)
            .await
            .unwrap();
        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].key, "PROJ-123");
    }
}
