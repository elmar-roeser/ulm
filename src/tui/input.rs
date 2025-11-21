//! TUI input handling.
//!
//! This module processes keyboard events and translates them into
//! user actions.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::{App, UserAction};

/// Handles a keyboard event and returns an optional user action.
///
/// Returns `Some(UserAction)` if the event should exit the TUI,
/// or `None` to continue.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_event(app: &mut App, event: Event) -> Option<UserAction> {
    if let Event::Key(key) = event {
        return handle_key(app, key);
    }
    None
}

/// Handles a key event.
fn handle_key(app: &mut App, key: KeyEvent) -> Option<UserAction> {
    // Clear status message on any key press
    app.clear_status();

    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
            None
        }

        // Execute
        KeyCode::Enter | KeyCode::Char('a' | 'A') => app
            .selected_suggestion()
            .map(|s| UserAction::Execute(s.command.clone())),

        // Copy
        KeyCode::Char('K') => app
            .selected_suggestion()
            .map(|s| UserAction::Copy(s.command.clone())),

        // Edit
        KeyCode::Char('b' | 'B') => app
            .selected_suggestion()
            .map(|s| UserAction::Edit(s.command.clone())),

        // Abort
        KeyCode::Esc | KeyCode::Char('q') => Some(UserAction::Abort),

        // Ctrl-C
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(UserAction::Abort)
        }

        // Number keys for direct selection
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let num = c.to_digit(10).unwrap_or(0) as usize;
            if num > 0 && num <= app.suggestions.len() {
                app.selected = num - 1;
            }
            None
        }

        // Ignore other keys
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{CommandSuggestion, RiskLevel};

    fn test_app() -> App {
        App::new(vec![
            CommandSuggestion {
                command: "ls -la".to_string(),
                title: "List files".to_string(),
                explanation: "Lists all files".to_string(),
                risk_level: RiskLevel::Safe,
            },
            CommandSuggestion {
                command: "pwd".to_string(),
                title: "Print dir".to_string(),
                explanation: "Prints current directory".to_string(),
                risk_level: RiskLevel::Safe,
            },
        ])
    }

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn test_navigate_down() {
        let mut app = test_app();
        assert_eq!(app.selected, 0);

        let result = handle_event(&mut app, key_event(KeyCode::Down));
        assert!(result.is_none());
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_navigate_up() {
        let mut app = test_app();
        app.selected = 1;

        let result = handle_event(&mut app, key_event(KeyCode::Up));
        assert!(result.is_none());
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_navigate_j_k() {
        let mut app = test_app();

        handle_event(&mut app, key_event(KeyCode::Char('j')));
        assert_eq!(app.selected, 1);

        handle_event(&mut app, key_event(KeyCode::Char('k')));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_execute_enter() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Enter));

        match result {
            Some(UserAction::Execute(cmd)) => assert_eq!(cmd, "ls -la"),
            _ => panic!("Expected Execute action"),
        }
    }

    #[test]
    fn test_execute_a() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Char('a')));

        assert!(matches!(result, Some(UserAction::Execute(_))));
    }

    #[test]
    fn test_copy_k() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Char('K')));

        match result {
            Some(UserAction::Copy(cmd)) => assert_eq!(cmd, "ls -la"),
            _ => panic!("Expected Copy action"),
        }
    }

    #[test]
    fn test_edit_b() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Char('b')));

        match result {
            Some(UserAction::Edit(cmd)) => assert_eq!(cmd, "ls -la"),
            _ => panic!("Expected Edit action"),
        }
    }

    #[test]
    fn test_abort_esc() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Esc));

        assert!(matches!(result, Some(UserAction::Abort)));
    }

    #[test]
    fn test_abort_q() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Char('q')));

        assert!(matches!(result, Some(UserAction::Abort)));
    }

    #[test]
    fn test_number_key_selection() {
        let mut app = test_app();
        assert_eq!(app.selected, 0);

        handle_event(&mut app, key_event(KeyCode::Char('2')));
        assert_eq!(app.selected, 1);

        // Out of range number should be ignored
        handle_event(&mut app, key_event(KeyCode::Char('5')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_ignore_unknown_key() {
        let mut app = test_app();
        let result = handle_event(&mut app, key_event(KeyCode::Char('x')));

        assert!(result.is_none());
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_clears_status_on_key() {
        let mut app = test_app();
        app.set_status("Test message".to_string());

        handle_event(&mut app, key_event(KeyCode::Down));
        assert!(app.status_message.is_none());
    }
}
