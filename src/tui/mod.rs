//! Terminal User Interface for command suggestions.
//!
//! This module provides an interactive TUI for displaying, navigating,
//! and acting on command suggestions.

pub mod render;

use crate::llm::{CommandSuggestion, RiskLevel};

/// Application state for the TUI.
#[derive(Debug)]
pub struct App {
    /// Command suggestions to display.
    pub suggestions: Vec<CommandSuggestion>,
    /// Currently selected suggestion index.
    pub selected: usize,
    /// Status message to display (e.g., "Copied!").
    pub status_message: Option<String>,
}

impl App {
    /// Creates a new App with the given suggestions.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(suggestions: Vec<CommandSuggestion>) -> Self {
        Self {
            suggestions,
            selected: 0,
            status_message: None,
        }
    }

    /// Moves selection to the previous item (with wrap-around).
    #[allow(clippy::missing_const_for_fn)]
    pub fn select_previous(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.suggestions.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Moves selection to the next item (with wrap-around).
    #[allow(clippy::missing_const_for_fn)]
    pub fn select_next(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.suggestions.len();
    }

    /// Returns the currently selected suggestion.
    #[must_use]
    pub fn selected_suggestion(&self) -> Option<&CommandSuggestion> {
        self.suggestions.get(self.selected)
    }

    /// Sets a status message.
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clears the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

/// Returns the color for a risk level.
#[must_use]
pub const fn risk_color(risk: &RiskLevel) -> ratatui::style::Color {
    match risk {
        RiskLevel::Safe => ratatui::style::Color::Green,
        RiskLevel::Moderate => ratatui::style::Color::Yellow,
        RiskLevel::Destructive => ratatui::style::Color::Red,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_suggestions() -> Vec<CommandSuggestion> {
        vec![
            CommandSuggestion {
                command: "ls -la".to_string(),
                title: "List files".to_string(),
                explanation: "Lists all files".to_string(),
                risk_level: RiskLevel::Safe,
            },
            CommandSuggestion {
                command: "rm -rf /tmp/*".to_string(),
                title: "Clean temp".to_string(),
                explanation: "Removes temp files".to_string(),
                risk_level: RiskLevel::Destructive,
            },
        ]
    }

    #[test]
    fn test_app_creation() {
        let app = App::new(test_suggestions());
        assert_eq!(app.selected, 0);
        assert_eq!(app.suggestions.len(), 2);
        assert!(app.status_message.is_none());
    }

    #[test]
    fn test_select_next() {
        let mut app = App::new(test_suggestions());
        assert_eq!(app.selected, 0);

        app.select_next();
        assert_eq!(app.selected, 1);

        // Wrap around
        app.select_next();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_select_previous() {
        let mut app = App::new(test_suggestions());
        assert_eq!(app.selected, 0);

        // Wrap around
        app.select_previous();
        assert_eq!(app.selected, 1);

        app.select_previous();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_selected_suggestion() {
        let app = App::new(test_suggestions());
        let selected = app.selected_suggestion();
        assert!(selected.is_some());
        assert_eq!(selected.map(|s| s.command.as_str()), Some("ls -la"));
    }

    #[test]
    fn test_status_message() {
        let mut app = App::new(test_suggestions());
        assert!(app.status_message.is_none());

        app.set_status("Copied!".to_string());
        assert_eq!(app.status_message, Some("Copied!".to_string()));

        app.clear_status();
        assert!(app.status_message.is_none());
    }

    #[test]
    fn test_empty_suggestions() {
        let mut app = App::new(vec![]);
        app.select_next();
        assert_eq!(app.selected, 0);

        app.select_previous();
        assert_eq!(app.selected, 0);

        assert!(app.selected_suggestion().is_none());
    }

    #[test]
    fn test_risk_color() {
        assert_eq!(risk_color(&RiskLevel::Safe), ratatui::style::Color::Green);
        assert_eq!(
            risk_color(&RiskLevel::Moderate),
            ratatui::style::Color::Yellow
        );
        assert_eq!(
            risk_color(&RiskLevel::Destructive),
            ratatui::style::Color::Red
        );
    }
}
