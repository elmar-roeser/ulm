# Epic 5: Auto-Installation - Technical Specification

## Overview

Epic 5 enhances the setup flow to automatically install Ollama (natively or via Docker) when not present, providing a true zero-to-working experience with `ulm setup`.

---

## Data Models

```rust
// setup/install.rs

/// Status of Ollama installation on the system.
#[derive(Debug, Clone, PartialEq)]
pub enum OllamaStatus {
    /// Ollama is running and accessible at the API endpoint.
    Running,
    /// Ollama binary is installed but not running.
    Installed,
    /// Ollama is not installed on the system.
    NotInstalled,
}

/// Available installation methods.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallMethod {
    /// Native installation via curl/brew.
    Native,
    /// Docker container.
    Docker,
    /// User will install manually.
    Manual,
}

/// Result of an installation attempt.
#[derive(Debug)]
pub struct InstallResult {
    /// Whether installation succeeded.
    pub success: bool,
    /// Human-readable message.
    pub message: String,
    /// Suggested next action if failed.
    pub next_action: Option<String>,
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
```

---

## APIs and Interfaces

```rust
// setup/install.rs

/// Detects current Ollama installation status and system capabilities.
pub async fn detect_system() -> Result<SystemCapabilities>;

/// Prompts user to choose installation method.
pub fn prompt_install_method(caps: &SystemCapabilities) -> Result<InstallMethod>;

/// Installs Ollama natively via curl or brew.
pub async fn install_native(os: &str) -> Result<InstallResult>;

/// Installs Ollama via Docker container.
pub async fn install_docker() -> Result<InstallResult>;

/// Starts Ollama service after installation.
pub async fn start_ollama() -> Result<()>;

/// Waits for Ollama API to become available.
pub async fn wait_for_ollama(timeout_secs: u64) -> Result<()>;
```

---

## Non-Functional Requirements

### Performance

| Metric | Target | Strategy |
|--------|--------|----------|
| Detection time | < 2s | Parallel checks |
| Install timeout | 5 min | Configurable timeout |
| Health check | < 10s | Retry with backoff |

### Security

- Request sudo only when necessary (native Linux install)
- Explain why elevated permissions needed
- Never store credentials
- Use official Ollama installer URLs only

### Reliability

- Graceful fallback chain: Native → Docker → Manual
- Clear error messages with actionable guidance
- Idempotent operations (re-run safe)

---

## Dependencies

No new Cargo dependencies required. Uses existing:
- `std::process::Command` for shell execution
- `tokio` for async operations
- `reqwest` for health checks

---

## Acceptance Criteria (Authoritative)

### Story 5.1: Ollama Detection & Status

1. **AC5.1.1:** Detect Ollama running (API accessible at localhost:11434)
2. **AC5.1.2:** Detect Ollama installed but not running (`which ollama`)
3. **AC5.1.3:** Detect Ollama not installed
4. **AC5.1.4:** Detect Docker availability (`which docker`)
5. **AC5.1.5:** Report clear status message to user
6. **AC5.1.6:** `cargo build` succeeds without errors
7. **AC5.1.7:** `cargo clippy -- -D warnings` passes

### Story 5.2: Native Ollama Installation

1. **AC5.2.1:** Install on Linux via `curl -fsSL https://ollama.com/install.sh | sh`
2. **AC5.2.2:** Install on macOS via `brew install ollama` (with curl fallback)
3. **AC5.2.3:** Request sudo with explanation when needed
4. **AC5.2.4:** Verify installation succeeded
5. **AC5.2.5:** Start Ollama service after install
6. **AC5.2.6:** Report success or failure with next steps
7. **AC5.2.7:** Timeout after 5 minutes
8. **AC5.2.8:** `cargo build` succeeds without errors
9. **AC5.2.9:** `cargo clippy -- -D warnings` passes

### Story 5.3: Docker Ollama Installation

1. **AC5.3.1:** Check Docker daemon is running
2. **AC5.3.2:** Run container with correct volume and port mapping
3. **AC5.3.3:** Wait for container health check
4. **AC5.3.4:** Verify API is accessible
5. **AC5.3.5:** Handle existing container (restart or recreate)
6. **AC5.3.6:** Handle port conflicts (11434 in use)
7. **AC5.3.7:** Report success with container info
8. **AC5.3.8:** `cargo build` succeeds without errors
9. **AC5.3.9:** `cargo clippy -- -D warnings` passes

---

## Implementation Notes

### Detection Flow

```
detect_system()
├── Check localhost:11434 (async HTTP)
│   └── Success → OllamaStatus::Running
├── Check `which ollama`
│   └── Found → OllamaStatus::Installed
│   └── Not found → OllamaStatus::NotInstalled
├── Check `which docker`
│   └── Found → docker_available = true
├── Check `which curl`
├── Check `which brew` (macOS only)
└── Return SystemCapabilities
```

### Installation Commands

**Linux Native:**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**macOS Native:**
```bash
# Prefer brew if available
brew install ollama

# Fallback to curl
curl -fsSL https://ollama.com/install.sh | sh
```

**Docker:**
```bash
docker run -d \
  --name ollama \
  -v ollama:/root/.ollama \
  -p 11434:11434 \
  ollama/ollama
```

### Starting Ollama

**Linux (systemd):**
```bash
systemctl --user start ollama
# or
ollama serve &
```

**macOS:**
```bash
brew services start ollama
# or
ollama serve &
```

### Health Check

```rust
async fn wait_for_ollama(timeout_secs: u64) -> Result<()> {
    let client = reqwest::Client::new();
    let start = Instant::now();

    loop {
        if start.elapsed().as_secs() > timeout_secs {
            bail!("Timeout waiting for Ollama");
        }

        match client.get("http://localhost:11434/api/tags").send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }
}
```

### Error Handling

| Error | User Message | Next Action |
|-------|--------------|-------------|
| curl not found | "curl is required for installation" | Install curl or use Docker |
| sudo denied | "Installation requires administrator access" | Run with sudo or use Docker |
| Download failed | "Failed to download Ollama installer" | Check network, try manual install |
| Docker not running | "Docker daemon is not running" | Start Docker or use native install |
| Port in use | "Port 11434 is already in use" | Stop conflicting service |
| Container exists | "Ollama container already exists" | Restart or remove existing |

### Integration with Existing Setup

Modify `setup/mod.rs`:

```rust
pub async fn run_setup() -> Result<()> {
    println!("ulm setup - Initializing manpage index\n");

    // NEW: Check and install Ollama if needed
    let caps = install::detect_system().await?;

    match caps.ollama_status {
        OllamaStatus::Running => {
            println!("✓ Ollama is running");
        }
        OllamaStatus::Installed => {
            println!("Ollama is installed but not running");
            install::start_ollama().await?;
            install::wait_for_ollama(30).await?;
        }
        OllamaStatus::NotInstalled => {
            println!("Ollama is not installed");
            let method = install::prompt_install_method(&caps)?;
            match method {
                InstallMethod::Native => install::install_native(&caps.os).await?,
                InstallMethod::Docker => install::install_docker().await?,
                InstallMethod::Manual => {
                    println!("Please install Ollama manually:");
                    println!("  https://ollama.com/download");
                    return Ok(());
                }
            };
            install::wait_for_ollama(60).await?;
        }
    }

    // Existing setup flow continues...
    check_ollama().await?;
    // ...
}
```

---

## Testing Strategy

### Unit Tests

- `test_detect_running_ollama` - Mock HTTP response
- `test_detect_installed_ollama` - Mock `which` command
- `test_detect_not_installed` - Mock missing binary
- `test_docker_available` - Mock `which docker`
- `test_install_command_linux` - Verify curl command
- `test_install_command_macos` - Verify brew command
- `test_docker_run_command` - Verify docker command

### Integration Tests

- Manual testing required (actually installs software)
- Test in clean VM/container
- Document manual test procedure

---

## File Structure

```
src/setup/
├── mod.rs          # Setup orchestration (modified)
├── ollama.rs       # Ollama health check (existing)
├── index.rs        # Manpage indexing (existing)
└── install.rs      # NEW: Installation logic
```

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ollama installer changes | Install fails | Pin to known-good URL, version check |
| Docker image unavailable | Docker install fails | Use official image, fallback to native |
| Permission issues | Install blocked | Clear sudo explanation, Docker alternative |
| Network issues | Download fails | Timeout, clear error message, manual fallback |

---

## Version Targeting

- **v0.2.0** - Epic 5 complete
- Update `Cargo.toml` version after implementation
- Update `CHANGELOG.md` with new features
