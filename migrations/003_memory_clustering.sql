-- @ version: 003
-- @ name: memory_clustering
-- @ description: Create memory clustering and improvement pattern tracking tables
-- @ rollback: DROP TABLE IF EXISTS conversation_threads; DROP TABLE IF EXISTS improvement_patterns;

-- Improvement patterns table for tracking what each stage typically improves
CREATE TABLE improvement_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    from_stage TEXT NOT NULL,
    to_stage TEXT NOT NULL,
    improvement_type TEXT NOT NULL,
    description TEXT NOT NULL,
    question_type TEXT NOT NULL,
    before_content TEXT,
    after_content TEXT,
    improvement_score REAL,
    meta_commentary TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Conversation thread tracking for follow-up detection
CREATE TABLE conversation_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    child_conversation_id TEXT NOT NULL,
    parent_conversation_id TEXT NOT NULL,
    thread_type TEXT CHECK (thread_type IN ('follow_up', 'clarification', 'list_reference', 'continuation')),
    reference_text TEXT, -- The text that indicated this was a follow-up
    confidence_score REAL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (child_conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (parent_conversation_id) REFERENCES conversations(id)
);