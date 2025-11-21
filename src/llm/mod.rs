//! LLM interaction via Ollama API.
//!
//! This module handles all LLM-related operations including:
//! - Ollama API client
//! - Prompt building
//! - Response parsing

pub mod ollama;
pub mod prompt;
pub mod response;

pub use ollama::{OllamaClient, DEFAULT_MODEL, DEFAULT_OLLAMA_URL};
pub use prompt::build_prompt;
pub use response::{parse_suggestions, CommandSuggestion, RiskLevel};
