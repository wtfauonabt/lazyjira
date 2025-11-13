use crate::domain::models::ticket::Ticket;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
    Frame,
};
use std::collections::HashSet;

/// State for the ticket list widget
#[derive(Debug, Clone)]
pub struct TicketListState {
    pub tickets: Vec<Ticket>,
    pub selected_indices: HashSet<usize>,
    pub focused_index: Option<usize>,
    pub scroll_offset: usize,
}

impl Default for TicketListState {
    fn default() -> Self {
        Self {
            tickets: vec![],
            selected_indices: HashSet::new(),
            focused_index: None,
            scroll_offset: 0,
        }
    }
}

impl TicketListState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tickets and reset selection/focus
    pub fn set_tickets(&mut self, tickets: Vec<Ticket>) {
        self.tickets = tickets;
        self.selected_indices.clear();
        self.focused_index = if self.tickets.is_empty() {
            None
        } else {
            Some(0)
        };
        self.scroll_offset = 0;
    }

    /// Move focus up
    pub fn move_up(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx > 0 {
                self.focused_index = Some(idx - 1);
                self.ensure_focused_visible();
            }
        } else if !self.tickets.is_empty() {
            self.focused_index = Some(0);
        }
    }

    /// Move focus down
    pub fn move_down(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx < self.tickets.len().saturating_sub(1) {
                self.focused_index = Some(idx + 1);
                self.ensure_focused_visible();
            }
        } else if !self.tickets.is_empty() {
            self.focused_index = Some(0);
        }
    }

    /// Toggle selection of focused ticket
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.focused_index {
            if self.selected_indices.contains(&idx) {
                self.selected_indices.remove(&idx);
            } else {
                self.selected_indices.insert(idx);
            }
        }
    }

    /// Ensure focused item is visible in viewport
    fn ensure_focused_visible(&mut self) {
        // This will be used when we have a viewport height
        // For now, just ensure scroll_offset is reasonable
    }

    /// Get the focused ticket
    pub fn focused_ticket(&self) -> Option<&Ticket> {
        self.focused_index
            .and_then(|idx| self.tickets.get(idx))
    }
}

/// Ticket list widget
pub struct TicketList<'a> {
    state: &'a TicketListState,
    theme: &'a Theme,
}

impl<'a> TicketList<'a> {
    pub fn new(state: &'a TicketListState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render the ticket list
    pub fn render(self, frame: &mut Frame, area: Rect) {
        if self.state.tickets.is_empty() {
            self.render_empty_state(frame, area);
            return;
        }

        // Create list items
        let items: Vec<ListItem> = self
            .state
            .tickets
            .iter()
            .enumerate()
            .map(|(idx, ticket)| self.create_list_item(idx, ticket))
            .collect();

        // Create list with state
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tickets")
                    .title_style(self.theme.focused),
            )
            .highlight_style(self.theme.selected)
            .highlight_symbol("> ");

        // Convert to ListState for rendering
        let mut list_state = ListState::default();
        if let Some(focused_idx) = self.state.focused_index {
            list_state.select(Some(focused_idx));
        }

        StatefulWidget::render(list, area, frame, &mut list_state);
    }

    /// Create a list item for a ticket
    fn create_list_item(&self, idx: usize, ticket: &Ticket) -> ListItem {
        let is_selected = self.state.selected_indices.contains(&idx);
        let is_focused = self.state.focused_index == Some(idx);

        // Build the line with ticket information
        let mut spans = vec![];

        // Selection indicator
        if is_selected {
            spans.push(Span::styled("✓ ", self.theme.success));
        } else {
            spans.push(Span::raw("  "));
        }

        // Ticket key
        spans.push(Span::styled(
            format!("{} ", ticket.key),
            if is_focused {
                self.theme.focused
            } else {
                self.theme.normal
            },
        ));

        // Status (color-coded)
        let status_category_str = match ticket.status.category {
            crate::domain::models::ticket::StatusCategory::ToDo => "new",
            crate::domain::models::ticket::StatusCategory::InProgress => "indeterminate",
            crate::domain::models::ticket::StatusCategory::Done => "done",
        };
        let status_style = self.theme.status_style(status_category_str);
        spans.push(Span::styled(
            format!("[{}] ", ticket.status.name),
            status_style,
        ));

        // Summary
        spans.push(Span::styled(
            ticket.summary.clone(),
            if is_focused {
                self.theme.focused
            } else {
                self.theme.normal
            },
        ));

        // Assignee (if present)
        if let Some(assignee) = &ticket.assignee {
            spans.push(Span::raw(" • "));
            spans.push(Span::styled(
                assignee.display_name.clone(),
                self.theme.normal,
            ));
        }

        ListItem::new(Line::from(spans))
    }

    /// Render empty state
    fn render_empty_state(&self, frame: &mut Frame, area: Rect) {
        let empty_text = "No tickets found. Press 'r' to refresh.";
        let paragraph = Paragraph::new(empty_text)
            .style(self.theme.normal)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tickets"),
            );

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ticket::{Priority, Status, StatusCategory};
    use chrono::Utc;

    fn create_test_ticket(key: &str, summary: &str) -> Ticket {
        Ticket {
            id: format!("{}", key),
            key: key.to_string(),
            summary: summary.to_string(),
            status: Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
            assignee: None,
            priority: Priority::Medium,
            issue_type: "Task".to_string(),
            project_key: "TEST".to_string(),
            description: None,
            created: Utc::now(),
            updated: Utc::now(),
        }
    }

    #[test]
    fn test_ticket_list_state_new() {
        let state = TicketListState::new();
        assert!(state.tickets.is_empty());
        assert!(state.selected_indices.is_empty());
        assert!(state.focused_index.is_none());
    }

    #[test]
    fn test_ticket_list_state_set_tickets() {
        let mut state = TicketListState::new();
        let tickets = vec![
            create_test_ticket("TEST-1", "Test ticket 1"),
            create_test_ticket("TEST-2", "Test ticket 2"),
        ];

        state.set_tickets(tickets);
        assert_eq!(state.tickets.len(), 2);
        assert_eq!(state.focused_index, Some(0));
        assert!(state.selected_indices.is_empty());
    }

    #[test]
    fn test_move_up_down() {
        let mut state = TicketListState::new();
        let tickets = vec![
            create_test_ticket("TEST-1", "Test ticket 1"),
            create_test_ticket("TEST-2", "Test ticket 2"),
            create_test_ticket("TEST-3", "Test ticket 3"),
        ];

        state.set_tickets(tickets);
        assert_eq!(state.focused_index, Some(0));

        state.move_down();
        assert_eq!(state.focused_index, Some(1));

        state.move_down();
        assert_eq!(state.focused_index, Some(2));

        state.move_down(); // Should not go beyond bounds
        assert_eq!(state.focused_index, Some(2));

        state.move_up();
        assert_eq!(state.focused_index, Some(1));

        state.move_up();
        assert_eq!(state.focused_index, Some(0));

        state.move_up(); // Should not go below 0
        assert_eq!(state.focused_index, Some(0));
    }

    #[test]
    fn test_toggle_selection() {
        let mut state = TicketListState::new();
        let tickets = vec![create_test_ticket("TEST-1", "Test ticket 1")];
        state.set_tickets(tickets);
        state.focused_index = Some(0);

        assert!(!state.selected_indices.contains(&0));

        state.toggle_selection();
        assert!(state.selected_indices.contains(&0));

        state.toggle_selection();
        assert!(!state.selected_indices.contains(&0));
    }

    #[test]
    fn test_focused_ticket() {
        let mut state = TicketListState::new();
        let tickets = vec![create_test_ticket("TEST-1", "Test ticket 1")];
        state.set_tickets(tickets);
        state.focused_index = Some(0);

        let focused = state.focused_ticket();
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().key, "TEST-1");
    }
}
