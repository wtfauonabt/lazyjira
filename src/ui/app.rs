use crate::domain::models::ticket::Ticket;
use crate::infrastructure::api::ApiClient;
use crate::ui::components::ticket_detail::TicketDetail;
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

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    List,
    Detail,
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
    view_mode: ViewMode,
    detail_ticket: Option<Ticket>,
    detail_loading: bool,
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
            view_mode: ViewMode::List,
            detail_ticket: None,
            detail_loading: false,
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
                    AppEvent::EnterDetail => {
                        if self.view_mode == ViewMode::List {
                            self.open_detail_view().await;
                        }
                    }
                    AppEvent::ExitDetail => {
                        if self.view_mode == ViewMode::Detail {
                            self.view_mode = ViewMode::List;
                            self.detail_ticket = None;
                        }
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

    /// Open detail view for focused ticket
    async fn open_detail_view(&mut self) {
        if let Some(ticket) = self.ticket_list_state.focused_ticket() {
            let ticket_key = ticket.key.clone();
            self.view_mode = ViewMode::Detail;
            self.detail_loading = true;
            self.detail_ticket = None;

            // Fetch full ticket details
            match self.ticket_service.get_issue(&ticket_key).await {
                Ok(full_ticket) => {
                    self.detail_ticket = Some(full_ticket);
                    self.detail_loading = false;
                }
                Err(e) => {
                    // On error, use the ticket from list (may be incomplete)
                    self.detail_ticket = Some(ticket.clone());
                    self.detail_loading = false;
                    // Could set an error state here if needed
                }
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

            // Render based on view mode
            match self.view_mode {
                ViewMode::List => {
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
                }
                ViewMode::Detail => {
                    // Render detail view
                    if self.detail_loading {
                        if let Err(e) = self.renderer.render_content_area(
                            frame,
                            chunks[1],
                            "Loading ticket details...",
                        ) {
                            eprintln!("Error rendering content: {}", e);
                        }
                    } else if let Some(ticket) = &self.detail_ticket {
                        let detail = TicketDetail::new(ticket, self.renderer.theme());
                        detail.render(frame, chunks[1]);
                    } else {
                        if let Err(e) = self.renderer.render_content_area(
                            frame,
                            chunks[1],
                            "No ticket selected.",
                        ) {
                            eprintln!("Error rendering content: {}", e);
                        }
                    }
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
