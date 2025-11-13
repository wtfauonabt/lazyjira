use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the application
#[derive(Debug, Clone)]
pub struct Theme {
    pub status_bar: Style,
    pub help_bar: Style,
    pub selected: Style,
    pub focused: Style,
    pub normal: Style,
    pub status_todo: Style,
    pub status_in_progress: Style,
    pub status_done: Style,
    pub priority_lowest: Style,
    pub priority_low: Style,
    pub priority_medium: Style,
    pub priority_high: Style,
    pub priority_highest: Style,
    pub error: Style,
    pub success: Style,
    pub warning: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            status_bar: Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan),
            help_bar: Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow),
            selected: Style::default()
                .add_modifier(Modifier::REVERSED),
            focused: Style::default()
                .add_modifier(Modifier::BOLD),
            normal: Style::default(),
            status_todo: Style::default()
                .fg(Color::Blue),
            status_in_progress: Style::default()
                .fg(Color::Yellow),
            status_done: Style::default()
                .fg(Color::Green),
            priority_lowest: Style::default()
                .fg(Color::DarkGray),
            priority_low: Style::default()
                .fg(Color::Blue),
            priority_medium: Style::default()
                .fg(Color::Yellow),
            priority_high: Style::default()
                .fg(Color::Magenta),
            priority_highest: Style::default()
                .fg(Color::Red),
            error: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Theme {
    /// Get style for a status category
    pub fn status_style(&self, category: &str) -> Style {
        match category {
            "new" | "To Do" => self.status_todo,
            "indeterminate" | "In Progress" => self.status_in_progress,
            "done" | "Done" => self.status_done,
            _ => self.normal,
        }
    }

    /// Get style for a priority level
    pub fn priority_style(&self, priority: &str) -> Style {
        match priority {
            "Lowest" => self.priority_lowest,
            "Low" => self.priority_low,
            "Medium" => self.priority_medium,
            "High" => self.priority_high,
            "Highest" => self.priority_highest,
            _ => self.normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.status_bar.fg, Some(Color::Black));
        assert_eq!(theme.status_bar.bg, Some(Color::Cyan));
    }

    #[test]
    fn test_status_style() {
        let theme = Theme::default();
        
        let todo_style = theme.status_style("new");
        assert_eq!(todo_style.fg, Some(Color::Blue));
        
        let in_progress_style = theme.status_style("indeterminate");
        assert_eq!(in_progress_style.fg, Some(Color::Yellow));
        
        let done_style = theme.status_style("done");
        assert_eq!(done_style.fg, Some(Color::Green));
    }

    #[test]
    fn test_priority_style() {
        let theme = Theme::default();
        
        assert_eq!(theme.priority_style("Lowest").fg, Some(Color::DarkGray));
        assert_eq!(theme.priority_style("Low").fg, Some(Color::Blue));
        assert_eq!(theme.priority_style("Medium").fg, Some(Color::Yellow));
        assert_eq!(theme.priority_style("High").fg, Some(Color::Magenta));
        assert_eq!(theme.priority_style("Highest").fg, Some(Color::Red));
    }
}
