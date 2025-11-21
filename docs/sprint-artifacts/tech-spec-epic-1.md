# Epic Technical Specification: Foundation

Date: 2025-11-21
Author: Elmar Röser
Epic ID: 1
Status: Draft

---

## Overview

Epic 1 establishes the foundational infrastructure for ulm, a Rust CLI tool that provides AI-powered manpage assistance. This epic creates the project skeleton, module structure, CLI parsing, and error handling that all subsequent epics will build upon.

The foundation must support async operations (for Ollama API calls), provide type-safe CLI argument handling, and establish consistent error propagation patterns that align with the strict linting rules defined in the architecture.

## Objectives and Scope

**In Scope:**
- Initialize Cargo project with all core dependencies
- Create module directory structure per architecture design
- Implement CLI argument parsing with clap derive
- Set up error handling with anyhow
- Configure tracing for structured logging
- Apply all linting rules from architecture

**Out of Scope:**
- Actual functionality (setup, query, TUI) - handled in Epics 2-4
- Testing infrastructure beyond basic setup
- CI/CD pipeline setup (separate task)
- Documentation beyond code comments

## System Architecture Alignment

**Architecture References:**
- Project Structure (architecture.md:37-76)
- Decision Summary table (architecture.md:20-35)
- Development Standards (architecture.md:406-504)

**Key Constraints:**
- Rust 2021 edition
- Async runtime: tokio (multi-threaded)
- Error handling: anyhow with context chains
- CLI parsing: clap 4.x with derive macros
- Logging: tracing with tracing-subscriber
- No `unwrap()` or `expect()` allowed (lint deny)

## Detailed Design

### Services and Modules

| Module | Responsibility | Dependencies |
|--------|---------------|--------------|
| `main.rs` | Entry point, CLI dispatch, runtime setup | cli, lib |
| `lib.rs` | Public API re-exports for testing | all modules |
| `cli.rs` | Clap argument definitions | clap |
| `error.rs` | Custom error types (if needed) | anyhow |
| `db.rs` | LanceDB operations (stub) | - |
| `setup/mod.rs` | Setup orchestration (stub) | - |
| `query/mod.rs` | Query orchestration (stub) | - |
| `llm/mod.rs` | LLM orchestration (stub) | - |
| `tui/mod.rs` | TUI orchestration (stub) | - |
| `exec/mod.rs` | Command execution (stub) | - |

### Data Models and Contracts

**CLI Arguments (cli.rs):**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "ulm")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Natural language query
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize ulm with Ollama and manpage index
    Setup,
    /// Refresh the manpage index
    Update,
}
```

**Core Types (lib.rs):**

```rust
pub use anyhow::Result;

// Re-export modules for testing
pub mod cli;
pub mod db;
pub mod error;
pub mod exec;
pub mod llm;
pub mod query;
pub mod setup;
pub mod tui;
```

### APIs and Interfaces

No external APIs in Epic 1. Internal interfaces:

| Function | Signature | Purpose |
|----------|-----------|---------|
| `main` | `fn main() -> Result<()>` | Entry point with error handling |
| `run` | `async fn run(args: Args) -> Result<()>` | Async dispatch logic |

### Workflows and Sequencing

**Startup Flow:**

1. Parse CLI arguments (clap)
2. Initialize tracing subscriber
3. Create tokio runtime
4. Dispatch to command handler (stub returns Ok)
5. Handle errors and exit

```
main()
  → parse Args
  → init tracing
  → tokio::runtime
  → run(args).await
  → match command
  → Ok(()) or Err
```

## Non-Functional Requirements

### Performance

- **Startup time:** < 50ms to argument parsing
- **Binary size:** Reasonable for Rust CLI (target < 10MB release)
- No performance requirements for stubs

### Security

- No unsafe code (`unsafe_code = "forbid"`)
- No credential handling in Epic 1
- All dependencies from crates.io (vetted)

### Reliability/Availability

- Graceful error messages on invalid arguments
- Non-zero exit code on errors
- Proper signal handling (Ctrl-C)

### Observability

- tracing configured with subscriber
- Log levels: ERROR, WARN, INFO, DEBUG
- Enable via `RUST_LOG=ulm=debug`
- Structured logging ready for all modules

## Dependencies and Integrations

**Cargo.toml dependencies:**

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# CLI parsing
clap = { version = "4", features = ["derive"] }

# Error handling
anyhow = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# HTTP client (for later epics)
reqwest = { version = "0.11", features = ["json"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Vector database (stub in Epic 1)
lancedb = "0.4"  # Pin specific version

# Terminal UI (stub in Epic 1)
crossterm = "0.27"

# Clipboard (stub in Epic 1)
arboard = "3"

# Line editor (stub in Epic 1)
rustyline = "14"

# XDG paths (stub in Epic 1)
directories = "5"

# Progress bars
indicatif = "0.17"

# Cleanup on panic
scopeguard = "1"
```

**Dev dependencies:**

```toml
[dev-dependencies]
cargo-tarpaulin = "0.27"  # Coverage
```

## Acceptance Criteria (Authoritative)

1. **AC1:** `cargo new ulm --bin` creates valid Rust 2021 project
2. **AC2:** All dependencies in Cargo.toml compile without errors
3. **AC3:** All linting rules from architecture pass (`cargo clippy`)
4. **AC4:** Module structure matches architecture exactly (10 modules)
5. **AC5:** `ulm --help` displays usage information
6. **AC6:** `ulm --version` displays version from Cargo.toml
7. **AC7:** `ulm setup` recognized as valid subcommand (stub OK)
8. **AC8:** `ulm update` recognized as valid subcommand (stub OK)
9. **AC9:** `ulm "query text"` captures query string correctly
10. **AC10:** Invalid commands show helpful error messages
11. **AC11:** Errors print to stderr with non-zero exit code
12. **AC12:** `RUST_LOG=ulm=debug` enables debug logging
13. **AC13:** `cargo fmt` passes without changes
14. **AC14:** `cargo test` runs (even if no tests yet)
15. **AC15:** README.md exists with basic project description
16. **AC16:** LICENSE files exist (MIT and Apache-2.0)

## Traceability Mapping

| AC | Spec Section | Component | Test Approach |
|----|--------------|-----------|---------------|
| AC1-2 | Dependencies | Cargo.toml | `cargo build` |
| AC3 | Linting | All | `cargo clippy -- -D warnings` |
| AC4 | Modules | src/* | Directory structure check |
| AC5-10 | CLI | cli.rs | Integration tests with clap |
| AC11 | Errors | main.rs, error.rs | Test error propagation |
| AC12 | Observability | main.rs | Manual verification |
| AC13 | Code Style | All | `cargo fmt --check` |
| AC14 | Testing | tests/ | `cargo test` |
| AC15-16 | Documentation | Root | File existence check |

## Risks, Assumptions, Open Questions

**Risks:**
- **R1:** LanceDB version compatibility - Mitigation: Pin version 0.4.x
- **R2:** Strict linting may slow initial development - Mitigation: Address warnings incrementally

**Assumptions:**
- **A1:** Developer has Rust toolchain installed (rustup)
- **A2:** All crates.io dependencies are available
- **A3:** MSRV 1.75+ is acceptable

**Open Questions:**
- **Q1:** Should we add `cargo-deny` for dependency auditing? (Deferred to CI setup)

## Test Strategy Summary

**Test Levels:**

1. **Unit Tests:** None required for stubs in Epic 1
2. **Integration Tests:** CLI argument parsing
3. **Manual Tests:** Verify --help, --version, error messages

**Coverage Target:** 80% (measured after Epic 2+ add real logic)

**Test Files:**
- `tests/cli_test.rs` - CLI argument parsing tests

**Edge Cases:**
- Empty query string
- Very long query string
- Special characters in query
- Unknown subcommands
