# Product Brief: Der ULMer (ulm)

**Date:** 2025-11-20
**Author:** Elmar Röser
**Context:** Open-Source Developer Tool

---

## Executive Summary

Der ULMer (ulm) is an AI-powered CLI tool that transforms how developers interact with system documentation. Instead of memorizing command names and flag combinations, users describe what they want to accomplish in natural language, and ulm finds the right tool, generates the correct command with multiple options, and explains why each approach works.

Built in Rust with an embedded vector database and local LLM inference via Ollama, ulm provides intelligent, context-aware command generation that understands the user's current project environment. The tool bridges the gap between "I know what I want to do" and "I know which command does it."

---

## Core Vision

### Problem Statement

Linux/Unix systems contain thousands of powerful command-line tools, but accessing their capabilities requires:

- **Knowing the tool exists**: Users must already know that `ffmpeg` handles video, `imagemagick` handles images, or `rsync` handles file sync
- **Memorizing cryptic flags**: Even simple operations require consulting manpages for flags like `-xvzf`, `-mtime +7`, or `-c:v libx264`
- **Navigating dense documentation**: Manpages are comprehensive but hard to search for specific use cases
- **Context switching**: Users must leave their workflow to search online for "how to do X with Y"

The result: developers repeatedly search Stack Overflow for commands they've used before, or avoid powerful tools because the learning curve is too steep.

### Proposed Solution

ulm provides an intelligent interface between natural language and system commands:

1. **Smart Discovery**: User asks "compress all images in this folder" → ulm searches its local index of all installed manpages, finds `imagemagick`, and reads the relevant documentation
2. **Context Awareness ("Sherlock Mode")**: ulm scans the current directory for project markers (Cargo.toml, package.json, Makefile) and infers the correct toolchain without being told
3. **Multiple Options with Explanations**: Instead of one answer, ulm provides 2-3 approaches (standard, safe, verbose) with explanations of why each works and what the flags do
4. **Interactive Selection**: Users navigate options with arrow keys, see explanations inline, then Execute/Copy/Edit with single-key shortcuts
5. **Local & Private**: All processing happens locally via Ollama - no data leaves the machine

### Key Differentiators

- **Embedded Vector Search**: Unlike web-based AI tools, ulm pre-indexes all local manpages during setup, enabling instant semantic search without internet
- **Project Context Awareness**: Automatically detects project type and suggests appropriate tools (cargo test vs npm test vs pytest)
- **Multiple Solution Paths**: Provides standard/safe/alternative approaches instead of single answers
- **Zero Cloud Dependencies**: Runs entirely locally with Ollama - works offline, no API keys, no privacy concerns
- **Cross-Platform Architecture**: Designed with abstraction layer (DocProvider trait) to support Linux manpages, macOS Homebrew, and Windows PowerShell Help

---

## Target Users

### Primary Users

**Linux/Unix CLI Power Users**

Developers and system administrators who:
- Work primarily in the terminal and value keyboard-driven workflows
- Know many commands but frequently forget specific flags or discover new tools
- Prefer local, privacy-respecting tools over cloud services
- Have Ollama or are willing to run local LLMs
- Are comfortable with Rust toolchain (cargo install)

**Typical scenarios:**
- "I know ffmpeg can do this, but which flags?"
- "There must be a tool for this, but I don't know its name"
- "I want to run tests but I'm in a new project and don't know the toolchain"

**Technical profile:**
- Intermediate to expert CLI skills
- Familiar with package managers, git, Docker
- Values efficiency and automation

---

## MVP Scope

### Core Features

**1. Setup Command (`ulm setup`)**
- Check/install Ollama (detect existing installation or guide Docker setup)
- Scan all manpages from system paths (/usr/share/man, Homebrew paths on macOS)
- Extract NAME and DESCRIPTION from each manpage
- Generate embeddings via Ollama and store in local LanceDB
- Optionally install shell alias (`ulm` → `ulm-ai`) in .bashrc/.zshrc

**2. Smart Query (`ulm "question"`)**
- Vector search against local manpage index to find relevant tool
- Load full manpage content for selected tool
- Inject current directory file list as context ("Sherlock Mode")
- Send to Ollama with structured prompt requesting JSON response
- Parse response into CommandSuggestion structs

**3. Interactive TUI**
- Display 1-3 command suggestions with inline explanations
- Arrow key navigation with live explanation preview
- Hotkey actions:
  - `Enter` / `A`: Execute command
  - `K`: Copy to clipboard
  - `B`: Edit before executing (rustyline)
  - `Esc`: Abort

**4. DocProvider Abstraction**
- Trait-based architecture for different documentation sources
- Linux: ManPageProvider (man -P cat)
- Fallback: HelpFlagProvider (--help)

### Out of Scope for MVP

- **Windows PowerShell support**: Deferred to V2 (requires Get-Help integration)
- **Visual Dry-Run**: Preview of affected files for destructive commands
- **Magic Pipes**: Complex multi-tool pipeline generation
- **Error Fix mode**: `ulm --fix` to correct last failed command
- **Web documentation**: Fetching docs from online sources
- **Custom model selection**: Using different Ollama models per query
- **Caching of LLM responses**: Avoiding repeated queries for same questions

### Future Vision

**V2 Features:**
- Windows PowerShell Help integration (Get-Help, Get-Command)
- Visual Dry-Run: Show affected files before rm, chmod, find -delete
- Magic Pipes: Generate complex pipelines (find | xargs | zip)

**V3 Features:**
- Error Fix: `ulm --fix` reads last command + error, suggests correction
- Shell integration: Inline suggestions as you type (like fish)
- Learning mode: Track which suggestions user executes, improve ranking

**Long-term Vision:**
- Community-shared command recipes
- Integration with tldr pages as additional context
- Plugin system for custom documentation sources

---

## Technical Preferences

### Language & Runtime
- **Rust** (2021 edition) - Performance, safety, single binary distribution

### Core Dependencies
- `clap` - CLI argument parsing
- `tokio` - Async runtime for HTTP calls
- `crossterm` - Terminal UI and keyboard events
- `reqwest` - HTTP client for Ollama API
- `serde` / `serde_json` - JSON serialization
- `lancedb` - Embedded vector database
- `arboard` - Cross-platform clipboard
- `rustyline` - Line editing for command modification
- `anyhow` - Error handling
- `directories` - XDG-compliant config paths

### External Dependencies
- **Ollama** - Local LLM inference (user-managed or Docker)

### Data Storage
- Config: `~/.config/ulm/`
- Database: `~/.local/share/ulm/index.lance`

### Project Structure
```
src/
├── main.rs          # Entry point, CLI parsing
├── setup.rs         # Installation & indexing
├── arch/            # OS abstraction
│   ├── mod.rs       # DocProvider trait
│   ├── linux.rs     # ManPageProvider
│   └── windows.rs   # (future) PowerShellProvider
├── brain/           # AI logic
│   ├── context.rs   # Sherlock mode (dir scanning)
│   └── ollama.rs    # LLM API client
├── db.rs            # Vector database operations
└── tui.rs           # Interactive selection UI
```

### Distribution
- `cargo install ulm` (crates.io)
- GitHub releases with pre-built binaries
- AUR package (Arch Linux)

---

_This Product Brief captures the vision and requirements for Der ULMer (ulm)._

_It was created through collaborative discovery and reflects the unique needs of this Open-Source Developer Tool project._

_Next: Use the PRD workflow to create detailed product requirements from this brief._
