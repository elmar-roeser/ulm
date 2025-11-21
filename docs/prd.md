# ulm - Product Requirements Document

**Author:** Elmar Röser
**Date:** 2025-11-20
**Version:** 1.0

---

## Executive Summary

ulm transforms CLI interaction from "memorize commands" to "describe intent." It's an AI-powered bridge between what users want to accomplish and the thousands of powerful but cryptic Unix tools available on their system.

The tool operates entirely locally using Ollama for LLM inference and LanceDB for semantic search, ensuring privacy and offline capability. By pre-indexing all installed manpages and understanding project context, ulm provides intelligent, contextual command suggestions with multiple options and explanations.

### What Makes This Special

**Triple Value Proposition:**

1. **Discovery** - Finds the right tool even when you don't know it exists. Ask "compress images" and ulm finds `imagemagick`, not because you mentioned it, but because it semantically matched your intent against its index of all installed tools.

2. **Education** - Explains WHY each flag works, not just WHAT to type. Users learn the underlying concepts, building lasting knowledge instead of copy-paste dependency.

3. **Efficiency** - Zero context switching. No opening browser, no Stack Overflow, no copying from tutorials. Describe → Select → Execute, all in the terminal.

**The "Sherlock Mode" Differentiator:**
ulm scans your current directory and infers project context. In a Rust project? It suggests `cargo test`. In a Node project? `npm test`. Without being told.

---

## Project Classification

**Technical Type:** CLI Tool
**Domain:** General (Developer Tooling)
**Complexity:** Low (no regulatory requirements)

ulm is a command-line tool targeting Linux/Unix power users. It combines several technical domains:
- **AI/ML**: Local LLM inference via Ollama
- **Information Retrieval**: Vector embeddings and semantic search
- **System Integration**: Manpage parsing, shell integration
- **TUI Development**: Interactive terminal interface

**Innovation Classification:** This project exhibits strong innovation signals for the CLI tool category - specifically "Natural Language CLI" and "AI Commands" - representing a paradigm shift from flag-based to intent-based interaction.

---

## Success Criteria

### MVP Success (Personal Validation)

**Primary Metric:** Daily personal use - ulm becomes part of the regular workflow

**Validation Points:**
- Setup completes successfully and indexes 1000+ manpages
- Query response time < 5 seconds (including LLM inference)
- Suggestions are accurate and actionable 80%+ of the time
- "Sherlock Mode" correctly identifies project type in familiar directories

### Growth Success (Community Adoption)

- GitHub repository gains traction (stars, forks)
- Community contributions (issues, PRs)
- crates.io downloads
- AUR package adoption

---

## Product Scope

### MVP - Minimum Viable Product

**1. Setup Command (`ulm setup`)**
- Detect/guide Ollama installation (existing install or Docker)
- Scan system manpages (/usr/share/man, macOS Homebrew paths)
- Extract NAME and DESCRIPTION from each manpage
- Generate embeddings via Ollama API
- Store vectors in local LanceDB (~/.local/share/ulm/)
- Optionally add shell alias to .bashrc/.zshrc

**2. Query Command (`ulm "question"`)**
- Accept natural language query
- Vector search against local index to find relevant tool(s)
- Load full manpage content for matched tool
- Scan current directory for project markers ("Sherlock Mode")
- Build prompt with context and send to Ollama
- Parse structured JSON response into suggestions

**3. Interactive TUI**
- Display 1-3 command suggestions with titles
- Show explanation text for selected option
- Arrow key navigation
- Hotkey actions: Enter/A (execute), K (copy), B (edit), Esc (abort)
- Execute command via shell or copy to clipboard

**4. DocProvider Abstraction**
- Trait-based architecture for OS abstraction
- Linux/macOS: ManPageProvider
- Fallback: HelpFlagProvider (--help)

### Growth Features (Post-MVP)

**V2:**
- Windows PowerShell Help integration (Get-Help, Get-Command)
- Visual Dry-Run preview for destructive commands
- Magic Pipes: Complex multi-tool pipeline generation
- Scriptable output formats (--json, --plain)
- Configuration file support

### Vision (Future)

**V3+:**
- Error Fix mode (`ulm --fix`)
- Shell integration (inline suggestions)
- Learning mode (track user preferences)
- Community-shared command recipes
- tldr pages integration
- Plugin system for custom documentation sources

---

## CLI Tool Specific Requirements

### MVP Focus: Interactive Experience

ulm MVP is purely interactive - no scriptable interface, no machine-readable output formats.

**Command Structure:**
- `ulm setup` - One-time initialization
- `ulm "natural language query"` - Main interaction
- `ulm update` - Refresh manpage index

**Interaction Model:**
- TUI-based selection (arrow keys, hotkeys)
- Human-readable output only
- Direct command execution or clipboard copy

### Deferred to Post-MVP

- **Scriptable Output**: `--json`, `--plain` flags for pipe compatibility
- **Configuration File**: `~/.config/ulm/config.toml` for model selection, defaults
- **Shell Completion**: Tab completion for bash/zsh/fish
- **Non-interactive Mode**: `ulm --no-tui "query"` for scripts

---

## Innovation & Novel Patterns

### Natural Language CLI Paradigm

ulm represents a fundamental shift in CLI interaction:

**Traditional CLI:** User must know tool name → look up flags → construct command
**ulm Paradigm:** User describes intent → tool discovers solution → user validates and executes

This inverts the knowledge requirement - instead of knowing the tool to find the documentation, users describe the outcome to find the tool.

### Semantic Search over System Documentation

Unlike keyword search or regex matching, ulm uses vector embeddings to understand semantic meaning:
- "compress images" matches imagemagick even without the word "image" in the query
- "find large files" matches both `find` and `du` with different approaches
- "secure copy" matches `scp`, `rsync`, and potentially `rclone`

### Context-Aware Inference ("Sherlock Mode")

The system doesn't just answer questions - it understands WHERE the question is asked:
- Directory contains `Cargo.toml` → Rust project → prefer cargo commands
- Directory contains `package.json` → Node project → prefer npm/yarn commands
- Directory contains `.git` → suggest git-related operations

### Validation Approach

**Technical Validation:**
- LanceDB performance with 5000+ vectors
- Ollama response latency acceptable for interactive use
- Embedding quality sufficient for semantic matching

**User Validation:**
- Does it find the right tool for common tasks?
- Are explanations clear and educational?
- Is the TUI interaction natural and fast?

**Innovation Risk Mitigation:**
- If semantic search fails, fall back to keyword matching
- If context inference is wrong, user can override
- Multiple suggestions reduce single-point-of-failure

---

## Functional Requirements

### Setup & Initialization

- **FR1:** User can run `ulm setup` to initialize the tool
- **FR2:** Setup detects if Ollama is running and accessible
- **FR3:** Setup guides user to install Ollama if not present (with Docker option)
- **FR4:** Setup verifies a suitable LLM model is available in Ollama
- **FR5:** Setup can pull a default model if none exists
- **FR6:** User can optionally install shell alias during setup

### Knowledge Base (Indexing)

- **FR7:** Setup scans all manpage directories on the system
- **FR8:** Setup extracts NAME and DESCRIPTION sections from each manpage
- **FR9:** Setup generates vector embeddings for each manpage description
- **FR10:** Setup stores embeddings in local LanceDB database
- **FR11:** User can run `ulm update` to refresh the index
- **FR12:** Setup reports indexing progress and final count

### Query Processing

- **FR13:** User can submit natural language queries via `ulm "query"`
- **FR14:** System converts query to vector embedding
- **FR15:** System performs semantic search against manpage index
- **FR16:** System retrieves top matching tool(s) from index
- **FR17:** System loads full manpage content for matched tool

### Context Awareness ("Sherlock Mode")

- **FR18:** System scans current directory for project marker files
- **FR19:** System identifies project type from markers (Rust, Node, Python, etc.)
- **FR20:** System includes directory context in LLM prompt
- **FR21:** Context influences tool and command suggestions

### Suggestion Generation

- **FR22:** System sends query + manpage + context to Ollama
- **FR23:** System requests structured JSON response from LLM
- **FR24:** System parses response into 1-3 command suggestions
- **FR25:** Each suggestion includes: command, title, explanation, risk level
- **FR26:** Explanations describe WHY each flag/approach works

### Interactive Selection (TUI)

- **FR27:** System displays suggestions in interactive terminal menu
- **FR28:** User can navigate suggestions with arrow keys
- **FR29:** Selected suggestion shows full explanation inline
- **FR30:** User can execute selected command with Enter or 'A' key
- **FR31:** User can copy command to clipboard with 'K' key
- **FR32:** User can edit command before executing with 'B' key
- **FR33:** User can abort with Esc key
- **FR34:** Copy action provides visual feedback

### Command Execution

- **FR35:** Executed commands run in user's shell
- **FR36:** Command output streams to terminal
- **FR37:** Edit mode uses line editor with cursor navigation

### Error Handling

- **FR38:** System provides clear error messages for Ollama connection failures
- **FR39:** System handles missing manpages gracefully
- **FR40:** System reports if no relevant tools found for query

### Model Management

- **FR41:** Setup displays list of recommended LLM models with RAM requirements
- **FR42:** User can select preferred model from list during setup
- **FR43:** Setup pulls selected model with progress indicator
- **FR44:** Selected model is saved as default for future queries

---

## Non-Functional Requirements

### Performance

- **NFR1:** Setup indexing completes within 5 minutes for 5000 manpages
- **NFR2:** Query-to-first-suggestion latency < 5 seconds (including LLM inference)
- **NFR3:** TUI navigation is instantaneous (< 50ms response)
- **NFR4:** Vector search completes in < 100ms
- **NFR5:** Database size remains reasonable (< 500MB for full index)

### Security & Privacy

- **NFR6:** All processing occurs locally - no data sent to external services
- **NFR7:** No telemetry or usage tracking
- **NFR8:** Executed commands run with user's permissions (no privilege escalation)
- **NFR9:** Clipboard access limited to explicit user action (K key)

### Reliability

- **NFR10:** Tool functions offline after initial setup (if Ollama is local)
- **NFR11:** Graceful degradation if Ollama unavailable
- **NFR12:** Index survives system restarts

### Compatibility

- **NFR13:** Runs on Linux (primary) and macOS
- **NFR14:** Works with standard terminal emulators
- **NFR15:** Compatible with Ollama API v1

---

_This PRD captures the essence of ulm - an AI-powered CLI companion that transforms command discovery from "I need to know the tool" to "I describe what I want." It bridges the gap between powerful Unix tools and the humans who want to use them, providing discovery, education, and efficiency in a privacy-respecting local-first package._

_Created through collaborative discovery between Elmar Röser and AI facilitator._
