use crate::domain::models::ticket::Ticket;
use std::collections::HashSet;

/// Application state
pub struct AppState {
    /// Current list of tickets
    pub tickets: Vec<Ticket>,
    
    /// Selected ticket indices
    pub selected_indices: HashSet<usize>,
    
    /// Currently focused ticket index
    #[allow(dead_code)] // Alternative state management - not currently used
    pub focused_index: Option<usize>,
    
    /// Current filter/search query
    #[allow(dead_code)] // Will be used when search is implemented
    pub filter_query: Option<String>,
    
    /// Current view mode
    #[allow(dead_code)] // Alternative state management - not currently used
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    #[allow(dead_code)] // Will be used when detail view is implemented
    Detail,
    #[allow(dead_code)] // Will be used when command palette is implemented
    Command,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tickets: vec![],
            selected_indices: HashSet::new(),
            focused_index: None,
            filter_query: None,
            view_mode: ViewMode::List,
        }
    }
}

impl AppState {
    #[allow(dead_code)] // Alternative state management - not currently used
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the focused ticket
    #[allow(dead_code)] // Alternative state management - not currently used
    pub fn focused_ticket(&self) -> Option<&Ticket> {
        self.focused_index
            .and_then(|idx| self.tickets.get(idx))
    }

    /// Get selected tickets
    #[allow(dead_code)] // Will be used when bulk operations are implemented
    pub fn selected_tickets(&self) -> Vec<&Ticket> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.tickets.get(idx))
            .collect()
    }

    /// Toggle selection of focused ticket
    #[allow(dead_code)] // Alternative state management - not currently used
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.focused_index {
            if self.selected_indices.contains(&idx) {
                self.selected_indices.remove(&idx);
            } else {
                self.selected_indices.insert(idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ticket::{Status, StatusCategory};

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.tickets.is_empty());
        assert!(state.selected_indices.is_empty());
        assert_eq!(state.view_mode, ViewMode::List);
    }

    #[test]
    fn test_focused_ticket() {
        let mut state = AppState::new();
        let ticket = Ticket::new(
            "PROJ-123".to_string(),
            "Test".to_string(),
            Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
        );
        state.tickets.push(ticket);
        state.focused_index = Some(0);

        assert!(state.focused_ticket().is_some());
        assert_eq!(state.focused_ticket().unwrap().key, "PROJ-123");
    }

    #[test]
    fn test_toggle_selection() {
        let mut state = AppState::new();
        let ticket = Ticket::new(
            "PROJ-123".to_string(),
            "Test".to_string(),
            Status {
                id: "1".to_string(),
                name: "To Do".to_string(),
                category: StatusCategory::ToDo,
            },
        );
        state.tickets.push(ticket);
        state.focused_index = Some(0);

        assert!(!state.selected_indices.contains(&0));
        state.toggle_selection();
        assert!(state.selected_indices.contains(&0));
        state.toggle_selection();
        assert!(!state.selected_indices.contains(&0));
    }
}
