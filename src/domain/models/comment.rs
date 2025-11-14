use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::user::User;

/// Represents a Jira comment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Comment {
    pub id: String,
    pub author: User,
    pub body: String,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

impl Comment {
    #[allow(dead_code)] // Used in tests
    pub fn new(id: String, author: User, body: String, created: DateTime<Utc>) -> Self {
        Self {
            id,
            author,
            body,
            created,
            updated: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_new() {
        let author = User::new("123456".to_string(), "John Doe".to_string());
        let comment = Comment::new(
            "10000".to_string(),
            author,
            "This is a comment".to_string(),
            Utc::now(),
        );

        assert_eq!(comment.id, "10000");
        assert_eq!(comment.body, "This is a comment");
        assert_eq!(comment.author.display_name, "John Doe");
    }
}
