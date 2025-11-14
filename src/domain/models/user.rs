use serde::{Deserialize, Serialize};

/// Represents a Jira user
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub account_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
}

impl User {
    #[allow(dead_code)] // Used in tests
    pub fn new(account_id: String, display_name: String) -> Self {
        Self {
            account_id,
            display_name,
            email_address: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(
            "123456".to_string(),
            "John Doe".to_string(),
        );

        assert_eq!(user.account_id, "123456");
        assert_eq!(user.display_name, "John Doe");
        assert_eq!(user.email_address, None);
    }
}
