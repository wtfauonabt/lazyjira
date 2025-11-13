use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

/// Application events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// Quit the application
    Quit,
    /// Move selection up
    MoveUp,
    /// Move selection down
    MoveDown,
    /// Move selection left
    MoveLeft,
    /// Move selection right
    MoveRight,
    /// Select current item
    Select,
    /// Toggle selection
    ToggleSelection,
    /// Refresh data
    Refresh,
    /// Enter detail view
    EnterDetail,
    /// Exit detail view
    ExitDetail,
    /// Unknown/unhandled key
    Unknown,
}

/// Event handler for terminal input
pub struct EventHandler {
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Check if it's time for a tick
    pub fn should_tick(&mut self) -> bool {
        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            true
        } else {
            false
        }
    }

    /// Read the next event from the terminal
    pub fn next(&self) -> Result<AppEvent, std::io::Error> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                Ok(Self::handle_key(key_event))
            }
            Event::Resize(_, _) => {
                // Resize events are handled separately
                Ok(AppEvent::Unknown)
            }
            _ => Ok(AppEvent::Unknown),
        }
    }

    /// Handle a key event and convert it to an AppEvent
    pub fn handle_key(key_event: KeyEvent) -> AppEvent {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc if key_event.modifiers.is_empty() => AppEvent::Quit,
            KeyCode::Char('Q') if key_event.modifiers.contains(KeyModifiers::SHIFT) => AppEvent::Quit,
            KeyCode::Up | KeyCode::Char('k') => AppEvent::MoveUp,
            KeyCode::Down | KeyCode::Char('j') => AppEvent::MoveDown,
            KeyCode::Left | KeyCode::Char('h') => AppEvent::MoveLeft,
            KeyCode::Right | KeyCode::Char('l') => AppEvent::MoveRight,
            KeyCode::Enter => AppEvent::EnterDetail,
            KeyCode::Char(' ') => AppEvent::ToggleSelection,
            KeyCode::Char('r') | KeyCode::Char('R') => AppEvent::Refresh,
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => AppEvent::Quit,
            _ => AppEvent::Unknown,
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn test_handle_key_quit() {
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Char('q'), KeyModifiers::empty())),
            AppEvent::Quit
        );
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Esc, KeyModifiers::empty())),
            AppEvent::Quit
        );
    }

    #[test]
    fn test_handle_key_movement() {
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Up, KeyModifiers::empty())),
            AppEvent::MoveUp
        );
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Char('k'), KeyModifiers::empty())),
            AppEvent::MoveUp
        );
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Down, KeyModifiers::empty())),
            AppEvent::MoveDown
        );
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Char('j'), KeyModifiers::empty())),
            AppEvent::MoveDown
        );
    }

    #[test]
    fn test_handle_key_ctrl_c() {
        assert_eq!(
            EventHandler::handle_key(create_key_event(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL
            )),
            AppEvent::Quit
        );
    }

    #[test]
    fn test_handle_key_enter() {
        assert_eq!(
            EventHandler::handle_key(create_key_event(KeyCode::Enter, KeyModifiers::empty())),
            AppEvent::EnterDetail
        );
    }

    #[test]
    fn test_should_tick() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        assert!(!handler.should_tick()); // Just created, shouldn't tick immediately
        
        std::thread::sleep(Duration::from_millis(150));
        assert!(handler.should_tick());
        assert!(!handler.should_tick()); // Should reset
    }
}
