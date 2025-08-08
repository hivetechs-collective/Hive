# Tauri Migration Plan - Preserving Existing Codebase

## Core Principle
**Replace ONLY the Dioxus UI layer with Tauri + Web UI. Keep ALL existing business logic, database, and features intact.**

## What We Keep (100% Unchanged)
1. **Database System** (`src/core/database.rs`)
   - DatabaseManager with connection pooling
   - WAL mode SQLite configuration  
   - All existing schemas and migrations
   - Global `get_database()` instance

2. **Consensus Engine** (`src/consensus/`)
   - 4-stage pipeline (Generator → Refiner → Validator → Curator)
   - OpenRouter integration
   - Streaming callbacks
   - Repository context
   - Cancellation tokens
   - AI operation parser

3. **Analytics System** (`src/analytics/`)
   - Cost intelligence
   - Performance tracking
   - Executive dashboards
   - ML models
   - Export functionality

4. **Core Features** (`src/core/`)
   - API key management (`api_keys.rs`)
   - Profile system (`profiles.rs`)
   - Memory system (`memory.rs`)
   - Security & trust (`security.rs`)
   - Temporal awareness (`temporal.rs`)
   - License management (`license.rs`)
   - Auto-updater (`updater.rs`)

5. **Analysis System** (`src/analysis/`)
   - Repository intelligence
   - Symbol indexing
   - Dependency analysis
   - Language detection
   - Performance analysis

6. **Desktop Features** (`src/desktop/`)
   - Terminal implementation (PTY-based)
   - Git integration (LazyGit)
   - Markdown rendering
   - File explorer logic
   - Model browser
   - Response coordinator

## What We Replace (Dioxus → Tauri)

### Old Stack (Remove)
```
src/bin/hive-consensus.rs (Dioxus app)
src/desktop/*.rs (Dioxus components)
dioxus-specific dependencies
```

### New Stack (Add)
```
src-tauri/ (Tauri backend)
  ├── src/
  │   ├── main.rs (Tauri entry point)
  │   ├── commands.rs (Tauri command handlers)
  │   └── bridge.rs (Connect to existing code)
  └── Cargo.toml
frontend/ (Web UI)
  ├── index.html
  ├── main.js
  └── styles.css
```

## Next Immediate Actions

1. **Delete** `src-tauri/src/state.rs` (duplicate database)
2. **Delete** duplicate command implementations
3. **Create** `src-tauri/src/bridge.rs` module
4. **Import** `hive_ai` crate in `src-tauri/Cargo.toml`
5. **Connect** to existing `get_database()`
6. **Wire** existing ConsensusEngine
7. **Test** with real consensus query

This approach ensures we keep 100% of our existing, working code and only replace the UI layer with Tauri.
