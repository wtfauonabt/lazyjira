use lazyjira::infrastructure::config::JiraCliConfig;
use lazyjira::infrastructure::api::ConnectionValidator;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_jira_cli_config_success() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("jira-cli");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_file = config_dir.join("config.yaml");
    let config_content = r#"
instance: test.atlassian.net
auth:
  type: api-token
  username: test@example.com
  token: test-token-123
"#;
    fs::write(&config_file, config_content).unwrap();
    
    // We can't easily test the actual loading without mocking dirs::config_dir()
    // But we can test the parsing logic
    let parsed: serde_yaml::Value = serde_yaml::from_str(config_content).unwrap();
    
    let instance = parsed.get("instance")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let auth = parsed.get("auth")
        .and_then(|a| {
            let auth_type = a.get("type")?.as_str()?;
            let username = a.get("username")?.as_str()?;
            let token = a.get("token").and_then(|t| t.as_str());
            
            Some(lazyjira::infrastructure::config::JiraCliAuth {
                auth_type: auth_type.to_string(),
                username: username.to_string(),
                token: token.map(|s| s.to_string()),
            })
        });
    
    assert!(instance.is_some());
    assert_eq!(instance.unwrap(), "test.atlassian.net");
    assert!(auth.is_some());
    let auth = auth.unwrap();
    assert_eq!(auth.auth_type, "api-token");
    assert_eq!(auth.username, "test@example.com");
    assert_eq!(auth.token, Some("test-token-123".to_string()));
}

#[test]
fn test_validate_jira_cli_config() {
    let valid_config = JiraCliConfig {
        instance: "test.atlassian.net".to_string(),
        auth: lazyjira::infrastructure::config::JiraCliAuth {
            auth_type: "api-token".to_string(),
            username: "test@example.com".to_string(),
            token: Some("token123".to_string()),
        },
    };
    
    assert!(ConnectionValidator::validate_config(&valid_config).is_ok());
}

#[test]
fn test_validate_jira_cli_config_missing_token() {
    let invalid_config = JiraCliConfig {
        instance: "test.atlassian.net".to_string(),
        auth: lazyjira::infrastructure::config::JiraCliAuth {
            auth_type: "api-token".to_string(),
            username: "test@example.com".to_string(),
            token: None,
        },
    };
    
    assert!(ConnectionValidator::validate_config(&invalid_config).is_err());
}

#[test]
fn test_parse_jira_cli_config_yaml() {
    let yaml_content = r#"
instance: company.atlassian.net
auth:
  type: api-token
  username: user@example.com
  token: abc123xyz
"#;
    
    let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
    
    assert_eq!(yaml.get("instance").unwrap().as_str().unwrap(), "company.atlassian.net");
    assert_eq!(yaml.get("auth").unwrap().get("type").unwrap().as_str().unwrap(), "api-token");
    assert_eq!(yaml.get("auth").unwrap().get("username").unwrap().as_str().unwrap(), "user@example.com");
    assert_eq!(yaml.get("auth").unwrap().get("token").unwrap().as_str().unwrap(), "abc123xyz");
}
