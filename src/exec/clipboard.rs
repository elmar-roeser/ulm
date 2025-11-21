//! Clipboard operations.
//!
//! This module handles copying text to the system clipboard.

use anyhow::{Context, Result};
use arboard::Clipboard;
use tracing::debug;

/// Copies text to the system clipboard.
///
/// Uses the arboard crate which supports X11, Wayland, and macOS.
///
/// # Errors
///
/// Returns an error if:
/// - Clipboard access fails (e.g., no display)
/// - Setting clipboard content fails
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    debug!(len = text.len(), "Copying to clipboard");

    let mut clipboard =
        Clipboard::new().context("Failed to access clipboard. Ensure a display is available.")?;

    clipboard
        .set_text(text)
        .context("Failed to set clipboard content")?;

    debug!("Text copied to clipboard");

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Clipboard tests are difficult in CI environments
    // These tests are marked as ignored and can be run manually

    use super::*;

    #[test]
    #[ignore = "Requires display environment"]
    fn test_copy_to_clipboard() {
        let result = copy_to_clipboard("test text");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "Requires display environment"]
    fn test_copy_empty_string() {
        let result = copy_to_clipboard("");
        assert!(result.is_ok());
    }
}
