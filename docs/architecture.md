# Architecture - ulm

## Executive Summary

ulm is a Rust CLI application using async architecture with tokio for Ollama API communication and LanceDB for embedded vector search. The architecture prioritizes local-first operation, fast response times, and a clean separation between query processing, LLM interaction, and terminal UI.

## Project Initialization

First implementation step:

```bash
cargo new ulm --bin
cd ulm
```

This establishes Rust 2021 edition with standard Cargo project structure.

## Decision Summary

| Category | Decision | Version | Affects FRs | Rationale |
| -------- | -------- | ------- | ----------- | --------- |
| Async Runtime | tokio | latest | All async FRs | De-facto standard, best reqwest integration |
| HTTP Client | reqwest | latest | FR2-5, FR22-26 | Async, feature-rich, tokio-native |
| Vector Database | lancedb | latest | FR7-12, FR14-16 | Embedded, Rust-native, no server needed |
| CLI Parsing | clap (derive) | 4.x | FR1, FR11, FR13 | Type-safe, derive macros, excellent UX |
| TUI/Events | crossterm | latest | FR27-34 | Low-level control, no widget overhead |
| Error Handling | anyhow | latest | FR38-40 | Simple propagation, good context support |
| Serialization | serde + serde_json | latest | FR22-26 | Standard for Rust JSON handling |
| Clipboard | arboard | latest | FR31, FR34 | Cross-platform, simple API |
| Line Editor | rustyline | latest | FR32, FR37 | Readline-like editing experience |
| Paths | directories | latest | FR6, FR10 | XDG-compliant config/data paths |
| Terminal Colors | crossterm (styles) | - | FR27-29 | Already a dependency |
| Manpage Parsing | shell-out + parse | - | FR7-8, FR17 | Reliable, uses system `man` |
| Logging | tracing | latest | All | Structured, async-friendly |

## Project Structure

```
ulm/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── main.rs              # Entry point, CLI setup
│   ├── lib.rs               # Public API for testing
│   ├── cli.rs               # Clap argument definitions
│   ├── setup/
│   │   ├── mod.rs           # Setup orchestration
│   │   ├── ollama.rs        # Ollama detection/installation
│   │   └── index.rs         # Manpage scanning & embedding
│   ├── query/
│   │   ├── mod.rs           # Query orchestration
│   │   ├── search.rs        # Vector search logic
│   │   └── context.rs       # Sherlock mode (dir scanning)
│   ├── llm/
│   │   ├── mod.rs           # LLM orchestration
│   │   ├── ollama.rs        # Ollama API client
│   │   ├── prompt.rs        # Prompt building
│   │   └── response.rs      # Response parsing
│   ├── tui/
│   │   ├── mod.rs           # TUI orchestration
│   │   ├── render.rs        # Display logic
│   │   └── input.rs         # Event loop & hotkeys
│   ├── exec/
│   │   ├── mod.rs           # Command execution
│   │   ├── shell.rs         # Shell-out logic
│   │   └── clipboard.rs     # Clipboard integration
│   ├── db.rs                # LanceDB operations
│   └── error.rs             # Custom error types
├── tests/
│   ├── setup_test.rs
│   ├── query_test.rs
│   └── integration_test.rs
└── assets/
    └── prompts/             # System prompts (optional)
```

## FR Category to Architecture Mapping

| FR Category | FRs | Module(s) | Key Components |
| ----------- | --- | --------- | -------------- |
| Setup & Initialization | FR1-6 | `setup/` | OllamaChecker, ShellAliasInstaller |
| Knowledge Base | FR7-12 | `setup/index.rs`, `db.rs` | ManpageScanner, EmbeddingGenerator, LanceDB |
| Query Processing | FR13-17 | `query/` | VectorSearch, ManpageLoader |
| Context Awareness | FR18-21 | `query/context.rs` | ProjectDetector, ContextBuilder |
| Suggestion Generation | FR22-26 | `llm/` | OllamaClient, PromptBuilder, ResponseParser |
| Interactive TUI | FR27-34 | `tui/` | Renderer, EventLoop, HotkeyHandler |
| Command Execution | FR35-37 | `exec/` | ShellExecutor, ClipboardManager, LineEditor |
| Error Handling | FR38-40 | `error.rs` | Throughout all modules |

## Technology Stack Details

### Core Technologies

**Runtime & Async:**
- Rust 2021 edition
- tokio (multi-threaded runtime)
- async/await throughout for I/O operations

**External Services:**
- Ollama API (localhost:11434)
  - `/api/embeddings` - vector generation
  - `/api/generate` - LLM inference

**Data Storage:**
- LanceDB (embedded) at `~/.local/share/ulm/index.lance`
- No external database server required

### Integration Points

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   User      │────▶│    ulm      │────▶│   Ollama    │
│  Terminal   │◀────│   (Rust)    │◀────│   (LLM)     │
└─────────────┘     └──────┬──────┘     └─────────────┘
                          │
                    ┌─────▼─────┐
                    │  LanceDB  │
                    │ (embedded)│
                    └───────────┘
```

**System Integration:**
- `man -P cat <tool>` - manpage retrieval
- Shell execution via `std::process::Command`
- Clipboard via arboard (X11/Wayland/macOS)

## Implementation Patterns

These patterns ensure consistent implementation across all AI agents:

### Async Pattern

```rust
// All Ollama/LanceDB calls are async
pub async fn query_ollama(&self, prompt: &str) -> Result<OllamaResponse> {
    self.client
        .post(&format!("{}/api/generate", self.base_url))
        .json(&request)
        .send()
        .await?
        .json()
        .await
        .context("Failed to parse Ollama response")
}

// TUI Event Loop is sync (crossterm is sync)
pub fn run_tui(suggestions: Vec<CommandSuggestion>) -> Result<UserAction> {
    // ...
}
```

### Result Propagation

```rust
// Use ? operator with context
fn load_manpage(tool: &str) -> Result<String> {
    let output = Command::new("man")
        .args(["-P", "cat", tool])
        .output()
        .context("Failed to execute man command")?;

    String::from_utf8(output.stdout)
        .context("Manpage contains invalid UTF-8")
}
```

### Struct Definitions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub title: String,
    pub explanation: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Moderate,
    Destructive,
}

#[derive(Debug, Clone)]
pub enum UserAction {
    Execute(String),
    Copy(String),
    Edit(String),
    Abort,
}
```

## Consistency Rules

### Naming Conventions

**Rust Standard:**
- Modules: `snake_case` (e.g., `query_context.rs`)
- Types/Structs: `PascalCase` (e.g., `CommandSuggestion`)
- Functions: `snake_case` (e.g., `find_manpages`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_MODEL`)
- Traits: `PascalCase` (e.g., `DocProvider`)

**Project-Specific:**
- CLI subcommands: `lowercase` (setup, update)
- Config keys in files: `snake_case`

### Code Organization

- One struct per file when > 100 lines
- Related functions grouped in impl blocks
- Traits in separate files from implementations
- Tests at bottom of file in `#[cfg(test)]` module

### Error Handling

```rust
// User-facing errors: Clear, actionable, English
"Ollama not reachable at localhost:11434. Start with: ollama serve"
"No manpages found. Ensure man-db is installed."
"Model 'llama3' not found. Pull with: ollama pull llama3"

// Developer errors: With context chain
.context("Failed to connect to Ollama API")?
.context("Failed to parse embedding response")?
.context("Failed to write to LanceDB")?
```

### Logging Strategy

```rust
use tracing::{info, debug, warn, error};

// Levels:
// ERROR - Unrecoverable failures
// WARN  - Recoverable issues, fallbacks
// INFO  - User-relevant progress (setup, indexing)
// DEBUG - Developer diagnostics

// Usage:
info!("Indexing {} manpages", count);
debug!(query = %query, "Performing vector search");
warn!("Ollama slow to respond, retrying");
error!("Failed to connect to Ollama: {}", err);

// Enable via: RUST_LOG=ulm=debug
```

## Data Architecture

### LanceDB Schema

```rust
// Stored in ~/.local/share/ulm/index.lance
struct ManpageEntry {
    tool_name: String,      // e.g., "ffmpeg"
    section: String,        // e.g., "1"
    description: String,    // NAME + DESCRIPTION text
    vector: Vec<f32>,       // Embedding from Ollama
}
```

### Runtime Data Structures

```rust
// Query context for LLM
struct QueryContext {
    query: String,
    matched_tool: String,
    manpage_content: String,
    directory_context: DirectoryContext,
}

// Directory scanning result
struct DirectoryContext {
    project_type: Option<ProjectType>,
    marker_files: Vec<String>,
}

enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    // ...
}
```

## API Contracts

### Ollama API

**Embeddings Request:**
```json
POST /api/embeddings
{
  "model": "llama3",
  "prompt": "ffmpeg - multimedia framework for video/audio"
}
```

**Generate Request:**
```json
POST /api/generate
{
  "model": "llama3",
  "prompt": "...",
  "stream": false,
  "format": "json"
}
```

**Expected LLM Response:**
```json
{
  "suggestions": [
    {
      "command": "ffmpeg -i input.mp4 -vf scale=1280:720 output.mp4",
      "title": "Standard resize",
      "explanation": "Uses -vf scale filter...",
      "risk_level": "Safe"
    }
  ]
}
```

## Security Architecture

- **No network egress**: All communication is localhost (Ollama)
- **No telemetry**: Zero tracking or analytics
- **User permissions**: Executed commands run as current user
- **No credential storage**: No passwords, API keys, or tokens
- **Clipboard isolation**: Only written on explicit user action (K key)

## Performance Considerations

| Requirement | Target | Strategy |
| ----------- | ------ | -------- |
| Setup indexing | < 5 min / 5000 pages | Batch embedding requests, progress bar |
| Vector search | < 100ms | LanceDB's built-in ANN search |
| LLM latency | < 5s total | Stream-ready, but batch for JSON parsing |
| TUI response | < 50ms | No blocking in event loop |
| DB size | < 500MB | 768-dim vectors, compressed storage |

**Optimizations:**
- Batch manpage processing during setup
- Cache frequently used manpages in memory
- Lazy-load full manpage only after vector match

## Deployment Architecture

**Distribution:**
- `cargo install ulm` (crates.io)
- Pre-built binaries (GitHub Releases)
- AUR package (Arch Linux)

**Dependencies:**
- Runtime: Ollama (user-managed)
- System: `man` command available

**Data Locations:**
- Config: `~/.config/ulm/` (future)
- Data: `~/.local/share/ulm/index.lance`
- Cache: `~/.cache/ulm/` (future)

## Development Environment

### Prerequisites

- Rust toolchain (rustup)
- Ollama installed and running
- man-db (for manpage access)

### Setup Commands

```bash
# Clone and build
git clone https://github.com/eroeser/ulm.git
cd ulm
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=ulm=debug cargo run -- "find large files"

# Install locally
cargo install --path .
```

### Development Workflow

```bash
# Check before commit
cargo fmt
cargo clippy
cargo test

# Build release
cargo build --release
```

## Development Standards

### Linting Configuration

Add to `Cargo.toml`:

```toml
[lints.rust]
unsafe_code = "forbid"
future_incompatible = { level = "deny", priority = 1 }
unused_crate_dependencies = "warn"
dead_code = "warn"
unused_imports = "warn"
explicit_outlives_requirements = "warn"
macro_use_extern_crate = "warn"
missing_docs = "warn"
missing_debug_implementations = "warn"

[lints.clippy]
cargo = "warn"
complexity = "warn"
correctness = "deny"
pedantic = "warn"
perf = "warn"
style = "warn"
expect_used = { level = "deny", priority = 1 }
unwrap_used = { level = "deny", priority = 1 }
panic = { level = "warn", priority = 1 }
large_stack_arrays = { level = "warn", priority = 1 }
enum_glob_use = { level = "deny", priority = 1 }
wildcard_imports = { level = "deny", priority = 1 }
multiple_crate_versions = { level = "allow", priority = 1 }
dbg_macro = { level = "deny", priority = 1 }
todo = { level = "deny", priority = 1 }
redundant_pub_crate = { level = "warn", priority = 1 }
unnecessary_wraps = { level = "deny", priority = 1 }
too_many_lines = { level = "warn", priority = 1 }
box_collection = { level = "deny", priority = 1 }
missing_const_for_fn = { level = "deny", priority = 1 }
missing_docs_in_private_items = { level = "warn", priority = 1 }
missing_errors_doc = { level = "warn", priority = 1 }
missing_panics_doc = { level = "warn", priority = 1 }
use_self = { level = "warn", priority = 1 }
```

### Commit Convention

Follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat:` - New feature (MINOR version)
- `fix:` - Bug fix (PATCH version)
- `docs:` - Documentation only
- `refactor:` - Code refactoring
- `test:` - Adding/updating tests
- `chore:` - Maintenance tasks
- `feat!:` or `BREAKING CHANGE:` - Breaking change (MAJOR version)

### Code Style

Follow [Rust Compiler Development Conventions](https://rustc-dev-guide.rust-lang.org/conventions.html).

### Testing Requirements

- **Minimum Code Coverage:** 80%
- **Tool:** `cargo-tarpaulin` or `cargo-llvm-cov`
- All stories must include adequate test coverage

### CI/CD Pipeline

GitHub Actions workflow:
- Run `cargo fmt --check`
- Run `cargo clippy -- -D warnings`
- Run `cargo test`
- Check code coverage ≥ 80%
- Automatic releases on version tags

### Pre-commit Hooks

Required checks before each commit:
- `cargo fmt` - Code formatting
- `cargo clippy` - Lint checks

### Git Branch & Commit Strategy

**Branching:**
- `main` - Stable releases only
- `epic-{num}-{name}` - One branch per epic
  - `epic-1-foundation`
  - `epic-2-setup-knowledge-base`
  - `epic-3-query-intelligence`
  - `epic-4-interactive-experience`

**Commits:**
- One commit per story (squash if needed)
- Format: `feat(epic-{num}): story {num}.{num} - {title}`
- Examples:
  - `feat(epic-1): story 1.1 - project initialization`
  - `feat(epic-2): story 2.3 - model verification and pull`
  - `fix(epic-3): story 3.1 - vector search`

**Workflow:**
1. Create epic branch from main: `git checkout -b epic-1-foundation`
2. Implement stories, one commit each
3. PR epic branch to main after all stories complete
4. Squash merge or rebase to keep history clean

### Changelog

Automatically generated from Conventional Commits using `git-cliff`.

### Version Requirements

- **MSRV (Minimum Supported Rust Version):** 1.75+
- **License:** MIT OR Apache-2.0 (Dual License)

## Architecture Decision Records (ADRs)

### ADR-001: Embedded Vector Database

**Decision:** Use LanceDB as embedded vector store instead of external service.

**Context:** Need to store and search ~5000 manpage embeddings locally.

**Rationale:**
- No server to manage
- Fast cold-start
- Single binary distribution
- Privacy (no external calls)

**Consequences:** Limited to single-machine use, but this matches the CLI tool use case.

### ADR-002: Shell-out for Manpages

**Decision:** Use `man -P cat <tool>` instead of parsing manpage files directly.

**Context:** Need to read manpage content for LLM context.

**Rationale:**
- Handles all manpage formats (groff, mdoc)
- Respects user's MANPATH
- No complex parsing logic
- Works on all Unix systems

**Consequences:** Requires `man` command available, slight performance overhead.

### ADR-003: Sync TUI with Async Backend

**Decision:** TUI event loop is synchronous, spawns async tasks for Ollama.

**Context:** crossterm is sync, but Ollama calls need to be async.

**Rationale:**
- crossterm event loop is inherently blocking
- Async backend for network I/O
- Clean separation of concerns

**Consequences:** Need tokio runtime in main, block_on for TUI integration.

### ADR-004: English Error Messages

**Decision:** All user-facing error messages in English.

**Context:** Open-source project targeting international developer community.

**Rationale:**
- Consistent with Rust ecosystem
- Easier to search/debug
- Community contributions easier

**Consequences:** Non-English speakers need to understand error messages.

---

_Generated by BMAD Decision Architecture Workflow v1.0_
_Date: 2025-11-20_
_For: Elmar Röser_
