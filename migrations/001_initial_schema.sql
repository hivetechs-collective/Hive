-- @ version: 001
-- @ name: initial_schema
-- @ description: Create initial database schema with all core tables
-- @ rollback: DROP TABLE IF EXISTS activity_log; DROP TABLE IF EXISTS performance_metrics; DROP TABLE IF EXISTS model_selection_history; DROP TABLE IF EXISTS profile_templates; DROP TABLE IF EXISTS feature_usage; DROP TABLE IF EXISTS cost_analytics; DROP TABLE IF EXISTS consensus_metrics; DROP TABLE IF EXISTS model_rankings; DROP TABLE IF EXISTS provider_performance; DROP TABLE IF EXISTS pending_sync; DROP TABLE IF EXISTS conversation_threads; DROP TABLE IF EXISTS improvement_patterns; DROP TABLE IF EXISTS conversation_keywords; DROP TABLE IF EXISTS conversation_topics; DROP TABLE IF EXISTS conversation_context; DROP TABLE IF EXISTS curator_truths; DROP TABLE IF EXISTS knowledge_conversations; DROP TABLE IF EXISTS consensus_settings; DROP TABLE IF EXISTS conversation_usage; DROP TABLE IF EXISTS settings; DROP TABLE IF EXISTS sync_metadata; DROP TABLE IF EXISTS budget_limits; DROP TABLE IF EXISTS usage_records; DROP TABLE IF EXISTS messages; DROP TABLE IF EXISTS conversations; DROP TABLE IF EXISTS consensus_profiles; DROP TABLE IF EXISTS pipeline_profiles; DROP TABLE IF EXISTS openrouter_models; DROP TABLE IF EXISTS openrouter_providers; DROP TABLE IF EXISTS configurations; DROP TABLE IF EXISTS users;

-- Users and authentication
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE,
    license_key TEXT,
    tier TEXT DEFAULT 'FREE',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Configuration storage (API keys, settings)
CREATE TABLE configurations (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    encrypted BOOLEAN DEFAULT 0,
    user_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- OpenRouter providers (consolidated from openrouter-data.db)
CREATE TABLE openrouter_providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    model_count INTEGER DEFAULT 0,
    average_cost REAL DEFAULT 0.0,
    capabilities TEXT DEFAULT '[]',
    last_updated TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- OpenRouter models (consolidated from openrouter-data.db)
CREATE TABLE openrouter_models (
    internal_id INTEGER PRIMARY KEY AUTOINCREMENT, -- Stable rowid, NEVER changes
    openrouter_id TEXT NOT NULL UNIQUE,            -- Current OpenRouter model ID (can change)
    name TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    description TEXT DEFAULT '',
    capabilities TEXT DEFAULT '[]',
    pricing_input REAL DEFAULT 0.0,
    pricing_output REAL DEFAULT 0.0,
    pricing_image REAL,
    pricing_request REAL,
    context_window INTEGER DEFAULT 4096,
    created_at INTEGER NOT NULL,
    input_modalities TEXT DEFAULT '["text"]',
    output_modalities TEXT DEFAULT '["text"]',
    is_active BOOLEAN DEFAULT 1,
    last_updated TEXT NOT NULL,
    FOREIGN KEY (provider_id) REFERENCES openrouter_providers(id)
);

-- Pipeline profiles (consolidated from secure-pipeline.db)
CREATE TABLE pipeline_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    user_id TEXT,
    is_default BOOLEAN DEFAULT 0,
    -- New stable foreign key references (bulletproof against model ID changes)
    generator_model_internal_id INTEGER NOT NULL,
    generator_temperature REAL DEFAULT 0.7,
    refiner_model_internal_id INTEGER NOT NULL,
    refiner_temperature REAL DEFAULT 0.7,
    validator_model_internal_id INTEGER NOT NULL,
    validator_temperature REAL DEFAULT 0.7,
    curator_model_internal_id INTEGER NOT NULL,
    curator_temperature REAL DEFAULT 0.7,
    -- Legacy columns for migration (will be removed after migration)
    generator_provider TEXT,
    generator_model TEXT,
    refiner_provider TEXT,
    refiner_model TEXT,
    validator_provider TEXT,
    validator_model TEXT,
    curator_provider TEXT,
    curator_model TEXT,
    -- Dynamic template columns
    is_dynamic_template BOOLEAN DEFAULT 0,
    selection_criteria TEXT,
    routing_preferences TEXT,
    cost_controls TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (generator_model_internal_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (refiner_model_internal_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (validator_model_internal_id) REFERENCES openrouter_models(internal_id),
    FOREIGN KEY (curator_model_internal_id) REFERENCES openrouter_models(internal_id)
);

-- Consensus profiles (existing from hive-ai-context.db)
CREATE TABLE consensus_profiles (
    id TEXT PRIMARY KEY,
    profile_name TEXT NOT NULL UNIQUE,
    pipeline_profile_id TEXT,
    generator_model TEXT NOT NULL,
    refiner_model TEXT NOT NULL,
    validator_model TEXT NOT NULL,
    curator_model TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pipeline_profile_id) REFERENCES pipeline_profiles(id)
);

-- Conversations (existing from hive-ai-context.db)
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    consensus_profile_id TEXT,
    total_cost REAL DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    start_time TEXT,
    end_time TEXT,
    success_rate TEXT DEFAULT "100",
    quality_score REAL DEFAULT 0.95,
    consensus_improvement INTEGER DEFAULT 15,
    confidence_level TEXT DEFAULT "High",
    success INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (consensus_profile_id) REFERENCES consensus_profiles(id)
);

-- Messages (existing from hive-ai-context.db)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    stage TEXT,
    model_used TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Usage tracking (replaces JSON files)
CREATE TABLE usage_records (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    conversation_id TEXT,
    action_type TEXT NOT NULL, -- 'conversation', 'model_call', 'sync'
    cost REAL DEFAULT 0.0,
    tokens_input INTEGER DEFAULT 0,
    tokens_output INTEGER DEFAULT 0,
    model_used TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Budget tracking (replaces budget.json)
CREATE TABLE budget_limits (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    period_type TEXT NOT NULL, -- 'daily', 'monthly', 'yearly'
    limit_amount REAL NOT NULL,
    current_spent REAL DEFAULT 0.0,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Sync metadata (from openrouter-data.db)
CREATE TABLE sync_metadata (
    id TEXT PRIMARY KEY,
    sync_type TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,
    providers_synced INTEGER DEFAULT 0,
    models_synced INTEGER DEFAULT 0,
    error_message TEXT,
    next_sync_due TEXT NOT NULL,
    rankings_last_synced TEXT,
    provider_performance_synced TEXT,
    intelligence_version TEXT DEFAULT '1.7.0',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Settings (global app settings)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Consensus settings (for active profile tracking)
CREATE TABLE consensus_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Legacy compatibility tables (for gradual migration)
CREATE TABLE conversation_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    user_id TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);