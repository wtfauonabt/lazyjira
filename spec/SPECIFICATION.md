# LazyJira Specification

## Project Overview

LazyJira is a terminal-based interactive interface for Jira, inspired by lazygit's intuitive and efficient workflow. The project aims to provide fast, keyboard-driven control over Jira tickets directly from the terminal, eliminating the need to switch between terminal and browser.

## Core Philosophy

- **Keyboard-first**: All operations should be accessible via keyboard shortcuts
- **Fast and efficient**: Minimize keystrokes and context switching
- **Visual feedback**: Clear, colorized terminal UI with status indicators
- **Context-aware**: Show relevant information based on current state
- **Non-destructive**: Safe operations with confirmation prompts for destructive actions

## Technology Stack

- **Language**: Rust
- **Testing**: Strict TDD (Test-Driven Development)
- **Dependencies**: 
  - `jira-cli` integration for API communication
  - Terminal UI library (e.g., `ratatui` or `cursive`)
- **Architecture**: Modular, testable components

## Core Features

### 1. Authentication & Configuration
- Integration with `jira-cli` authentication system
- Support for multiple Jira instances
- Configuration file management (`~/.lazyjira/config.toml`)
- Environment variable support for sensitive data
- Session management and token refresh

### 2. Ticket Views

#### 2.1 List View
- Display tickets in a scrollable list
- Multiple view modes:
  - **My Tickets**: Assigned to current user
  - **Recent**: Recently viewed/updated
  - **Sprint**: Current sprint tickets
  - **Board**: Tickets from selected board
  - **Search**: Custom JQL queries
- Sortable columns (priority, status, assignee, updated date)
- Filtering capabilities
- Color-coded status indicators
- Compact vs detailed view toggle

#### 2.2 Detail View
- Full ticket information display
- Expandable sections (description, comments, history, links)
- Inline editing capabilities
- Related tickets display
- Attachment preview/download

### 3. Ticket Operations

#### 3.1 Viewing
- Quick preview (peek view)
- Full detail view
- Navigate between related tickets
- View ticket history/changelog

#### 3.2 Creation
- Interactive ticket creation form
- Template support for common ticket types
- Field validation
- Quick create shortcuts

#### 3.3 Editing
- Inline field editing
- Bulk operations (multi-select)
- Quick field updates (status, assignee, priority)
- Comment addition

#### 3.4 Transitions
- Workflow state transitions
- Quick transition shortcuts
- Transition with comment
- Bulk transitions

### 4. Search & Filtering

#### 4.1 Quick Search
- Real-time filtering of current view
- JQL query builder
- Saved search queries
- Search history

#### 4.2 Filters
- Predefined filters (My Open, In Progress, etc.)
- Custom filter creation
- Filter presets management
- Quick filter toggle

### 5. Interactive Modes

#### 5.1 Main Panel Mode
- Default view with ticket list
- Context panel showing selected ticket details
- Status bar with current context

#### 5.2 Command Mode
- Command palette (similar to lazygit)
- Fuzzy search for commands
- Command history
- Keyboard shortcut hints

#### 5.3 Diff/Compare Mode
- Compare ticket states
- View changelog differences
- Side-by-side comparison

### 6. Quick Actions

- **Assign to me**: Quick self-assignment
- **Start progress**: Transition to "In Progress"
- **Resolve**: Quick resolution with default resolution
- **Add comment**: Quick comment addition
- **Link ticket**: Create ticket links
- **Watch/Unwatch**: Toggle watch status
- **Copy ticket key**: Copy ticket ID to clipboard

### 7. Board & Sprint Views

- Board visualization
- Sprint planning view
- Drag-and-drop ticket movement (if terminal supports)
- Sprint burndown visualization
- Capacity planning

### 8. Time Tracking

- Log work on tickets
- View time logged
- Time tracking reports
- Quick time entry

## User Interface Design

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│ [LazyJira] My Tickets (15)                    [Filter] │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────┐  ┌────────────────────────────┐ │
│  │                  │  │                            │ │
│  │  Ticket List     │  │  Ticket Details            │ │
│  │                  │  │                            │ │
│  │  [ ] PROJ-123    │  │  PROJ-123: Fix bug        │ │
│  │  [x] PROJ-124    │  │  Status: In Progress      │ │
│  │  [ ] PROJ-125    │  │  Assignee: John Doe       │ │
│  │  [ ] PROJ-126    │  │                            │ │
│  │                  │  │  Description:             │ │
│  │                  │  │  ...                      │ │
│  │                  │  │                            │ │
│  └──────────────────┘  └────────────────────────────┘ │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ Status: Connected | Jira: company.atlassian.net        │
└─────────────────────────────────────────────────────────┘
```

### Color Scheme

- **Status colors**: Green (Done), Yellow (In Progress), Red (Blocked), Blue (To Do)
- **Priority colors**: Red (Critical), Orange (High), Yellow (Medium), Gray (Low)
- **Syntax highlighting**: For descriptions, comments (markdown support)

## Keyboard Shortcuts

### Navigation
- `j/k` or `↑/↓`: Navigate list
- `h/l` or `←/→`: Switch panels
- `g`: Go to top
- `G`: Go to bottom
- `Ctrl+u/d`: Page up/down

### Actions
- `Enter`: View ticket details
- `c`: Create ticket
- `e`: Edit ticket
- `a`: Assign to me
- `s`: Start progress
- `r`: Resolve ticket
- `m`: Add comment
- `t`: Transition
- `f`: Filter/search
- `?`: Show help

### Quick Actions
- `Space`: Select/deselect ticket
- `x`: Toggle selection
- `*`: Select all
- `u`: Unselect all

### Mode Switching
- `:`: Command mode
- `Esc`: Return to normal mode
- `q`: Quit (with confirmation)

## Integration Points

### jira-cli Integration
- Use `jira-cli` for authentication
- Leverage existing configuration
- API calls through jira-cli or direct REST API
- Support for jira-cli plugins/extensions

### Jira REST API
- Direct API integration as fallback
- Rate limiting handling
- Pagination support
- Webhook support (future)

## Configuration

### Configuration File (`~/.lazyjira/config.toml`)

```toml
[jira]
instance = "company.atlassian.net"
username = "user@example.com"
# Use jira-cli config or environment variables for auth

[ui]
theme = "default"  # default, dark, light
show_avatars = true
compact_mode = false
refresh_interval = 30  # seconds

[shortcuts]
# Custom keyboard shortcuts

[filters]
# Saved filter presets

[templates]
# Ticket creation templates
```

## Error Handling

- Network errors: Retry with exponential backoff
- Authentication errors: Prompt for re-authentication
- API errors: Display user-friendly error messages
- Validation errors: Inline field validation feedback

## Performance Requirements

- Initial load: < 2 seconds
- List refresh: < 1 second
- Detail view: < 500ms
- Search response: < 300ms (with debouncing)
- Smooth scrolling: 60 FPS

## Testing Strategy (TDD)

### Unit Tests
- All business logic functions
- Data transformation functions
- Validation functions
- Mock API responses

### Integration Tests
- API integration (with mock server)
- Configuration loading
- Authentication flow

### E2E Tests
- User workflows
- Keyboard shortcuts
- UI interactions (using headless terminal)

### Test Coverage
- Minimum 80% code coverage
- 100% coverage for critical paths
- All public APIs must have tests

## Development Phases

### Phase 1: Foundation (MVP)
- [ ] Project setup with TDD structure
- [ ] Authentication integration
- [ ] Basic ticket list view
- [ ] Ticket detail view
- [ ] Basic navigation

### Phase 2: Core Operations
- [ ] Ticket creation
- [ ] Ticket editing
- [ ] Status transitions
- [ ] Commenting
- [ ] Search and filtering

### Phase 3: Enhanced Features
- [ ] Quick actions
- [ ] Board/sprint views
- [ ] Time tracking
- [ ] Bulk operations
- [ ] Advanced filtering

### Phase 4: Polish
- [ ] Performance optimization
- [ ] UI/UX improvements
- [ ] Documentation
- [ ] Error handling refinement
- [ ] Accessibility improvements

## Future Enhancements

- Multi-instance support (switch between Jira instances)
- Offline mode with sync
- Git integration (link commits to tickets)
- Custom workflows
- Plugin system
- Export/import functionality
- Dashboard view
- Notification system
- Collaboration features (real-time updates)

## Non-Goals

- Full browser replacement (focus on common workflows)
- Visual diagramming (keep it terminal-focused)
- Advanced reporting (use Jira web UI)
- Admin functions (user management, etc.)
