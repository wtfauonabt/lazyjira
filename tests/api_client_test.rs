use lazyjira::infrastructure::api::{ApiClient, JiraApiClient};
use lazyjira::infrastructure::config::{JiraCliConfig, JiraCliAuth};
use mockito::Server;
use serde_json::json;

/// Helper to create a mock Jira API client for testing
async fn create_test_client(server: &Server) -> JiraApiClient {
    let config = JiraCliConfig {
        instance: "test.atlassian.net".to_string(), // Dummy, we'll override base_url
        auth: JiraCliAuth {
            auth_type: "api-token".to_string(),
            username: "test@example.com".to_string(),
            token: Some("test-token".to_string()),
        },
    };
    
    // Use the mock server URL
    let base_url = format!("{}/rest/api/3", server.url());
    JiraApiClient::new(&config, Some(base_url)).unwrap()
}

#[tokio::test]
async fn test_get_issue_success() {
    let mut server = Server::new_async().await;
    
    let mock_response = json!({
        "id": "10000",
        "key": "PROJ-123",
        "fields": {
            "summary": "Test issue",
            "status": {
                "id": "3",
                "name": "In Progress",
                "statusCategory": {
                    "key": "indeterminate"
                }
            },
            "priority": {
                "name": "High",
                "id": "2"
            },
            "assignee": {
                "accountId": "123456",
                "displayName": "John Doe",
                "emailAddress": "john@example.com"
            },
            "issuetype": {
                "name": "Bug"
            },
            "project": {
                "key": "PROJ"
            },
            "description": null,
            "created": "2024-01-15T10:30:00.000+0000",
            "updated": "2024-01-16T14:20:00.000+0000"
        }
    });

    let mock = server
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let client = create_test_client(&server).await;
    let ticket = client.get_issue("PROJ-123").await.unwrap();

    assert_eq!(ticket.key, "PROJ-123");
    assert_eq!(ticket.summary, "Test issue");
    assert_eq!(ticket.status.name, "In Progress");
    mock.assert();
}

#[tokio::test]
async fn test_get_issue_not_found() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/rest/api/3/issue/PROJ-999")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages":["Issue does not exist"]}"#)
        .expect_at_least(1)
        .expect_at_most(1) // Should not retry on 404
        .create();

    let client = create_test_client(&server).await;
    let result = client.get_issue("PROJ-999").await;

    assert!(result.is_err());
    // Check it's an API error, not a network error (which would indicate retries)
    assert!(matches!(
        result.unwrap_err(),
        lazyjira::utils::LazyJiraError::Api(_)
    ));
    mock.assert();
}

#[tokio::test]
async fn test_get_issue_unauthorized() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorMessages":["Unauthorized"]}"#)
        .create();

    let client = create_test_client(&server).await;
    let result = client.get_issue("PROJ-123").await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        lazyjira::utils::LazyJiraError::Authentication(_)
    ));
    mock.assert();
}

#[tokio::test]
async fn test_search_issues_success() {
    let mut server = Server::new_async().await;

    let mock_response = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 2,
        "issues": [
            {
                "id": "10000",
                "key": "PROJ-123",
                "fields": {
                    "summary": "First issue",
                    "status": {
                        "id": "3",
                        "name": "In Progress",
                        "statusCategory": { "key": "indeterminate" }
                    },
                    "priority": { "name": "High", "id": "2" },
                    "assignee": {
                        "accountId": "123456",
                        "displayName": "John Doe"
                    },
                    "issuetype": { "name": "Bug" },
                    "project": { "key": "PROJ" },
                    "description": null,
                    "created": "2024-01-15T10:30:00.000+0000",
                    "updated": "2024-01-16T14:20:00.000+0000"
                }
            },
            {
                "id": "10001",
                "key": "PROJ-124",
                "fields": {
                    "summary": "Second issue",
                    "status": {
                        "id": "1",
                        "name": "To Do",
                        "statusCategory": { "key": "new" }
                    },
                    "priority": { "name": "Medium", "id": "3" },
                    "assignee": null,
                    "issuetype": { "name": "Task" },
                    "project": { "key": "PROJ" },
                    "description": null,
                    "created": "2024-01-14T09:00:00.000+0000",
                    "updated": "2024-01-14T09:00:00.000+0000"
                }
            }
        ]
    });

    let mock = server
        .mock("GET", "/rest/api/3/search")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("jql".to_string(), "assignee = currentUser()".to_string()),
            mockito::Matcher::UrlEncoded("startAt".to_string(), "0".to_string()),
            mockito::Matcher::UrlEncoded("maxResults".to_string(), "50".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let client = create_test_client(&server).await;
    let result = client
        .search_issues("assignee = currentUser()", 0, 50)
        .await
        .unwrap();

    assert_eq!(result.total, 2);
    assert_eq!(result.issues.len(), 2);
    assert_eq!(result.issues[0].key, "PROJ-123");
    assert_eq!(result.issues[1].key, "PROJ-124");
    mock.assert();
}

#[tokio::test]
async fn test_search_issues_empty() {
    let mut server = Server::new_async().await;

    let mock_response = json!({
        "startAt": 0,
        "maxResults": 50,
        "total": 0,
        "issues": []
    });

    let mock = server
        .mock("GET", "/rest/api/3/search")
        .match_query(mockito::Matcher::AnyOf(vec![
            mockito::Matcher::UrlEncoded("jql".to_string(), "project = PROJ".to_string()),
            mockito::Matcher::UrlEncoded("jql".to_string(), "project%20%3D%20PROJ".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let client = create_test_client(&server).await;
    let result = client.search_issues("project = PROJ", 0, 50).await.unwrap();

    assert_eq!(result.total, 0);
    assert_eq!(result.issues.len(), 0);
    mock.assert();
}
