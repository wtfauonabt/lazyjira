use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::user::User;

/// Represents a Jira ticket/issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ticket {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub status: Status,
    pub assignee: Option<User>,
    pub priority: Priority,
    pub issue_type: String,
    pub project_key: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Ticket status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub id: String,
    pub name: String,
    pub category: StatusCategory,
}

/// Status category (To Do, In Progress, Done)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusCategory {
    #[serde(rename = "new")]
    ToDo,
    #[serde(rename = "indeterminate")]
    InProgress,
    #[serde(rename = "done")]
    Done,
}

/// Ticket priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Lowest,
    Low,
    Medium,
    High,
    Highest,
    Critical,
}

impl Ticket {
    /// Create a new ticket (for testing)
    #[allow(dead_code)] // Used in tests
    pub fn new(
        key: String,
        summary: String,
        status: Status,
    ) -> Self {
        let project_key = key.split('-').next().unwrap_or("PROJ").to_string();
        Self {
            id: format!("{}", uuid::Uuid::new_v4()),
            key,
            summary,
            status,
            assignee: None,
            priority: Priority::Medium,
            issue_type: "Task".to_string(),
            project_key,
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        }
    }

    /// Check if ticket is in "Done" status category
    #[allow(dead_code)] // Will be used for filtering
    pub fn is_done(&self) -> bool {
        matches!(self.status.category, StatusCategory::Done)
    }

    /// Check if ticket is in "In Progress" status category
    #[allow(dead_code)] // Will be used for filtering
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status.category, StatusCategory::InProgress)
    }

    /// Check if ticket is in "To Do" status category
    #[allow(dead_code)] // Will be used for filtering
    pub fn is_todo(&self) -> bool {
        matches!(self.status.category, StatusCategory::ToDo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_new() {
        let ticket = Ticket::new(
            "PROJ-123".to_string(),
            "Test ticket".to_string(),
            Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
        );

        assert_eq!(ticket.key, "PROJ-123");
        assert_eq!(ticket.summary, "Test ticket");
        assert!(ticket.is_todo());
    }

    #[test]
    fn test_ticket_status_checks() {
        let todo_ticket = Ticket::new(
            "PROJ-1".to_string(),
            "Todo".to_string(),
            Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
        );

        let in_progress_ticket = Ticket::new(
            "PROJ-2".to_string(),
            "In Progress".to_string(),
            Status {
                id: "3".to_string(),
                name: "In Progress".to_string(),
                category: StatusCategory::InProgress,
            },
        );

        let done_ticket = Ticket::new(
            "PROJ-3".to_string(),
            "Done".to_string(),
            Status {
                id: "10000".to_string(),
                name: "Done".to_string(),
                category: StatusCategory::Done,
            },
        );

        assert!(todo_ticket.is_todo());
        assert!(!todo_ticket.is_in_progress());
        assert!(!todo_ticket.is_done());

        assert!(!in_progress_ticket.is_todo());
        assert!(in_progress_ticket.is_in_progress());
        assert!(!in_progress_ticket.is_done());

        assert!(!done_ticket.is_todo());
        assert!(!done_ticket.is_in_progress());
        assert!(done_ticket.is_done());
    }
}
