use crate::infrastructure::api::ApiClient;
use crate::ui::components::ticket_list::{TicketList, TicketListState};
use crate::ui::events::{AppEvent, EventHandler};
use crate::ui::renderer::Renderer;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::{self, stdout, Stdout};
use std::sync::Arc;
use std::time::Duration;

/// Loading state for tickets
#[derive(Debug, Clone, PartialEq, Eq)]
enum LoadingState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

/// Main UI application
pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_handler: EventHandler,
    renderer: Renderer,
    running: bool,
    connection_status: String,
    ticket_list_state: TicketListState,
    ticket_service: Arc<dyn ApiClient>,
    loading_state: LoadingState,
}

impl App {
    /// Create a new application instance
    pub fn new(
        connection_status: String,
        ticket_service: Arc<dyn ApiClient>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let renderer = Renderer::new();
        
        Ok(Self {
            terminal,
            event_handler: EventHandler::default(),
            renderer,
            running: true,
            connection_status,
            ticket_list_state: TicketListState::new(),
            ticket_service,
            loading_state: LoadingState::Idle,
        })
    }

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load initial tickets
        self.load_tickets().await;

        while self.running {
            // Draw UI
            self.draw()?;

            // Handle events with timeout
            if crossterm::event::poll(Duration::from_millis(100))? {
                match self.event_handler.next()? {
                    AppEvent::Quit => {
                        self.running = false;
                    }
                    AppEvent::Refresh => {
                        self.load_tickets().await;
                    }
                    AppEvent::MoveUp => {
                        self.ticket_list_state.move_up();
                    }
                    AppEvent::MoveDown => {
                        self.ticket_list_state.move_down();
                    }
                    AppEvent::ToggleSelection => {
                        self.ticket_list_state.toggle_selection();
                    }
                    _ => {
                        // Other events handled elsewhere
                    }
                }
            }

            // Handle ticks
            if self.event_handler.should_tick() {
                // Periodic updates can go here
            }
        }

        Ok(())
    }

    /// Load tickets from API
    async fn load_tickets(&mut self) {
        self.loading_state = LoadingState::Loading;
        
        // Default JQL: get assigned tickets, ordered by updated date
        let jql = "assignee = currentUser() ORDER BY updated DESC";
        match self
            .ticket_service
            .search_issues(jql, 0, 50)
            .await
        {
            Ok(result) => {
                self.ticket_list_state.set_tickets(result.issues);
                self.loading_state = LoadingState::Loaded;
            }
            Err(e) => {
                self.loading_state = LoadingState::Error(format!("Failed to load tickets: {}", e));
            }
        }
    }

    /// Draw the UI
    fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|frame| {
            let area = frame.size();
            
            // Render main layout
            if let Err(e) = self.renderer.render_main_layout(frame, area, &self.connection_status) {
                eprintln!("Error rendering: {}", e);
            }

            // Content area is already split in render_main_layout
            // We just need to get the middle chunk for content
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(1),
                    ratatui::layout::Constraint::Min(1),
                    ratatui::layout::Constraint::Length(1),
                ])
                .split(area);

            // Render ticket list or loading/error state
            match &self.loading_state {
                LoadingState::Loading => {
                    if let Err(e) = self.renderer.render_content_area(
                        frame,
                        chunks[1],
                        "Loading tickets...",
                    ) {
                        eprintln!("Error rendering content: {}", e);
                    }
                }
                LoadingState::Error(msg) => {
                    if let Err(e) = self.renderer.render_content_area(frame, chunks[1], msg) {
                        eprintln!("Error rendering content: {}", e);
                    }
                }
                _ => {
                    // Render ticket list
                    let ticket_list = TicketList::new(&self.ticket_list_state, self.renderer.theme());
                    ticket_list.render(frame, chunks[1]);
                }
            }
        })?;

        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // Can't easily test without a real terminal, but we can test that
        // the structure is correct
        // This would require mocking the terminal
    }
}
