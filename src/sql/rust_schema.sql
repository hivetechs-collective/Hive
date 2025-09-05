-- Hive AI Rust Database Schema
-- Production-ready schema matching TypeScript implementation with enhancements
-- Compatible with existing TypeScript data while adding new Rust-specific optimizations

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;
PRAGMA temp_store = memory;
PRAGMA mmap_size = 268435456; -- 256MB

-- Conversations: Core question-answer pairs with metadata
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    question TEXT NOT NULL,                    -- Original user question
    final_answer TEXT NOT NULL,               -- Final pipeline result
    source_of_truth TEXT NOT NULL,            -- Curator output for knowledge retrieval
    conversation_context TEXT,                -- Any additional context
    profile_id TEXT,                          -- User/project profile
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_updated TEXT DEFAULT CURRENT_TIMESTAMP
);

-- PERFORMANCE INDEXES for timestamp-based queries
CREATE INDEX IF NOT EXISTS idx_conversations_created_at ON conversations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversations_source_search ON conversations(created_at DESC, source_of_truth);
CREATE INDEX IF NOT EXISTS idx_conversations_question_search ON conversations(created_at DESC, question);

-- Stage Outputs: Every word from every stage preserved forever
CREATE TABLE IF NOT EXISTS stage_outputs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    stage_name TEXT NOT NULL CHECK (stage_name IN ('generator', 'refiner', 'validator', 'curator')),
    stage_number INTEGER NOT NULL CHECK (stage_number IN (1, 2, 3, 4)),
    provider TEXT NOT NULL,                   -- Gemini, Grok, OpenAI, etc.
    model TEXT NOT NULL,                      -- Specific model used
    full_output TEXT NOT NULL,                -- Complete unedited AI response
    character_count INTEGER NOT NULL,
    word_count INTEGER NOT NULL,
    temperature REAL,
    processing_time_ms INTEGER,
    tokens_used INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- PERFORMANCE INDEXES for stage outputs queries
CREATE INDEX IF NOT EXISTS idx_stage_outputs_conversation ON stage_outputs(conversation_id, stage_name);
CREATE INDEX IF NOT EXISTS idx_stage_outputs_created_at ON stage_outputs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_stage_outputs_stage_time ON stage_outputs(stage_name, created_at DESC);

-- Knowledge Base: Curator outputs flagged as source of truth
CREATE TABLE IF NOT EXISTS knowledge_base (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    curator_content TEXT NOT NULL,            -- Full curator output - source of truth
    topics TEXT NOT NULL,                     -- JSON array of topics
    keywords TEXT NOT NULL,                   -- JSON array of keywords  
    semantic_embedding BLOB,                  -- Vector embedding for similarity search
    is_source_of_truth INTEGER DEFAULT 1,    -- Always 1 for curator outputs
    relevance_score REAL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Topics: Searchable topic extraction (TypeScript compatibility)
CREATE TABLE IF NOT EXISTS conversation_topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    weight REAL DEFAULT 1.0,                  -- Matches TypeScript schema
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Keywords: Full-text search support
CREATE TABLE IF NOT EXISTS conversation_keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    frequency INTEGER DEFAULT 1,
    weight REAL DEFAULT 1.0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Stage Comparisons: Track how content evolves through pipeline
CREATE TABLE IF NOT EXISTS stage_evolution (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    generator_length INTEGER,
    refiner_length INTEGER,
    validator_length INTEGER,
    curator_length INTEGER,
    total_evolution_percentage REAL,
    quality_score REAL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Conversation context tracking for cross-conversation memory
CREATE TABLE IF NOT EXISTS conversation_context (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    referenced_conversation_id TEXT NOT NULL,
    relevance_score REAL DEFAULT 1.0,
    context_type TEXT CHECK (context_type IN ('recent_24h', 'thematic', 'direct_reference')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (referenced_conversation_id) REFERENCES conversations(id)
);

-- Stage confidence and reasoning tracking
CREATE TABLE IF NOT EXISTS stage_confidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stage_output_id TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    reasoning TEXT,
    sources_used TEXT, -- JSON array of source conversation IDs
    content_quality_score REAL,
    technical_depth_score REAL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (stage_output_id) REFERENCES stage_outputs(id)
);

-- Consensus metrics for each conversation
CREATE TABLE IF NOT EXISTS consensus_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    final_confidence REAL NOT NULL,
    stage_agreement REAL,
    content_quality REAL,
    provider_reliability REAL,
    context_utilization REAL,
    agreement_matrix TEXT, -- JSON representation of agreement matrix
    processing_time_ms INTEGER,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Curator truths tracking (single source of truth)
CREATE TABLE IF NOT EXISTS curator_truths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    curator_output TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    topic_summary TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Knowledge conversations for authoritative answers (TypeScript compatibility)
CREATE TABLE IF NOT EXISTS knowledge_conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    question TEXT NOT NULL,
    final_answer TEXT NOT NULL,
    source_of_truth TEXT NOT NULL, -- Curator output
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Conversation threading for follow-up detection (TypeScript compatibility)
CREATE TABLE IF NOT EXISTS conversation_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    child_conversation_id TEXT NOT NULL,
    parent_conversation_id TEXT NOT NULL,
    thread_type TEXT CHECK (thread_type IN ('follow_up', 'clarification', 'list_reference', 'continuation')),
    reference_text TEXT,
    confidence_score REAL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (child_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (parent_conversation_id) REFERENCES conversations(id)
);

-- Performance indices for fast querying
CREATE INDEX IF NOT EXISTS idx_conversations_created ON conversations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversations_question ON conversations(question);
CREATE INDEX IF NOT EXISTS idx_stage_outputs_conversation ON stage_outputs(conversation_id);
CREATE INDEX IF NOT EXISTS idx_stage_outputs_stage ON stage_outputs(stage_name);
CREATE INDEX IF NOT EXISTS idx_knowledge_base_conversation ON knowledge_base(conversation_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_base_source ON knowledge_base(is_source_of_truth);
CREATE INDEX IF NOT EXISTS idx_topics_conversation ON conversation_topics(conversation_id);
CREATE INDEX IF NOT EXISTS idx_topics_topic ON conversation_topics(topic);
CREATE INDEX IF NOT EXISTS idx_keywords_conversation ON conversation_keywords(conversation_id);
CREATE INDEX IF NOT EXISTS idx_keywords_keyword ON conversation_keywords(keyword);

-- Indexes for enhanced features
CREATE INDEX IF NOT EXISTS idx_conversation_context_conv ON conversation_context(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_context_ref ON conversation_context(referenced_conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_context_type ON conversation_context(context_type);
CREATE INDEX IF NOT EXISTS idx_stage_confidence_stage ON stage_confidence(stage_output_id);
CREATE INDEX IF NOT EXISTS idx_consensus_metrics_conv ON consensus_metrics(conversation_id);
CREATE INDEX IF NOT EXISTS idx_consensus_metrics_confidence ON consensus_metrics(final_confidence DESC);
CREATE INDEX IF NOT EXISTS idx_curator_truths_conv ON curator_truths(conversation_id);
CREATE INDEX IF NOT EXISTS idx_curator_truths_topic ON curator_truths(topic_summary);
CREATE INDEX IF NOT EXISTS idx_curator_truths_confidence ON curator_truths(confidence_score DESC);
CREATE INDEX IF NOT EXISTS idx_knowledge_conversations_conv ON knowledge_conversations(conversation_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_conversations_created ON knowledge_conversations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_threads_child ON conversation_threads(child_conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_threads_parent ON conversation_threads(parent_conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversation_threads_type ON conversation_threads(thread_type);

-- Full-text search virtual table
CREATE VIRTUAL TABLE IF NOT EXISTS conversations_fts USING fts5(
    conversation_id,
    question,
    source_of_truth,
    content='knowledge_base',
    content_rowid='id'
);

-- Enhanced FTS table for curator truths
CREATE VIRTUAL TABLE IF NOT EXISTS curator_truths_fts USING fts5(
    conversation_id,
    curator_output,
    topic_summary,
    content='curator_truths',
    content_rowid='id'
);

-- Trigger to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS knowledge_base_fts_insert AFTER INSERT ON knowledge_base BEGIN
    INSERT INTO conversations_fts(conversation_id, question, source_of_truth) 
    SELECT NEW.conversation_id, c.question, NEW.curator_content 
    FROM conversations c WHERE c.id = NEW.conversation_id;
END;

-- Trigger to keep curator truths FTS in sync
CREATE TRIGGER IF NOT EXISTS curator_truths_fts_insert AFTER INSERT ON curator_truths BEGIN
    INSERT INTO curator_truths_fts(conversation_id, curator_output, topic_summary)
    VALUES (NEW.conversation_id, NEW.curator_output, NEW.topic_summary);
END;

-- Rust-specific enhancements for performance

-- Migration tracking table
CREATE TABLE IF NOT EXISTS migration_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    migration_version TEXT NOT NULL,
    typescript_source_version TEXT,
    migration_timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    migration_stats TEXT, -- JSON with migration statistics
    validation_checksum TEXT,
    notes TEXT
);

-- Performance monitoring table
CREATE TABLE IF NOT EXISTS performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,
    operation_duration_ms INTEGER NOT NULL,
    memory_usage_mb INTEGER,
    rows_processed INTEGER,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT -- JSON with additional metrics
);

-- User preferences and settings for Rust version
CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    setting_key TEXT NOT NULL UNIQUE,
    setting_value TEXT NOT NULL,
    setting_type TEXT CHECK (setting_type IN ('string', 'number', 'boolean', 'json')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Repository analysis cache (new Rust feature)
CREATE TABLE IF NOT EXISTS repository_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repository_path TEXT NOT NULL,
    analysis_type TEXT NOT NULL,
    analysis_result TEXT NOT NULL, -- JSON with analysis results
    file_count INTEGER,
    total_lines_of_code INTEGER,
    analysis_timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    analysis_duration_ms INTEGER,
    checksum TEXT, -- SHA256 of repository state
    UNIQUE(repository_path, analysis_type, checksum)
);

-- Code intelligence cache (new Rust feature)
CREATE TABLE IF NOT EXISTS code_intelligence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    file_language TEXT NOT NULL,
    symbols TEXT NOT NULL, -- JSON array of extracted symbols
    dependencies TEXT, -- JSON array of dependencies
    complexity_score REAL,
    last_modified TEXT,
    analysis_timestamp TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Planning session storage (new Rust feature)
CREATE TABLE IF NOT EXISTS planning_sessions (
    id TEXT PRIMARY KEY,
    session_name TEXT NOT NULL,
    project_context TEXT NOT NULL,
    task_breakdown TEXT NOT NULL, -- JSON with task hierarchy
    status TEXT CHECK (status IN ('active', 'completed', 'paused', 'cancelled')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    completion_percentage REAL DEFAULT 0.0
);

-- TUI state management (new Rust feature)
CREATE TABLE IF NOT EXISTS tui_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    state_key TEXT NOT NULL UNIQUE,
    state_value TEXT NOT NULL, -- JSON with TUI state
    last_updated TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Enterprise hooks configuration (new Rust feature)
CREATE TABLE IF NOT EXISTS enterprise_hooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hook_name TEXT NOT NULL UNIQUE,
    hook_type TEXT CHECK (hook_type IN ('pre_consensus', 'post_consensus', 'validation', 'approval')),
    hook_config TEXT NOT NULL, -- JSON with hook configuration
    is_enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Additional indexes for new features
CREATE INDEX IF NOT EXISTS idx_migration_history_version ON migration_history(migration_version);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_operation ON performance_metrics(operation_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_user_settings_key ON user_settings(setting_key);
CREATE INDEX IF NOT EXISTS idx_repository_analysis_path ON repository_analysis(repository_path);
CREATE INDEX IF NOT EXISTS idx_repository_analysis_type ON repository_analysis(analysis_type, analysis_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_code_intelligence_path ON code_intelligence(file_path);
CREATE INDEX IF NOT EXISTS idx_code_intelligence_language ON code_intelligence(file_language);
CREATE INDEX IF NOT EXISTS idx_planning_sessions_status ON planning_sessions(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_enterprise_hooks_type ON enterprise_hooks(hook_type, is_enabled);