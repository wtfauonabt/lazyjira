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
use std::time::Duration;

/// Main UI application
pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_handler: EventHandler,
    renderer: Renderer,
    running: bool,
    connection_status: String,
}

impl App {
    /// Create a new application instance
    pub fn new(connection_status: String) -> Result<Self, Box<dyn std::error::Error>> {
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
        })
    }

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
                        // TODO: Refresh data
                    }
                    _ => {
                        // TODO: Handle other events
                    }
                }
            }

            // Handle ticks
            if self.event_handler.should_tick() {
                // TODO: Update UI state
            }
        }

        Ok(())
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

            if let Err(e) = self.renderer.render_content_area(frame, chunks[1], "Welcome to LazyJira! Press 'q' to quit.") {
                eprintln!("Error rendering content: {}", e);
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
