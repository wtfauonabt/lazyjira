# LazyJira Architecture

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    User Interface Layer                 │
│  (Terminal UI, Event Handling, Rendering)              │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Application Layer                      │
│  (State Management, Command Processing, Workflows)     │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                    Domain Layer                         │
│  (Business Logic, Models, Validators)                  │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Infrastructure Layer                   │
│  (API Client, Config, Storage, jira-cli integration)   │
└─────────────────────────────────────────────────────────┘
```

## Module Structure

```
lazyjira/
├── src/
│   ├── main.rs                 # Entry point
│   ├── app/                    # Application layer
│   │   ├── mod.rs
│   │   ├── state.rs           # Application state
│   │   ├── commands.rs        # Command processing
│   │   └── workflows.rs       # Business workflows
│   ├── ui/                     # User interface layer
│   │   ├── mod.rs
│   │   ├── components/        # UI components
│   │   │   ├── mod.rs
│   │   │   ├── ticket_list.rs
│   │   │   ├── ticket_detail.rs
│   │   │   ├── command_palette.rs
│   │   │   └── status_bar.rs
│   │   ├── events.rs          # Event handling
│   │   ├── renderer.rs        # Rendering logic
│   │   └── theme.rs           # UI theming
│   ├── domain/                # Domain layer
│   │   ├── mod.rs
│   │   ├── models/           # Domain models
│   │   │   ├── mod.rs
│   │   │   ├── ticket.rs
│   │   │   ├── user.rs
│   │   │   ├── board.rs
│   │   │   └── sprint.rs
│   │   ├── services/         # Domain services
│   │   │   ├── mod.rs
│   │   │   ├── ticket_service.rs
│   │   │   └── filter_service.rs
│   │   └── validators.rs     # Validation logic
│   ├── infrastructure/       # Infrastructure layer
│   │   ├── mod.rs
│   │   ├── api/              # API client
│   │   │   ├── mod.rs
│   │   │   ├── client.rs
│   │   │   ├── jira_client.rs
│   │   │   └── jira_cli_adapter.rs
│   │   ├── config/           # Configuration
│   │   │   ├── mod.rs
│   │   │   └── config.rs
│   │   └── storage/          # Local storage
│   │       ├── mod.rs
│   │       └── cache.rs
│   └── utils/                # Utilities
│       ├── mod.rs
│       ├── error.rs          # Error types
│       └── logger.rs         # Logging
├── tests/                    # Integration tests
│   ├── common/
│   └── integration/
├── Cargo.toml
└── README.md
```

## Component Responsibilities

### Application Layer (`app/`)

**State Management (`state.rs`)**
- Manages application-wide state
- Current view mode
- Selected tickets
- Active filters
- UI state (panels, scroll positions)

**Commands (`commands.rs`)**
- Command parsing and routing
- Command execution
- Command history
- Undo/redo support

**Workflows (`workflows.rs`)**
- Complex multi-step operations
- Ticket creation workflow
- Bulk operation workflows
- Transition workflows

### UI Layer (`ui/`)

**Components**
- Reusable UI components
- Each component handles its own rendering
- Event handling delegated to app layer

**Events (`events.rs`)**
- Keyboard event processing
- Mouse event handling (if supported)
- Event routing to appropriate handlers

**Renderer (`renderer.rs`)**
- Terminal rendering coordination
- Layout management
- Screen refresh logic

### Domain Layer (`domain/`)

**Models**
- Pure data structures
- No side effects
- Immutable by default
- Serialization support

**Services**
- Business logic
- Domain rules enforcement
- Data transformation
- Validation

**Validators**
- Input validation
- Business rule validation
- Error message generation

### Infrastructure Layer (`infrastructure/`)

**API Client**
- HTTP client abstraction
- Jira REST API wrapper
- jira-cli integration adapter
- Rate limiting
- Retry logic
- Error handling

**Configuration**
- Configuration file parsing
- Environment variable handling
- Default value management
- Configuration validation

**Storage**
- Local cache management
- State persistence
- Search history
- Filter presets

## Design Patterns

### Repository Pattern
- Abstract data access
- Easy to mock for testing
- Support multiple backends (API, cache, mock)

### Command Pattern
- All user actions as commands
- Undo/redo support
- Command queuing for bulk operations

### Observer Pattern
- State change notifications
- UI updates on state changes
- Event-driven architecture

### Strategy Pattern
- Different rendering strategies
- Multiple authentication methods
- Various filter implementations

## Data Flow

### Reading Tickets
```
User Input → UI Event → Command Handler → Ticket Service → API Client → Jira API
                                                                    ↓
UI Update ← State Change ← Domain Model ← Response Parser ← HTTP Response
```

### Creating Ticket
```
User Input → UI Form → Validator → Ticket Service → API Client → Jira API
                                                           ↓
UI Update ← State Change ← Domain Model ← Response Parser ← HTTP Response
```

## Error Handling Strategy

### Error Types
```rust
pub enum LazyJiraError {
    Network(NetworkError),
    Authentication(AuthError),
    Validation(ValidationError),
    Api(ApiError),
    Config(ConfigError),
    Internal(InternalError),
}
```

### Error Propagation
- Use `Result<T, LazyJiraError>` for fallible operations
- Error context preservation
- User-friendly error messages
- Error recovery strategies

## Testing Architecture

### Test Structure
```
tests/
├── unit/              # Unit tests (alongside source)
│   └── #[cfg(test)] mod tests
├── integration/       # Integration tests
│   ├── api/
│   ├── workflows/
│   └── ui/
└── fixtures/          # Test data
    └── mock_responses/
```

### Mocking Strategy
- Mock API client for unit tests
- Mock server for integration tests
- In-memory storage for testing
- Test fixtures for common scenarios

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `serde`: Serialization
- `ratatui`: Terminal UI
- `crossterm`: Terminal control

### Testing Dependencies
- `mockito`: HTTP mocking
- `tempfile`: Temporary files
- `assert_matches`: Test assertions

### Development Dependencies
- `criterion`: Benchmarking
- `cargo-tarpaulin`: Code coverage

## Performance Considerations

### Caching Strategy
- Ticket list cache (TTL: 30s)
- Ticket detail cache (TTL: 5min)
- User/board metadata cache (TTL: 1hour)

### Lazy Loading
- Load ticket details on demand
- Paginate large lists
- Virtual scrolling for long lists

### Background Operations
- Refresh data in background
- Prefetch next page
- Cache warming

## Security Considerations

### Authentication
- Never store passwords in plain text
- Use jira-cli credential store
- Token refresh handling
- Secure credential storage

### API Security
- HTTPS only
- Certificate validation
- Rate limiting respect
- Input sanitization

## Extension Points

### Plugin System (Future)
- Custom commands
- Custom views
- Custom workflows
- Third-party integrations

### Configuration Extensions
- Custom themes
- Custom keyboard shortcuts
- Custom filters
- Custom templates
