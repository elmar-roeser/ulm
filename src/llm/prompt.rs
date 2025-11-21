//! Prompt building for LLM queries.
//!
//! This module constructs prompts that combine system instructions,
//! manpage content, directory context, and user queries.

use crate::query::DirectoryContext;

/// System instructions for the command-line assistant.
const SYSTEM_INSTRUCTIONS: &str = r#"You are a command-line expert assistant. Your task is to suggest relevant commands based on the user's query and the provided manpage documentation.

Analyze the manpage content and generate practical command suggestions that solve the user's problem. Consider the working directory context when suggesting commands.

IMPORTANT: Respond ONLY with valid JSON in the following format:
{
  "suggestions": [
    {
      "command": "the exact command to run",
      "title": "brief title (3-5 words)",
      "explanation": "why this command helps and what it does",
      "risk_level": "safe|moderate|destructive"
    }
  ]
}

Risk levels:
- "safe": Read-only operations, no side effects
- "moderate": Modifies files but recoverable (e.g., creates/edits files)
- "destructive": Irreversible operations (e.g., rm -rf, force push)

Provide 1-3 command suggestions, ordered by relevance. If the query cannot be answered with the provided manpage, respond with an empty suggestions array."#;

/// Maximum total prompt length in characters (~12000 tokens).
const MAX_PROMPT_LENGTH: usize = 48000;

/// Builds a complete prompt for the LLM.
///
/// Combines system instructions, manpage content, directory context,
/// and user query into a formatted prompt string.
///
/// # Arguments
///
/// * `query` - The user's natural language query
/// * `manpage_content` - Full or truncated manpage text
/// * `context` - Directory context information
///
/// # Returns
///
/// A formatted prompt string ready for LLM consumption.
#[must_use]
pub fn build_prompt(query: &str, manpage_content: &str, context: &DirectoryContext) -> String {
    let context_str = context.format_for_prompt();

    // Build initial prompt
    let mut prompt = format!(
        "{SYSTEM_INSTRUCTIONS}

---

## Context

{context_str}

---

## Manpage Content

{manpage_content}

---

## User Query

{query}

---

Respond with JSON only:"
    );

    // Truncate if too long
    if prompt.len() > MAX_PROMPT_LENGTH {
        // Calculate how much to truncate from manpage content
        let overhead = prompt.len() - manpage_content.len();
        let max_content = MAX_PROMPT_LENGTH
            .saturating_sub(overhead)
            .saturating_sub(100);

        // Truncate manpage content
        let truncated_content = truncate_at_boundary(manpage_content, max_content);

        prompt = format!(
            "{SYSTEM_INSTRUCTIONS}

---

## Context

{context_str}

---

## Manpage Content

{truncated_content}

[Content truncated for length]

---

## User Query

{query}

---

Respond with JSON only:"
        );
    }

    prompt
}

/// Truncates text at a safe UTF-8 boundary.
fn truncate_at_boundary(text: &str, max_len: usize) -> &str {
    if text.len() <= max_len {
        return text;
    }

    let mut end = max_len;
    while !text.is_char_boundary(end) && end > 0 {
        end -= 1;
    }

    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_context() -> DirectoryContext {
        DirectoryContext {
            project_type: Some(crate::query::ProjectType::Rust),
            marker_files: vec!["Cargo.toml".to_string()],
            cwd: PathBuf::from("/home/user/project"),
        }
    }

    #[test]
    fn test_build_prompt_contains_all_parts() {
        let context = test_context();
        let prompt = build_prompt("find large files", "NAME\n    find - search", &context);

        assert!(prompt.contains("command-line expert"));
        assert!(prompt.contains("find large files"));
        assert!(prompt.contains("NAME\n    find - search"));
        assert!(prompt.contains("Rust"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_build_prompt_json_format() {
        let context = test_context();
        let prompt = build_prompt("test", "content", &context);

        assert!(prompt.contains("suggestions"));
        assert!(prompt.contains("command"));
        assert!(prompt.contains("risk_level"));
    }

    #[test]
    fn test_build_prompt_truncation() {
        let context = test_context();
        let long_content = "x".repeat(100_000);
        let prompt = build_prompt("query", &long_content, &context);

        assert!(prompt.len() <= MAX_PROMPT_LENGTH + 100);
        assert!(prompt.contains("[Content truncated"));
    }

    #[test]
    fn test_truncate_at_boundary() {
        let text = "Hello, 世界!";
        let truncated = truncate_at_boundary(text, 10);
        assert!(truncated.len() <= 10);
        assert!(truncated.is_char_boundary(truncated.len()));
    }

    #[test]
    fn test_truncate_short_text() {
        let text = "short";
        let truncated = truncate_at_boundary(text, 100);
        assert_eq!(truncated, text);
    }
}
