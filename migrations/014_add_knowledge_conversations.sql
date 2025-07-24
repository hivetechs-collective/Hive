-- Migration to add knowledge_conversations table for storing Claude Code curator outputs
-- This table stores authoritative knowledge from consensus-validated Claude executions

CREATE TABLE IF NOT EXISTS knowledge_conversations (
    id TEXT PRIMARY KEY,
    query TEXT NOT NULL,
    claude_plan TEXT NOT NULL,
    consensus_evaluation TEXT NOT NULL, -- JSON containing full consensus result
    curator_output TEXT NOT NULL,       -- The authoritative answer from curator
    execution_result TEXT NOT NULL,     -- JSON containing execution outcome
    created_at TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    
    -- Metadata for search and analytics
    execution_mode TEXT DEFAULT 'ConsensusAssisted', -- Direct, ConsensusAssisted, ConsensusRequired
    confidence_score REAL DEFAULT 0.0,
    tokens_used INTEGER DEFAULT 0,
    cost REAL DEFAULT 0.0,
    
    -- Tags and categorization for knowledge retrieval
    tags TEXT,                          -- JSON array of tags
    category TEXT,                      -- General category (architecture, refactoring, etc.)
    
    -- Track which consensus profile was used
    profile_id TEXT,
    
    -- Enable full-text search
    search_content TEXT GENERATED ALWAYS AS (
        query || ' ' || curator_output
    ) STORED
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_knowledge_created_at ON knowledge_conversations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_knowledge_category ON knowledge_conversations(category);
CREATE INDEX IF NOT EXISTS idx_knowledge_profile ON knowledge_conversations(profile_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_mode ON knowledge_conversations(execution_mode);

-- Enable FTS5 full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_conversations_fts USING fts5(
    id UNINDEXED,
    query,
    curator_output,
    content=knowledge_conversations,
    content_rowid=rowid
);

-- Trigger to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS knowledge_conversations_ai AFTER INSERT ON knowledge_conversations BEGIN
    INSERT INTO knowledge_conversations_fts(rowid, query, curator_output)
    VALUES (new.rowid, new.query, new.curator_output);
END;

CREATE TRIGGER IF NOT EXISTS knowledge_conversations_ad AFTER DELETE ON knowledge_conversations BEGIN
    DELETE FROM knowledge_conversations_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS knowledge_conversations_au AFTER UPDATE ON knowledge_conversations BEGIN
    DELETE FROM knowledge_conversations_fts WHERE rowid = old.rowid;
    INSERT INTO knowledge_conversations_fts(rowid, query, curator_output)
    VALUES (new.rowid, new.query, new.curator_output);
END;