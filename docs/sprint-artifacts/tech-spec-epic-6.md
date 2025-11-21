# Epic Technical Specification: Model Selection & Auto-Pull

Date: 2025-11-21
Author: Elmar Röser
Epic ID: 6
Status: Draft

---

## Overview

Epic 6 enables users to select their preferred LLM model during setup with automatic download and configuration. This eliminates the need for manual `ollama pull` commands and provides intelligent model recommendations based on system resources.

The feature extends the existing setup workflow (Epic 5) by adding model discovery, selection UI, streaming download with progress, and persistent configuration storage.

## Objectives and Scope

**In Scope:**
- Fetch and display available Ollama models
- Show RAM requirements and speed/quality ratings
- Detect installed models via Ollama API
- Stream model downloads with progress indication
- Save selected model to config file
- Use configured model for all future queries

**Out of Scope:**
- Custom model fine-tuning
- Multiple model profiles
- Remote Ollama server configuration
- Model performance benchmarking
- Automatic model updates

## System Architecture Alignment

**Components Referenced:**
- `setup/` module - Extended with `models.rs` for model management
- `llm/ollama.rs` - Existing client used for API calls
- New config module at `~/.config/ulm/config.toml`

**Architecture Constraints:**
- Async pattern for all Ollama API calls
- Error handling with anyhow `.context()`
- XDG-compliant paths via `directories` crate
- No external dependencies beyond Ollama

## Detailed Design

### Services and Modules

| Module | Responsibility | Inputs | Outputs |
|--------|---------------|--------|---------|
| `setup/models.rs` | Model discovery and selection | Ollama API | `ModelInfo`, `Vec<ModelInfo>` |
| `setup/config.rs` | Config file management | User selection | `config.toml` |
| `llm/ollama.rs` | API client (extended) | Model name | Pull progress stream |

### Data Models and Contracts

```rust
/// Information about an Ollama model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,           // e.g., "llama3.2:3b"
    pub ram_gb: f32,            // e.g., 4.0
    pub speed_rating: u8,       // 1-5 (5 = fastest)
    pub quality_rating: u8,     // 1-5 (5 = best)
    pub installed: bool,        // from Ollama /api/tags
}

/// Recommended models with metadata
pub const RECOMMENDED_MODELS: &[ModelInfo] = &[
    ModelInfo { name: "llama3.2:3b", ram_gb: 4.0, speed_rating: 5, quality_rating: 3, installed: false },
    ModelInfo { name: "mistral:7b", ram_gb: 6.0, speed_rating: 4, quality_rating: 4, installed: false },
    ModelInfo { name: "llama3.1:8b", ram_gb: 8.0, speed_rating: 3, quality_rating: 5, installed: false },
    ModelInfo { name: "phi3:mini", ram_gb: 3.0, speed_rating: 5, quality_rating: 2, installed: false },
];

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub model_name: String,
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

/// Pull progress update from Ollama
#[derive(Debug, Clone, Deserialize)]
pub struct PullProgress {
    pub status: String,
    pub digest: Option<String>,
    pub total: Option<u64>,
    pub completed: Option<u64>,
}
```

### APIs and Interfaces

**Ollama API Endpoints Used:**

| Endpoint | Method | Purpose | Response |
|----------|--------|---------|----------|
| `/api/tags` | GET | List installed models | `{ "models": [...] }` |
| `/api/pull` | POST | Download model | Stream of JSON lines |

**Pull Request:**
```json
POST /api/pull
{
  "name": "llama3.2:3b",
  "stream": true
}
```

**Pull Response (streaming JSON lines):**
```json
{"status": "pulling manifest"}
{"status": "downloading", "digest": "sha256:...", "total": 2000000000, "completed": 500000000}
{"status": "verifying sha256 digest"}
{"status": "success"}
```

**New Public Functions:**

```rust
// setup/models.rs
pub async fn get_available_models(client: &OllamaClient) -> Result<Vec<ModelInfo>>;
pub fn display_model_selection(models: &[ModelInfo], system_ram_gb: f32) -> Result<usize>;
pub async fn pull_model(client: &OllamaClient, model_name: &str) -> Result<()>;

// setup/config.rs
pub fn load_config() -> Result<Config>;
pub fn save_config(config: &Config) -> Result<()>;
pub fn get_config_path() -> PathBuf;
```

### Workflows and Sequencing

**Setup Flow with Model Selection:**

```
1. Detect Ollama (existing)
2. Install Ollama if needed (existing)
3. [NEW] Fetch installed models
4. [NEW] Display model selection UI
5. [NEW] Pull selected model if not installed
6. [NEW] Save config with model choice
7. Scan manpages (existing)
8. Generate embeddings (existing)
9. Store in LanceDB (existing)
```

**Model Selection Sequence:**

```
User                    ulm                     Ollama
  |                      |                        |
  |---ulm setup--------->|                        |
  |                      |--GET /api/tags-------->|
  |                      |<--installed models-----|
  |<--display table------|                        |
  |---select model------>|                        |
  |                      |--POST /api/pull------->|
  |<--progress updates---|<--stream---------------|
  |                      |--write config.toml     |
  |<--success------------|                        |
```

## Non-Functional Requirements

### Performance

| Metric | Target | Source |
|--------|--------|--------|
| Model list fetch | < 1s | Ollama API latency |
| Model download | Network-limited | 30 min timeout |
| Config read/write | < 50ms | Local file I/O |
| RAM detection | < 100ms | sysinfo crate |

### Security

- No credentials stored in config
- Config file has user-only permissions (0600)
- Model downloads use Ollama's verified sources
- No external network calls beyond Ollama

### Reliability/Availability

- Graceful handling of Ollama connection failures
- Resume support for interrupted downloads (Ollama built-in)
- Default model fallback if config missing
- Offline operation after initial model download

### Observability

```rust
// Logging signals
info!("Fetching available models from Ollama");
info!(model = %name, "Selected model for download");
debug!(completed = completed, total = total, "Pull progress");
warn!("Model download slow, still in progress");
error!("Failed to pull model: {}", err);
```

## Dependencies and Integrations

### Rust Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `toml` | latest | Config file serialization |
| `sysinfo` | latest | System RAM detection |
| `directories` | latest | XDG config paths (already used) |
| `indicatif` | latest | Progress bar (already used) |
| `reqwest` | latest | HTTP streaming (already used) |
| `tokio` | latest | Async runtime (already used) |

**Cargo.toml additions:**
```toml
[dependencies]
toml = "0.8"
sysinfo = "0.31"
```

### Integration Points

- **Ollama API**: `/api/tags`, `/api/pull`
- **File System**: `~/.config/ulm/config.toml`
- **Existing Code**: `OllamaClient` in `llm/ollama.rs`

## Acceptance Criteria (Authoritative)

1. **AC1**: Setup displays list of recommended LLM models with RAM requirements (FR41)
2. **AC2**: User can select preferred model from list during setup (FR42)
3. **AC3**: Setup pulls selected model with progress indicator (FR43)
4. **AC4**: Selected model is saved as default for future queries (FR44)
5. **AC5**: Already-installed models are marked in selection UI
6. **AC6**: System RAM is detected to highlight recommended model
7. **AC7**: Network errors during pull show retry option
8. **AC8**: Config file uses XDG-compliant path
9. **AC9**: `cargo build` succeeds without errors
10. **AC10**: `cargo clippy -- -D warnings` passes

## Traceability Mapping

| AC | Spec Section | Component | Test Idea |
|----|-------------|-----------|-----------|
| AC1 | APIs/Interfaces | `setup/models.rs` | Mock Ollama, verify table output |
| AC2 | Workflows | `display_model_selection()` | Test stdin selection |
| AC3 | Data Models | `pull_model()` | Mock streaming response |
| AC4 | Data Models | `setup/config.rs` | Verify file written |
| AC5 | APIs/Interfaces | `get_available_models()` | Test installed flag |
| AC6 | Dependencies | `sysinfo` integration | Mock RAM values |
| AC7 | NFR/Reliability | `pull_model()` | Simulate network error |
| AC8 | Dependencies | `get_config_path()` | Check XDG path |
| AC9 | - | All | CI build |
| AC10 | - | All | CI clippy |

## Risks, Assumptions, Open Questions

### Risks

- **R1**: Large model downloads may timeout on slow connections
  - *Mitigation*: 30-minute timeout, clear progress indication
- **R2**: sysinfo crate may not detect RAM on all systems
  - *Mitigation*: Fallback to showing all models without recommendation

### Assumptions

- **A1**: Ollama API remains stable for /api/tags and /api/pull
- **A2**: Users have at least 4GB RAM for smallest recommended model
- **A3**: Config directory is writable by user

### Open Questions

- **Q1**: Should we support custom Ollama URLs in config? (Deferred to future)
- **Q2**: What happens if user's selected model is deleted? (Use default)

## Test Strategy Summary

### Test Levels

| Level | Framework | Coverage |
|-------|-----------|----------|
| Unit | `#[test]`, `#[tokio::test]` | Core logic |
| Integration | `tests/` directory | Ollama interaction |

### Test Plan

1. **Unit Tests** (`setup/models.rs`)
   - Parse model info from API response
   - Recommend model based on RAM
   - Parse pull progress stream

2. **Unit Tests** (`setup/config.rs`)
   - Serialize/deserialize config
   - Handle missing config file
   - Create config directory

3. **Integration Tests**
   - Full model selection flow (with mock Ollama)
   - Config persistence across runs

### Coverage Target

- Minimum 80% code coverage for new modules
- All acceptance criteria have corresponding tests
