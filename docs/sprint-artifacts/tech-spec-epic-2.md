# Epic Technical Specification: Setup & Knowledge Base

Date: 2025-11-21
Author: Elmar Röser
Epic ID: 2
Status: Draft

---

## Overview

Epic 2 implements the complete setup and knowledge base creation flow for ulm. It enables users to initialize the tool with a single `ulm setup` command that detects/configures Ollama, verifies/pulls an LLM model, scans all system manpages, generates vector embeddings, and stores them in a local LanceDB database. After setup completion, the system is ready to answer natural language queries with a searchable index of 1000+ installed tools.

This epic transforms ulm from a CLI skeleton (Epic 1) into a fully functional knowledge base system. It covers FR1-12 from the PRD, focusing on local-first operation with no external dependencies beyond Ollama.

## Objectives and Scope

### In Scope

- Ollama API client implementation (reqwest async)
- Ollama health detection and user guidance
- Model verification and automatic pull
- Manpage directory scanning (Linux/macOS paths)
- NAME/DESCRIPTION extraction via `man -P cat`
- Vector embedding generation via Ollama API
- LanceDB storage at ~/.local/share/ulm/index.lance
- Setup orchestration with progress reporting
- Update command for index refresh
- Optional shell alias installation

### Out of Scope

- TUI interface (Epic 4)
- Query processing (Epic 3)
- Windows PowerShell support (V2)
- Custom model configuration (future)
- Incremental index updates (future)

## System Architecture Alignment

This epic implements components from the architecture document:

**Modules:**
- `llm/ollama.rs` - Ollama API client (reqwest, async)
- `setup/ollama.rs` - Detection and health check
- `setup/index.rs` - Manpage scanning and embedding
- `db.rs` - LanceDB operations

**Key Decisions:**
- ADR-001: Embedded LanceDB (no external server)
- ADR-002: Shell-out for manpages (`man -P cat`)

**Integration Points:**
- Ollama API at localhost:11434
- System `man` command
- LanceDB embedded database

---

## Detailed Design

### Services and Modules

| Module | Responsibility | Inputs | Outputs |
|--------|---------------|--------|---------|
| `llm/ollama.rs` | HTTP client for Ollama API | Prompts, model name | Embeddings, responses |
| `setup/ollama.rs` | Ollama detection & model management | None | Health status, model list |
| `setup/index.rs` | Manpage scanning & processing | Directory paths | ManpageEntry list |
| `db.rs` | LanceDB storage operations | ManpageEntry list | Persisted database |
| `setup/mod.rs` | Setup orchestration | CLI args | Completion status |

### Data Models and Contracts

```rust
// llm/ollama.rs
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
}

// db.rs
#[derive(Debug, Clone)]
pub struct ManpageEntry {
    pub tool_name: String,
    pub section: String,
    pub description: String,
    pub vector: Vec<f32>,
}

// setup/index.rs
pub struct ManpageScanner {
    paths: Vec<PathBuf>,
}

pub struct ManpageContent {
    pub tool_name: String,
    pub section: String,
    pub description: String,
}
```

### APIs and Interfaces

**Ollama API Endpoints:**

```
POST /api/tags
Response: { "models": [{ "name": "llama3", "size": 4000000000 }] }

POST /api/embeddings
Request: { "model": "llama3", "prompt": "ffmpeg - video converter" }
Response: { "embedding": [0.1, 0.2, ...] }

POST /api/pull
Request: { "name": "llama3", "stream": true }
Response: Streaming progress updates
```

**Internal Module APIs:**

```rust
// OllamaClient
impl OllamaClient {
    pub fn new(base_url: &str) -> Self;
    pub async fn health_check(&self) -> Result<bool>;
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    pub async fn generate_embedding(&self, model: &str, text: &str) -> Result<Vec<f32>>;
    pub async fn pull_model(&self, name: &str) -> Result<()>;
}

// ManpageScanner
impl ManpageScanner {
    pub fn new() -> Self;
    pub fn scan_directories(&self) -> Result<Vec<PathBuf>>;
    pub fn extract_content(&self, path: &Path) -> Result<ManpageContent>;
}

// Database
pub async fn create_index(entries: Vec<ManpageEntry>) -> Result<()>;
pub async fn index_exists() -> Result<bool>;
```

### Workflows and Sequencing

**Setup Flow:**

```
1. User runs `ulm setup`
   │
2. Check Ollama connection
   ├── Success → Continue
   └── Failure → Display install instructions, exit
   │
3. Verify model availability
   ├── Model found → Continue
   └── No model → Prompt to pull → Pull with progress
   │
4. Scan manpage directories
   ├── /usr/share/man
   ├── /usr/local/share/man
   ├── $MANPATH entries
   └── Report: "Found 4,523 manpages"
   │
5. Extract descriptions (parallel)
   └── For each: `man -P cat <tool>` → Parse NAME/DESCRIPTION
   │
6. Generate embeddings (batched)
   └── Batch of 10 → Ollama API → Progress bar
   │
7. Store in LanceDB
   └── ~/.local/share/ulm/index.lance
   │
8. Optional: Install shell alias
   └── Detect shell → Append to rc file
   │
9. Report completion
   └── "✓ Indexed 4,523 manpages in 3m 42s"
```

---

## Non-Functional Requirements

### Performance

| Metric | Target | Strategy |
|--------|--------|----------|
| Full setup time | < 5 minutes for 5000 manpages | Batch embeddings, parallel extraction |
| Embedding request | < 500ms per item | Ollama local inference |
| Manpage extraction | < 50ms per item | Shell-out to `man -P cat` |
| Database write | < 60 seconds total | Batch insert to LanceDB |
| Database size | < 500MB | 768-dim vectors, compressed |

### Security

- **No external network**: All traffic to localhost:11434 only
- **No credentials**: No API keys or tokens stored
- **User permissions**: All files created with user ownership
- **Data location**: XDG-compliant paths only

### Reliability/Availability

- **Offline operation**: Works without internet after Ollama setup
- **Graceful degradation**: Skip failed manpages with warning
- **Idempotent setup**: Can re-run safely (overwrites index)
- **Retry logic**: 3 attempts with backoff for Ollama calls

### Observability

- **Logging**: tracing crate with levels (INFO for progress, DEBUG for details)
- **Progress reporting**: Count/total for each phase
- **Timing**: Log duration for each major step
- **Debug mode**: RUST_LOG=ulm=debug for verbose output

---

## Dependencies and Integrations

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
# Already in Epic 1
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }

# New for Epic 2
lancedb = "0.22"
arrow-array = "54"
directories = "5"
indicatif = "0.17"  # Progress bars
```

### System Dependencies

- **Ollama**: Running at localhost:11434
- **man-db**: System `man` command available
- **Disk space**: ~500MB for full index

### Integration Points

| Service | Protocol | Purpose |
|---------|----------|---------|
| Ollama | HTTP REST | Embeddings, model management |
| man command | Shell | Manpage content extraction |
| LanceDB | Embedded | Vector storage and search |
| Filesystem | XDG paths | Data persistence |

---

## Acceptance Criteria (Authoritative)

### Story 2.1: Ollama API Client
1. AC2.1.1: OllamaClient can POST to /api/embeddings
2. AC2.1.2: OllamaClient can POST to /api/generate
3. AC2.1.3: Requests use serde JSON serialization
4. AC2.1.4: Responses deserialize to typed structs
5. AC2.1.5: Connection errors return descriptive messages

### Story 2.2: Ollama Detection & Health Check
1. AC2.2.1: Health check pings /api/tags endpoint
2. AC2.2.2: Success displays "✓ Ollama detected"
3. AC2.2.3: Failure displays install instructions

### Story 2.3: Model Verification & Pull
1. AC2.3.1: Query /api/tags for installed models
2. AC2.3.2: Display available model when found
3. AC2.3.3: Prompt user to pull if no model
4. AC2.3.4: Pull displays progress
5. AC2.3.5: Confirm when pull complete

### Story 2.4: Manpage Directory Scanner
1. AC2.4.1: Scan /usr/share/man, /usr/local/share/man
2. AC2.4.2: Include $MANPATH directories
3. AC2.4.3: Find man1 and man8 sections
4. AC2.4.4: Handle missing directories gracefully
5. AC2.4.5: Complete scan in < 5 seconds

### Story 2.5: Manpage Content Extraction
1. AC2.5.1: Run `man -P cat <tool>` for each page
2. AC2.5.2: Parse NAME section
3. AC2.5.3: Parse DESCRIPTION (first paragraph)
4. AC2.5.4: Handle malformed manpages gracefully
5. AC2.5.5: Validate UTF-8 output

### Story 2.6: Embedding Generation
1. AC2.6.1: Call Ollama /api/embeddings for each description
2. AC2.6.2: Receive vector (768+ dimensions)
3. AC2.6.3: Batch requests for efficiency
4. AC2.6.4: Display progress indicator
5. AC2.6.5: Retry failed requests 3 times

### Story 2.7: LanceDB Storage
1. AC2.7.1: Create database at ~/.local/share/ulm/index.lance
2. AC2.7.2: Schema: tool_name, section, description, vector
3. AC2.7.3: Use directories crate for XDG paths
4. AC2.7.4: Database size < 500MB for 5000 entries
5. AC2.7.5: Overwrite existing index on re-run

### Story 2.8: Setup Orchestration
1. AC2.8.1: `ulm setup` runs all steps in order
2. AC2.8.2: Display progress for each step
3. AC2.8.3: Report final count: "Indexed N manpages"
4. AC2.8.4: `ulm update` refreshes index
5. AC2.8.5: Optional shell alias installation
6. AC2.8.6: Total time < 5 minutes for 5000 pages

---

## Traceability Mapping

| AC | Spec Section | Component | Test Idea |
|----|-------------|-----------|-----------|
| AC2.1.1 | APIs and Interfaces | OllamaClient | Mock server test for embeddings endpoint |
| AC2.1.2 | APIs and Interfaces | OllamaClient | Mock server test for generate endpoint |
| AC2.2.1 | Workflows | setup/ollama.rs | Test with mock /api/tags response |
| AC2.3.3 | Workflows | setup/ollama.rs | Test pull prompt UI |
| AC2.4.1 | Workflows | ManpageScanner | Test with temp directories |
| AC2.5.1 | Workflows | ManpageScanner | Test with mock man command |
| AC2.6.3 | Performance | setup/index.rs | Verify batching behavior |
| AC2.7.1 | Data Models | db.rs | Test database creation path |
| AC2.8.1 | Workflows | setup/mod.rs | Integration test full flow |
| AC2.8.6 | Performance | setup/mod.rs | Benchmark with 1000+ pages |

---

## Risks, Assumptions, Open Questions

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Ollama not installed | Setup fails | Clear install instructions, Docker option |
| Model pull takes too long | User abandons | Show progress, allow cancel |
| Malformed manpages | Partial index | Skip with warning, don't fail setup |
| LanceDB version incompatibility | Build fails | Pin version in Cargo.toml |

### Assumptions

- User has internet access for initial model pull
- System has `man` command available
- User has write access to ~/.local/share
- Ollama supports the embedding API
- 768-dimensional embeddings are sufficient

### Open Questions

- **Q1:** Should we support custom Ollama port? → A: Not for MVP, use default 11434
- **Q2:** Which model to default to? → A: llama3 (good balance)
- **Q3:** Should alias installation be automatic? → A: No, prompt user

---

## Test Strategy Summary

### Test Levels

| Level | Scope | Tools |
|-------|-------|-------|
| Unit | OllamaClient, ManpageScanner | cargo test, mockall |
| Integration | Full setup flow | cargo test --test |
| E2E | Real Ollama | Manual testing |

### Test Coverage Targets

- **Unit tests**: 80% coverage for llm/, setup/, db.rs
- **Integration tests**: Full setup flow with mocked Ollama
- **Edge cases**: Empty directories, malformed manpages, network errors

### Key Test Scenarios

1. **Happy path**: Full setup with working Ollama
2. **No Ollama**: Clear error message and instructions
3. **No model**: Pull prompt and progress
4. **Empty manpath**: Handle gracefully
5. **Invalid manpage**: Skip and continue
6. **Network timeout**: Retry logic
7. **Re-run setup**: Overwrites existing index

### Test Dependencies

```toml
[dev-dependencies]
mockall = "0.13"
tempfile = "3"
wiremock = "0.6"
```
