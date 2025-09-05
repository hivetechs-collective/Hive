-- Migration: Add consensus_facts table for AI context enhancement
-- This table stores curator-validated facts with semantic fingerprinting

CREATE TABLE IF NOT EXISTS consensus_facts (
    id TEXT PRIMARY KEY,
    semantic_fingerprint TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL,
    curator_confidence REAL NOT NULL DEFAULT 0.0,
    source_question TEXT NOT NULL,
    source_conversation_id TEXT,
    consensus_stages TEXT NOT NULL DEFAULT '[]', -- JSON array of StageContribution
    created_at TEXT NOT NULL,
    last_accessed TEXT NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    related_facts TEXT NOT NULL DEFAULT '[]', -- JSON array of fact IDs
    topics TEXT NOT NULL DEFAULT '[]', -- JSON array of topic strings
    entities TEXT NOT NULL DEFAULT '[]', -- JSON array of Entity objects
    metadata TEXT NOT NULL DEFAULT '{}' -- JSON object for FactMetadata
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_consensus_facts_fingerprint ON consensus_facts(semantic_fingerprint);
CREATE INDEX IF NOT EXISTS idx_consensus_facts_created_at ON consensus_facts(created_at);
CREATE INDEX IF NOT EXISTS idx_consensus_facts_topics ON consensus_facts(topics);
CREATE INDEX IF NOT EXISTS idx_consensus_facts_access_count ON consensus_facts(access_count);
CREATE INDEX IF NOT EXISTS idx_consensus_facts_confidence ON consensus_facts(curator_confidence);