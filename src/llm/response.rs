//! LLM response parsing.
//!
//! This module parses JSON responses from the LLM into structured
//! command suggestions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Risk level for a command suggestion.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Read-only operations, no side effects.
    #[default]
    Safe,
    /// Modifies files but recoverable.
    Moderate,
    /// Irreversible operations.
    Destructive,
}

/// A command suggestion from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    /// The exact command to run.
    pub command: String,
    /// Brief title (3-5 words).
    pub title: String,
    /// Explanation of what the command does.
    pub explanation: String,
    /// Risk level of the command.
    #[serde(default)]
    pub risk_level: RiskLevel,
}

/// Response wrapper for JSON parsing.
#[derive(Debug, Deserialize)]
struct SuggestionsResponse {
    /// List of command suggestions from the LLM.
    suggestions: Vec<CommandSuggestion>,
}

/// Parses LLM response into command suggestions.
///
/// Expects JSON in the format:
/// ```json
/// {
///   "suggestions": [
///     {
///       "command": "...",
///       "title": "...",
///       "explanation": "...",
///       "risk_level": "safe|moderate|destructive"
///     }
///   ]
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - JSON parsing fails
/// - Command field is empty
pub fn parse_suggestions(response: &str) -> Result<Vec<CommandSuggestion>> {
    // Try to parse the JSON
    let parsed: SuggestionsResponse =
        serde_json::from_str(response).context("Failed to parse LLM response as JSON")?;

    // Validate suggestions
    let mut suggestions = Vec::new();
    for (i, suggestion) in parsed.suggestions.into_iter().enumerate() {
        if suggestion.command.trim().is_empty() {
            anyhow::bail!("Suggestion {} has empty command", i + 1);
        }

        suggestions.push(suggestion);
    }

    Ok(suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_response() {
        let json = r#"{
            "suggestions": [
                {
                    "command": "find . -size +100M",
                    "title": "Find large files",
                    "explanation": "Finds files larger than 100MB",
                    "risk_level": "safe"
                }
            ]
        }"#;

        let suggestions = parse_suggestions(json).unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].command, "find . -size +100M");
        assert_eq!(suggestions[0].title, "Find large files");
        assert_eq!(suggestions[0].risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_parse_multiple_suggestions() {
        let json = r#"{
            "suggestions": [
                {
                    "command": "ls -la",
                    "title": "List files",
                    "explanation": "Lists all files",
                    "risk_level": "safe"
                },
                {
                    "command": "rm -rf /tmp/*",
                    "title": "Clean temp",
                    "explanation": "Removes temp files",
                    "risk_level": "destructive"
                }
            ]
        }"#;

        let suggestions = parse_suggestions(json).unwrap();
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[1].risk_level, RiskLevel::Destructive);
    }

    #[test]
    fn test_parse_empty_suggestions() {
        let json = r#"{"suggestions": []}"#;

        let suggestions = parse_suggestions(json).unwrap();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_parse_default_risk_level() {
        let json = r#"{
            "suggestions": [
                {
                    "command": "echo test",
                    "title": "Echo",
                    "explanation": "Prints test"
                }
            ]
        }"#;

        let suggestions = parse_suggestions(json).unwrap();
        assert_eq!(suggestions[0].risk_level, RiskLevel::Safe);
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_suggestions("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_parse_empty_command() {
        let json = r#"{
            "suggestions": [
                {
                    "command": "",
                    "title": "Empty",
                    "explanation": "Bad"
                }
            ]
        }"#;

        let result = parse_suggestions(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty command"));
    }

    #[test]
    fn test_parse_whitespace_command() {
        let json = r#"{
            "suggestions": [
                {
                    "command": "   ",
                    "title": "Whitespace",
                    "explanation": "Bad"
                }
            ]
        }"#;

        let result = parse_suggestions(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_level_serialize() {
        let suggestion = CommandSuggestion {
            command: "test".to_string(),
            title: "Test".to_string(),
            explanation: "Testing".to_string(),
            risk_level: RiskLevel::Moderate,
        };

        let json = serde_json::to_string(&suggestion).unwrap();
        assert!(json.contains("\"risk_level\":\"moderate\""));
    }

    #[test]
    fn test_risk_level_default() {
        assert_eq!(RiskLevel::default(), RiskLevel::Safe);
    }
}
