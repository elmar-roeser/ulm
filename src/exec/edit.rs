//! Command editing with line input.
//!
//! This module provides command editing functionality using rustyline.

use anyhow::{Context, Result};
use rustyline::DefaultEditor;
use tracing::debug;

/// Allows the user to edit a command before execution.
///
/// Uses rustyline for line editing with history and readline-like shortcuts.
///
/// # Returns
///
/// - `Ok(Some(command))` - User edited and confirmed the command
/// - `Ok(None)` - User cancelled (Ctrl-C or Ctrl-D)
///
/// # Errors
///
/// Returns an error if:
/// - Rustyline initialization fails
/// - Reading input fails
pub fn edit_command(initial: &str) -> Result<Option<String>> {
    debug!(initial = %initial, "Editing command");

    let mut editor = DefaultEditor::new().context("Failed to create line editor")?;

    // Read line with initial value
    match editor.readline_with_initial("Edit: ", (initial, "")) {
        Ok(line) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                debug!("Empty command entered, cancelling");
                Ok(None)
            } else {
                debug!(edited = %trimmed, "Command edited");
                Ok(Some(trimmed.to_string()))
            }
        }
        Err(
            rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof,
        ) => {
            debug!("Edit cancelled by user");
            Ok(None)
        }
        Err(e) => Err(e).context("Failed to read edited command"),
    }
}

#[cfg(test)]
mod tests {
    // Note: Interactive tests are difficult to automate
    // The edit functionality is tested manually

    #[test]
    fn test_module_compiles() {
        // Ensure the module compiles correctly
        assert!(true);
    }
}
