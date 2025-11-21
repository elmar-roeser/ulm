//! Ollama API client for embeddings and LLM inference.
//!
//! This module provides the HTTP client for communicating with Ollama's REST API.

use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Default Ollama API base URL.
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Default model for embeddings and generation.
pub const DEFAULT_MODEL: &str = "llama3";

/// Client for interacting with Ollama API.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    /// HTTP client.
    client: reqwest::Client,
    /// Base URL for Ollama API.
    base_url: String,
}

/// Request for generating embeddings.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    /// Model to use for embedding generation.
    pub model: String,
    /// Text to generate embedding for.
    pub prompt: String,
}

/// Response from embedding generation.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    /// Vector embedding.
    pub embedding: Vec<f32>,
}

/// Request for LLM text generation.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    /// Model to use for generation.
    pub model: String,
    /// Prompt text.
    pub prompt: String,
    /// Whether to stream the response.
    pub stream: bool,
    /// Output format (e.g., "json").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Response from LLM text generation.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    /// Generated text.
    pub response: String,
}

/// Response from /api/tags endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TagsResponse {
    /// List of available models.
    pub models: Vec<ModelInfo>,
}

/// Information about an installed model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    /// Model name (e.g., "llama3:latest").
    pub name: String,
    /// Model size in bytes.
    pub size: u64,
}

impl OllamaClient {
    /// Creates a new Ollama client with the default URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new() -> Result<Self> {
        Self::with_url(DEFAULT_OLLAMA_URL)
    }

    /// Creates a new Ollama client with a custom URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_url(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Returns the base URL of this client.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Checks if Ollama is reachable.
    ///
    /// # Errors
    ///
    /// Returns an error if Ollama is not reachable.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to Ollama at {}. Start with: ollama serve",
                    self.base_url
                )
            })?;

        Ok(response.status().is_success())
    }

    /// Lists available models.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or response cannot be parsed.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to Ollama at {}. Start with: ollama serve",
                    self.base_url
                )
            })?;

        let tags: TagsResponse = response
            .json()
            .await
            .context("Failed to parse model list response")?;

        Ok(tags.models)
    }

    /// Generates an embedding for the given text.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or response cannot be parsed.
    pub async fn generate_embedding(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let request = EmbeddingRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(30))
            .json(&request)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to Ollama at {}. Start with: ollama serve",
                    self.base_url
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama embedding request failed ({status}): {body}");
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        Ok(embedding_response.embedding)
    }

    /// Generates text using the LLM.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or response cannot be parsed.
    pub async fn generate(&self, model: &str, prompt: &str, json_format: bool) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            format: if json_format {
                Some("json".to_string())
            } else {
                None
            },
        };

        let response = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(60))
            .json(&request)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to Ollama at {}. Start with: ollama serve",
                    self.base_url
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama generate request failed ({status}): {body}");
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .context("Failed to parse generate response")?;

        Ok(generate_response.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_request_serialization() {
        let request = EmbeddingRequest {
            model: "llama3".to_string(),
            prompt: "test prompt".to_string(),
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("\"model\":\"llama3\""));
        assert!(json.contains("\"prompt\":\"test prompt\""));
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{"embedding": [0.1, 0.2, 0.3]}"#;
        let response: EmbeddingResponse =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_generate_request_serialization() {
        let request = GenerateRequest {
            model: "llama3".to_string(),
            prompt: "Hello".to_string(),
            stream: false,
            format: Some("json".to_string()),
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("\"model\":\"llama3\""));
        assert!(json.contains("\"stream\":false"));
        assert!(json.contains("\"format\":\"json\""));
    }

    #[test]
    fn test_generate_request_no_format() {
        let request = GenerateRequest {
            model: "llama3".to_string(),
            prompt: "Hello".to_string(),
            stream: false,
            format: None,
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(!json.contains("format"));
    }

    #[test]
    fn test_generate_response_deserialization() {
        let json = r#"{"response": "Hello, world!"}"#;
        let response: GenerateResponse =
            serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.response, "Hello, world!");
    }

    #[test]
    fn test_tags_response_deserialization() {
        let json = r#"{"models": [{"name": "llama3:latest", "size": 4000000000}]}"#;
        let response: TagsResponse = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(response.models.len(), 1);
        assert_eq!(response.models[0].name, "llama3:latest");
        assert_eq!(response.models[0].size, 4_000_000_000);
    }

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new().expect("Failed to create client");
        assert_eq!(client.base_url(), DEFAULT_OLLAMA_URL);
    }

    #[test]
    fn test_client_custom_url() {
        let client =
            OllamaClient::with_url("http://custom:8080/").expect("Failed to create client");
        assert_eq!(client.base_url(), "http://custom:8080");
    }
}
