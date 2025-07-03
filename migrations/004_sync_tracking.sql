-- @ version: 004
-- @ name: sync_tracking
-- @ description: Create sync tracking and analytics tables for enterprise features
-- @ rollback: DROP TABLE IF EXISTS activity_log; DROP TABLE IF EXISTS performance_metrics; DROP TABLE IF EXISTS model_selection_history; DROP TABLE IF EXISTS profile_templates; DROP TABLE IF EXISTS feature_usage; DROP TABLE IF EXISTS cost_analytics; DROP TABLE IF EXISTS consensus_metrics; DROP TABLE IF EXISTS model_rankings; DROP TABLE IF EXISTS provider_performance; DROP TABLE IF EXISTS pending_sync;

-- Pending sync table for tracking conversations awaiting server verification
-- Note: No foreign keys as this is a temporary queue, not permanent data
CREATE TABLE pending_sync (
    conversation_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    question_hash TEXT NOT NULL,
    conversation_token TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    sync_attempts INTEGER DEFAULT 0,
    last_attempt_at TEXT,
    error_message TEXT
);

-- ===== DYNAMIC CONSENSUS & OPENROUTER INTELLIGENCE TABLES =====

-- Provider performance tracking for intelligent routing
CREATE TABLE provider_performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_internal_id INTEGER NOT NULL,
    provider_name TEXT NOT NULL,
    routing_variant TEXT, -- :nitro, :floor, :online, :free, :default
    avg_latency_ms INTEGER,
    success_rate REAL DEFAULT 1.0,
    cost_per_token REAL,
    uptime_24h REAL DEFAULT 1.0,
    error_rate REAL DEFAULT 0.0,
    last_success_at TEXT,
    last_failure_at TEXT,
    measured_at TEXT DEFAULT CURRENT_TIMESTAMP,
    measurement_period_hours INTEGER DEFAULT 24,
    FOREIGN KEY (model_internal_id) REFERENCES openrouter_models(internal_id)
);

-- OpenRouter programming model rankings
CREATE TABLE model_rankings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_internal_id INTEGER NOT NULL,
    ranking_source TEXT NOT NULL, -- 'openrouter_programming_weekly', 'openrouter_programming_monthly'
    rank_position INTEGER NOT NULL,
    usage_percentage REAL NOT NULL,
    relative_score REAL, -- Normalized score for comparison
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    collected_at TEXT DEFAULT CURRENT_TIMESTAMP,
    data_quality TEXT DEFAULT 'scraped', -- 'scraped', 'api', 'estimated'
    FOREIGN KEY (model_internal_id) REFERENCES openrouter_models(internal_id)
);

-- Consensus effectiveness measurement and A/B testing
CREATE TABLE consensus_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    baseline_model TEXT NOT NULL, -- Single model used for comparison
    baseline_result TEXT NOT NULL, -- Single model output
    consensus_result TEXT NOT NULL, -- 4-stage consensus output
    improvement_score REAL NOT NULL, -- 0-1 score: consensus improvement over baseline
    quality_metrics TEXT NOT NULL, -- JSON: correctness, completeness, architecture, etc.
    cost_comparison TEXT NOT NULL, -- JSON: single model cost vs consensus cost
    time_comparison TEXT NOT NULL, -- JSON: single model time vs consensus time
    user_rating INTEGER, -- 1-5 user satisfaction rating
    question_complexity TEXT NOT NULL, -- minimal/basic/production
    question_category TEXT, -- coding/architecture/debugging/etc.
    effectiveness_notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Cost optimization and budget tracking per conversation
CREATE TABLE cost_analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    total_cost REAL NOT NULL,
    cost_per_stage TEXT NOT NULL, -- JSON: {generator: 0.05, refiner: 0.03, etc.}
    tokens_per_stage TEXT NOT NULL, -- JSON: {generator: {input: 150, output: 300}, etc.}
    routing_optimizations TEXT, -- JSON: {stage: variant_used, savings: amount}
    budget_threshold REAL,
    budget_utilized_percentage REAL,
    cost_efficiency_score REAL, -- quality improvement per dollar spent
    optimization_opportunities TEXT, -- JSON: suggestions for cost reduction
    provider_breakdown TEXT, -- JSON: cost per provider used
    savings_achieved REAL DEFAULT 0.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- OpenRouter feature utilization tracking
CREATE TABLE feature_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    features_used TEXT NOT NULL, -- JSON: {streaming: true, json_mode: false, tools: ['web_search']}
    routing_variants_used TEXT, -- JSON: {generator: ':nitro', refiner: ':default', etc.}
    fallback_events INTEGER DEFAULT 0,
    provider_switches INTEGER DEFAULT 0,
    rate_limit_hits INTEGER DEFAULT 0,
    error_recoveries INTEGER DEFAULT 0,
    performance_metrics TEXT, -- JSON: {avg_latency: 1200, total_tokens: 1500, etc.}
    advanced_features_used TEXT, -- JSON: {function_calling: false, multimodal: false}
    cost_optimizations_applied TEXT, -- JSON: optimization strategies used
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Dynamic profile template definitions
CREATE TABLE profile_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    template_version TEXT DEFAULT '1.0',
    selection_criteria TEXT NOT NULL, -- JSON: complex criteria for model selection
    routing_preferences TEXT, -- JSON: preferred routing variants per stage
    cost_controls TEXT, -- JSON: budget limits and optimization rules
    performance_targets TEXT, -- JSON: latency, quality targets
    scope_applicability TEXT NOT NULL, -- JSON: ['minimal', 'basic', 'production']
    auto_update BOOLEAN DEFAULT 1, -- Whether to auto-update with new rankings
    learning_enabled BOOLEAN DEFAULT 1, -- Whether to learn from usage patterns
    expert_mode BOOLEAN DEFAULT 0, -- Whether this requires expert knowledge
    created_by TEXT, -- User ID or 'system'
    usage_count INTEGER DEFAULT 0,
    effectiveness_score REAL, -- Measured effectiveness over time
    last_updated TEXT DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Model selection history and learning
CREATE TABLE model_selection_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    selection_criteria TEXT NOT NULL, -- JSON: criteria used for selection
    selected_models TEXT NOT NULL, -- JSON: {generator: 'model_id', refiner: 'model_id', etc.}
    selection_reasoning TEXT, -- JSON: why each model was selected
    alternative_models TEXT, -- JSON: other models that were considered
    selection_performance TEXT, -- JSON: how well the selection performed
    user_feedback INTEGER, -- 1-5 rating of selection quality
    learning_applied BOOLEAN DEFAULT 0, -- Whether selection influenced future choices
    template_used TEXT, -- Template ID if selection was template-based
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Performance metrics for analytics and monitoring
CREATE TABLE performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    conversation_id TEXT,
    total_duration INTEGER NOT NULL, -- Total pipeline duration in ms
    overall_efficiency REAL DEFAULT 0, -- Overall efficiency score
    openrouter_latency INTEGER DEFAULT 0, -- OpenRouter API latency
    memory_usage INTEGER DEFAULT 0, -- Memory usage in bytes
    total_cost REAL DEFAULT 0, -- Total cost in USD
    quality_score REAL DEFAULT 0, -- Quality score 0-10
    metrics_data TEXT, -- JSON: Full PerformanceMetrics object
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Real-time activity tracking for dashboards
CREATE TABLE activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL CHECK (event_type IN (
        'query_start', 'query_complete', 'query_error',
        'model_switch', 'fallback', 'rate_limit',
        'stage_start', 'stage_complete', 'stage_error',
        'pipeline_complete', 'export', 'budget_alert',
        'cost_alert', 'quality_alert', 'performance_alert'
    )),
    conversation_id TEXT,
    user_id TEXT,
    stage TEXT CHECK (stage IN ('generator', 'refiner', 'validator', 'curator')),
    model_used TEXT,
    provider_name TEXT,
    cost REAL,
    duration_ms INTEGER,
    tokens_used INTEGER,
    error_message TEXT,
    metadata TEXT, -- JSON for additional event-specific data
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);