# Epic Technical Specification: Query & Intelligence

Date: 2025-11-21
Author: Elmar Röser
Epic ID: 3
Status: Draft

---

## Overview

Epic 3 implements the intelligent query processing pipeline that transforms natural language questions into contextual command suggestions. It combines vector similarity search, directory context awareness, and LLM-powered response generation to provide accurate, relevant Unix command suggestions.

This epic builds on the knowledge base from Epic 2 to deliver the core intelligence of ulm. It covers FR13-26 from the PRD, focusing on semantic search, context building, and structured LLM responses.

## Objectives and Scope

### In Scope

- Vector similarity search against manpage embeddings
- Full manpage content loading for matched tools
- Directory context scanning (project type detection)
- Context builder for LLM prompts
- Structured prompt construction
- LLM response parsing into typed suggestions
- Query orchestration pipeline

### Out of Scope

- TUI rendering (Epic 4)
- Command execution (Epic 4)
- Multiple model support (future)
- Conversation history (future)

## System Architecture Alignment

This epic implements components from the architecture document:

**Modules:**
- `query/search.rs` - Vector search and manpage loading
- `query/context.rs` - Directory scanning and context building
- `llm/prompt.rs` - Prompt construction
- `llm/response.rs` - Response parsing
- `query/mod.rs` - Query orchestration

**Key Decisions:**
- ADR-001: Use LanceDB vector search
- ADR-003: JSON format for LLM responses

**Integration Points:**
- LanceDB for vector search
- Ollama for embeddings and generation
- Filesystem for context detection

---

## Detailed Design

### Services and Modules

| Module | Responsibility | Inputs | Outputs |
|--------|---------------|--------|---------|
| `query/search.rs` | Vector search & manpage loading | Query string | Matched tools, content |
| `query/context.rs` | Directory context detection | Current directory | ProjectContext |
| `llm/prompt.rs` | Prompt construction | Query, context, manpage | Full prompt string |
| `llm/response.rs` | Response parsing | LLM JSON output | Vec<CommandSuggestion> |
| `query/mod.rs` | Query orchestration | User query | Vec<CommandSuggestion> |

### Data Models and Contracts

```rust
// query/search.rs
pub struct SearchMatch {
    pub tool_name: String,
    pub section: String,
    pub description: String,
    pub score: f32,
}

// query/context.rs
#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    CMake,
    Git,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DirectoryContext {
    pub project_type: Option<ProjectType>,
    pub marker_files: Vec<String>,
    pub cwd: PathBuf,
}

// llm/response.rs
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
```

### APIs and Interfaces

```rust
// query/search.rs
pub async fn search_tools(query: &str, limit: usize) -> Result<Vec<SearchMatch>>;
pub fn load_manpage_content(tool_name: &str) -> Result<String>;

// query/context.rs
pub fn scan_directory_context() -> Result<DirectoryContext>;
impl DirectoryContext {
    pub fn format_for_prompt(&self) -> String;
}

// llm/prompt.rs
pub fn build_prompt(
    query: &str,
    manpage_content: &str,
    context: &DirectoryContext,
) -> String;

// llm/response.rs
pub fn parse_suggestions(response: &str) -> Result<Vec<CommandSuggestion>>;

// query/mod.rs
pub async fn process_query(query: &str) -> Result<Vec<CommandSuggestion>>;
```

### Workflows and Sequencing

**Query Flow:**

```
1. User runs `ulm "find large files"`
   │
2. Generate query embedding
   └── Ollama /api/embeddings
   │
3. Vector search
   └── LanceDB nearest neighbors → top 3 tools
   │
4. Load manpage content
   └── `man -P cat <tool>` → full content
   │
5. Scan directory context
   └── Check for Cargo.toml, package.json, etc.
   │
6. Build prompt
   └── System instructions + manpage + context + query
   │
7. Call Ollama generate
   └── /api/generate with JSON format
   │
8. Parse response
   └── JSON → Vec<CommandSuggestion>
   │
9. Return suggestions to TUI (Epic 4)
```

---

## Non-Functional Requirements

### Performance

| Metric | Target | Strategy |
|--------|--------|----------|
| Query embedding | < 500ms | Ollama local inference |
| Vector search | < 100ms | LanceDB ANN search |
| Manpage load | < 100ms | Shell out to man |
| Context scan | < 50ms | Top-level only |
| LLM generation | < 3s | Ollama local inference |
| Total query | < 5s | Pipeline optimization |

### Security

- **No external network**: All queries local to Ollama
- **Read-only context**: Only reads directory markers
- **No execution**: Commands not executed in this epic

### Reliability/Availability

- **Graceful degradation**: Return partial results if possible
- **Retry logic**: 3 attempts for Ollama calls
- **Clear errors**: Actionable error messages

### Observability

- **Logging**: tracing with timing for each step
- **Debug mode**: RUST_LOG=ulm=debug for details
- **Metrics**: Log search scores, response times

---

## Dependencies and Integrations

### Rust Dependencies

Already in Cargo.toml from previous epics:
- lancedb, arrow-array (vector search)
- reqwest (Ollama API)
- serde, serde_json (JSON parsing)
- tokio (async runtime)
- tracing (logging)

### System Dependencies

- Ollama running at localhost:11434
- man command available
- LanceDB index created (Epic 2)

---

## Acceptance Criteria (Authoritative)

### Story 3.1: Vector Search
1. AC3.1.1: Generate embedding for query via Ollama
2. AC3.1.2: Search LanceDB for nearest neighbors
3. AC3.1.3: Return top 3 matches with scores
4. AC3.1.4: Search completes in < 100ms
5. AC3.1.5: Handle empty index gracefully

### Story 3.2: Manpage Content Loading
1. AC3.2.1: Run `man -P cat <tool>` for matched tools
2. AC3.2.2: Capture full output as string
3. AC3.2.3: Clean escape sequences
4. AC3.2.4: Truncate to ~8000 chars for LLM context
5. AC3.2.5: Handle missing manpages with error

### Story 3.3: Directory Context Scanner
1. AC3.3.1: Detect Cargo.toml → Rust
2. AC3.3.2: Detect package.json → Node
3. AC3.3.3: Detect requirements.txt/pyproject.toml → Python
4. AC3.3.4: Detect go.mod → Go
5. AC3.3.5: Detect .git → Git repository
6. AC3.3.6: Scan only top-level (no recursion)

### Story 3.4: Context Builder
1. AC3.4.1: Create DirectoryContext struct
2. AC3.4.2: Format context for prompt inclusion
3. AC3.4.3: Handle no project type gracefully
4. AC3.4.4: Limit marker list to 20 items

### Story 3.5: Prompt Builder
1. AC3.5.1: Include system instructions
2. AC3.5.2: Include manpage content
3. AC3.5.3: Include directory context
4. AC3.5.4: Include user query
5. AC3.5.5: Request JSON output format
6. AC3.5.6: Total prompt < 12000 tokens

### Story 3.6: LLM Response Parser
1. AC3.6.1: Deserialize JSON to Vec<CommandSuggestion>
2. AC3.6.2: Each suggestion has command, title, explanation, risk_level
3. AC3.6.3: Handle malformed JSON with error
4. AC3.6.4: Validate command is not empty
5. AC3.6.5: Default risk_level to Safe if missing

### Story 3.7: Query Orchestration
1. AC3.7.1: Orchestrate full query pipeline
2. AC3.7.2: Return Vec<CommandSuggestion>
3. AC3.7.3: Total latency < 5 seconds
4. AC3.7.4: Handle "no matching tools" error
5. AC3.7.5: Include explanations of WHY

---

## Traceability Mapping

| AC | Spec Section | Component | Test Idea |
|----|-------------|-----------|-----------|
| AC3.1.1 | APIs | search.rs | Mock Ollama embedding response |
| AC3.1.2 | APIs | search.rs | Test with populated index |
| AC3.2.1 | Workflows | search.rs | Test man command execution |
| AC3.3.1-5 | Data Models | context.rs | Test with temp directories |
| AC3.4.2 | APIs | context.rs | Test format output |
| AC3.5.1-5 | Workflows | prompt.rs | Test prompt structure |
| AC3.6.1 | Data Models | response.rs | Test JSON parsing |
| AC3.7.1 | Workflows | query/mod.rs | Integration test |

---

## Risks, Assumptions, Open Questions

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLM returns invalid JSON | Parse failure | Retry with different prompt |
| No matching tools | Empty results | Clear error message |
| Manpage too long | Token limit | Truncate with priority sections |
| Slow LLM response | Poor UX | Show loading indicator |

### Assumptions

- Index exists and is populated (Epic 2 complete)
- Ollama is running and model available
- User has read access to manpages
- JSON format works reliably with model

### Open Questions

- **Q1:** How many suggestions to return? → A: 1-3, start with 1 for MVP
- **Q2:** Similarity threshold? → A: 0.7 default
- **Q3:** Which sections to prioritize in manpage? → A: NAME, SYNOPSIS, OPTIONS

---

## Test Strategy Summary

### Test Levels

| Level | Scope | Tools |
|-------|-------|-------|
| Unit | Individual functions | cargo test, mockall |
| Integration | Full query pipeline | cargo test --test |
| E2E | Real Ollama | Manual testing |

### Test Coverage Targets

- **Unit tests**: 80% coverage for query/, llm/
- **Integration tests**: Full query flow with mocked Ollama
- **Edge cases**: Empty index, no context, malformed JSON

### Key Test Scenarios

1. **Happy path**: Query with matching tool
2. **No matches**: Return appropriate error
3. **Multiple matches**: Return ranked results
4. **Context detection**: Each project type
5. **JSON parsing**: Valid and invalid responses
6. **Manpage loading**: Success and failure cases
