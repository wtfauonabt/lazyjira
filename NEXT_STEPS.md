# LazyJira Next Steps

## Current Status

✅ **Foundation Complete**
- Project structure initialized
- Core domain models implemented
- Configuration system ready
- Test infrastructure in place (21 tests passing)
- API client skeleton created

## Phase 1: MVP (Minimum Viable Product)

### Step 1: Complete API Client Implementation 🔴 **HIGH PRIORITY**

**Why First**: Everything depends on being able to fetch data from Jira.

**Tasks**:
1. **Implement JSON Parsing** (TDD)
   - [ ] Write tests for parsing Jira API ticket response
   - [ ] Implement `Ticket` deserialization from Jira JSON
   - [ ] Handle nested fields (status, assignee, priority)
   - [ ] Parse description (Atlassian Document Format)

2. **Complete Search Implementation**
   - [ ] Parse search results with pagination
   - [ ] Handle empty results
   - [ ] Implement proper error handling

3. **Add Rate Limiting**
   - [ ] Implement rate limiter
   - [ ] Add retry logic with exponential backoff
   - [ ] Handle 429 (Too Many Requests) responses

4. **Integration Tests**
   - [ ] Set up mockito server for API tests
   - [ ] Test successful API calls
   - [ ] Test error scenarios

**Estimated Time**: 2-3 days

---

### Step 2: Authentication & Connection 🔴 **HIGH PRIORITY**

**Why Second**: Need to verify we can connect before building UI.

**Tasks**:
1. **Test jira-cli Integration**
   - [ ] Verify jira-cli config loading works
   - [ ] Test with real jira-cli setup (if available)
   - [ ] Handle missing jira-cli config gracefully

2. **Connection Validation**
   - [ ] Add connection test on startup
   - [ ] Display connection status
   - [ ] Handle authentication errors

3. **Error Handling**
   - [ ] User-friendly error messages
   - [ ] Retry mechanism
   - [ ] Fallback to manual config

**Estimated Time**: 1-2 days

---

### Step 3: Basic Terminal UI Setup 🟡 **MEDIUM PRIORITY**

**Why Third**: Need UI framework before building views.

**Tasks**:
1. **Set up ratatui**
   - [ ] Initialize terminal UI
   - [ ] Create basic event loop
   - [ ] Handle terminal resize
   - [ ] Cleanup on exit

2. **Layout System**
   - [ ] Create main layout (list + detail panels)
   - [ ] Implement status bar
   - [ ] Add help bar (keyboard shortcuts)

3. **Event Handling**
   - [ ] Keyboard event processing
   - [ ] Map keys to commands
   - [ ] Handle Ctrl+C gracefully

4. **Theme System**
   - [ ] Define color scheme
   - [ ] Status colors (To Do, In Progress, Done)
   - [ ] Priority colors

**Estimated Time**: 2-3 days

---

### Step 4: Ticket List View 🟡 **MEDIUM PRIORITY**

**Why Fourth**: Core feature - users need to see their tickets.

**Tasks**:
1. **Display Tickets**
   - [ ] Render ticket list component
   - [ ] Show key, summary, status, assignee
   - [ ] Color-code by status
   - [ ] Handle empty state

2. **Navigation**
   - [ ] Implement j/k and arrow key navigation
   - [ ] Focus highlighting
   - [ ] Scroll to focused item

3. **Selection**
   - [ ] Toggle selection with Space
   - [ ] Visual selection indicator
   - [ ] Multi-select support

4. **Loading States**
   - [ ] Show loading indicator
   - [ ] Handle fetch errors
   - [ ] Refresh capability (R key)

**Estimated Time**: 2-3 days

---

### Step 5: Ticket Detail View 🟡 **MEDIUM PRIORITY**

**Why Fifth**: Users need to see full ticket information.

**Tasks**:
1. **Display Details**
   - [ ] Show all ticket fields
   - [ ] Format description (markdown support)
   - [ ] Display metadata (created, updated)
   - [ ] Show related tickets

2. **Comments**
   - [ ] Display comment thread
   - [ ] Format comment text
   - [ ] Show timestamps and authors

3. **Navigation**
   - [ ] Switch between list and detail views
   - [ ] Navigate to related tickets
   - [ ] Return to list (Esc)

**Estimated Time**: 2-3 days

---

### Step 6: Basic Operations 🟢 **LOWER PRIORITY**

**Why Sixth**: Core functionality after viewing works.

**Tasks**:
1. **Ticket Creation**
   - [ ] Create ticket form
   - [ ] Field validation
   - [ ] Submit to API
   - [ ] Refresh list after creation

2. **Status Transitions**
   - [ ] Show available transitions
   - [ ] Execute transition
   - [ ] Update UI after transition

3. **Quick Actions**
   - [ ] Assign to me (a)
   - [ ] Start progress (s)
   - [ ] Resolve (r)

**Estimated Time**: 3-4 days

---

## Recommended Development Order

```
Week 1:
├── Day 1-2: Complete API Client (JSON parsing, search)
├── Day 3: Authentication & Connection
└── Day 4-5: Basic UI Setup

Week 2:
├── Day 1-2: Ticket List View
├── Day 3-4: Ticket Detail View
└── Day 5: Testing & Bug Fixes

Week 3:
├── Day 1-2: Ticket Creation
├── Day 3: Status Transitions
└── Day 4-5: Quick Actions & Polish
```

## Quick Wins (Can Do Anytime)

These are smaller tasks that can be done between major features:

- [ ] Add more unit tests for edge cases
- [ ] Improve error messages
- [ ] Add logging throughout
- [ ] Document code with doc comments
- [ ] Fix compiler warnings
- [ ] Add integration tests
- [ ] Performance profiling
- [ ] Code cleanup and refactoring

## Testing Strategy

For each feature:
1. **Write tests first** (TDD)
2. **Unit tests** for business logic
3. **Integration tests** for API calls (with mocks)
4. **Manual testing** for UI components

## Success Criteria for MVP

- [ ] Can connect to Jira instance
- [ ] Can view list of tickets
- [ ] Can view ticket details
- [ ] Can navigate with keyboard
- [ ] Can create a ticket
- [ ] Can transition ticket status
- [ ] All tests passing
- [ ] No critical bugs

## After MVP (Phase 2)

Once MVP is complete, move to:
- Search and filtering
- Commenting
- Multi-select operations
- Command palette
- Board/sprint views

## Getting Started Right Now

**Immediate Next Task**: Complete API Client JSON Parsing

1. Start with a test:
   ```rust
   #[tokio::test]
   async fn test_parse_jira_ticket_response() {
       // Test parsing a sample Jira API response
   }
   ```

2. Create a sample JSON response (from Jira API docs)

3. Implement parsing logic

4. Make test pass

5. Repeat for search results

Would you like me to help you start with any specific step?
