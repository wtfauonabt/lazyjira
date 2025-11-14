use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Terminal renderer
pub struct Renderer {
    theme: Theme,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
        }
    }

    /// Get the theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Render the main layout
    pub fn render_main_layout(&mut self, frame: &mut Frame, area: Rect, connection_status: &str) -> Result<(), std::io::Error> {
        // Create main layout: [status bar] [content] [help bar]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status bar
                Constraint::Min(1),    // Content area
                Constraint::Length(1), // Help bar
            ])
            .split(area);

        // Render status bar
        self.render_status_bar(frame, chunks[0], connection_status)?;
        
        // Render help bar
        self.render_help_bar(frame, chunks[2])?;

        Ok(())
    }

    /// Render the status bar
    fn render_status_bar(&mut self, frame: &mut Frame, area: Rect, status: &str) -> Result<(), std::io::Error> {
        let status_text = format!(" LazyJira | Status: {} ", status);
        let paragraph = Paragraph::new(status_text)
            .style(self.theme.status_bar)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        
        frame.render_widget(paragraph, area);
        Ok(())
    }

    /// Render the help bar
    fn render_help_bar(&mut self, frame: &mut Frame, area: Rect) -> Result<(), std::io::Error> {
        let help_text = " [q]uit [↑↓/jk]move [Enter]detail [Esc]back [a]ssign [s]tart [R]esolve [t]ransitions [r]efresh ";
        let paragraph = Paragraph::new(help_text)
            .style(self.theme.help_bar)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        
        frame.render_widget(paragraph, area);
        Ok(())
    }

    /// Render content area (to be implemented by views)
    pub fn render_content_area(&mut self, frame: &mut Frame, area: Rect, content: &str) -> Result<(), std::io::Error> {
        let paragraph = Paragraph::new(content)
            .style(self.theme.normal)
            .block(Block::default().borders(Borders::ALL).title("Content"));
        
        frame.render_widget(paragraph, area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        // Can't easily test without a real terminal, but we can test theme access
        let renderer = Renderer::new();
        
        let theme = renderer.theme();
        assert_eq!(theme.status_bar.fg, Some(ratatui::style::Color::Black));
    }
}
