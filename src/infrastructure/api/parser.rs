use crate::domain::models::ticket::{Priority, Status, StatusCategory, Ticket};
use crate::domain::models::user::User;
use crate::domain::models::comment::Comment;
use crate::utils::{LazyJiraError, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Parse a Jira issue JSON response into a Ticket
pub fn parse_issue(json: &Value) -> Result<Ticket> {
    let key = json
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing 'key' field".to_string()))?
        .to_string();

    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing 'id' field".to_string()))?
        .to_string();

    let fields = json
        .get("fields")
        .ok_or_else(|| LazyJiraError::Parse("Missing 'fields' object".to_string()))?;

    let summary = fields
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing 'summary' field".to_string()))?
        .to_string();

    let status = parse_status(fields)?;
    let priority = parse_priority(fields)?;
    let assignee = parse_assignee(fields)?;
    let issue_type = parse_issue_type(fields)?;
    let project_key = parse_project_key(fields)?;
    let description = parse_description(fields)?;
    let created = parse_datetime(fields, "created")?;
    let updated = parse_datetime(fields, "updated")?;

    Ok(Ticket {
        id,
        key,
        summary,
        status,
        assignee,
        priority,
        issue_type,
        project_key,
        description,
        created,
        updated,
    })
}

/// Parse status from fields object
fn parse_status(fields: &Value) -> Result<Status> {
    let status_obj = fields
        .get("status")
        .ok_or_else(|| LazyJiraError::Parse("Missing 'status' field".to_string()))?;

    let id = status_obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing status 'id' field".to_string()))?
        .to_string();

    let name = status_obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing status 'name' field".to_string()))?
        .to_string();

    let category = status_obj
        .get("statusCategory")
        .and_then(|sc| sc.get("key"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing 'statusCategory.key' field".to_string()))?;

    let status_category = match category {
        "new" => StatusCategory::ToDo,
        "indeterminate" => StatusCategory::InProgress,
        "done" => StatusCategory::Done,
        _ => {
            return Err(LazyJiraError::Parse(format!(
                "Unknown status category: {}",
                category
            )));
        }
    };

    Ok(Status {
        id,
        name,
        category: status_category,
    })
}

/// Parse priority from fields object
fn parse_priority(fields: &Value) -> Result<Priority> {
    let priority_obj = fields.get("priority");

    // Priority is optional, default to Medium if missing
    if priority_obj.is_none() {
        return Ok(Priority::Medium);
    }

    let priority_obj = priority_obj.unwrap();
    let name = priority_obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Medium");

    match name {
        "Lowest" => Ok(Priority::Lowest),
        "Low" => Ok(Priority::Low),
        "Medium" => Ok(Priority::Medium),
        "High" => Ok(Priority::High),
        "Highest" => Ok(Priority::Highest),
        "Critical" => Ok(Priority::Critical),
        _ => {
            // Try to parse by ID as fallback
            let id = priority_obj.get("id").and_then(|v| v.as_str()).unwrap_or("3");
            match id {
                "1" => Ok(Priority::Lowest),
                "2" => Ok(Priority::Low),
                "3" => Ok(Priority::Medium),
                "4" => Ok(Priority::High),
                "5" => Ok(Priority::Highest),
                _ => Ok(Priority::Medium), // Default fallback
            }
        }
    }
}

/// Parse assignee from fields object
fn parse_assignee(fields: &Value) -> Result<Option<User>> {
    let assignee_obj = fields.get("assignee");

    if assignee_obj.is_none() || assignee_obj.unwrap().is_null() {
        return Ok(None);
    }

    let assignee_obj = assignee_obj.unwrap();

    let account_id = assignee_obj
        .get("accountId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse("Missing assignee 'accountId' field".to_string()))?
        .to_string();

    let display_name = assignee_obj
        .get("displayName")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let email_address = assignee_obj
        .get("emailAddress")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Some(User {
        account_id,
        display_name,
        email_address,
    }))
}

/// Parse issue type from fields object
fn parse_issue_type(fields: &Value) -> Result<String> {
    let issue_type_obj = fields
        .get("issuetype")
        .ok_or_else(|| LazyJiraError::Parse("Missing 'issuetype' field".to_string()))?;

    issue_type_obj
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| LazyJiraError::Parse("Missing issuetype 'name' field".to_string()))
}

/// Parse project key from fields object
fn parse_project_key(fields: &Value) -> Result<String> {
    let project_obj = fields
        .get("project")
        .ok_or_else(|| LazyJiraError::Parse("Missing 'project' field".to_string()))?;

    project_obj
        .get("key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| LazyJiraError::Parse("Missing project 'key' field".to_string()))
}

/// Parse description from fields object (simplified - converts Atlassian Document Format to plain text)
fn parse_description(fields: &Value) -> Result<Option<String>> {
    let description_obj = fields.get("description");

    if description_obj.is_none() || description_obj.unwrap().is_null() {
        return Ok(None);
    }

    let description_obj = description_obj.unwrap();

    // Try to extract text from Atlassian Document Format
    // This is a simplified version - full implementation would handle all ADF node types
    if let Some(content) = description_obj.get("content").and_then(|c| c.as_array()) {
        let mut text_parts = Vec::new();
        extract_text_from_adf(content, &mut text_parts);
        if text_parts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(text_parts.join("\n")))
        }
    } else {
        Ok(None)
    }
}

/// Recursively extract text from Atlassian Document Format
fn extract_text_from_adf(content: &[Value], text_parts: &mut Vec<String>) {
    for node in content {
        if let Some(node_type) = node.get("type").and_then(|v| v.as_str()) {
            match node_type {
                "paragraph" | "heading" | "listItem" => {
                    if let Some(node_content) = node.get("content").and_then(|c| c.as_array()) {
                        extract_text_from_adf(node_content, text_parts);
                    }
                }
                "text" => {
                    if let Some(text) = node.get("text").and_then(|v| v.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                "hardBreak" => {
                    text_parts.push("\n".to_string());
                }
                _ => {
                    // Recursively process nested content
                    if let Some(node_content) = node.get("content").and_then(|c| c.as_array()) {
                        extract_text_from_adf(node_content, text_parts);
                    }
                }
            }
        }
    }
}

/// Parse datetime from fields object
fn parse_datetime(fields: &Value, field_name: &str) -> Result<DateTime<Utc>> {
    let datetime_str = fields
        .get(field_name)
        .and_then(|v| v.as_str())
        .ok_or_else(|| LazyJiraError::Parse(format!("Missing '{}' field", field_name)))?;

    // Jira returns ISO 8601 format: "2024-01-15T10:30:00.000+0000"
    // Parse with chrono
    DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f%z")
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try without timezone
            DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f")
                .map(|dt| dt.with_timezone(&Utc))
        })
        .or_else(|_| {
            // Try ISO 8601 format
            datetime_str.parse::<DateTime<Utc>>()
        })
        .map_err(|e| {
            LazyJiraError::Parse(format!(
                "Failed to parse {} datetime '{}': {}",
                field_name, datetime_str, e
            ))
        })
}

/// Parse comments from Jira comments API response
pub fn parse_comments(json: &Value) -> Result<Vec<Comment>> {
    log::debug!("parse_comments: Starting to parse comments");
    log::debug!("parse_comments: JSON type: {}", if json.is_array() { "array" } else if json.is_object() { "object" } else { "other" });
    
    // The Jira comments API returns a response with a "comments" array
    // Handle both direct array response and wrapped response
    let comments_array = if json.is_array() {
        log::debug!("parse_comments: Response is an array");
        // If the response is directly an array
        json.as_array().ok_or_else(|| {
            LazyJiraError::Parse("Expected array or object with 'comments' field".to_string())
        })?
    } else {
        log::debug!("parse_comments: Response is an object, looking for 'comments' field");
        // If the response is an object with a "comments" field
        match json.get("comments") {
            Some(comments_val) => {
                log::debug!("parse_comments: Found 'comments' field");
                if let Some(arr) = comments_val.as_array() {
                    log::debug!("parse_comments: 'comments' is an array with {} items", arr.len());
                    arr
                } else {
                    let available_keys: Vec<String> = json
                        .as_object()
                        .map(|obj| obj.keys().map(|k| k.clone()).collect())
                        .unwrap_or_default();
                    log::error!("parse_comments: 'comments' field is not an array. Available keys: {:?}", available_keys);
                    return Err(LazyJiraError::Parse(format!(
                        "'comments' field is not an array. Available keys: {:?}",
                        available_keys
                    )));
                }
            }
            None => {
                let available_keys: Vec<String> = json
                    .as_object()
                    .map(|obj| obj.keys().map(|k| k.clone()).collect())
                    .unwrap_or_default();
                log::error!("parse_comments: Missing 'comments' field. Available keys: {:?}", available_keys);
                return Err(LazyJiraError::Parse(format!(
                    "Missing 'comments' array in response. Available keys: {:?}",
                    available_keys
                )));
            }
        }
    };
    
    log::debug!("parse_comments: Processing {} comments", comments_array.len());

    let mut comments = Vec::new();
    for (idx, comment_json) in comments_array.iter().enumerate() {
        log::debug!("parse_comments: Processing comment at index {}", idx);
        
        // Skip invalid comments instead of failing completely
        let id = match comment_json
            .get("id")
            .and_then(|v| v.as_str())
        {
            Some(id_str) => {
                log::debug!("parse_comments: Comment {} has id: {}", idx, id_str);
                id_str.to_string()
            }
            None => {
                log::warn!("parse_comments: Skipping comment at index {}: missing 'id' field", idx);
                continue;
            }
        };

        let author_obj = match comment_json.get("author") {
            Some(author) => author,
            None => {
                eprintln!("Warning: Skipping comment {}: missing 'author' field", id);
                continue;
            }
        };

        let account_id = match author_obj
            .get("accountId")
            .and_then(|v| v.as_str())
        {
            Some(account_id_str) => account_id_str.to_string(),
            None => {
                eprintln!("Warning: Skipping comment {}: missing author 'accountId' field", id);
                continue;
            }
        };

        let display_name = author_obj
            .get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let email_address = author_obj
            .get("emailAddress")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let author = User {
            account_id,
            display_name,
            email_address,
        };

        // Extract text from Atlassian Document Format
        let mut body_parts = Vec::new();
        if let Some(body_obj) = comment_json.get("body") {
            if let Some(content) = body_obj.get("content").and_then(|c| c.as_array()) {
                extract_text_from_adf(content, &mut body_parts);
            }
        } else {
            eprintln!("Warning: Comment {} has no body, using empty string", id);
        }
        
        let body = if body_parts.is_empty() {
            "".to_string()
        } else {
            body_parts.join("\n")
        };

        let created = match parse_datetime(comment_json, "created") {
            Ok(dt) => dt,
            Err(e) => {
                eprintln!("Warning: Skipping comment {}: failed to parse 'created' datetime: {}", id, e);
                continue;
            }
        };
        
        let updated = comment_json
            .get("updated")
            .and_then(|v| v.as_str())
            .and_then(|_| {
                parse_datetime(comment_json, "updated").ok()
            });

        comments.push(Comment {
            id,
            author,
            body,
            created,
            updated,
        });
        log::debug!("parse_comments: Successfully parsed comment {}", comments.len());
    }

    log::debug!("parse_comments: Successfully parsed {} comments total", comments.len());
    Ok(comments)
}

/// Parse search results from Jira search API response
/// Note: This function is kept for potential future use or testing
#[allow(dead_code)]
pub fn parse_search_results(json: &Value) -> Result<(usize, usize, usize, Vec<Ticket>)> {
    let start_at = json
        .get("startAt")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let max_results = json
        .get("maxResults")
        .and_then(|v| v.as_u64())
        .unwrap_or(50) as usize;

    let total = json
        .get("total")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    // Try to get issues array - could be "issues" or "values" depending on endpoint
    let issues_array = json
        .get("issues")
        .or_else(|| json.get("values"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            // Debug: log what keys are available
            let available_keys: Vec<String> = json
                .as_object()
                .map(|obj| obj.keys().map(|k| k.clone()).collect())
                .unwrap_or_default();
            LazyJiraError::Parse(format!(
                "Missing 'issues' or 'values' array. Available keys: {:?}",
                available_keys
            ))
        })?;

    let mut tickets = Vec::new();
    for (idx, issue) in issues_array.iter().enumerate() {
        match parse_issue(issue) {
            Ok(ticket) => tickets.push(ticket),
            Err(e) => {
                // Log error with more context
                eprintln!(
                    "Warning: Failed to parse issue at index {}: {:?}. Issue data: {:?}",
                    idx,
                    e,
                    issue
                );
            }
        }
    }

    Ok((start_at, max_results, total, tickets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_basic() {
        let json_str = r#"
        {
          "id": "10000",
          "key": "PROJ-123",
          "fields": {
            "summary": "Fix bug in authentication",
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
              "emailAddress": "john.doe@example.com"
            },
            "issuetype": {
              "name": "Bug"
            },
            "project": {
              "key": "PROJ"
            },
            "description": {
              "type": "doc",
              "version": 1,
              "content": [
                {
                  "type": "paragraph",
                  "content": [
                    {
                      "type": "text",
                      "text": "This is a description."
                    }
                  ]
                }
              ]
            },
            "created": "2024-01-15T10:30:00.000+0000",
            "updated": "2024-01-16T14:20:00.000+0000"
          }
        }
        "#;
        let json: Value = serde_json::from_str(json_str).unwrap();
        let ticket = parse_issue(&json).unwrap();

        assert_eq!(ticket.key, "PROJ-123");
        assert_eq!(ticket.summary, "Fix bug in authentication");
        assert_eq!(ticket.status.name, "In Progress");
        assert_eq!(ticket.status.category, StatusCategory::InProgress);
        assert_eq!(ticket.priority, Priority::High);
        assert!(ticket.assignee.is_some());
        assert_eq!(ticket.assignee.as_ref().unwrap().display_name, "John Doe");
    }

    #[test]
    fn test_parse_issue_without_assignee() {
        let json_str = r#"
        {
          "id": "10001",
          "key": "PROJ-124",
          "fields": {
            "summary": "Test ticket",
            "status": {
              "id": "1",
              "name": "To Do",
              "statusCategory": {
                "key": "new"
              }
            },
            "priority": {
              "name": "Medium",
              "id": "3"
            },
            "assignee": null,
            "issuetype": {
              "name": "Task"
            },
            "project": {
              "key": "PROJ"
            },
            "description": null,
            "created": "2024-01-15T10:30:00.000+0000",
            "updated": "2024-01-15T10:30:00.000+0000"
          }
        }
        "#;
        let json: Value = serde_json::from_str(json_str).unwrap();
        let ticket = parse_issue(&json).unwrap();

        assert_eq!(ticket.key, "PROJ-124");
        assert!(ticket.assignee.is_none());
    }

    #[test]
    fn test_parse_search_results() {
        let json_str = r#"
        {
          "startAt": 0,
          "maxResults": 50,
          "total": 2,
          "issues": [
            {
              "id": "10000",
              "key": "PROJ-123",
              "fields": {
                "summary": "Fix bug",
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
                "summary": "Add feature",
                "status": {
                  "id": "1",
                  "name": "To Do",
                  "statusCategory": { "key": "new" }
                },
                "priority": { "name": "Medium", "id": "3" },
                "assignee": null,
                "issuetype": { "name": "Story" },
                "project": { "key": "PROJ" },
                "description": null,
                "created": "2024-01-14T09:00:00.000+0000",
                "updated": "2024-01-14T09:00:00.000+0000"
              }
            }
          ]
        }
        "#;
        let json: Value = serde_json::from_str(json_str).unwrap();
        let (start_at, max_results, total, tickets) = parse_search_results(&json).unwrap();

        assert_eq!(start_at, 0);
        assert_eq!(max_results, 50);
        assert_eq!(total, 2);
        assert_eq!(tickets.len(), 2);
        assert_eq!(tickets[0].key, "PROJ-123");
        assert_eq!(tickets[1].key, "PROJ-124");
    }

    #[test]
    fn test_parse_empty_search_results() {
        let json_str = r#"
        {
          "startAt": 0,
          "maxResults": 50,
          "total": 0,
          "issues": []
        }
        "#;
        let json: Value = serde_json::from_str(json_str).unwrap();
        let (start_at, max_results, total, tickets) = parse_search_results(&json).unwrap();

        assert_eq!(start_at, 0);
        assert_eq!(max_results, 50);
        assert_eq!(total, 0);
        assert_eq!(tickets.len(), 0);
    }

    #[test]
    fn test_parse_status_categories() {
        let test_cases = vec![
            ("new", StatusCategory::ToDo),
            ("indeterminate", StatusCategory::InProgress),
            ("done", StatusCategory::Done),
        ];

        for (key, expected_category) in test_cases {
            let json_str = format!(
                r#"
                {{
                  "id": "1",
                  "key": "PROJ-1",
                  "fields": {{
                    "summary": "Test",
                    "status": {{
                      "id": "1",
                      "name": "Test Status",
                      "statusCategory": {{
                        "key": "{}"
                      }}
                    }},
                    "priority": {{ "name": "Medium", "id": "3" }},
                    "assignee": null,
                    "issuetype": {{ "name": "Task" }},
                    "project": {{ "key": "PROJ" }},
                    "description": null,
                    "created": "2024-01-15T10:30:00.000+0000",
                    "updated": "2024-01-15T10:30:00.000+0000"
                  }}
                }}
                "#,
                key
            );
            let json: Value = serde_json::from_str(&json_str).unwrap();
            let ticket = parse_issue(&json).unwrap();
            assert_eq!(ticket.status.category, expected_category);
        }
    }
}
