//! Ollama detection and health checking for setup.
//!
//! This module provides functionality to detect if Ollama is running
//! and guide users through installation if needed.

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::llm::OllamaClient;

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
