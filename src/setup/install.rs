//! Ollama installation detection and management.
//!
//! This module provides functionality to detect Ollama installation status
//! and system capabilities for installation.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use tracing::{debug, info, warn};

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

/// Result of an installation attempt.
#[derive(Debug)]
pub struct InstallResult {
    /// Whether installation succeeded.
    pub success: bool,
    /// Human-readable message about the result.
    pub message: String,
    /// Suggested next action for the user.
    pub next_action: Option<String>,
}

/// Installs Ollama natively using the official installer.
///
/// # Arguments
///
/// * `os` - Operating system name ("linux" or "macos")
///
/// # Errors
///
/// Returns an error if the installation command fails or times out.
pub async fn install_native(os: &str) -> Result<InstallResult> {
    info!(os, "Starting native Ollama installation");

    // Request sudo explanation
    println!("\nInstalling Ollama requires administrator privileges.");
    println!("You may be prompted for your password.\n");

    let result = match os {
        "linux" => install_linux().await,
        "macos" => install_macos().await,
        _ => bail!("Native installation not supported on {os}"),
    };

    match result {
        Ok(()) => {
            // Verify installation
            if check_command_exists("ollama") {
                // Start the service
                if let Err(e) = start_ollama().await {
                    warn!(error = %e, "Failed to start Ollama service");
                    return Ok(InstallResult {
                        success: true,
                        message: "Ollama installed successfully".to_string(),
                        next_action: Some("Run 'ollama serve' to start the service".to_string()),
                    });
                }

                // Wait for API to be ready
                match wait_for_ollama(60).await {
                    Ok(()) => Ok(InstallResult {
                        success: true,
                        message: "Ollama installed and running".to_string(),
                        next_action: None,
                    }),
                    Err(_) => Ok(InstallResult {
                        success: true,
                        message: "Ollama installed but not yet responding".to_string(),
                        next_action: Some("Wait a moment and run 'ulm setup' again".to_string()),
                    }),
                }
            } else {
                Ok(InstallResult {
                    success: false,
                    message: "Installation completed but ollama binary not found".to_string(),
                    next_action: Some("Check installation logs and try again".to_string()),
                })
            }
        }
        Err(e) => Ok(InstallResult {
            success: false,
            message: format!("Installation failed: {e}"),
            next_action: Some("Check error message and try manual installation".to_string()),
        }),
    }
}

/// Installs Ollama on Linux via curl.
async fn install_linux() -> Result<()> {
    println!("Installing Ollama via official installer...");

    let start = Instant::now();
    let timeout = Duration::from_secs(300); // 5 minutes

    let mut child = Command::new("sh")
        .arg("-c")
        .arg("curl -fsSL https://ollama.com/install.sh | sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start installer")?;

    // Poll for completion with timeout
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                bail!("Installer exited with status: {status}");
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    bail!("Installation timed out after 5 minutes");
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => bail!("Failed to check installer status: {e}"),
        }
    }
}

/// Installs Ollama on macOS via brew or curl fallback.
async fn install_macos() -> Result<()> {
    // Try brew first
    if check_command_exists("brew") {
        println!("Installing Ollama via Homebrew...");

        let start = Instant::now();
        let timeout = Duration::from_secs(300);

        let mut child = Command::new("brew")
            .args(["install", "ollama"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start brew")?;

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        return Ok(());
                    }
                    // Brew failed, try curl fallback
                    warn!("Brew installation failed, trying curl fallback");
                    break;
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        bail!("Installation timed out after 5 minutes");
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => bail!("Failed to check brew status: {e}"),
            }
        }
    }

    // Curl fallback (same as Linux)
    install_linux().await
}

/// Starts the Ollama service.
///
/// # Errors
///
/// Returns an error if the service fails to start.
pub async fn start_ollama() -> Result<()> {
    info!("Starting Ollama service");

    // Start ollama serve in background
    Command::new("ollama")
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start ollama serve")?;

    // Give it a moment to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(())
}

/// Waits for Ollama API to become available.
///
/// # Arguments
///
/// * `timeout_secs` - Maximum seconds to wait
///
/// # Errors
///
/// Returns an error if the API is not available within the timeout.
pub async fn wait_for_ollama(timeout_secs: u64) -> Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    debug!(timeout_secs, "Waiting for Ollama API");

    while start.elapsed() < timeout {
        if check_ollama_api().await {
            info!(elapsed_ms = start.elapsed().as_millis(), "Ollama API ready");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    bail!("Ollama API not available after {timeout_secs} seconds")
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
