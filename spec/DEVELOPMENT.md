# Development Guide

## Getting Started

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **jira-cli**: Required for authentication integration
  ```bash
  # Install jira-cli (check their documentation for latest method)
  # Example: cargo install jira-cli
  # Or: brew install jira-cli (if available)
  ```

- **Terminal**: Terminal with color and Unicode support
  - iTerm2 (macOS)
  - Alacritty
  - Windows Terminal (Windows)
  - Any modern terminal emulator

### Initial Setup

```bash
# Clone repository (when available)
git clone <repository-url>
cd lazyjira

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests
cargo test

# Run with watch mode for TDD
cargo watch -x test
```

## Development Workflow

### TDD Workflow

1. **Write failing test** (`cargo test` should fail)
2. **Write minimal code** to make test pass
3. **Refactor** while keeping tests green
4. **Commit** with descriptive message

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_ticket_service_create_ticket

# Run integration tests only
cargo test --test '*'

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Watch mode (auto-run tests on file change)
cargo watch -x test
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without compiling
cargo check

# Run all checks
cargo fmt && cargo clippy && cargo test
```

## Project Structure

```
lazyjira/
├── src/
│   ├── main.rs              # Entry point
│   ├── app/                 # Application layer
│   ├── ui/                  # UI layer
│   ├── domain/              # Domain layer
│   ├── infrastructure/      # Infrastructure layer
│   └── utils/               # Utilities
├── tests/                   # Integration tests
│   ├── common/              # Test utilities
│   └── integration/         # Integration test suites
├── docs/                    # Additional documentation
├── Cargo.toml              # Dependencies
├── Cargo.lock              # Lock file (git ignored)
└── README.md               # Project README
```

## Adding a New Feature

### Step 1: Write Tests First

```rust
// src/domain/services/ticket_service.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_new_feature() {
        // Arrange
        let service = TicketService::new(mock_client());
        
        // Act
        let result = service.new_feature().await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Step 2: Implement Feature

```rust
// src/domain/services/ticket_service.rs

impl TicketService {
    pub async fn new_feature(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

### Step 3: Integration Test

```rust
// tests/integration/ticket_service.rs

#[tokio::test]
async fn test_new_feature_integration() {
    // Integration test
}
```

## Code Style

### Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Prefer `Result<T, E>` over panics
- Use `?` operator for error propagation
- Document public APIs with `///`

### Naming Conventions

- **Modules**: `snake_case`
- **Types**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Tests**: `test_<function>_<scenario>_<expected>`

### Example

```rust
/// Service for managing tickets.
pub struct TicketService {
    api_client: Box<dyn ApiClient>,
}

impl TicketService {
    /// Creates a new ticket with the given data.
    ///
    /// # Errors
    ///
    /// Returns an error if the API call fails or validation fails.
    pub async fn create_ticket(
        &self,
        data: TicketData,
    ) -> Result<Ticket> {
        // Implementation
    }
}
```

## Testing Guidelines

### Unit Tests

- Located in same file with `#[cfg(test)]`
- Test one behavior per test
- Use descriptive test names
- Mock external dependencies

### Integration Tests

- Located in `tests/` directory
- Test multiple components together
- Use real dependencies where possible
- Mock external services (Jira API)

### Test Data

- Use builders for complex test data
- Create fixtures for common scenarios
- Keep test data minimal and focused

See [TDD Guidelines](./TDD_GUIDELINES.md) for detailed testing practices.

## Debugging

### Debug Build

```bash
# Build with debug symbols
cargo build

# Run with debugger
rust-gdb target/debug/lazyjira
```

### Logging

```rust
use log::{debug, info, warn, error};

debug!("Debug message: {}", value);
info!("Info message");
warn!("Warning message");
error!("Error message");
```

```bash
# Run with logging
RUST_LOG=debug cargo run
RUST_LOG=lazyjira=debug cargo run
```

## Dependencies

### Adding Dependencies

1. Add to `Cargo.toml`:
   ```toml
   [dependencies]
   new-crate = "1.0.0"
   ```

2. Run `cargo build` to update `Cargo.lock`

3. Document why the dependency is needed

### Dependency Guidelines

- Prefer well-maintained crates
- Check license compatibility
- Minimize dependencies
- Document non-obvious choices

## Git Workflow

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add ticket creation functionality
fix: resolve authentication token refresh issue
test: add tests for ticket service
docs: update architecture documentation
refactor: simplify state management
```

### Branch Naming

- `feature/feature-name` - New features
- `fix/bug-description` - Bug fixes
- `test/test-description` - Test additions
- `refactor/refactor-description` - Refactoring
- `docs/doc-update` - Documentation

## CI/CD (Future)

### Planned CI Steps

1. Format check (`cargo fmt --check`)
2. Lint check (`cargo clippy`)
3. Test suite (`cargo test`)
4. Coverage check (`cargo tarpaulin`)
5. Build check (`cargo build --release`)

## Performance Considerations

### Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin lazyjira
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Use criterion for custom benchmarks
```

## Common Tasks

### Adding a New Command

1. Add test in `app/commands/tests.rs`
2. Implement command handler
3. Add to command registry
4. Add keyboard shortcut mapping
5. Update help text

### Adding a New UI Component

1. Create component file in `ui/components/`
2. Write tests for rendering
3. Integrate into main UI
4. Add event handling

### Adding API Integration

1. Define API client trait
2. Implement mock for testing
3. Implement real client
4. Add integration tests with mock server

## Troubleshooting

### Common Issues

**Tests fail with "cannot find test"**
- Ensure test function has `#[test]` or `#[tokio::test]` attribute
- Check module visibility (`pub` or `pub(crate)`)

**Async test issues**
- Use `#[tokio::test]` for async tests
- Ensure `tokio` test features enabled

**Terminal UI not rendering**
- Check terminal capabilities
- Verify color support
- Check terminal size

**jira-cli integration issues**
- Verify jira-cli is installed
- Check configuration file location
- Test jira-cli authentication separately

## Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [ratatui Documentation](https://docs.rs/ratatui/)
- [jira-cli Documentation](https://github.com/ankitpokhrel/jira-cli)

## Getting Help

- Check existing documentation
- Review test cases for examples
- Check architecture documentation
- Review similar implementations in codebase
