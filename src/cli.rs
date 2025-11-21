//! CLI argument parsing using clap derive macros.
//!
//! Defines the command-line interface for ulm including subcommands
//! and argument handling.

use clap::{Parser, Subcommand};

/// Command-line arguments for ulm.
///
/// ulm supports two modes:
/// - Subcommand mode: `ulm setup` or `ulm update`
/// - Query mode: `ulm "find large files"`
#[derive(Parser, Debug)]
#[command(name = "ulm")]
#[command(
    author,
    version,
    about = "AI-powered manpage assistant using local LLM"
)]
#[command(
    long_about = "ulm transforms CLI interaction from 'memorize commands' to 'describe intent'. \
    It provides an AI-powered bridge between what you want to accomplish and the \
    thousands of powerful but cryptic Unix tools available on your system."
)]
pub struct Args {
    /// Subcommand to execute (setup, update).
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Natural language query describing what you want to do.
    ///
    /// Example: `ulm "find large files in current directory"`
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,
}

/// Available subcommands for ulm.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize ulm with Ollama and build the manpage index.
    ///
    /// This command will:
    /// - Check for Ollama installation
    /// - Verify or pull a suitable LLM model
    /// - Scan system manpage directories
    /// - Generate embeddings and build the search index
    Setup,

    /// Refresh the manpage index.
    ///
    /// Re-scans manpage directories and updates the search index
    /// with any new or modified manpages.
    Update,
}

impl Args {
    /// Parse command-line arguments.
    ///
    /// This is a convenience wrapper around clap's parse function.
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Check if a query was provided.
    #[must_use]
    pub const fn has_query(&self) -> bool {
        !self.query.is_empty()
    }

    /// Get the query as a single string.
    ///
    /// Joins all query arguments with spaces.
    #[must_use]
    pub fn query_string(&self) -> String {
        self.query.join(" ")
    }
}
