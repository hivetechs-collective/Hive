-- Memory-Context Enhancement Migration
-- This migration enhances the database for complete Memory → Context → Consensus → Storage loop

-- 1. Add missing columns to messages table (if they don't exist)
-- Using ALTER TABLE carefully to maintain ACID compliance

-- Add columns for tracking consensus details
ALTER TABLE messages ADD COLUMN tokens_used INTEGER DEFAULT 0;
ALTER TABLE messages ADD COLUMN cost REAL DEFAULT 0.0;
ALTER TABLE messages ADD COLUMN consensus_path TEXT; -- 'SIMPLE' or 'COMPLEX'
ALTER TABLE messages ADD COLUMN consensus_rounds INTEGER DEFAULT 1;
ALTER TABLE messages ADD COLUMN parent_message_id TEXT;
ALTER TABLE messages ADD COLUMN metadata TEXT; -- JSON for additional data

-- 2. Create indexes for efficient memory retrieval
-- These support our layered retrieval strategy

-- Index for timestamp-based retrieval (most recent first)
CREATE INDEX IF NOT EXISTS idx_messages_timestamp 
    ON messages(timestamp DESC);

-- Index for conversation continuity
CREATE INDEX IF NOT EXISTS idx_messages_conversation_timestamp 
    ON messages(conversation_id, timestamp DESC);

-- Index for parent-child message relationships
CREATE INDEX IF NOT EXISTS idx_messages_parent 
    ON messages(parent_message_id);

-- Full-text search index for semantic retrieval
-- Note: SQLite FTS5 would be better, but using LIKE for compatibility
CREATE INDEX IF NOT EXISTS idx_messages_content_search 
    ON messages(content);

-- 3. Create memory_context_logs table for tracking Memory and Context stages
CREATE TABLE IF NOT EXISTS memory_context_logs (
    log_id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL,
    conversation_id TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    
    -- Memory Stage Metrics
    memory_stage_start TEXT,
    memory_stage_end TEXT,
    memory_stage_duration_ms INTEGER,
    memories_retrieved_recent INTEGER DEFAULT 0,  -- Last 2 hours
    memories_retrieved_today INTEGER DEFAULT 0,   -- Last 24 hours
    memories_retrieved_week INTEGER DEFAULT 0,    -- Last 7 days
    memories_retrieved_semantic INTEGER DEFAULT 0, -- All time semantic
    memory_search_query TEXT,
    memory_relevance_scores TEXT, -- JSON array of scores
    
    -- Context Stage Metrics
    context_stage_start TEXT,
    context_stage_end TEXT,
    context_stage_duration_ms INTEGER,
    patterns_identified TEXT, -- JSON array
    topics_extracted TEXT, -- JSON array
    context_summary TEXT,
    user_preferences_detected TEXT, -- JSON array
    
    -- Routing Impact
    context_influenced_routing BOOLEAN DEFAULT 0,
    routing_decision TEXT, -- 'SIMPLE' or 'COMPLEX'
    routing_confidence REAL,
    
    -- Performance Metrics
    total_memory_bytes INTEGER,
    cache_hit_rate REAL,
    
    -- Links
    consensus_id TEXT,
    final_message_id TEXT, -- Links to the message created from this consensus
    
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- 4. Create indexes for memory_context_logs
CREATE INDEX IF NOT EXISTS idx_memory_context_logs_request 
    ON memory_context_logs(request_id);

CREATE INDEX IF NOT EXISTS idx_memory_context_logs_conversation 
    ON memory_context_logs(conversation_id);

CREATE INDEX IF NOT EXISTS idx_memory_context_logs_timestamp 
    ON memory_context_logs(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_memory_context_logs_consensus 
    ON memory_context_logs(consensus_id);

-- 5. Create a view for easy memory retrieval with relevance
CREATE VIEW IF NOT EXISTS memory_retrieval_view AS
SELECT 
    m.id,
    m.conversation_id,
    m.role,
    m.content,
    m.timestamp,
    m.tokens_used,
    m.cost,
    m.consensus_path,
    c.title as conversation_title,
    c.context_type,
    -- Calculate recency score (higher for more recent)
    CASE 
        WHEN datetime(m.timestamp) > datetime('now', '-2 hours') THEN 4  -- Ongoing
        WHEN datetime(m.timestamp) > datetime('now', '-24 hours') THEN 3 -- Recent
        WHEN datetime(m.timestamp) > datetime('now', '-7 days') THEN 2   -- This week
        ELSE 1  -- Older
    END as recency_score
FROM messages m
LEFT JOIN conversations c ON m.conversation_id = c.id
WHERE m.role IN ('user', 'assistant');

-- 6. Create a trigger to maintain parent-child relationships
CREATE TRIGGER IF NOT EXISTS maintain_message_chain
AFTER INSERT ON messages
FOR EACH ROW
WHEN NEW.role = 'assistant'
BEGIN
    UPDATE messages 
    SET parent_message_id = (
        SELECT id FROM messages 
        WHERE conversation_id = NEW.conversation_id 
        AND role = 'user'
        AND timestamp < NEW.timestamp
        ORDER BY timestamp DESC 
        LIMIT 1
    )
    WHERE id = NEW.id;
END;

-- 7. Create a table for tracking conversation continuity
CREATE TABLE IF NOT EXISTS conversation_continuity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    current_conversation_id TEXT NOT NULL,
    previous_conversation_id TEXT,
    continuity_score REAL DEFAULT 0.0,
    time_gap_minutes INTEGER,
    topic_similarity REAL DEFAULT 0.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (current_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (previous_conversation_id) REFERENCES conversations(id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_continuity_current 
    ON conversation_continuity(current_conversation_id);

-- 8. Add a configuration for Memory-Context settings
INSERT OR IGNORE INTO configurations (key, value, encrypted) 
VALUES 
    ('memory_search_limit', '20', 0),
    ('memory_recency_weight', '0.6', 0),
    ('memory_semantic_weight', '0.4', 0),
    ('context_pattern_threshold', '0.3', 0),
    ('memory_context_enabled', 'true', 0);