# API Integration Specification

## Overview

LazyJira integrates with Jira through two primary mechanisms:
1. **jira-cli integration** - Leverage existing authentication and configuration
2. **Direct Jira REST API** - Direct API calls for full control

## jira-cli Integration

### Authentication

LazyJira will use jira-cli's authentication system to avoid duplicating credential management.

#### Configuration Discovery

```rust
// Check for jira-cli config location
// macOS/Linux: ~/.config/jira-cli/config.yaml
// Windows: %APPDATA%\jira-cli\config.yaml

pub fn discover_jira_cli_config() -> Option<PathBuf> {
    // Implementation
}
```

#### Reading Credentials

```rust
// Read jira-cli config file
// Extract:
// - Jira instance URL
// - Authentication method (API token, OAuth, etc.)
// - User credentials (if available)

pub struct JiraCliConfig {
    pub instance: String,
    pub auth_method: AuthMethod,
    pub credentials: Credentials,
}
```

#### Authentication Methods

1. **API Token**
   - Username + API token
   - Stored in jira-cli config

2. **OAuth2** (if supported by jira-cli)
   - Use jira-cli's OAuth tokens
   - Refresh token handling

3. **Basic Auth** (fallback)
   - Username + password
   - Less secure, avoid if possible

### Configuration File Format

jira-cli typically uses YAML:

```yaml
# ~/.config/jira-cli/config.yaml
instance: company.atlassian.net
auth:
  type: api-token
  username: user@example.com
  token: <api-token>
```

### Integration Strategy

#### Option 1: Use jira-cli as CLI Wrapper

```rust
// Execute jira-cli commands and parse output
pub async fn execute_jira_cli(args: &[&str]) -> Result<String> {
    // Run: jira-cli <args>
    // Parse JSON output
}
```

**Pros:**
- Leverages jira-cli's full feature set
- Automatic updates when jira-cli updates

**Cons:**
- Subprocess overhead
- Parsing text/JSON output
- Less control over API calls

#### Option 2: Direct API with jira-cli Config

```rust
// Read jira-cli config
// Use credentials directly with Jira REST API
pub struct JiraApiClient {
    base_url: String,
    credentials: Credentials,
}
```

**Pros:**
- Direct control over API calls
- Better error handling
- More efficient

**Cons:**
- Need to reimplement API logic
- Must keep up with Jira API changes

#### Recommended Approach: Hybrid

- Use jira-cli config for authentication
- Make direct REST API calls for control
- Fallback to jira-cli CLI for complex operations if needed

## Jira REST API Integration

### Base URL

```
https://{instance}.atlassian.net/rest/api/3
```

### Authentication

#### API Token Method

```http
Authorization: Basic {base64(username:api_token)}
```

#### OAuth2 Method

```http
Authorization: Bearer {access_token}
```

### Core Endpoints

#### Get Issue

```http
GET /rest/api/3/issue/{issueIdOrKey}
```

**Response:**
```json
{
  "id": "10000",
  "key": "PROJ-123",
  "fields": {
    "summary": "Fix bug",
    "status": {
      "name": "In Progress",
      "id": "3"
    },
    "assignee": {
      "displayName": "John Doe",
      "accountId": "123456"
    },
    "priority": {
      "name": "High",
      "id": "2"
    },
    "description": {
      "type": "doc",
      "version": 1,
      "content": [...]
    }
  }
}
```

#### Search Issues (JQL)

```http
GET /rest/api/3/search?jql={jql}&startAt=0&maxResults=50
```

**Example JQL:**
```
assignee = currentUser() AND status != Done ORDER BY updated DESC
```

#### Create Issue

```http
POST /rest/api/3/issue
Content-Type: application/json

{
  "fields": {
    "project": {"key": "PROJ"},
    "summary": "New issue",
    "issuetype": {"name": "Bug"},
    "description": {
      "type": "doc",
      "version": 1,
      "content": [...]
    }
  }
}
```

#### Update Issue

```http
PUT /rest/api/3/issue/{issueIdOrKey}
Content-Type: application/json

{
  "fields": {
    "summary": "Updated summary",
    "status": {"id": "3"}
  }
}
```

#### Transition Issue

```http
POST /rest/api/3/issue/{issueIdOrKey}/transitions
Content-Type: application/json

{
  "transition": {"id": "11"},
  "fields": {
    "resolution": {"name": "Fixed"}
  },
  "comment": {
    "body": {
      "type": "doc",
      "version": 1,
      "content": [...]
    }
  }
}
```

#### Add Comment

```http
POST /rest/api/3/issue/{issueIdOrKey}/comment
Content-Type: application/json

{
  "body": {
    "type": "doc",
    "version": 1,
    "content": [...]
  }
}
```

#### Get Transitions

```http
GET /rest/api/3/issue/{issueIdOrKey}/transitions
```

#### Get Projects

```http
GET /rest/api/3/project
```

#### Get Issue Types

```http
GET /rest/api/3/issuetype
```

### API Client Implementation

```rust
pub trait ApiClient: Send + Sync {
    async fn get_issue(&self, key: &str) -> Result<Issue>;
    async fn search_issues(&self, jql: &str, start_at: usize, max_results: usize) -> Result<SearchResult>;
    async fn create_issue(&self, data: CreateIssueData) -> Result<Issue>;
    async fn update_issue(&self, key: &str, data: UpdateIssueData) -> Result<()>;
    async fn transition_issue(&self, key: &str, transition_id: &str, comment: Option<Comment>) -> Result<()>;
    async fn get_transitions(&self, key: &str) -> Result<Vec<Transition>>;
    async fn add_comment(&self, key: &str, comment: Comment) -> Result<Comment>;
    async fn get_projects(&self) -> Result<Vec<Project>>;
    async fn get_issue_types(&self) -> Result<Vec<IssueType>>;
}
```

### Error Handling

#### HTTP Status Codes

- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid request
- `401 Unauthorized` - Authentication failed
- `403 Forbidden` - Permission denied
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limited
- `500 Internal Server Error` - Server error

#### Error Response Format

```json
{
  "errorMessages": ["Error message 1", "Error message 2"],
  "errors": {
    "field": "Error message for field"
  }
}
```

### Rate Limiting

Jira API has rate limits:
- **Cloud**: ~100 requests per minute per user
- **Server/Data Center**: Varies by configuration

#### Implementation

```rust
pub struct RateLimiter {
    requests_per_minute: usize,
    requests: Vec<Instant>,
}

impl RateLimiter {
    pub async fn wait_if_needed(&mut self) {
        // Remove requests older than 1 minute
        // If at limit, wait until oldest request expires
    }
}
```

### Pagination

Jira API uses pagination for large result sets:

```rust
pub struct PaginatedResult<T> {
    pub start_at: usize,
    pub max_results: usize,
    pub total: usize,
    pub values: Vec<T>,
}

impl<T> PaginatedResult<T> {
    pub fn has_more(&self) -> bool {
        self.start_at + self.values.len() < self.total
    }
    
    pub fn next_start_at(&self) -> usize {
        self.start_at + self.values.len()
    }
}
```

### Caching Strategy

#### Cache Levels

1. **In-Memory Cache**
   - Ticket details: 5 minutes TTL
   - Ticket list: 30 seconds TTL
   - Metadata (projects, issue types): 1 hour TTL

2. **Disk Cache** (optional)
   - Offline support
   - Search history
   - Filter presets

#### Cache Invalidation

- Invalidate on write operations (create, update, delete)
- Invalidate on explicit refresh
- TTL-based expiration

### Request/Response Models

```rust
// Request models
pub struct CreateIssueData {
    pub project_key: String,
    pub issue_type: String,
    pub summary: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
}

pub struct UpdateIssueData {
    pub fields: HashMap<String, serde_json::Value>,
}

// Response models
pub struct Issue {
    pub id: String,
    pub key: String,
    pub fields: IssueFields,
}

pub struct IssueFields {
    pub summary: String,
    pub status: Status,
    pub assignee: Option<User>,
    pub priority: Priority,
    pub description: Option<Document>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
```

### Testing API Integration

#### Mock Server

Use `mockito` or `wiremock` for integration tests:

```rust
use mockito::Server;

#[tokio::test]
async fn test_get_issue() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"10000","key":"PROJ-123","fields":{...}}"#)
        .create();
    
    let client = JiraApiClient::new(server.url());
    let issue = client.get_issue("PROJ-123").await.unwrap();
    
    assert_eq!(issue.key, "PROJ-123");
    mock.assert();
}
```

#### Test Fixtures

```rust
// tests/fixtures/api_responses.rs

pub fn sample_issue_json() -> &'static str {
    r#"
    {
      "id": "10000",
      "key": "PROJ-123",
      "fields": {
        "summary": "Test issue",
        "status": {"name": "To Do", "id": "1"}
      }
    }
    "#
}
```

## Security Considerations

### Credential Storage

- Never store credentials in plain text
- Use OS credential store when possible
- Encrypt sensitive configuration
- Support environment variables for CI/CD

### API Token Security

- Tokens should be read-only when possible
- Rotate tokens regularly
- Never commit tokens to version control
- Use `.gitignore` for config files with tokens

### HTTPS Only

- Always use HTTPS for API calls
- Validate SSL certificates
- Support custom CA certificates for self-hosted instances

## Future Enhancements

### Webhook Support

- Listen for Jira webhooks
- Real-time updates
- Event-driven UI refresh

### Batch Operations

- Batch API endpoints for bulk operations
- Reduce API calls
- Improve performance

### GraphQL API (Future)

- Jira is adding GraphQL support
- More efficient queries
- Better type safety

## References

- [Jira REST API Documentation](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [jira-cli GitHub](https://github.com/ankitpokhrel/jira-cli)
- [Atlassian API Token Guide](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/)
