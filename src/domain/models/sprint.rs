use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a Jira sprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub state: SprintState,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SprintState {
    Future,
    Active,
    Closed,
}
