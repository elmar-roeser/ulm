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
        docker_available, curl_available, brew_available, os, "System capabilities detected"
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

/// Checks if Docker daemon is running and responsive.
///
/// # Returns
///
/// Returns `true` if Docker daemon responds to `docker info`, `false` otherwise.
fn is_docker_daemon_running() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Docker container status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerStatus {
    /// Container exists and is running.
    Running,
    /// Container exists but is stopped.
    Stopped,
    /// Container does not exist.
    NotFound,
}

/// Checks if a container with the given name exists and its status.
///
/// # Arguments
///
/// * `name` - Container name to check
///
/// # Returns
///
/// Returns the container status (Running, Stopped, or `NotFound`).
fn check_container_status(name: &str) -> ContainerStatus {
    // Check if container exists and is running
    let running = Command::new("docker")
        .args(["ps", "-q", "-f", &format!("name=^{name}$")])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if running {
        return ContainerStatus::Running;
    }

    // Check if container exists but stopped
    let exists = Command::new("docker")
        .args(["ps", "-aq", "-f", &format!("name=^{name}$")])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if exists {
        ContainerStatus::Stopped
    } else {
        ContainerStatus::NotFound
    }
}

/// Installs Ollama using Docker.
///
/// # Arguments
///
/// * `action` - What to do if container exists: "restart", "recreate", or "create"
///
/// # Errors
///
/// Returns an error if Docker is not available, daemon not running,
/// or container operations fail.
pub async fn install_docker(action: &str) -> Result<InstallResult> {
    info!(action, "Starting Docker Ollama installation");

    // Check Docker is available
    if !check_command_exists("docker") {
        return Ok(InstallResult {
            success: false,
            message: "Docker not found. Install from https://docs.docker.com/get-docker/"
                .to_string(),
            next_action: Some("Install Docker or use native installation instead".to_string()),
        });
    }

    // Check Docker daemon is running
    if !is_docker_daemon_running() {
        return Ok(InstallResult {
            success: false,
            message: "Docker daemon is not running".to_string(),
            next_action: Some("Start Docker daemon with: sudo systemctl start docker".to_string()),
        });
    }

    // Check existing container status
    let container_status = check_container_status("ollama");
    debug!(?container_status, "Container status check");

    match (action, container_status) {
        ("restart", ContainerStatus::Running) => {
            info!("Container already running");
            return Ok(InstallResult {
                success: true,
                message: "Ollama container is already running".to_string(),
                next_action: None,
            });
        }
        ("restart", ContainerStatus::Stopped) => {
            // Start existing container
            let start_result = Command::new("docker")
                .args(["start", "ollama"])
                .output()
                .context("Failed to start container")?;

            if !start_result.status.success() {
                let stderr = String::from_utf8_lossy(&start_result.stderr);
                return Ok(InstallResult {
                    success: false,
                    message: format!("Failed to start container: {stderr}"),
                    next_action: Some("Try recreating the container".to_string()),
                });
            }
        }
        ("recreate", ContainerStatus::Running | ContainerStatus::Stopped) => {
            // Remove existing container
            info!("Removing existing container");
            let rm_result = Command::new("docker")
                .args(["rm", "-f", "ollama"])
                .output()
                .context("Failed to remove container")?;

            if !rm_result.status.success() {
                let stderr = String::from_utf8_lossy(&rm_result.stderr);
                warn!(stderr = %stderr, "Failed to remove container");
            }

            // Create new container
            return create_ollama_container().await;
        }
        (_, ContainerStatus::Running | ContainerStatus::Stopped) => {
            // Container exists but action is "create" - should not happen
            return Ok(InstallResult {
                success: false,
                message: "Container 'ollama' already exists".to_string(),
                next_action: Some("Choose 'restart' or 'recreate' action".to_string()),
            });
        }
        (_, ContainerStatus::NotFound) => {
            // Create new container
            return create_ollama_container().await;
        }
    }

    // Wait for API to be ready
    match wait_for_ollama(60).await {
        Ok(()) => {
            // Get container info
            let container_id = get_container_id("ollama").unwrap_or_else(|| "unknown".to_string());
            Ok(InstallResult {
                success: true,
                message: format!("Ollama container started (ID: {container_id})"),
                next_action: None,
            })
        }
        Err(_) => Ok(InstallResult {
            success: true,
            message: "Container started but API not yet responding".to_string(),
            next_action: Some(
                "Wait a moment and check with: curl http://localhost:11434/api/tags".to_string(),
            ),
        }),
    }
}

/// Creates a new Ollama container.
async fn create_ollama_container() -> Result<InstallResult> {
    info!("Creating new Ollama container");

    let start = Instant::now();
    let timeout = Duration::from_secs(300); // 5 minutes

    let mut child = Command::new("docker")
        .args([
            "run",
            "-d",
            "-v",
            "ollama:/root/.ollama",
            "-p",
            "11434:11434",
            "--name",
            "ollama",
            "ollama/ollama",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start docker run")?;

    // Wait for command to complete
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    // Wait for API
                    match wait_for_ollama(60).await {
                        Ok(()) => {
                            let container_id =
                                get_container_id("ollama").unwrap_or_else(|| "unknown".to_string());
                            return Ok(InstallResult {
                                success: true,
                                message: format!(
                                    "Ollama container created and running (ID: {container_id})"
                                ),
                                next_action: None,
                            });
                        }
                        Err(_) => {
                            return Ok(InstallResult {
                                success: true,
                                message: "Container created but API not yet responding".to_string(),
                                next_action: Some(
                                    "Wait a moment, container may be pulling the image".to_string(),
                                ),
                            });
                        }
                    }
                }

                // Check for port conflict
                let stderr = child
                    .stderr
                    .take()
                    .and_then(|mut s| {
                        let mut buf = String::new();
                        std::io::Read::read_to_string(&mut s, &mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();

                if stderr.contains("port is already allocated")
                    || stderr.contains("address already in use")
                {
                    return Ok(InstallResult {
                        success: false,
                        message: "Port 11434 is already in use".to_string(),
                        next_action: Some(
                            "Stop the process using port 11434 or use a different port".to_string(),
                        ),
                    });
                }

                return Ok(InstallResult {
                    success: false,
                    message: format!("Docker run failed: {stderr}"),
                    next_action: Some("Check Docker logs for details".to_string()),
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Ok(InstallResult {
                        success: false,
                        message: "Docker run timed out after 5 minutes".to_string(),
                        next_action: Some("Check network connection and try again".to_string()),
                    });
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                return Ok(InstallResult {
                    success: false,
                    message: format!("Failed to check docker status: {e}"),
                    next_action: Some("Try running docker manually".to_string()),
                });
            }
        }
    }
}

/// Gets the container ID for a named container.
fn get_container_id(name: &str) -> Option<String> {
    Command::new("docker")
        .args(["ps", "-q", "-f", &format!("name=^{name}$")])
        .output()
        .ok()
        .and_then(|output| {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if id.is_empty() {
                None
            } else {
                Some(id)
            }
        })
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

    #[test]
    fn test_container_status_eq() {
        assert_eq!(ContainerStatus::Running, ContainerStatus::Running);
        assert_ne!(ContainerStatus::Running, ContainerStatus::Stopped);
        assert_ne!(ContainerStatus::Stopped, ContainerStatus::NotFound);
    }

    #[test]
    fn test_container_status_debug() {
        let status = ContainerStatus::Running;
        let debug_str = format!("{status:?}");
        assert!(debug_str.contains("Running"));
    }

    #[test]
    fn test_is_docker_daemon_running_no_docker() {
        // If docker is not installed, should return false
        // This test just verifies it doesn't panic
        let _ = is_docker_daemon_running();
    }

    #[test]
    fn test_check_container_status_nonexistent() {
        // Check for a container that definitely doesn't exist
        let status = check_container_status("nonexistent_container_12345");
        assert_eq!(status, ContainerStatus::NotFound);
    }

    #[test]
    fn test_get_container_id_nonexistent() {
        // Container that doesn't exist should return None
        let id = get_container_id("nonexistent_container_12345");
        assert!(id.is_none());
    }

    #[tokio::test]
    async fn test_install_docker_no_docker() {
        // This test will pass if docker command exists but may fail on systems without docker
        // The function should handle missing docker gracefully
        let result = install_docker("create").await;
        assert!(result.is_ok());
        // If docker is not available, success should be false
        let install_result = result.expect("Result should be Ok");
        if !check_command_exists("docker") {
            assert!(!install_result.success);
            assert!(install_result.message.contains("Docker not found"));
        }
    }
}
