//! Setup and initialization orchestration.
//!
//! This module handles the `ulm setup` command, including:
//! - Ollama detection and health checking
//! - Model verification and pulling
//! - Manpage scanning and indexing

pub mod ollama;

// Submodules (to be implemented in Epic 2)
// pub mod index;

pub use ollama::OllamaChecker;
