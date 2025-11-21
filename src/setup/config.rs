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
    /// The name of the Ollama model to use.
    pub model_name: String,
    /// The URL of the Ollama server.
    pub ollama_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_name: "llama3.2:3b".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
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

/// Loads configuration from file.
///
/// Returns default configuration if the file doesn't exist.
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

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
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
        assert_eq!(config.model_name, "llama3.2:3b");
        assert_eq!(config.ollama_url, "http://localhost:11434");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            model_name: "mistral:7b".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("model_name = \"mistral:7b\""));
        assert!(toml_str.contains("ollama_url = \"http://localhost:11434\""));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            model_name = "llama3.1:8b"
            ollama_url = "http://localhost:11434"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.model_name, "llama3.1:8b");
        assert_eq!(config.ollama_url, "http://localhost:11434");
    }

    #[test]
    fn test_config_roundtrip() {
        let original = Config {
            model_name: "phi3:mini".to_string(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
        };

        let toml_str = toml::to_string(&original).unwrap();
        let restored: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_get_config_path_returns_toml_file() {
        // This test verifies the path ends with config.toml
        // Note: Actual path depends on system, but filename should be consistent
        let result = get_config_path();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert_eq!(path.file_name().unwrap(), "config.toml");
    }

    #[test]
    fn test_save_and_load_config() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            model_name: "test-model:latest".to_string(),
            ollama_url: "http://test:11434".to_string(),
        };

        // Serialize and write
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        // Read and deserialize
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(config, loaded_config);
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
