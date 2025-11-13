use crate::utils::{LazyJiraError, Result};

/// Validate Jira instance URL
pub fn validate_instance(instance: &str) -> Result<()> {
    if instance.trim().is_empty() {
        return Err(LazyJiraError::Validation(
            "Instance cannot be empty".to_string(),
        ));
    }

    // Basic validation - should be a valid domain
    if !instance.contains('.') {
        return Err(LazyJiraError::Validation(
            "Instance must be a valid domain".to_string(),
        ));
    }

    Ok(())
}

/// Validate ticket key format (e.g., PROJ-123)
pub fn validate_ticket_key(key: &str) -> Result<()> {
    if key.trim().is_empty() {
        return Err(LazyJiraError::Validation(
            "Ticket key cannot be empty".to_string(),
        ));
    }

    // Basic format: PROJECT-NUMBER
    if !key.contains('-') {
        return Err(LazyJiraError::Validation(
            "Ticket key must be in format PROJECT-NUMBER".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_instance() {
        assert!(validate_instance("company.atlassian.net").is_ok());
        assert!(validate_instance("").is_err());
        assert!(validate_instance("invalid").is_err());
    }

    #[test]
    fn test_validate_ticket_key() {
        assert!(validate_ticket_key("PROJ-123").is_ok());
        assert!(validate_ticket_key("TEST-456").is_ok());
        assert!(validate_ticket_key("").is_err());
        assert!(validate_ticket_key("PROJ123").is_err());
        assert!(validate_ticket_key("invalid").is_err());
    }
}
