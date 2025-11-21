//! ulm - AI-powered manpage assistant using local LLM inference.
//!
//! This tool transforms CLI interaction from "memorize commands" to "describe intent."
//! It provides an AI-powered bridge between what users want to accomplish and the
//! thousands of powerful but cryptic Unix tools available on their system.

use std::process::ExitCode;

use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use ulm::cli::{Args, Commands};
use ulm::setup;
use ulm::Result;

/// Application entry point.
///
/// Sets up logging, parses CLI arguments, and dispatches to the appropriate
/// command handler. Errors are printed to stderr with exit code 1.
fn main() -> ExitCode {
    // Initialize tracing subscriber with env filter
    // Enable with: RUST_LOG=ulm=debug
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("ulm=info")),
        )
        .with_target(false)
        .init();

    debug!("ulm starting");

    // Create tokio runtime for async operations
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error: Failed to create tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Run the application and handle errors
    match runtime.block_on(run()) {
        Ok(()) => {
            debug!("ulm completed successfully");
            ExitCode::SUCCESS
        }
        Err(err) => {
            error!("{err:?}");
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Main application logic.
///
/// # Errors
///
/// Returns an error if any part of the application fails.
async fn run() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse_args();
    debug!(?args, "parsed arguments");

    // Dispatch based on command or query
    match args.command {
        Some(Commands::Setup) => {
            info!("running setup");
            setup::run_setup().await?;
        }
        Some(Commands::Update) => {
            info!("running update");
            setup::run_update().await?;
        }
        None => {
            if args.has_query() {
                let query = args.query_string();
                info!(%query, "processing query");
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
