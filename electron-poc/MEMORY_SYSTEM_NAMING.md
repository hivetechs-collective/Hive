# Memory System - Exact Naming Reference

## Database
**Path**: `~/.hive/hive-ai.db`
- NOT `~/.hive-consensus/consensus.db` ❌
- NOT `consensus.db` ❌

## Core Tables

### 1. `messages` Table
**Purpose**: Store all user questions and AI responses
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,                    -- 'user' or 'assistant'
    content TEXT NOT NULL,
    stage TEXT,
    model_used TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    cost REAL DEFAULT 0.0,
    consensus_path TEXT,                   -- 'SIMPLE' or 'COMPLEX'
    parent_message_id TEXT,
    metadata TEXT,
    tokens_used INTEGER DEFAULT 0,         -- Added by us
    consensus_rounds INTEGER,              -- Added by us
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
```

### 2. `memory_context_logs` Table
**Purpose**: Track memory retrieval and context building operations
```sql
CREATE TABLE memory_context_logs (
    log_id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    conversation_id TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    memory_stage_start TEXT,
    memory_stage_end TEXT,
    memory_stage_duration_ms INTEGER,
    memories_retrieved_recent INTEGER DEFAULT 0,    -- Count from 2-hour layer
    memories_retrieved_today INTEGER DEFAULT 0,     -- Count from 24-hour layer
    memories_retrieved_week INTEGER DEFAULT 0,      -- Count from 7-day layer
    memories_retrieved_semantic INTEGER DEFAULT 0,  -- Count from semantic search
    memory_search_query TEXT,
    memory_relevance_scores TEXT,
    context_stage_start TEXT,
    context_stage_end TEXT,
    context_stage_duration_ms INTEGER,
    patterns_identified TEXT,
    topics_extracted TEXT,
    context_summary TEXT,
    user_preferences_detected TEXT,
    context_influenced_routing BOOLEAN DEFAULT 0,
    routing_decision TEXT,                          -- 'SIMPLE' or 'COMPLEX'
    routing_confidence REAL,
    total_memory_bytes INTEGER,
    cache_hit_rate REAL,
    consensus_id TEXT,
    final_message_id TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);
```

### 3. `conversations` Table
**Purpose**: Track conversation sessions
```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT,
    total_cost REAL DEFAULT 0.0,
    total_tokens_input INTEGER DEFAULT 0,
    total_tokens_output INTEGER DEFAULT 0,
    -- other columns...
);
```

## Memory Retrieval Layers

### Layer Time Ranges (SQL datetime functions)
1. **RECENT**: `datetime(timestamp) > datetime('now', '-2 hours')`
2. **TODAY**: `datetime(timestamp) > datetime('now', '-24 hours') AND datetime(timestamp) <= datetime('now', '-2 hours')`
3. **WEEK**: `datetime(timestamp) > datetime('now', '-7 days') AND datetime(timestamp) <= datetime('now', '-24 hours')`
4. **SEMANTIC**: All older messages with keyword matching

## Code Files Using These Names

### 1. `src/database/OptimizedMemoryService.ts`
- Must use table: `messages`
- Must use database from constructor parameter (already connected to `hive-ai.db`)
- Inserts into `messages` table
- Queries from `messages` table for all 4 layers

### 2. `src/consensus/SimpleConsensusEngine.ts`
- Creates `OptimizedMemoryService` with database connection
- Generates `conversation_id` like: `conv_${Date.now()}_${randomString}`
- Stores messages via `optimizedMemory.storeMessage()`
- Retrieves memories via `optimizedMemory.retrieveMemories()`

### 3. `src/database/MemoryContextDatabase.ts`
- Also uses `messages` table
- Logs to `memory_context_logs` table

## Key ID Formats
- **message_id**: `msg_${Date.now()}_${randomString}`
- **conversation_id**: `conv_${Date.now()}_${randomString}`
- **log_id**: `memlog_${Date.now()}_${randomString}`

## Common Mistakes to Avoid
1. ❌ Using `~/.hive-consensus/consensus.db` instead of `~/.hive/hive-ai.db`
2. ❌ Creating a `conversations` record without proper fields
3. ❌ Not including all required columns in INSERT statements
4. ❌ Using wrong datetime functions for time-based queries
5. ❌ Forgetting that foreign keys are disabled (so missing conversations won't block inserts)

## Testing Queries
Always use the correct database:
```bash
sqlite3 ~/.hive/hive-ai.db "SELECT * FROM messages WHERE date(timestamp) = date('now');"
sqlite3 ~/.hive/hive-ai.db "SELECT * FROM memory_context_logs ORDER BY timestamp DESC LIMIT 5;"
```

## Current Issues (2025-09-05)
1. Messages are NOT being stored (storeMessage might be failing silently)
2. Memory retrieval returns 0 from all layers (because no messages exist)
3. Context can't be built without memories
4. Need to verify the exact error in storeMessage function