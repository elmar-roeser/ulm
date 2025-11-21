//! Ollama detection and health checking for setup.
//!
//! This module provides functionality to detect if Ollama is running
//! and guide users through installation if needed.

use std::io::{self, Write};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

use crate::llm::{OllamaClient, DEFAULT_MODEL};

/// Checks Ollama availability and guides installation.
#[derive(Debug)]
pub struct OllamaChecker {
    /// Ollama API client.
    client: OllamaClient,
}

impl OllamaChecker {
    /// Creates a new Ollama checker.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new() -> Result<Self> {
        let client = OllamaClient::new().context("Failed to create Ollama client")?;
        Ok(Self { client })
    }

    /// Creates a new Ollama checker with a custom URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_url(url: &str) -> Result<Self> {
        let client = OllamaClient::with_url(url).context("Failed to create Ollama client")?;
        Ok(Self { client })
    }

    /// Checks if Ollama is running and accessible.
    ///
    /// Displays success or failure message to the user.
    ///
    /// # Errors
    ///
    /// Returns an error if Ollama is not reachable.
    pub async fn check_connection(&self) -> Result<()> {
        debug!("Checking Ollama connection at {}", self.client.base_url());

        match self.client.health_check().await {
            Ok(true) => {
                info!("Ollama detected");
                println!("✓ Ollama detected at {}", self.client.base_url());
                Ok(())
            }
            Ok(false) => {
                self.print_install_instructions();
                anyhow::bail!(
                    "Ollama returned unsuccessful status at {}",
                    self.client.base_url()
                );
            }
            Err(e) => {
                self.print_install_instructions();
                Err(e).context("Failed to connect to Ollama")
            }
        }
    }

    /// Prints installation instructions to stderr.
    fn print_install_instructions(&self) {
        eprintln!("✗ Ollama not found at {}", self.client.base_url());
        eprintln!();
        eprintln!("Please install Ollama:");
        eprintln!("  • Download from https://ollama.ai");
        eprintln!("  • Or run via Docker: docker run -d -p 11434:11434 ollama/ollama");
        eprintln!();
        eprintln!("Then start with: ollama serve");
    }

    /// Returns a reference to the underlying client.
    #[must_use]
    pub const fn client(&self) -> &OllamaClient {
        &self.client
    }

    /// Checks for a suitable model and optionally pulls one.
    ///
    /// Returns the name of the available model.
    ///
    /// # Errors
    ///
    /// Returns an error if no model is available and user declines to pull.
    pub async fn check_model(&self) -> Result<String> {
        debug!("Checking for available models");

        let models = self
            .client
            .list_models()
            .await
            .context("Failed to list models")?;

        // Check for suitable models (in order of preference)
        let preferred = ["llama3", "llama3.2", "mistral", "gemma2", "phi3"];

        for pref in &preferred {
            if let Some(model) = models.iter().find(|m| m.name.starts_with(pref)) {
                info!(model = %model.name, "Model found");
                println!("✓ Model '{}' available", model.name);
                return Ok(model.name.clone());
            }
        }

        // No suitable model found
        warn!("No suitable model found");
        self.prompt_pull_model().await
    }

    /// Prompts user to pull a model.
    async fn prompt_pull_model(&self) -> Result<String> {
        println!("No suitable model found.");
        print!("Pull default model '{DEFAULT_MODEL}'? [Y/n] ");
        io::stdout().flush().context("Failed to flush stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        let input = input.trim().to_lowercase();
        if input.is_empty() || input == "y" || input == "yes" {
            self.pull_model(DEFAULT_MODEL).await?;
            Ok(DEFAULT_MODEL.to_string())
        } else {
            eprintln!("No model available. Pull manually with: ollama pull {DEFAULT_MODEL}");
            anyhow::bail!("No model available")
        }
    }

    /// Pulls a model from Ollama registry.
    ///
    /// # Errors
    ///
    /// Returns an error if the pull fails.
    pub async fn pull_model(&self, name: &str) -> Result<()> {
        println!("Pulling {name}... (this may take a few minutes)");

        self.client
            .pull_model(name)
            .await
            .with_context(|| format!("Failed to pull model '{name}'"))?;

        info!(model = %name, "Model pulled successfully");
        println!("✓ Model '{name}' pulled successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_creation() {
        let checker = OllamaChecker::new().expect("Failed to create checker");
        assert!(checker.client.base_url().contains("11434"));
    }

    #[test]
    fn test_checker_custom_url() {
        let checker =
            OllamaChecker::with_url("http://custom:8080").expect("Failed to create checker");
        assert_eq!(checker.client.base_url(), "http://custom:8080");
    }
}
