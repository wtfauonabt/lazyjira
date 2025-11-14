use crate::infrastructure::api::client::Transition;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// State for transition list widget
#[derive(Debug, Clone)]
pub struct TransitionListState {
    pub transitions: Vec<Transition>,
    pub focused_index: Option<usize>,
}

impl Default for TransitionListState {
    fn default() -> Self {
        Self {
            transitions: vec![],
            focused_index: None,
        }
    }
}

impl TransitionListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_transitions(&mut self, transitions: Vec<Transition>) {
        self.transitions = transitions;
        self.focused_index = if self.transitions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn move_up(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx > 0 {
                self.focused_index = Some(idx - 1);
            }
        } else if !self.transitions.is_empty() {
            self.focused_index = Some(0);
        }
    }

    pub fn move_down(&mut self) {
        if let Some(idx) = self.focused_index {
            if idx < self.transitions.len().saturating_sub(1) {
                self.focused_index = Some(idx + 1);
            }
        } else if !self.transitions.is_empty() {
            self.focused_index = Some(0);
        }
    }

    pub fn focused_transition(&self) -> Option<&Transition> {
        self.focused_index
            .and_then(|idx| self.transitions.get(idx))
    }
}

/// Transition list widget
pub struct TransitionList<'a> {
    state: &'a TransitionListState,
    theme: &'a Theme,
}

impl<'a> TransitionList<'a> {
    pub fn new(state: &'a TransitionListState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        if self.state.transitions.is_empty() {
            let paragraph = Paragraph::new("No transitions available")
                .style(self.theme.normal)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Transitions"),
                );
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .state
            .transitions
            .iter()
            .enumerate()
            .map(|(_idx, transition)| {
                let text = format!("{} → {}", transition.name, transition.to_status);
                ListItem::new(Line::from(text))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Available Transitions")
                    .title_style(self.theme.focused),
            )
            .highlight_style(self.theme.selected)
            .highlight_symbol("> ");

        let mut list_state = ListState::default();
        if let Some(focused_idx) = self.state.focused_index {
            list_state.select(Some(focused_idx));
        }

        frame.render_stateful_widget(list, area, &mut list_state);
    }
}
