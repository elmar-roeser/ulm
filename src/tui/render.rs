//! TUI rendering with ratatui.
//!
//! This module handles the visual rendering of command suggestions
//! in the terminal.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use super::{risk_color, App};

/// Renders the TUI to the given frame.
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Suggestions list
            Constraint::Length(5), // Explanation
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

    render_suggestions(frame, app, chunks[0]);
    render_explanation(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

/// Renders the list of suggestions.
fn render_suggestions(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .suggestions
        .iter()
        .enumerate()
        .map(|(i, suggestion)| {
            let is_selected = i == app.selected;
            let color = risk_color(&suggestion.risk_level);

            // Build the item content
            let mut lines = vec![
                // Title line with index
                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", i + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        &suggestion.title,
                        Style::default()
                            .fg(if is_selected { Color::White } else { color })
                            .add_modifier(if is_selected {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                ]),
                // Command line
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        &suggestion.command,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]),
            ];

            // Add risk indicator for non-safe
            if suggestion.risk_level != crate::llm::RiskLevel::Safe {
                let risk_text = match suggestion.risk_level {
                    crate::llm::RiskLevel::Moderate => " [!] moderate",
                    crate::llm::RiskLevel::Destructive => " [!] DESTRUCTIVE",
                    crate::llm::RiskLevel::Safe => "",
                };
                lines[0]
                    .spans
                    .push(Span::styled(risk_text, Style::default().fg(color)));
            }

            ListItem::new(lines).style(if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            })
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" ulm - Command Suggestions "),
    );

    frame.render_widget(list, area);
}

/// Renders the explanation panel for the selected suggestion.
fn render_explanation(frame: &mut Frame, app: &App, area: Rect) {
    let explanation_text = app
        .selected_suggestion()
        .map_or("No suggestion selected", |s| &s.explanation);

    let paragraph = Paragraph::new(Text::from(explanation_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Explanation "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Renders the footer with key bindings and status.
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let footer_text = if let Some(status) = &app.status_message {
        Span::styled(status, Style::default().fg(Color::Green))
    } else {
        Span::styled(
            " Up/Down Navigate  Enter/A Execute  K Copy  Esc/q Quit ",
            Style::default().fg(Color::DarkGray),
        )
    };

    let paragraph = Paragraph::new(Line::from(footer_text));
    frame.render_widget(paragraph, area);
}
