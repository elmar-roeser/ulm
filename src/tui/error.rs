//! Error display formatting for user-friendly error messages.
//!
//! This module provides functions to format errors with actionable guidance.

use std::io::{self, Write};

/// Displays an error message to stderr with actionable guidance.
///
/// Formats the error in a clear, user-friendly way with suggestions
/// for how to resolve the issue.
pub fn display_error(error: &anyhow::Error) {
    let mut stderr = io::stderr();

    // Write error header
    let _ = writeln!(stderr, "Error: {error}");

    // Show error chain for context
    for (i, cause) in error.chain().skip(1).enumerate() {
        let _ = writeln!(stderr, "  {}: {cause}", i + 1);
    }

    // Add actionable guidance based on error message
    let error_str = error.to_string().to_lowercase();
    let guidance = get_guidance(&error_str);

    if !guidance.is_empty() {
        let _ = writeln!(stderr);
        let _ = writeln!(stderr, "Suggestions:");
        for suggestion in guidance {
            let _ = writeln!(stderr, "  - {suggestion}");
        }
    }
}

/// Returns actionable guidance based on the error message.
fn get_guidance(error_str: &str) -> Vec<&'static str> {
    let mut suggestions = Vec::new();

    // Ollama connection issues
    if error_str.contains("connection refused") || error_str.contains("connect error") {
        suggestions.push("Ensure Ollama is running: ollama serve");
        suggestions.push("Check if Ollama is listening on port 11434");
    }

    // Model not found
    if error_str.contains("model") && error_str.contains("not found") {
        suggestions.push("Pull the required model: ollama pull <model>");
        suggestions.push("List available models: ollama list");
    }

    // Database issues
    if error_str.contains("lancedb") || error_str.contains("database") {
        suggestions.push("Run setup first: ulm --setup");
        suggestions.push("Check XDG_DATA_HOME permissions");
    }

    // No results
    if error_str.contains("no results") || error_str.contains("not found") {
        suggestions.push("Try a different search query");
        suggestions.push("Run setup to index manpages: ulm --setup");
    }

    // Clipboard issues
    if error_str.contains("clipboard") {
        suggestions.push("Check if a display server is running (X11/Wayland)");
        suggestions.push("Try running from a graphical terminal");
    }

    // Terminal issues
    if error_str.contains("terminal") || error_str.contains("tty") {
        suggestions.push("Ensure running in a terminal emulator");
        suggestions.push("Check TERM environment variable is set");
    }

    // Generic fallback
    if suggestions.is_empty() {
        suggestions.push("Check the error message above for details");
        suggestions.push("Run with RUST_LOG=debug for more information");
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_get_guidance_connection_refused() {
        let guidance = get_guidance("connection refused to localhost");
        assert!(guidance.iter().any(|s| s.contains("ollama serve")));
    }

    #[test]
    fn test_get_guidance_model_not_found() {
        let guidance = get_guidance("model llama3 not found");
        assert!(guidance.iter().any(|s| s.contains("ollama pull")));
    }

    #[test]
    fn test_get_guidance_database() {
        let guidance = get_guidance("lancedb error");
        assert!(guidance.iter().any(|s| s.contains("--setup")));
    }

    #[test]
    fn test_get_guidance_clipboard() {
        let guidance = get_guidance("clipboard error");
        assert!(guidance.iter().any(|s| s.contains("display server")));
    }

    #[test]
    fn test_get_guidance_generic() {
        let guidance = get_guidance("some unknown error");
        assert!(guidance.iter().any(|s| s.contains("RUST_LOG")));
    }

    #[test]
    fn test_display_error_runs() {
        // Just verify it doesn't panic
        let error = anyhow!("Test error").context("Test context");
        display_error(&error);
    }
}
