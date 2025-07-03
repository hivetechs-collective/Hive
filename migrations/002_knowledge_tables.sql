-- @ version: 002
-- @ name: knowledge_tables
-- @ description: Create knowledge database tables for conversation memory and analysis
-- @ rollback: DROP TABLE IF EXISTS conversation_keywords; DROP TABLE IF EXISTS conversation_topics; DROP TABLE IF EXISTS conversation_context; DROP TABLE IF EXISTS curator_truths; DROP TABLE IF EXISTS knowledge_conversations;

-- Knowledge Database Tables (from hive-ai-knowledge.db)
-- Extended conversation data with questions, answers, and source-of-truth
CREATE TABLE knowledge_conversations (
    id TEXT PRIMARY KEY,
    conversation_id TEXT,  -- Links to main conversations table
    question TEXT NOT NULL,
    final_answer TEXT NOT NULL,
    source_of_truth TEXT NOT NULL,
    conversation_context TEXT,
    profile_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    last_updated TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

CREATE TABLE curator_truths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    curator_output TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    topic_summary TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

CREATE TABLE conversation_context (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    referenced_conversation_id TEXT NOT NULL,
    relevance_score REAL DEFAULT 1.0,
    context_type TEXT CHECK (context_type IN ('recent_24h', 'thematic', 'direct_reference')),
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (referenced_conversation_id) REFERENCES conversations(id)
);

CREATE TABLE conversation_topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    relevance_score REAL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

CREATE TABLE conversation_keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    frequency INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);