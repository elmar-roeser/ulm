//! Setup and initialization orchestration.
//!
//! This module handles the `ulm setup` command, including:
//! - Ollama detection and health checking
//! - Model verification and pulling
//! - Manpage scanning and indexing

pub mod index;
pub mod ollama;

pub use index::{ManpageContent, ManpageScanner};
pub use ollama::OllamaChecker;
