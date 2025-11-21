//! Query processing and intelligence.
//!
//! This module handles user queries by combining semantic search,
//! directory context awareness, and LLM-powered response generation.

pub mod search;

pub use search::{search_tools, SearchMatch};
