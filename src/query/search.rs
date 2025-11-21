//! Vector search for finding relevant manpages.
//!
//! This module provides semantic search functionality using the manpage
//! embedding index to find tools relevant to user queries.

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::db;
use crate::llm::{OllamaClient, DEFAULT_MODEL};

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

    // Generate query embedding
    let client = OllamaClient::new().context("Failed to create Ollama client")?;
    let embedding = client
        .generate_embedding(DEFAULT_MODEL, query)
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
