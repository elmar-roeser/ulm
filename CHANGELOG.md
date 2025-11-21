# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Epic 5: Auto-Installation (Ollama/Docker auto-install during setup)

## [0.1.0] - 2025-11-21

### Added

#### Setup & Knowledge Base (Epic 2)
- `ulm setup` command for one-time initialization
- Ollama API client with health check and model verification
- Automatic model pull if not available
- Manpage directory scanner (supports /usr/share/man, macOS paths)
- Manpage content extraction (NAME, DESCRIPTION sections)
- Embedding generation via Ollama API
- LanceDB vector storage at `~/.local/share/ulm/index.lance`
- `ulm update` command to refresh index
- Progress reporting during indexing

#### Query & Intelligence (Epic 3)
- Natural language query via `ulm "question"`
- Vector embedding for semantic search
- LanceDB nearest neighbor search
- Manpage content loading for matched tools
- Directory context scanning ("Sherlock Mode")
- Project type detection (Rust, Node, Python, Go, Git)
- Context-aware prompt building
- Ollama LLM integration for suggestion generation
- JSON response parsing with 1-3 command suggestions
- Risk level classification (Safe, Moderate, Destructive)

#### Interactive Experience (Epic 4)
- Terminal UI with ratatui
- Suggestions list with navigation (↑/↓, j/k)
- Explanation panel for selected suggestion
- Risk level color coding (green/yellow/red)
- Keyboard shortcuts footer
- Execute command (Enter/A)
- Copy to clipboard (K)
- Edit before execute (B) with rustyline
- Abort (Esc/q)
- Status message feedback ("Copied!")
- Error display with actionable guidance
- Panic hook for terminal cleanup
- Auto-execute single suggestion

#### Foundation (Epic 1)
- Rust 2021 project structure
- Modular architecture (setup, query, llm, tui, exec, db)
- CLI argument parsing with clap
- Comprehensive error handling with anyhow
- Structured logging with tracing
- Strict clippy linting

### Technical Details

- **95 tests** across 14 modules
- **40 functional requirements** from PRD
- **95% test coverage** of requirements
- Supports Linux and macOS
- XDG Base Directory compliance
- Cross-platform clipboard (X11/Wayland/macOS)

### Dependencies

- tokio 1.x (async runtime)
- clap 4.x (CLI parsing)
- reqwest 0.12 (HTTP client)
- lancedb 0.17 (vector database)
- ratatui 0.29 (TUI)
- crossterm 0.28 (terminal control)
- arboard 3 (clipboard)
- rustyline 14 (line editing)
- serde/serde_json 1.x (serialization)
- anyhow 1.x (error handling)
- tracing (logging)

[Unreleased]: https://github.com/eroeser/ulm/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/eroeser/ulm/releases/tag/v0.1.0
