//! ulm - AI-powered manpage assistant using local LLM inference.
//!
//! This crate provides the core functionality for ulm, including:
//! - CLI argument parsing
//! - Setup and initialization
//! - Query processing and context building
//! - LLM interaction via Ollama
//! - Terminal UI for suggestion display
//! - Command execution

// Re-export anyhow::Result for convenience
pub use anyhow::Result;

// Core modules
pub mod cli;
pub mod db;
pub mod error;

// Feature modules
pub mod exec;
pub mod llm;
pub mod query;
pub mod setup;
pub mod tui;
