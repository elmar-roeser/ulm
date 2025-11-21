//! ulm - AI-powered manpage assistant using local LLM inference.
//!
//! This tool transforms CLI interaction from "memorize commands" to "describe intent."
//! It provides an AI-powered bridge between what users want to accomplish and the
//! thousands of powerful but cryptic Unix tools available on their system.

use ulm::Result;

/// Application entry point.
///
/// Sets up the CLI, initializes logging, and dispatches to the appropriate
/// command handler.
///
/// # Errors
///
/// Returns an error if any part of the application fails.
#[allow(clippy::unnecessary_wraps)] // Will return errors in Story 1.4
fn main() -> Result<()> {
    // Placeholder: CLI parsing will be added in Story 1.3
    // Placeholder: Tracing setup will be added in Story 1.4

    println!("ulm - AI-powered manpage assistant");
    println!("Run 'ulm --help' for usage information.");

    Ok(())
}
