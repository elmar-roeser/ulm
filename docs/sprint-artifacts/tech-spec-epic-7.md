# Epic 7: Multi-Model Support

**Goal:** Enable separate configuration of embedding and LLM models with automatic index validation.

**User Value:** Users can optimize performance by using specialized embedding models for search and powerful LLM models for response generation.

**Bug Fix:** Resolves dimension mismatch error when model changes between setup and query.

---

## Stories

### Story 7.1: Config Migration to Multi-Model Structure

As a user,
I want my config to support separate embedding and LLM models,
So that I can optimize each for its purpose.

**Acceptance Criteria:**

**Given** existing config with single `model_name`
**When** ulm starts
**Then** migrates to new structure:
```toml
[models]
embedding_model = "nomic-embed-text"
llm_model = "llama3.1:8b"

[ollama]
url = "http://localhost:11434"

[index]
embedding_dimension = 768
last_model = "nomic-embed-text"
```

**And** preserves existing model as both embedding and LLM (backward compatible)
**And** stores embedding dimension used for index

**Prerequisites:** None

**Technical Notes:**
- Implement in setup/config.rs
- Version config schema
- Auto-migrate on first run

---

### Story 7.2: Embedding Model Selection in Setup

As a user,
I want to select an embedding model during setup,
So that I get optimal search performance.

**Acceptance Criteria:**

**Given** user runs `ulm setup`
**When** model selection step
**Then** shows embedding-specific models:
- nomic-embed-text (768-dim, fast, recommended)
- mxbai-embed-large (1024-dim, accurate)
- all-minilm (384-dim, very fast)
- llama3.1:8b (4096-dim, general purpose)

**And** displays dimension and speed characteristics
**And** recommends specialized embedding model
**And** pulls model if not installed

**Prerequisites:** Story 7.1

**Technical Notes:**
- Separate from LLM model selection
- Show which models are already installed
- Warn about index rebuild if changing

---

### Story 7.3: LLM Model Selection in Setup

As a user,
I want to select an LLM model for response generation,
So that I get high-quality command suggestions.

**Acceptance Criteria:**

**Given** embedding model selected
**When** LLM selection step
**Then** shows LLM models:
- llama3.2:3b (~4GB RAM, fast)
- mistral:7b (~6GB RAM, balanced)
- llama3.1:8b (~8GB RAM, quality)
- phi3:mini (~3GB RAM, lightweight)

**And** can choose same model as embedding (if capable)
**And** recommends based on available RAM
**And** pulls model if not installed

**Prerequisites:** Story 7.2

**Technical Notes:**
- Filter out pure embedding models
- Show RAM requirements
- Allow "use same as embedding" option

---

### Story 7.4: Index Dimension Validation

As a user,
I want ulm to detect when my embedding model changed,
So that I don't get dimension mismatch errors.

**Acceptance Criteria:**

**Given** config has `last_model` and `embedding_dimension`
**When** user runs query
**Then** checks if current embedding model matches stored model

**Given** model mismatch detected
**When** query starts
**Then** displays clear error:
"Index was built with 'nomic-embed-text' (768-dim) but config uses 'llama3.1:8b' (4096-dim).
Run 'ulm setup' to rebuild index or change embedding_model in config."

**And** suggests specific fix
**And** does not attempt search (would fail)

**Given** user runs `ulm update`
**When** embedding model differs from index
**Then** warns and asks to confirm rebuild

**Prerequisites:** Story 7.1

**Technical Notes:**
- Check dimension at query start
- Store model name in LanceDB metadata or config
- Clear error messaging

---

### Story 7.5: Recommended Model Presets

As a user,
I want preset configurations for common use cases,
So that I can quickly choose optimal settings.

**Acceptance Criteria:**

**Given** user runs `ulm setup`
**When** asked for model configuration
**Then** offers presets:

1. **Fast** (recommended for most users)
   - Embedding: nomic-embed-text
   - LLM: llama3.2:3b

2. **Balanced**
   - Embedding: mxbai-embed-large
   - LLM: mistral:7b

3. **Quality**
   - Embedding: mxbai-embed-large
   - LLM: llama3.1:8b

4. **Custom** - manual selection

**And** shows total RAM requirement for preset
**And** allows switching between presets and custom

**Prerequisites:** Stories 7.2, 7.3

**Technical Notes:**
- Display as numbered menu
- Show which models need download
- Calculate combined RAM usage

---

## Summary

**Total Stories:** 5

| Story | Description | Bug Fix |
|-------|-------------|---------|
| 7.1 | Config Migration | Partial - adds dimension tracking |
| 7.2 | Embedding Model Selection | - |
| 7.3 | LLM Model Selection | - |
| 7.4 | Index Dimension Validation | **Yes** - prevents mismatch error |
| 7.5 | Recommended Model Presets | - |

**Dependencies:**
- Stories 7.2 and 7.3 depend on 7.1
- Story 7.5 depends on 7.2 and 7.3
- Story 7.4 can be done after 7.1

**Recommended Order:** 7.1 → 7.4 → 7.2 → 7.3 → 7.5

---

_Implementation: Use the `dev-story` workflow for each story._
