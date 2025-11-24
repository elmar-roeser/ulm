//! Configuration file management.
//!
//! This module handles loading and saving application configuration
//! using XDG-compliant paths and TOML format.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Model configuration.
    pub models: ModelsConfig,
    /// Ollama server configuration.
    pub ollama: OllamaConfig,
    /// Index metadata.
    pub index: IndexConfig,
}

/// Model configuration for embedding and LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelsConfig {
    /// Model used for generating embeddings (index + query).
    pub embedding_model: String,
    /// Model used for LLM response generation.
    pub llm_model: String,
}

/// Ollama server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OllamaConfig {
    /// The URL of the Ollama server.
    pub url: String,
}

/// Index metadata for validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexConfig {
    /// Embedding dimension used when building index.
    pub embedding_dimension: Option<u32>,
    /// Model name used when building index.
    pub last_embedding_model: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            models: ModelsConfig {
                embedding_model: "nomic-embed-text".to_string(),
                llm_model: "llama3.2:3b".to_string(),
            },
            ollama: OllamaConfig {
                url: "http://localhost:11434".to_string(),
            },
            index: IndexConfig {
                embedding_dimension: None,
                last_embedding_model: None,
            },
        }
    }
}

impl Config {
    /// Get the embedding model name.
    #[must_use]
    pub fn embedding_model(&self) -> &str {
        &self.models.embedding_model
    }

    /// Get the LLM model name.
    #[must_use]
    pub fn llm_model(&self) -> &str {
        &self.models.llm_model
    }

    /// Get the Ollama URL.
    #[must_use]
    pub fn ollama_url(&self) -> &str {
        &self.ollama.url
    }

    /// Update index metadata after building index.
    pub fn update_index_metadata(&mut self, dimension: u32) {
        self.index.embedding_dimension = Some(dimension);
        self.index.last_embedding_model = Some(self.models.embedding_model.clone());
    }

    /// Check if index needs rebuild due to model change.
    #[must_use]
    pub fn needs_index_rebuild(&self) -> bool {
        match &self.index.last_embedding_model {
            Some(last_model) => last_model != &self.models.embedding_model,
            None => false, // No previous index, not a "rebuild"
        }
    }

    /// Get index dimension if available.
    #[must_use]
    pub const fn index_dimension(&self) -> Option<u32> {
        self.index.embedding_dimension
    }
}

/// Legacy configuration format for migration.
#[derive(Debug, Deserialize)]
#[allow(clippy::missing_docs_in_private_items)]
struct LegacyConfig {
    model_name: String,
    ollama_url: String,
}

/// Gets the XDG-compliant config file path.
///
/// Returns `~/.config/ulm/config.toml` on Linux/macOS.
///
/// # Errors
///
/// Returns an error if the config directory cannot be determined.
pub fn get_config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "ulm")
        .context("Could not determine config directory")?;

    let config_dir = dirs.config_dir();

    // Create config directory if needed
    fs::create_dir_all(config_dir).with_context(|| {
        format!(
            "Failed to create config directory: {}",
            config_dir.display()
        )
    })?;

    Ok(config_dir.join("config.toml"))
}

/// Loads configuration from file with automatic migration.
///
/// Returns default configuration if the file doesn't exist.
/// Automatically migrates legacy single-model format to new multi-model format.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    // Try to parse as new format first
    if let Ok(config) = toml::from_str::<Config>(&content) {
        return Ok(config);
    }

    // Try to parse as legacy format and migrate
    if let Ok(legacy) = toml::from_str::<LegacyConfig>(&content) {
        tracing::info!("Migrating legacy config to new multi-model format");

        let config = Config {
            models: ModelsConfig {
                // Use legacy model for both (user can change later)
                embedding_model: legacy.model_name.clone(),
                llm_model: legacy.model_name,
            },
            ollama: OllamaConfig {
                url: legacy.ollama_url,
            },
            index: IndexConfig {
                embedding_dimension: None,
                last_embedding_model: None,
            },
        };

        // Save migrated config
        save_config(&config)?;
        tracing::info!("Config migrated successfully");

        return Ok(config);
    }

    // Neither format worked
    anyhow::bail!("Failed to parse config file: {}", config_path.display())
}

/// Saves configuration to file.
///
/// Creates the config directory if it doesn't exist.
/// Sets file permissions to 0600 (user-only) on Unix systems.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

    fs::write(&config_path, &content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    // Set file permissions to user-only on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&config_path, permissions)
            .with_context(|| format!("Failed to set permissions on: {}", config_path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.models.embedding_model, "nomic-embed-text");
        assert_eq!(config.models.llm_model, "llama3.2:3b");
        assert_eq!(config.ollama.url, "http://localhost:11434");
        assert_eq!(config.index.embedding_dimension, None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            models: ModelsConfig {
                embedding_model: "nomic-embed-text".to_string(),
                llm_model: "mistral:7b".to_string(),
            },
            ollama: OllamaConfig {
                url: "http://localhost:11434".to_string(),
            },
            index: IndexConfig {
                embedding_dimension: Some(768),
                last_embedding_model: Some("nomic-embed-text".to_string()),
            },
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("embedding_model = \"nomic-embed-text\""));
        assert!(toml_str.contains("llm_model = \"mistral:7b\""));
        assert!(toml_str.contains("embedding_dimension = 768"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [models]
            embedding_model = "mxbai-embed-large"
            llm_model = "llama3.1:8b"

            [ollama]
            url = "http://localhost:11434"

            [index]
            embedding_dimension = 1024
            last_embedding_model = "mxbai-embed-large"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.models.embedding_model, "mxbai-embed-large");
        assert_eq!(config.models.llm_model, "llama3.1:8b");
        assert_eq!(config.index.embedding_dimension, Some(1024));
    }

    #[test]
    fn test_legacy_config_migration() {
        let legacy_toml = r#"
            model_name = "llama3.1:8b"
            ollama_url = "http://localhost:11434"
        "#;

        // Parse as legacy
        let legacy: LegacyConfig = toml::from_str(legacy_toml).unwrap();

        // Migrate
        let config = Config {
            models: ModelsConfig {
                embedding_model: legacy.model_name.clone(),
                llm_model: legacy.model_name,
            },
            ollama: OllamaConfig {
                url: legacy.ollama_url,
            },
            index: IndexConfig {
                embedding_dimension: None,
                last_embedding_model: None,
            },
        };

        assert_eq!(config.models.embedding_model, "llama3.1:8b");
        assert_eq!(config.models.llm_model, "llama3.1:8b");
    }

    #[test]
    fn test_config_roundtrip() {
        let original = Config {
            models: ModelsConfig {
                embedding_model: "nomic-embed-text".to_string(),
                llm_model: "phi3:mini".to_string(),
            },
            ollama: OllamaConfig {
                url: "http://127.0.0.1:11434".to_string(),
            },
            index: IndexConfig {
                embedding_dimension: Some(768),
                last_embedding_model: Some("nomic-embed-text".to_string()),
            },
        };

        let toml_str = toml::to_string(&original).unwrap();
        let restored: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_get_config_path_returns_toml_file() {
        let result = get_config_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert_eq!(path.file_name().unwrap(), "config.toml");
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            models: ModelsConfig {
                embedding_model: "test-embed".to_string(),
                llm_model: "test-llm".to_string(),
            },
            ollama: OllamaConfig {
                url: "http://test:11434".to_string(),
            },
            index: IndexConfig {
                embedding_dimension: Some(512),
                last_embedding_model: Some("test-embed".to_string()),
            },
        };

        // Serialize and write
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        // Read and deserialize
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(config, loaded_config);
    }

    #[test]
    fn test_needs_index_rebuild() {
        let mut config = Config::default();

        // No previous index
        assert!(!config.needs_index_rebuild());

        // Set index metadata
        config.update_index_metadata(768);
        assert!(!config.needs_index_rebuild());

        // Change embedding model
        config.models.embedding_model = "different-model".to_string();
        assert!(config.needs_index_rebuild());
    }

    #[test]
    fn test_update_index_metadata() {
        let mut config = Config::default();
        config.update_index_metadata(1024);

        assert_eq!(config.index.embedding_dimension, Some(1024));
        assert_eq!(
            config.index.last_embedding_model,
            Some("nomic-embed-text".to_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config::default();
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        // Set permissions
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&config_path, permissions).unwrap();

        // Verify permissions
        let metadata = fs::metadata(&config_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
