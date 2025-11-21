//! Ollama installation detection and management.
//!
//! This module provides functionality to detect Ollama installation status
//! and system capabilities for installation.

use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use tracing::{debug, info};

/// Status of Ollama installation on the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OllamaStatus {
    /// Ollama is running and accessible at the API endpoint.
    Running,
    /// Ollama binary is installed but not running.
    Installed,
    /// Ollama is not installed on the system.
    NotInstalled,
}

/// System capabilities for installation.
#[derive(Debug)]
pub struct SystemCapabilities {
    /// Ollama current status.
    pub ollama_status: OllamaStatus,
    /// Whether Docker is available.
    pub docker_available: bool,
    /// Whether curl is available.
    pub curl_available: bool,
    /// Whether brew is available (macOS).
    pub brew_available: bool,
    /// Current operating system.
    pub os: String,
}

/// Detects current Ollama installation status and system capabilities.
///
/// Performs the following checks:
/// 1. Is Ollama API accessible at localhost:11434?
/// 2. Is `ollama` binary installed?
/// 3. Is Docker available?
/// 4. Is curl available?
/// 5. Is brew available (macOS)?
///
/// # Errors
///
/// Currently this function does not return errors, but the signature
/// allows for future error conditions during detection.
pub async fn detect_system() -> Result<SystemCapabilities> {
    info!("Detecting system capabilities");

    // Check if Ollama is running (API accessible)
    let ollama_running = check_ollama_api().await;
    debug!(running = ollama_running, "Ollama API check");

    // Check if Ollama binary is installed
    let ollama_installed = check_command_exists("ollama");
    debug!(installed = ollama_installed, "Ollama binary check");

    // Determine Ollama status
    let ollama_status = if ollama_running {
        OllamaStatus::Running
    } else if ollama_installed {
        OllamaStatus::Installed
    } else {
        OllamaStatus::NotInstalled
    };

    // Check other capabilities
    let docker_available = check_command_exists("docker");
    let curl_available = check_command_exists("curl");
    let brew_available = check_command_exists("brew");
    let os = std::env::consts::OS.to_string();

    debug!(
        ?ollama_status,
        docker_available,
        curl_available,
        brew_available,
        os,
        "System capabilities detected"
    );

    Ok(SystemCapabilities {
        ollama_status,
        docker_available,
        curl_available,
        brew_available,
        os,
    })
}

/// Checks if the Ollama API is accessible.
async fn check_ollama_api() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok();

    let Some(client) = client else {
        return false;
    };

    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Checks if a command exists in PATH.
fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Displays the current system status to the user.
pub fn display_status(caps: &SystemCapabilities) {
    match caps.ollama_status {
        OllamaStatus::Running => {
            println!("✓ Ollama is running and accessible");
        }
        OllamaStatus::Installed => {
            println!("! Ollama is installed but not running");
            println!("  Run: ollama serve");
        }
        OllamaStatus::NotInstalled => {
            println!("✗ Ollama is not installed");
            if caps.docker_available {
                println!("  Docker is available for container installation");
            }
            if caps.curl_available {
                println!("  curl is available for native installation");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_status_eq() {
        assert_eq!(OllamaStatus::Running, OllamaStatus::Running);
        assert_ne!(OllamaStatus::Running, OllamaStatus::Installed);
        assert_ne!(OllamaStatus::Installed, OllamaStatus::NotInstalled);
    }

    #[test]
    fn test_check_command_exists_ls() {
        // `ls` should exist on all Unix systems
        assert!(check_command_exists("ls"));
    }

    #[test]
    fn test_check_command_exists_nonexistent() {
        // This command should not exist
        assert!(!check_command_exists("nonexistent_command_12345"));
    }

    #[test]
    fn test_system_capabilities_debug() {
        let caps = SystemCapabilities {
            ollama_status: OllamaStatus::NotInstalled,
            docker_available: true,
            curl_available: true,
            brew_available: false,
            os: "linux".to_string(),
        };
        let debug_str = format!("{caps:?}");
        assert!(debug_str.contains("NotInstalled"));
        assert!(debug_str.contains("docker_available: true"));
    }

    #[tokio::test]
    async fn test_detect_system_runs() {
        // Just verify it doesn't panic
        let result = detect_system().await;
        assert!(result.is_ok());

        let caps = result.unwrap();
        // OS should be detected
        assert!(!caps.os.is_empty());
        // curl should exist on most systems
        assert!(caps.curl_available);
    }

    #[test]
    fn test_display_status_running() {
        let caps = SystemCapabilities {
            ollama_status: OllamaStatus::Running,
            docker_available: false,
            curl_available: true,
            brew_available: false,
            os: "linux".to_string(),
        };
        // Just verify it doesn't panic
        display_status(&caps);
    }

    #[test]
    fn test_display_status_not_installed() {
        let caps = SystemCapabilities {
            ollama_status: OllamaStatus::NotInstalled,
            docker_available: true,
            curl_available: true,
            brew_available: false,
            os: "linux".to_string(),
        };
        // Just verify it doesn't panic
        display_status(&caps);
    }
}
