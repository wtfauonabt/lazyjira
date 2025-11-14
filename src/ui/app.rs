use crate::domain::models::ticket::Ticket;
use crate::infrastructure::api::ApiClient;
use crate::infrastructure::api::client::{CreateIssueData, Transition};
use crate::ui::components::ticket_detail::TicketDetail;
use crate::ui::components::ticket_list::{TicketList, TicketListState};
use crate::ui::components::transition_list::{TransitionList, TransitionListState};
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
    Transitions,
    CreateTicket,
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
    transition_list_state: TransitionListState,
    transitions_loading: bool,
    current_ticket_key: Option<String>,
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
            transition_list_state: TransitionListState::new(),
            transitions_loading: false,
            current_ticket_key: None,
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
                        match self.view_mode {
                            ViewMode::List => {
                                self.ticket_list_state.move_up();
                            }
                            ViewMode::Transitions => {
                                self.transition_list_state.move_up();
                            }
                            _ => {}
                        }
                    }
                    AppEvent::MoveDown => {
                        match self.view_mode {
                            ViewMode::List => {
                                self.ticket_list_state.move_down();
                            }
                            ViewMode::Transitions => {
                                self.transition_list_state.move_down();
                            }
                            _ => {}
                        }
                    }
                    AppEvent::EnterDetail => {
                        match self.view_mode {
                            ViewMode::List => {
                                self.open_detail_view().await;
                            }
                            ViewMode::Transitions => {
                                // Execute selected transition
                                if let Some(transition) = self.transition_list_state.focused_transition() {
                                    if let Some(ticket_key) = &self.current_ticket_key {
                                        if let Err(_e) = self.ticket_service.transition_issue(
                                            ticket_key,
                                            &transition.id,
                                            None,
                                        ).await {
                                            // Error handling
                                        } else {
                                            // Refresh ticket and return to detail view
                                            self.view_mode = ViewMode::Detail;
                                            if let Ok(updated_ticket) = self.ticket_service.get_issue(ticket_key).await {
                                                self.detail_ticket = Some(updated_ticket);
                                            }
                                            self.load_tickets().await;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    AppEvent::ToggleSelection => {
                        self.ticket_list_state.toggle_selection();
                    }
                    AppEvent::ExitDetail => {
                        match self.view_mode {
                            ViewMode::Detail | ViewMode::Transitions | ViewMode::CreateTicket => {
                                self.view_mode = ViewMode::List;
                                self.detail_ticket = None;
                                self.transition_list_state = TransitionListState::new();
                                self.current_ticket_key = None;
                            }
                            _ => {}
                        }
                    }
                    AppEvent::AssignToMe => {
                        if self.view_mode == ViewMode::Detail {
                            self.assign_to_me().await;
                        }
                    }
                    AppEvent::StartProgress => {
                        if self.view_mode == ViewMode::Detail {
                            self.start_progress().await;
                        }
                    }
                    AppEvent::Resolve => {
                        if self.view_mode == ViewMode::Detail {
                            self.resolve_ticket().await;
                        }
                    }
                    AppEvent::ShowTransitions => {
                        if self.view_mode == ViewMode::Detail {
                            self.show_transitions().await;
                        }
                    }
                    AppEvent::CreateTicket => {
                        if self.view_mode == ViewMode::List {
                            // TODO: Open create ticket form
                            // For now, just show a message
                        }
                    }
                    AppEvent::AddComment => {
                        if self.view_mode == ViewMode::Detail {
                            // TODO: Open comment input
                            // For now, just show a message
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
            self.current_ticket_key = Some(ticket_key.clone());

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

    /// Assign ticket to current user
    async fn assign_to_me(&mut self) {
        if let Some(ticket_key) = &self.current_ticket_key {
            // Get current user from config or API
            // For now, use a placeholder - in real implementation, get from config
            let _assignee = "currentUser()"; // This would need to be the actual account ID
            
            // Update ticket assignee
            // Note: This requires update_issue to be implemented
            // For now, just refresh the ticket
            if let Ok(updated_ticket) = self.ticket_service.get_issue(ticket_key).await {
                self.detail_ticket = Some(updated_ticket);
            }
        }
    }

    /// Start progress (transition to In Progress)
    async fn start_progress(&mut self) {
        if let Some(ticket_key) = &self.current_ticket_key {
            // Get transitions and find "Start Progress" or "In Progress"
            if let Ok(transitions) = self.ticket_service.get_transitions(ticket_key).await {
                if let Some(transition) = transitions.iter().find(|t| {
                    t.name.to_lowercase().contains("start") || 
                    t.to_status.to_lowercase().contains("progress")
                }) {
                    if let Err(_e) = self.ticket_service.transition_issue(
                        ticket_key,
                        &transition.id,
                        None,
                    ).await {
                        // Error handling - could show message
                    } else {
                        // Refresh ticket after transition
                        if let Ok(updated_ticket) = self.ticket_service.get_issue(ticket_key).await {
                            self.detail_ticket = Some(updated_ticket);
                            // Refresh list as well
                            self.load_tickets().await;
                        }
                    }
                }
            }
        }
    }

    /// Resolve ticket
    async fn resolve_ticket(&mut self) {
        if let Some(ticket_key) = &self.current_ticket_key {
            // Get transitions and find "Resolve" or "Done"
            if let Ok(transitions) = self.ticket_service.get_transitions(ticket_key).await {
                if let Some(transition) = transitions.iter().find(|t| {
                    t.name.to_lowercase().contains("resolve") || 
                    t.name.to_lowercase().contains("done") ||
                    t.to_status.to_lowercase().contains("done")
                }) {
                    if let Err(_e) = self.ticket_service.transition_issue(
                        ticket_key,
                        &transition.id,
                        None,
                    ).await {
                        // Error handling - could show message
                    } else {
                        // Refresh ticket after transition
                        if let Ok(updated_ticket) = self.ticket_service.get_issue(ticket_key).await {
                            self.detail_ticket = Some(updated_ticket);
                            // Refresh list as well
                            self.load_tickets().await;
                        }
                    }
                }
            }
        }
    }

    /// Show available transitions
    async fn show_transitions(&mut self) {
        if let Some(ticket_key) = &self.current_ticket_key {
            self.view_mode = ViewMode::Transitions;
            self.transitions_loading = true;
            
            match self.ticket_service.get_transitions(ticket_key).await {
                Ok(transitions) => {
                    self.transition_list_state.set_transitions(transitions);
                    self.transitions_loading = false;
                }
                Err(_e) => {
                    self.transitions_loading = false;
                    // Could show error message
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
                ViewMode::Transitions => {
                    // Render transitions list
                    if self.transitions_loading {
                        if let Err(e) = self.renderer.render_content_area(
                            frame,
                            chunks[1],
                            "Loading transitions...",
                        ) {
                            eprintln!("Error rendering content: {}", e);
                        }
                    } else {
                        let transition_list = TransitionList::new(&self.transition_list_state, self.renderer.theme());
                        transition_list.render(frame, chunks[1]);
                    }
                }
                ViewMode::CreateTicket => {
                    // TODO: Render create ticket form
                    if let Err(e) = self.renderer.render_content_area(
                        frame,
                        chunks[1],
                        "Create ticket form (not yet implemented)",
                    ) {
                        eprintln!("Error rendering content: {}", e);
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
