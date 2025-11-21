//! Query processing and intelligence.
//!
//! This module handles user queries by combining semantic search,
//! directory context awareness, and LLM-powered response generation.

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::llm::{build_prompt, parse_suggestions, OllamaClient, DEFAULT_MODEL};

pub mod context;
pub mod search;

pub use context::{scan_directory_context, DirectoryContext, ProjectType};
pub use search::{load_manpage_content, search_tools, SearchMatch};

// Re-export for convenience
pub use crate::llm::{CommandSuggestion, RiskLevel};

/// Maximum number of tools to retrieve from vector search.
const MAX_SEARCH_RESULTS: usize = 3;

/// Processes a user query and returns command suggestions.
///
/// Orchestrates the full query pipeline:
/// 1. Search for relevant tools via vector similarity
/// 2. Load manpage content for top match
/// 3. Scan current directory context
/// 4. Build LLM prompt
/// 5. Generate response via Ollama
/// 6. Parse JSON response
///
/// # Arguments
///
/// * `query` - Natural language query describing desired functionality
///
/// # Errors
///
/// Returns an error if:
/// - No matching tools found in index
/// - Ollama API call fails
/// - Response parsing fails
pub async fn process_query(query: &str) -> Result<Vec<CommandSuggestion>> {
    info!(query = %query, "Processing query");

    // Step 1: Search for relevant tools
    let matches = search_tools(query, MAX_SEARCH_RESULTS)
        .await
        .context("Failed to search for tools")?;

    if matches.is_empty() {
        anyhow::bail!(
            "No matching tools found for query: '{query}'. \
             Try running 'ulm setup' to build the index."
        );
    }

    debug!(
        count = matches.len(),
        top_tool = %matches[0].tool_name,
        top_score = matches[0].score,
        "Found matching tools"
    );

    // Step 2: Load manpage content for top match
    let top_tool = &matches[0].tool_name;
    let manpage_content =
        load_manpage_content(top_tool).context("Failed to load manpage content")?;

    debug!(
        tool = %top_tool,
        content_len = manpage_content.len(),
        "Loaded manpage content"
    );

    // Step 3: Scan directory context
    let context = scan_directory_context().context("Failed to scan directory context")?;

    debug!(
        project_type = context.project_type.as_ref().map_or("None", |p| p.as_str()),
        cwd = %context.cwd.display(),
        "Scanned directory context"
    );

    // Step 4: Build prompt
    let prompt = build_prompt(query, &manpage_content, &context);

    debug!(prompt_len = prompt.len(), "Built prompt");

    // Step 5: Call Ollama to generate response
    let client = OllamaClient::new().context("Failed to create Ollama client")?;

    info!("Calling Ollama for response generation");

    let response = client
        .generate(DEFAULT_MODEL, &prompt, true)
        .await
        .context("Failed to generate LLM response")?;

    debug!(response_len = response.len(), "Received LLM response");

    // Step 6: Parse response
    let suggestions = parse_suggestions(&response).context("Failed to parse LLM response")?;

    info!(suggestions = suggestions.len(), "Query processing complete");

    Ok(suggestions)
}
