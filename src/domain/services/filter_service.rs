use crate::domain::models::ticket::{StatusCategory, Ticket};

/// Service for filtering tickets
#[allow(dead_code)] // Will be used when filtering is implemented
pub struct FilterService;

impl FilterService {
    /// Filter tickets by status category
    #[allow(dead_code)] // Will be used when filtering is implemented
    pub fn filter_by_status_category(
        tickets: &[Ticket],
        category: StatusCategory,
    ) -> Vec<Ticket> {
        tickets
            .iter()
            .filter(|ticket| ticket.status.category == category)
            .cloned()
            .collect()
    }

    /// Filter tickets assigned to a user
    #[allow(dead_code)] // Will be used when filtering is implemented
    pub fn filter_by_assignee(tickets: &[Ticket], account_id: &str) -> Vec<Ticket> {
        tickets
            .iter()
            .filter(|ticket| {
                ticket
                    .assignee
                    .as_ref()
                    .map(|u| u.account_id == account_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Filter tickets by text search in summary
    #[allow(dead_code)] // Will be used when filtering is implemented
    pub fn filter_by_text(tickets: &[Ticket], query: &str) -> Vec<Ticket> {
        let query_lower = query.to_lowercase();
        tickets
            .iter()
            .filter(|ticket| {
                ticket.summary.to_lowercase().contains(&query_lower)
                    || ticket
                        .key
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ticket::{Status, StatusCategory};
    use crate::domain::models::user::User;

    fn create_test_tickets() -> Vec<Ticket> {
        vec![
            Ticket {
                id: "1".to_string(),
                key: "PROJ-1".to_string(),
                summary: "Todo ticket".to_string(),
                status: Status {
                    id: "1".to_string(),
                    name: "To Do".to_string(),
                    category: StatusCategory::ToDo,
                },
                assignee: Some(User::new("user1".to_string(), "User 1".to_string())),
                priority: crate::domain::models::ticket::Priority::Medium,
                issue_type: "Task".to_string(),
                project_key: "PROJ".to_string(),
                description: None,
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
            Ticket {
                id: "2".to_string(),
                key: "PROJ-2".to_string(),
                summary: "In progress ticket".to_string(),
                status: Status {
                    id: "3".to_string(),
                    name: "In Progress".to_string(),
                    category: StatusCategory::InProgress,
                },
                assignee: Some(User::new("user2".to_string(), "User 2".to_string())),
                priority: crate::domain::models::ticket::Priority::High,
                issue_type: "Bug".to_string(),
                project_key: "PROJ".to_string(),
                description: None,
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
            Ticket {
                id: "3".to_string(),
                key: "PROJ-3".to_string(),
                summary: "Done ticket".to_string(),
                status: Status {
                    id: "10000".to_string(),
                    name: "Done".to_string(),
                    category: StatusCategory::Done,
                },
                assignee: Some(User::new("user1".to_string(), "User 1".to_string())),
                priority: crate::domain::models::ticket::Priority::Low,
                issue_type: "Task".to_string(),
                project_key: "PROJ".to_string(),
                description: None,
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
            },
        ]
    }

    #[test]
    fn test_filter_by_status_category() {
        let tickets = create_test_tickets();
        let todo_tickets =
            FilterService::filter_by_status_category(&tickets, StatusCategory::ToDo);
        assert_eq!(todo_tickets.len(), 1);
        assert_eq!(todo_tickets[0].key, "PROJ-1");

        let done_tickets =
            FilterService::filter_by_status_category(&tickets, StatusCategory::Done);
        assert_eq!(done_tickets.len(), 1);
        assert_eq!(done_tickets[0].key, "PROJ-3");
    }

    #[test]
    fn test_filter_by_assignee() {
        let tickets = create_test_tickets();
        let user1_tickets = FilterService::filter_by_assignee(&tickets, "user1");
        assert_eq!(user1_tickets.len(), 2);
        assert!(user1_tickets.iter().all(|t| {
            t.assignee
                .as_ref()
                .map(|u| u.account_id == "user1")
                .unwrap_or(false)
        }));
    }

    #[test]
    fn test_filter_by_text() {
        let tickets = create_test_tickets();
        let filtered = FilterService::filter_by_text(&tickets, "todo");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].key, "PROJ-1");

        let filtered_by_key = FilterService::filter_by_text(&tickets, "PROJ-2");
        assert_eq!(filtered_by_key.len(), 1);
        assert_eq!(filtered_by_key[0].key, "PROJ-2");
    }
}
