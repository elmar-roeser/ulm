//! LLM interaction via Ollama API.
//!
//! This module handles all LLM-related operations including:
//! - Ollama API client
//! - Prompt building
//! - Response parsing

pub mod ollama;

// Submodules (to be implemented in Epic 3)
// pub mod prompt;
// pub mod response;

pub use ollama::{OllamaClient, DEFAULT_MODEL, DEFAULT_OLLAMA_URL};
