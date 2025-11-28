//! Vector search for finding relevant manpages.
//!
//! This module provides semantic search functionality using the manpage
//! embedding index to find tools relevant to user queries.

use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::db;
use crate::llm::OllamaClient;
use crate::setup::load_config;

/// Maximum characters to include in manpage content for LLM context.
const MAX_CONTENT_LENGTH: usize = 8000;

/// A search result matching a user query.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Tool name (e.g., "ffmpeg").
    pub tool_name: String,
    /// Man section (e.g., "1").
    pub section: String,
    /// Description text.
    pub description: String,
    /// Similarity score (distance - lower is better).
    pub score: f32,
}

/// Searches for tools matching the given query.
///
/// Generates an embedding for the query and performs vector similarity
/// search against the manpage index.
///
/// # Arguments
///
/// * `query` - Natural language query describing desired functionality
/// * `limit` - Maximum number of results to return
///
/// # Errors
///
/// Returns an error if:
/// - Ollama cannot generate embedding
/// - Database search fails
/// - Index doesn't exist
pub async fn search_tools(query: &str, limit: usize) -> Result<Vec<SearchMatch>> {
    info!(query = %query, limit = limit, "Searching for tools");

    // Check if index exists
    if !db::index_exists().await.context("Failed to check index")? {
        anyhow::bail!("Index not found. Please run 'ulm setup' first.");
    }

    // Load config to get embedding model
    let config = load_config().context("Failed to load config")?;
    let embedding_model = config.embedding_model();

    // Validate embedding model matches index
    if config.needs_index_rebuild() {
        let last_model = config
            .index
            .last_embedding_model
            .as_deref()
            .unwrap_or("unknown");
        anyhow::bail!(
            "Index was built with '{last_model}' but config uses '{embedding_model}'.\n\
             Run 'ulm setup' to rebuild index with the current embedding model."
        );
    }

    // If index exists but no model metadata, warn user to rebuild
    if config.index.last_embedding_model.is_none() {
        anyhow::bail!(
            "Index was built with an unknown embedding model.\n\
             Run 'ulm setup' to rebuild index with '{embedding_model}'."
        );
    }

    // Generate query embedding
    let client = OllamaClient::with_config(
        config.ollama_url(),
        config.generate_timeout_secs(),
        config.embedding_timeout_secs(),
    )
    .context("Failed to create Ollama client")?;
    let embedding = client
        .generate_embedding(embedding_model, query)
        .await
        .context("Failed to generate query embedding")?;

    debug!(dimensions = embedding.len(), "Generated query embedding");

    // Perform vector search
    let results = db::search(&embedding, limit)
        .await
        .context("Failed to search database")?;

    // Map to SearchMatch
    let matches: Vec<SearchMatch> = results
        .into_iter()
        .map(|r| SearchMatch {
            tool_name: r.tool_name,
            section: r.section,
            description: r.description,
            score: r.score,
        })
        .collect();

    info!(count = matches.len(), "Search completed");

    if matches.is_empty() {
        debug!("No matching tools found for query");
    } else {
        for (i, m) in matches.iter().enumerate() {
            debug!(
                rank = i + 1,
                tool = %m.tool_name,
                score = m.score,
                "Search result"
            );
        }
    }

    Ok(matches)
}

/// Loads the full content of a manpage.
///
/// Runs `man -P cat <tool>` to get the raw manpage content, cleans
/// escape sequences, and truncates to fit LLM context limits.
///
/// # Errors
///
/// Returns an error if:
/// - The manpage doesn't exist
/// - The man command fails
/// - Output contains invalid UTF-8
pub fn load_manpage_content(tool_name: &str) -> Result<String> {
    debug!(tool = %tool_name, "Loading manpage content");

    // Run man -P cat to get raw content
    let output = Command::new("man")
        .args(["-P", "cat", tool_name])
        .output()
        .with_context(|| format!("Failed to execute man command for '{tool_name}'"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Manpage for '{}' not found: {}", tool_name, stderr.trim());
    }

    // Convert to UTF-8
    let content = String::from_utf8(output.stdout)
        .with_context(|| format!("Manpage '{tool_name}' contains invalid UTF-8"))?;

    // Clean escape sequences
    let cleaned = clean_escape_sequences(&content);

    // Truncate to max length
    let truncated = truncate_content(&cleaned, MAX_CONTENT_LENGTH);

    debug!(
        original_len = content.len(),
        cleaned_len = cleaned.len(),
        final_len = truncated.len(),
        "Loaded manpage content"
    );

    Ok(truncated)
}

/// Removes ANSI escape sequences from text.
fn clean_escape_sequences(text: &str) -> String {
    // Simple approach: remove common escape patterns
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                              // Skip until we hit a letter (end of sequence)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    // Normalize whitespace: collapse multiple spaces/newlines
    let mut normalized = String::with_capacity(result.len());
    let mut prev_whitespace = false;
    let mut prev_newline = false;

    for c in result.chars() {
        if c == '\n' {
            if !prev_newline {
                normalized.push('\n');
                prev_newline = true;
            }
            prev_whitespace = true;
        } else if c.is_whitespace() {
            if !prev_whitespace {
                normalized.push(' ');
                prev_whitespace = true;
            }
            prev_newline = false;
        } else {
            normalized.push(c);
            prev_whitespace = false;
            prev_newline = false;
        }
    }

    normalized
}

/// Truncates content to maximum length, respecting UTF-8 boundaries.
fn truncate_content(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Find last valid char boundary
    let mut end = max_len;
    while !text.is_char_boundary(end) && end > 0 {
        end -= 1;
    }

    let mut result = text[..end].to_string();
    result.push_str("\n\n[Content truncated...]");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_match_debug() {
        let match_result = SearchMatch {
            tool_name: "ffmpeg".to_string(),
            section: "1".to_string(),
            description: "video converter".to_string(),
            score: 0.5,
        };
        let debug_str = format!("{match_result:?}");
        assert!(debug_str.contains("ffmpeg"));
    }

    #[test]
    fn test_search_match_clone() {
        let match_result = SearchMatch {
            tool_name: "ls".to_string(),
            section: "1".to_string(),
            description: "list directory".to_string(),
            score: 0.3,
        };
        let cloned = match_result.clone();
        assert_eq!(cloned.tool_name, "ls");
    }
}
