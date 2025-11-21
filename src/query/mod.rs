//! Query processing and intelligence.
//!
//! This module handles user queries by combining semantic search,
//! directory context awareness, and LLM-powered response generation.

pub mod context;
pub mod search;

pub use context::{scan_directory_context, DirectoryContext, ProjectType};
pub use search::{load_manpage_content, search_tools, SearchMatch};
