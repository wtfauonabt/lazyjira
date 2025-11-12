# Test-Driven Development (TDD) Guidelines

## TDD Workflow

### Red-Green-Refactor Cycle

1. **Red**: Write a failing test
2. **Green**: Write minimal code to make test pass
3. **Refactor**: Improve code while keeping tests green
4. **Repeat**

## Testing Principles

### 1. Write Tests First
- Never write production code without a failing test
- Tests define the expected behavior
- Tests serve as documentation

### 2. Test Behavior, Not Implementation
- Test what the code does, not how it does it
- Focus on public APIs and observable behavior
- Avoid testing private implementation details

### 3. One Assertion Per Test (When Possible)
- Each test should verify one behavior
- Makes failures easier to diagnose
- Easier to understand test intent

### 4. Test Naming Convention
```rust
#[test]
fn test_<unit_under_test>_<condition>_<expected_result>() {
    // Given
    // When
    // Then
}
```

Example:
```rust
#[test]
fn test_ticket_service_create_ticket_with_valid_data_returns_ticket() {
    // Given
    let service = TicketService::new(mock_api_client());
    let ticket_data = valid_ticket_data();
    
    // When
    let result = service.create_ticket(ticket_data).await;
    
    // Then
    assert!(result.is_ok());
    assert_eq!(result.unwrap().key, "PROJ-123");
}
```

## Test Organization

### Unit Tests
- Located in the same file as source code
- Use `#[cfg(test)]` module
- Test individual functions/methods
- Mock external dependencies

```rust
// src/domain/services/ticket_service.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::api::MockApiClient;
    
    #[tokio::test]
    async fn test_create_ticket_success() {
        // Test implementation
    }
}
```

### Integration Tests
- Located in `tests/` directory
- Test multiple components together
- Use real dependencies where possible
- Mock external services (Jira API)

```rust
// tests/integration/ticket_workflow.rs

#[tokio::test]
async fn test_create_and_transition_ticket() {
    // Integration test implementation
}
```

## Test Structure (AAA Pattern)

### Arrange
- Set up test data
- Create mocks
- Configure test environment

### Act
- Execute the code under test
- Capture results

### Assert
- Verify expected outcomes
- Check side effects
- Validate state changes

```rust
#[test]
fn test_example() {
    // Arrange
    let input = "test";
    let expected = "TEST";
    
    // Act
    let result = transform(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Mocking Strategy

### API Client Mocking
```rust
pub struct MockApiClient {
    // Mock implementation
}

impl ApiClient for MockApiClient {
    async fn get_ticket(&self, key: &str) -> Result<Ticket> {
        // Return mock data
    }
}
```

### Using Mockito for HTTP
```rust
use mockito::Server;

#[tokio::test]
async fn test_api_call() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_body(r#"{"key": "PROJ-123"}"#)
        .create();
    
    // Test code
    
    mock.assert();
}
```

## Test Data Builders

### Builder Pattern for Test Data
```rust
pub struct TicketBuilder {
    key: String,
    summary: String,
    status: Status,
    // ... other fields
}

impl TicketBuilder {
    pub fn new() -> Self {
        Self {
            key: "PROJ-123".to_string(),
            summary: "Test ticket".to_string(),
            status: Status::ToDo,
            // ... defaults
        }
    }
    
    pub fn with_key(mut self, key: &str) -> Self {
        self.key = key.to_string();
        self
    }
    
    pub fn build(self) -> Ticket {
        Ticket {
            key: self.key,
            summary: self.summary,
            status: self.status,
            // ...
        }
    }
}

// Usage in tests
let ticket = TicketBuilder::new()
    .with_key("PROJ-456")
    .with_status(Status::InProgress)
    .build();
```

## Test Coverage Requirements

### Coverage Goals
- **Minimum**: 80% overall coverage
- **Critical paths**: 100% coverage
- **Public APIs**: 100% coverage
- **Error handling**: All error paths tested

### Coverage Tools
- Use `cargo-tarpaulin` for coverage reports
- Run coverage in CI/CD
- Fail build if coverage drops below threshold

```bash
cargo tarpaulin --out Html --output-dir coverage
```

## Testing Async Code

### Async Test Setup
```rust
#[tokio::test]
async fn test_async_function() {
    // Test async code
}
```

### Testing with Timeouts
```rust
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_with_timeout() {
    let result = timeout(
        Duration::from_secs(5),
        async_function()
    ).await;
    
    assert!(result.is_ok());
}
```

## Testing Error Cases

### Test All Error Paths
```rust
#[test]
fn test_create_ticket_with_invalid_data_returns_error() {
    // Test validation errors
}

#[test]
fn test_create_ticket_with_network_error_returns_error() {
    // Test network errors
}

#[test]
fn test_create_ticket_with_auth_error_returns_error() {
    // Test authentication errors
}
```

## Property-Based Testing

### Using QuickCheck (Optional)
```rust
use quickcheck::QuickCheck;

#[test]
fn test_ticket_serialization_roundtrip() {
    QuickCheck::new()
        .quickcheck(|ticket: Ticket| {
            let serialized = serde_json::to_string(&ticket).unwrap();
            let deserialized: Ticket = serde_json::from_str(&serialized).unwrap();
            ticket == deserialized
        });
}
```

## Test Fixtures

### Common Test Fixtures
```rust
// tests/fixtures/mod.rs

pub mod tickets {
    pub fn sample_ticket() -> Ticket {
        // Return sample ticket
    }
    
    pub fn sample_ticket_list() -> Vec<Ticket> {
        // Return sample list
    }
}

pub mod api {
    pub fn mock_jira_response() -> String {
        // Return mock JSON response
    }
}
```

## Continuous Testing

### Watch Mode
```bash
cargo watch -x test
```

### Test Commands
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'

# Run with coverage
cargo tarpaulin
```

## Test Documentation

### Document Test Intent
```rust
/// Tests that creating a ticket with valid data succeeds
/// and returns a ticket with the correct key.
#[test]
fn test_create_ticket_success() {
    // ...
}
```

### Use Descriptive Assertions
```rust
// Good
assert_eq!(ticket.status, Status::Done, 
    "Ticket should be marked as Done after resolution");

// Less clear
assert!(ticket.status == Status::Done);
```

## Common Testing Patterns

### Testing State Machines
```rust
#[test]
fn test_ticket_transition_valid() {
    let mut ticket = Ticket::new(Status::ToDo);
    
    assert!(ticket.transition_to(Status::InProgress).is_ok());
    assert_eq!(ticket.status, Status::InProgress);
}

#[test]
fn test_ticket_transition_invalid() {
    let mut ticket = Ticket::new(Status::Done);
    
    assert!(ticket.transition_to(Status::ToDo).is_err());
}
```

### Testing Filters
```rust
#[test]
fn test_filter_by_status() {
    let tickets = vec![
        ticket_with_status(Status::ToDo),
        ticket_with_status(Status::InProgress),
        ticket_with_status(Status::Done),
    ];
    
    let filtered = filter_by_status(tickets, Status::InProgress);
    
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].status, Status::InProgress);
}
```

## TDD Checklist

Before writing production code:
- [ ] Write failing test
- [ ] Test compiles
- [ ] Test fails for expected reason
- [ ] Write minimal code to pass
- [ ] All tests pass
- [ ] Refactor if needed
- [ ] All tests still pass
- [ ] Code review

## Anti-Patterns to Avoid

### ❌ Don't Skip Tests
- Every feature needs tests
- No "quick fixes" without tests

### ❌ Don't Test Implementation Details
- Test public APIs, not private methods
- Avoid testing internal state

### ❌ Don't Write Tests After Code
- Follow TDD strictly
- Tests should drive design

### ❌ Don't Ignore Failing Tests
- Fix or remove failing tests immediately
- Never commit failing tests

### ❌ Don't Over-Mock
- Use real objects when possible
- Mock only external dependencies

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testing Async Code](https://tokio.rs/tokio/tutorial/testing)
- [Mockito Documentation](https://docs.rs/mockito/)
