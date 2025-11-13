use serde::{Deserialize, Serialize};

/// Represents a Jira board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub board_type: BoardType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoardType {
    Scrum,
    Kanban,
}
