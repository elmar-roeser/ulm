//! ulm - AI-powered manpage assistant using local LLM inference.
//!
//! This tool transforms CLI interaction from "memorize commands" to "describe intent."
//! It provides an AI-powered bridge between what users want to accomplish and the
//! thousands of powerful but cryptic Unix tools available on their system.

use ulm::cli::{Args, Commands};
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
    // Parse command-line arguments
    let args = Args::parse_args();

    // Placeholder: Tracing setup will be added in Story 1.4

    // Dispatch based on command or query
    match args.command {
        Some(Commands::Setup) => {
            println!("Running setup...");
            // Placeholder: Will be implemented in Epic 2
        }
        Some(Commands::Update) => {
            println!("Running update...");
            // Placeholder: Will be implemented in Epic 2
        }
        None => {
            if args.has_query() {
                let query = args.query_string();
                println!("Query: {query}");
                // Placeholder: Will be implemented in Epic 3
            } else {
                println!("ulm - AI-powered manpage assistant");
                println!("Run 'ulm --help' for usage information.");
            }
        }
    }

    Ok(())
}
