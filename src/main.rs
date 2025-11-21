//! ulm - AI-powered manpage assistant using local LLM inference.
//!
//! This tool transforms CLI interaction from "memorize commands" to "describe intent."
//! It provides an AI-powered bridge between what users want to accomplish and the
//! thousands of powerful but cryptic Unix tools available on their system.

use std::io::{self, Write};
use std::panic;
use std::process::ExitCode;

use crossterm::terminal::{disable_raw_mode, is_raw_mode_enabled};
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use ulm::cli::{Args, Commands};
use ulm::exec::{copy_to_clipboard, edit_command, execute_command};
use ulm::query;
use ulm::setup;
use ulm::tui::{display_error, run_tui, UserAction};
use ulm::Result;

/// Application entry point.
///
/// Sets up logging, parses CLI arguments, and dispatches to the appropriate
/// command handler. Errors are printed to stderr with exit code 1.
fn main() -> ExitCode {
    // Set up panic hook to restore terminal on panic
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        // Try to restore terminal state
        if is_raw_mode_enabled().unwrap_or(false) {
            let _ = disable_raw_mode();
        }
        default_hook(info);
    }));

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
        Ok(code) => {
            debug!("ulm completed with code {}", code);
            if code == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(code)
            }
        }
        Err(err) => {
            error!("{err:?}");
            display_error(&err);
            ExitCode::FAILURE
        }
    }
}

/// Main application logic.
///
/// # Errors
///
/// Returns an error if any part of the application fails.
async fn run() -> Result<u8> {
    // Parse command-line arguments
    let args = Args::parse_args();
    debug!(?args, "parsed arguments");

    // Dispatch based on command or query
    match args.command {
        Some(Commands::Setup) => {
            info!("running setup");
            setup::run_setup().await?;
            Ok(0)
        }
        Some(Commands::Update) => {
            info!("running update");
            setup::run_update().await?;
            Ok(0)
        }
        None => {
            if args.has_query() {
                let query = args.query_string();
                info!(%query, "processing query");
                process_query_flow(&query).await
            } else {
                println!("ulm - AI-powered manpage assistant");
                println!("Run 'ulm --help' for usage information.");
                Ok(0)
            }
        }
    }
}

/// Processes a query and shows the TUI for user interaction.
async fn process_query_flow(query: &str) -> Result<u8> {
    // Get suggestions from the query pipeline
    let suggestions = query::process_query(query).await?;

    if suggestions.is_empty() {
        println!("No suggestions found for: {query}");
        return Ok(0);
    }

    // Auto-execute if single suggestion
    if suggestions.len() == 1 {
        let cmd = &suggestions[0].command;
        info!(command = %cmd, "auto-executing single suggestion");
        let exit_code = execute_command(cmd)?;
        return Ok(exit_code.try_into().unwrap_or(1));
    }

    // Run TUI for multiple suggestions
    let action = run_tui(suggestions)?;

    // Handle user action
    match action {
        UserAction::Execute(cmd) => {
            info!(command = %cmd, "executing command");
            let exit_code = execute_command(&cmd)?;
            Ok(exit_code.try_into().unwrap_or(1))
        }
        UserAction::Copy(cmd) => {
            copy_to_clipboard(&cmd)?;
            println!("Copied to clipboard: {cmd}");
            Ok(0)
        }
        UserAction::Edit(cmd) => {
            if let Some(edited) = edit_command(&cmd)? {
                info!(command = %edited, "executing edited command");
                let exit_code = execute_command(&edited)?;
                Ok(exit_code.try_into().unwrap_or(1))
            } else {
                debug!("edit cancelled");
                Ok(0)
            }
        }
        UserAction::Abort => {
            debug!("user aborted");
            Ok(0)
        }
    }
}

/// Prints a message to stdout and flushes.
#[allow(dead_code)]
fn print_flush(msg: &str) {
    print!("{msg}");
    let _ = io::stdout().flush();
}
