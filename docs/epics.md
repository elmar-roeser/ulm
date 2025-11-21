# ulm - Epic Breakdown

**Author:** Elmar Röser
**Date:** 2025-11-20
**Project Type:** Rust CLI Tool

---

## Overview

This document provides the complete epic and story breakdown for ulm, decomposing the 40 functional requirements from the PRD into implementable stories.

**Epic Summary:**
- Epic 1: Foundation (4 stories)
- Epic 2: Setup & Knowledge Base (8 stories)
- Epic 3: Query & Intelligence (7 stories)
- Epic 4: Interactive Experience (8 stories)
- Epic 5: Auto-Installation (3 stories)

**Total: 30 stories**

---

## Functional Requirements Inventory

### Setup & Initialization (FR1-6)
### Knowledge Base (FR7-12)
### Query Processing (FR13-17)
### Context Awareness (FR18-21)
### Suggestion Generation (FR22-26)
### Interactive TUI (FR27-34)
### Command Execution (FR35-37)
### Error Handling (FR38-40)

---

## FR Coverage Map

| Epic | FRs Covered | Story Count |
|------|-------------|-------------|
| Epic 1: Foundation | Infrastructure for all | 4 |
| Epic 2: Setup & Knowledge Base | FR1-12 | 8 |
| Epic 3: Query & Intelligence | FR13-26 | 7 |
| Epic 4: Interactive Experience | FR27-40 | 8 |
| Epic 5: Auto-Installation | FR3, FR5 (enhanced) | 3 |

---

## Epic 1: Foundation

**Goal:** Establish project infrastructure, core dependencies, and basic architecture that enables all subsequent development.

**User Value:** Developer can build and run the project with proper tooling and structure in place.

---

### Story 1.1: Project Initialization

As a developer,
I want to initialize the Rust project with proper structure,
So that I have a solid foundation for building ulm.

**Acceptance Criteria:**

**Given** no existing project
**When** I run `cargo new ulm --bin`
**Then** a new Rust project is created with Cargo.toml and src/main.rs

**And** Cargo.toml includes all core dependencies:
- tokio (full features)
- clap (derive feature)
- anyhow
- serde, serde_json
- reqwest (json feature)
- tracing, tracing-subscriber

**And** .gitignore is configured for Rust projects
**And** README.md exists with basic project description
**And** LICENSE file is added (MIT or Apache-2.0)

**Prerequisites:** None (first story)

**Technical Notes:**
- Use Rust 2021 edition
- Configure for release optimizations in Cargo.toml
- Add workspace-level clippy lints

---

### Story 1.2: Module Structure Setup

As a developer,
I want the project organized into logical modules,
So that code is maintainable and follows architecture design.

**Acceptance Criteria:**

**Given** initialized project from Story 1.1
**When** I create the module structure
**Then** the following directories and mod.rs files exist:
```
src/
├── main.rs
├── lib.rs
├── cli.rs
├── error.rs
├── db.rs
├── setup/mod.rs
├── query/mod.rs
├── llm/mod.rs
├── tui/mod.rs
└── exec/mod.rs
```

**And** each mod.rs properly declares submodules
**And** lib.rs re-exports public API for testing
**And** main.rs imports from lib and sets up CLI

**Prerequisites:** Story 1.1

**Technical Notes:**
- Follow architecture document structure exactly
- Each module should compile (can have placeholder implementations)
- Use `pub(crate)` for internal APIs

---

### Story 1.3: CLI Argument Parsing

As a user,
I want to run ulm with different commands,
So that I can setup, query, or update the system.

**Acceptance Criteria:**

**Given** the module structure from Story 1.2
**When** I define CLI arguments using clap derive
**Then** the following commands are available:
- `ulm setup` - Initialize the system
- `ulm update` - Refresh manpage index
- `ulm "query"` - Main query command (positional argument)

**And** `--help` shows usage information
**And** `--version` shows version from Cargo.toml
**And** invalid commands show helpful error messages

**Given** user runs `ulm "find large files"`
**When** the query is parsed
**Then** the query string is captured correctly with quotes preserved

**Prerequisites:** Story 1.2

**Technical Notes:**
- Use clap's derive macros for type-safe parsing
- Define Args struct in cli.rs
- Subcommands enum for setup/update
- Default command is query if string provided

---

### Story 1.4: Error Handling Infrastructure

As a developer,
I want consistent error handling across the application,
So that errors are properly propagated and displayed.

**Acceptance Criteria:**

**Given** the module structure
**When** I implement error handling
**Then** anyhow::Result is used throughout the codebase

**And** main() returns Result<()> with proper exit codes
**And** user-facing errors are clear and actionable (English)
**And** developer errors include context chain

**Given** an error occurs
**When** it propagates to main
**Then** the error message is printed to stderr
**And** non-zero exit code is returned

**And** tracing is configured for debug logging
**And** RUST_LOG=ulm=debug enables verbose output

**Prerequisites:** Story 1.2

**Technical Notes:**
- Configure tracing-subscriber in main()
- Use .context() for error chain
- Error messages follow architecture patterns
- Exit code 1 for errors, 0 for success

---

## Epic 2: Setup & Knowledge Base

**Goal:** Enable users to configure ulm with Ollama and build a searchable index of all system manpages.

**User Value:** After running `ulm setup`, the system is ready to answer questions with a complete knowledge base of installed tools.

**FRs Covered:** FR1-12

---

### Story 2.1: Ollama API Client

As a developer,
I want a client to communicate with Ollama API,
So that I can generate embeddings and LLM responses.

**Acceptance Criteria:**

**Given** Ollama is running on localhost:11434
**When** I create an OllamaClient
**Then** it can send POST requests to /api/embeddings
**And** it can send POST requests to /api/generate

**And** requests are serialized as JSON using serde
**And** responses are deserialized into typed structs
**And** connection errors return descriptive messages

**Prerequisites:** Story 1.4

**Technical Notes:**
- Implement in llm/ollama.rs
- Use reqwest with async/await
- Base URL configurable (default localhost:11434)
- Timeout of 30s for embeddings, 60s for generate

---

### Story 2.2: Ollama Detection & Health Check (FR2, FR3)

As a user,
I want setup to detect if Ollama is running,
So that I know if I need to install or start it.

**Acceptance Criteria:**

**Given** user runs `ulm setup`
**When** checking for Ollama
**Then** system pings localhost:11434/api/tags

**Given** Ollama is running
**When** health check succeeds
**Then** display "✓ Ollama detected at localhost:11434"

**Given** Ollama is not running
**When** health check fails
**Then** display clear instructions:
"Ollama not found. Install from https://ollama.ai or start with: ollama serve"

**Prerequisites:** Story 2.1

**Technical Notes:**
- Implement in setup/ollama.rs
- Use /api/tags endpoint (lightweight check)
- Timeout 5s for detection
- Consider Docker detection as future enhancement

---

### Story 2.3: Model Verification & Pull (FR4, FR5)

As a user,
I want setup to ensure a suitable LLM model is available,
So that I can generate embeddings and responses.

**Acceptance Criteria:**

**Given** Ollama is running
**When** checking for models
**Then** system queries /api/tags for installed models

**Given** a suitable model exists (llama3, mistral, etc.)
**When** model is found
**Then** display "✓ Model 'llama3' available"

**Given** no suitable model exists
**When** user confirms pull
**Then** system runs `ollama pull llama3`
**And** displays download progress
**And** confirms when complete

**Given** user declines pull
**When** no model available
**Then** display instructions for manual pull

**Prerequisites:** Story 2.2

**Technical Notes:**
- Default model: llama3 (good balance of speed/quality)
- Check for embedding capability
- Progress display via Ollama's streaming response

---

### Story 2.4: Manpage Directory Scanner (FR7)

As a developer,
I want to scan system manpage directories,
So that I can find all available documentation.

**Acceptance Criteria:**

**Given** a Linux/macOS system
**When** scanning for manpages
**Then** system checks these paths:
- /usr/share/man
- /usr/local/share/man
- /opt/homebrew/share/man (macOS)
- Paths from $MANPATH

**And** finds all man1, man8 sections (user commands, admin commands)
**And** returns list of manpage file paths
**And** handles missing directories gracefully

**Given** 5000 manpages exist
**When** scan completes
**Then** all are discovered in < 5 seconds

**Prerequisites:** Story 1.4

**Technical Notes:**
- Implement in setup/index.rs
- Use std::fs for directory traversal
- Filter by .gz, .1, .8 extensions
- Skip man3 (library functions) for MVP

---

### Story 2.5: Manpage Content Extraction (FR8)

As a developer,
I want to extract NAME and DESCRIPTION from manpages,
So that I can create searchable descriptions.

**Acceptance Criteria:**

**Given** a manpage file path
**When** extracting content
**Then** system runs `man -P cat <tool>`
**And** parses output for NAME section
**And** parses output for DESCRIPTION (first paragraph)

**Given** manpage for "ffmpeg"
**When** extraction completes
**Then** returns: "ffmpeg - ffmpeg video converter"

**And** handles malformed manpages gracefully
**And** returns empty string if parsing fails

**Prerequisites:** Story 2.4

**Technical Notes:**
- Shell out to `man -P cat` (handles all formats)
- Parse NAME section (usually first line after header)
- Limit DESCRIPTION to ~500 chars for embedding
- UTF-8 validation on output

---

### Story 2.6: Embedding Generation (FR9)

As a developer,
I want to generate vector embeddings for manpage descriptions,
So that I can perform semantic search.

**Acceptance Criteria:**

**Given** extracted manpage description
**When** generating embedding
**Then** system calls Ollama /api/embeddings
**And** receives 768-dimensional vector (or model-specific)

**Given** batch of 100 descriptions
**When** processing embeddings
**Then** system batches requests efficiently
**And** displays progress indicator
**And** completes within reasonable time

**And** handles API errors with retry logic
**And** skips failed embeddings with warning

**Prerequisites:** Story 2.1, Story 2.5

**Technical Notes:**
- Batch size: 10-20 for memory efficiency
- Retry: 3 attempts with exponential backoff
- Progress: indicatif crate or simple counter
- Store tool_name with each embedding

---

### Story 2.7: LanceDB Storage (FR10)

As a developer,
I want to store embeddings in LanceDB,
So that I can perform fast vector search.

**Acceptance Criteria:**

**Given** generated embeddings
**When** storing in LanceDB
**Then** creates database at ~/.local/share/ulm/index.lance

**And** schema includes: tool_name, section, description, vector
**And** uses directories crate for XDG paths
**And** creates parent directories if needed

**Given** 5000 embeddings
**When** storage completes
**Then** database size < 500MB
**And** write completes in < 60 seconds

**And** overwrites existing index on re-run
**And** handles disk errors gracefully

**Prerequisites:** Story 2.6

**Technical Notes:**
- Implement in db.rs
- Use lancedb crate
- Arrow array format for vectors
- Create table "manpages"

---

### Story 2.8: Setup Orchestration & Progress (FR1, FR6, FR11, FR12)

As a user,
I want `ulm setup` to orchestrate the complete setup process,
So that I can get started with one command.

**Acceptance Criteria:**

**Given** user runs `ulm setup`
**When** setup executes
**Then** steps run in order:
1. Check Ollama connection
2. Verify/pull model
3. Scan manpage directories
4. Extract descriptions
5. Generate embeddings
6. Store in LanceDB

**And** progress is displayed for each step
**And** final count reported: "Indexed 4,523 manpages"

**Given** user runs `ulm update` (FR11)
**When** update executes
**Then** re-runs indexing steps (skip Ollama check)
**And** overwrites existing index

**Given** setup completes successfully
**When** user opts to install alias (FR6)
**Then** adds `alias ulm='ulm'` to .bashrc or .zshrc
**And** detects shell from $SHELL

**Prerequisites:** Stories 2.2-2.7

**Technical Notes:**
- Implement in setup/mod.rs
- Orchestrate all substeps
- Alias installation is optional prompt
- Total time target: < 5 minutes for 5000 pages

---

## Epic 3: Query & Intelligence

**Goal:** Enable intelligent command suggestions by combining semantic search, context awareness, and LLM generation.

**User Value:** User asks a question in natural language and receives accurate, contextual command suggestions with explanations.

**FRs Covered:** FR13-26

---

### Story 3.1: Vector Search (FR14-16)

As a developer,
I want to perform semantic search against the manpage index,
So that I can find relevant tools for user queries.

**Acceptance Criteria:**

**Given** a user query "compress images"
**When** performing vector search
**Then** system generates embedding for query via Ollama
**And** searches LanceDB for nearest neighbors
**And** returns top 3 matching tools with scores

**Given** query matches "imagemagick"
**When** search completes
**Then** returns tool_name, description, similarity score
**And** search completes in < 100ms

**And** handles empty index gracefully
**And** returns empty results if no good matches (threshold)

**Prerequisites:** Story 2.7

**Technical Notes:**
- Implement in query/search.rs
- Use LanceDB's built-in ANN search
- Similarity threshold: 0.7 (configurable)
- Return top 1 for MVP (can expand later)

---

### Story 3.2: Manpage Content Loading (FR17)

As a developer,
I want to load full manpage content for matched tools,
So that I can provide complete context to the LLM.

**Acceptance Criteria:**

**Given** a matched tool name "ffmpeg"
**When** loading manpage content
**Then** system runs `man -P cat ffmpeg`
**And** captures full output as string
**And** cleans escape sequences and formatting

**Given** manpage is very long (> 10000 chars)
**When** loading content
**Then** truncates to reasonable size for LLM context
**And** preserves most important sections (SYNOPSIS, OPTIONS)

**And** handles missing manpages with error
**And** caches loaded content for session

**Prerequisites:** Story 3.1

**Technical Notes:**
- Implement in query/search.rs
- Max content: ~8000 chars (LLM context limit)
- Prioritize: NAME, SYNOPSIS, DESCRIPTION, OPTIONS
- Strip ANSI codes and man formatting

---

### Story 3.3: Directory Context Scanner (FR18-19)

As a developer,
I want to scan the current directory for project markers,
So that I can infer project type for context-aware suggestions.

**Acceptance Criteria:**

**Given** user is in a directory with Cargo.toml
**When** scanning for context
**Then** system identifies project type as "Rust"

**Given** directory contains package.json
**When** scanning for context
**Then** system identifies project type as "Node"

**And** detects these markers:
- Cargo.toml → Rust
- package.json → Node
- requirements.txt, pyproject.toml → Python
- go.mod → Go
- Makefile → C/C++
- .git → Git repository

**And** returns list of detected marker files
**And** handles nested projects (use closest marker)

**Prerequisites:** Story 1.4

**Technical Notes:**
- Implement in query/context.rs
- Scan only top-level (no recursion)
- Return ProjectType enum + marker list
- Max 50 files scanned for performance

---

### Story 3.4: Context Builder (FR20-21)

As a developer,
I want to build a context object for LLM prompts,
So that suggestions are tailored to the user's environment.

**Acceptance Criteria:**

**Given** detected project type and markers
**When** building context
**Then** creates DirectoryContext struct with:
- project_type: Option<ProjectType>
- marker_files: Vec<String>
- cwd: PathBuf

**Given** context is built
**When** formatting for prompt
**Then** produces string like:
"User is in a Rust project (Cargo.toml found)"

**And** handles no project type gracefully
**And** limits marker list to 20 most relevant

**Prerequisites:** Story 3.3

**Technical Notes:**
- Implement in query/context.rs
- Context influences prompt but doesn't override
- Example: "run tests" → cargo test (in Rust project)

---

### Story 3.5: Prompt Builder (FR22)

As a developer,
I want to construct effective prompts for the LLM,
So that responses are accurate and well-structured.

**Acceptance Criteria:**

**Given** query, manpage content, and directory context
**When** building prompt
**Then** creates structured prompt with:
1. System instructions (role, output format)
2. Manpage content
3. Directory context
4. User query

**And** requests JSON output format
**And** specifies CommandSuggestion schema
**And** limits response to 1-3 suggestions

**Given** prompt is built
**When** sent to Ollama
**Then** total prompt size < 12000 tokens

**Prerequisites:** Story 3.2, Story 3.4

**Technical Notes:**
- Implement in llm/prompt.rs
- Use clear system prompt with examples
- Request JSON with specific schema
- Include "explain WHY" instruction

---

### Story 3.6: LLM Response Parser (FR23-25)

As a developer,
I want to parse LLM responses into typed suggestions,
So that I can display them in the TUI.

**Acceptance Criteria:**

**Given** Ollama returns JSON response
**When** parsing response
**Then** deserializes into Vec<CommandSuggestion>

**And** each suggestion contains:
- command: String
- title: String
- explanation: String
- risk_level: RiskLevel (Safe/Moderate/Destructive)

**Given** malformed JSON response
**When** parsing fails
**Then** returns helpful error message
**And** logs raw response for debugging

**Given** empty suggestions array
**When** no commands found
**Then** returns appropriate error

**Prerequisites:** Story 3.5

**Technical Notes:**
- Implement in llm/response.rs
- Use serde for JSON parsing
- Validate command is not empty
- Risk level defaults to Safe if missing

---

### Story 3.7: Query Orchestration (FR13, FR26)

As a user,
I want to run `ulm "question"` and get intelligent suggestions,
So that I can find the right command quickly.

**Acceptance Criteria:**

**Given** user runs `ulm "find large files"`
**When** query executes
**Then** orchestrates these steps:
1. Generate query embedding
2. Search vector index
3. Load matched manpage
4. Scan directory context
5. Build prompt
6. Call Ollama generate
7. Parse response

**And** returns Vec<CommandSuggestion> to caller
**And** total latency < 5 seconds

**Given** no matching tools found
**When** search returns empty
**Then** returns error: "No relevant tools found for query"

**And** suggestions include explanations of WHY (FR26)
**And** handles all errors with clear messages

**Prerequisites:** Stories 3.1-3.6

**Technical Notes:**
- Implement in query/mod.rs
- Orchestrate full pipeline
- Pass results to TUI (Epic 4)
- Log timing for performance tracking

---

## Epic 4: Interactive Experience

**Goal:** Provide a smooth, keyboard-driven interface for selecting and executing command suggestions.

**User Value:** User can navigate suggestions, see explanations, and execute/copy/edit commands with single keystrokes.

**FRs Covered:** FR27-40

---

### Story 4.1: TUI Renderer (FR27, FR29)

As a user,
I want to see command suggestions displayed clearly,
So that I can understand my options.

**Acceptance Criteria:**

**Given** Vec<CommandSuggestion> from query
**When** rendering TUI
**Then** displays each suggestion with:
- Index number (1, 2, 3)
- Title
- Command (syntax highlighted)
- Explanation (for selected item)

**And** selected item is visually highlighted
**And** risk level shown with color (green/yellow/red)
**And** footer shows available hotkeys

**Given** terminal is 80 chars wide
**When** rendering
**Then** content wraps appropriately
**And** explanation text is readable

**Prerequisites:** Story 3.7

**Technical Notes:**
- Implement in tui/render.rs
- Use crossterm for styling
- Colors: green (Safe), yellow (Moderate), red (Destructive)
- Clear screen before render, restore on exit

---

### Story 4.2: Event Loop & Navigation (FR28)

As a user,
I want to navigate suggestions with arrow keys,
So that I can select the right option.

**Acceptance Criteria:**

**Given** TUI is displayed
**When** user presses Up/Down arrows
**Then** selection moves between suggestions
**And** display updates to show new selection
**And** explanation updates for selected item

**And** wraps around (down from last → first)
**And** responds in < 50ms

**Given** user presses invalid key
**When** key is not recognized
**Then** nothing happens (no error)

**Prerequisites:** Story 4.1

**Technical Notes:**
- Implement in tui/input.rs
- Use crossterm::event for key capture
- Raw mode for immediate input
- Event loop runs until action selected

---

### Story 4.3: Execute Action (FR30, FR35, FR36)

As a user,
I want to execute the selected command with Enter or 'A',
So that I can run it immediately.

**Acceptance Criteria:**

**Given** suggestion is selected
**When** user presses Enter or 'A'
**Then** TUI closes
**And** command executes in user's shell

**Given** command executes
**When** running
**Then** stdout/stderr stream to terminal
**And** user sees real-time output
**And** exit code is captured

**Given** command completes
**When** finished
**Then** ulm exits with command's exit code

**Prerequisites:** Story 4.2

**Technical Notes:**
- Implement in exec/shell.rs
- Use std::process::Command
- Inherit stdin/stdout/stderr
- Exit TUI before execution (restore terminal)

---

### Story 4.4: Copy Action (FR31, FR34)

As a user,
I want to copy the command with 'K',
So that I can paste it elsewhere.

**Acceptance Criteria:**

**Given** suggestion is selected
**When** user presses 'K'
**Then** command string copied to clipboard
**And** visual feedback shown: "Copied to clipboard!"

**And** feedback disappears after 1 second
**And** user stays in TUI (can continue)

**Given** clipboard unavailable (Wayland without wl-copy)
**When** copy fails
**Then** shows error message
**And** suggests: "Install wl-clipboard for Wayland support"

**Prerequisites:** Story 4.2

**Technical Notes:**
- Implement in exec/clipboard.rs
- Use arboard crate
- Works on X11, Wayland, macOS
- Keep TUI open after copy

---

### Story 4.5: Edit Action (FR32, FR37)

As a user,
I want to edit the command before executing with 'B',
So that I can modify paths or parameters.

**Acceptance Criteria:**

**Given** suggestion is selected
**When** user presses 'B'
**Then** command appears in editable line
**And** cursor is at end of command

**Given** user is editing
**When** typing
**Then** can use arrow keys to move cursor
**And** can delete/insert characters
**And** has readline-like shortcuts (Ctrl-A, Ctrl-E)

**Given** user presses Enter in edit mode
**When** confirmed
**Then** edited command executes
**And** same as Execute action (FR30)

**Given** user presses Escape in edit mode
**When** cancelled
**Then** returns to TUI selection

**Prerequisites:** Story 4.3

**Technical Notes:**
- Implement in tui/input.rs
- Use rustyline for line editing
- Pre-fill with selected command
- Exit edit mode on Enter or Esc

---

### Story 4.6: Abort Action (FR33)

As a user,
I want to abort with Escape,
So that I can exit without running anything.

**Acceptance Criteria:**

**Given** TUI is displayed
**When** user presses Escape
**Then** TUI closes
**And** terminal is restored
**And** ulm exits with code 0

**And** no command is executed
**And** nothing is copied

**Prerequisites:** Story 4.2

**Technical Notes:**
- Implement in tui/input.rs
- Clean terminal restoration
- Return UserAction::Abort

---

### Story 4.7: Error Display (FR38-40)

As a user,
I want to see clear error messages,
So that I know what went wrong and how to fix it.

**Acceptance Criteria:**

**Given** Ollama connection fails (FR38)
**When** error occurs
**Then** displays: "Cannot connect to Ollama at localhost:11434. Start with: ollama serve"

**Given** manpage not found (FR39)
**When** loading fails
**Then** displays: "Manpage for '{tool}' not found"

**Given** no relevant tools found (FR40)
**When** search returns empty
**Then** displays: "No relevant tools found for '{query}'. Try rephrasing your question."

**And** all errors are printed to stderr
**And** exit code is 1 for errors

**Prerequisites:** Story 1.4

**Technical Notes:**
- Error messages in English
- Actionable guidance included
- Use anyhow for error chain
- Consistent formatting

---

### Story 4.8: TUI Orchestration

As a user,
I want the complete TUI flow to work seamlessly,
So that I have a smooth interactive experience.

**Acceptance Criteria:**

**Given** query returns suggestions
**When** TUI starts
**Then** orchestrates complete flow:
1. Enter raw mode
2. Render suggestions
3. Handle events
4. Execute action
5. Restore terminal

**And** handles Ctrl-C gracefully
**And** cleans up on panic (terminal restored)

**Given** single suggestion returned
**When** TUI displays
**Then** auto-selects it (still shows menu)

**Given** suggestions have Destructive risk
**When** rendering
**Then** shows warning indicator

**Prerequisites:** Stories 4.1-4.7

**Technical Notes:**
- Implement in tui/mod.rs
- Orchestrate all TUI components
- Use scopeguard for cleanup
- Main entry point for TUI

---

## FR Coverage Matrix

| FR | Description | Epic | Story |
|----|-------------|------|-------|
| FR1 | User can run `ulm setup` | Epic 2 | 2.8 |
| FR2 | Setup detects Ollama | Epic 2 | 2.2 |
| FR3 | Setup guides Ollama install | Epic 2 | 2.2 |
| FR4 | Setup verifies model | Epic 2 | 2.3 |
| FR5 | Setup can pull model | Epic 2 | 2.3 |
| FR6 | Shell alias installation | Epic 2 | 2.8 |
| FR7 | Scan manpage directories | Epic 2 | 2.4 |
| FR8 | Extract NAME/DESCRIPTION | Epic 2 | 2.5 |
| FR9 | Generate embeddings | Epic 2 | 2.6 |
| FR10 | Store in LanceDB | Epic 2 | 2.7 |
| FR11 | User can run `ulm update` | Epic 2 | 2.8 |
| FR12 | Report indexing progress | Epic 2 | 2.8 |
| FR13 | Submit natural language query | Epic 3 | 3.7 |
| FR14 | Convert query to embedding | Epic 3 | 3.1 |
| FR15 | Semantic search | Epic 3 | 3.1 |
| FR16 | Retrieve top matches | Epic 3 | 3.1 |
| FR17 | Load manpage content | Epic 3 | 3.2 |
| FR18 | Scan directory markers | Epic 3 | 3.3 |
| FR19 | Identify project type | Epic 3 | 3.3 |
| FR20 | Include context in prompt | Epic 3 | 3.4 |
| FR21 | Context influences suggestions | Epic 3 | 3.4 |
| FR22 | Send to Ollama | Epic 3 | 3.5 |
| FR23 | Request JSON response | Epic 3 | 3.5 |
| FR24 | Parse into suggestions | Epic 3 | 3.6 |
| FR25 | Suggestion includes all fields | Epic 3 | 3.6 |
| FR26 | Explain WHY | Epic 3 | 3.7 |
| FR27 | Display in terminal menu | Epic 4 | 4.1 |
| FR28 | Navigate with arrows | Epic 4 | 4.2 |
| FR29 | Show explanation inline | Epic 4 | 4.1 |
| FR30 | Execute with Enter/A | Epic 4 | 4.3 |
| FR31 | Copy with K | Epic 4 | 4.4 |
| FR32 | Edit with B | Epic 4 | 4.5 |
| FR33 | Abort with Esc | Epic 4 | 4.6 |
| FR34 | Copy visual feedback | Epic 4 | 4.4 |
| FR35 | Run in user shell | Epic 4 | 4.3 |
| FR36 | Stream output | Epic 4 | 4.3 |
| FR37 | Line editor navigation | Epic 4 | 4.5 |
| FR38 | Ollama error messages | Epic 4 | 4.7 |
| FR39 | Missing manpage handling | Epic 4 | 4.7 |
| FR40 | No results message | Epic 4 | 4.7 |

**Validation:** ✅ All 40 FRs are covered by stories.

---

## Epic 5: Auto-Installation

**Goal:** Provide seamless Ollama installation as part of setup, eliminating manual installation steps.

**User Value:** User can go from zero to working ulm with a single `ulm setup` command.

**FRs Covered:** FR3 (enhanced), FR5 (enhanced)

---

### Story 5.1: Ollama Detection & Status

As a user,
I want setup to detect my current Ollama installation status,
So that it can guide me appropriately.

**Acceptance Criteria:**

**Given** user runs `ulm setup`
**When** checking Ollama status
**Then** system detects one of:
- Ollama running and accessible
- Ollama installed but not running
- Ollama not installed
- Docker available (alternative)

**And** reports current status clearly
**And** suggests appropriate next action

**Prerequisites:** None

**Technical Notes:**
- Check localhost:11434 for running Ollama
- Check `which ollama` for installed binary
- Check `which docker` for Docker availability
- Implement in setup/install.rs

---

### Story 5.2: Native Ollama Installation

As a user,
I want to install Ollama natively via the official installer,
So that I get optimal performance.

**Acceptance Criteria:**

**Given** Ollama not installed and user chooses native install
**When** installation proceeds
**Then** runs appropriate installer:
- Linux: `curl -fsSL https://ollama.com/install.sh | sh`
- macOS: `brew install ollama` or curl fallback

**And** requests sudo if needed (with explanation)
**And** verifies installation succeeded
**And** starts Ollama service
**And** reports success or failure with next steps

**Given** installation fails
**When** error occurs
**Then** provides clear error message
**And** suggests manual installation URL
**And** offers Docker as alternative

**Prerequisites:** Story 5.1

**Technical Notes:**
- Use std::process::Command for shell execution
- Detect OS via std::env::consts::OS
- Handle permission errors gracefully
- Timeout after 5 minutes

---

### Story 5.3: Docker Ollama Installation

As a user,
I want to run Ollama in Docker,
So that I can use it without system-wide installation.

**Acceptance Criteria:**

**Given** user chooses Docker installation
**When** Docker is available
**Then** runs:
```
docker run -d -v ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama
```

**And** waits for container to be healthy
**And** verifies API is accessible
**And** reports success with container info

**Given** Docker not available
**When** user chooses Docker option
**Then** reports Docker not found
**And** suggests Docker installation
**And** offers native install as alternative

**Given** container already exists
**When** installation proceeds
**Then** offers to restart existing container
**Or** remove and recreate

**Prerequisites:** Story 5.1

**Technical Notes:**
- Check docker daemon running
- Handle port conflicts (11434 in use)
- Volume mount for persistence
- Health check with timeout

---

## Summary

**Total Epics:** 5
**Total Stories:** 30

| Epic | Stories | FRs Covered |
|------|---------|-------------|
| Epic 1: Foundation | 4 | Infrastructure |
| Epic 2: Setup & Knowledge Base | 8 | FR1-12 |
| Epic 3: Query & Intelligence | 7 | FR13-26 |
| Epic 4: Interactive Experience | 8 | FR27-40 |
| Epic 5: Auto-Installation | 3 | FR3, FR5 (enhanced) |

**Context Incorporated:**
- ✅ PRD requirements (40 FRs)
- ✅ Architecture technical decisions
- ✅ v0.2.0 enhancement for seamless setup

**Next Steps:**
- Ready for Phase 4: Sprint Planning
- Use `sprint-planning` workflow to create sprint status tracking

---

_For implementation: Use the `dev-story` workflow to implement individual stories from this epic breakdown._
