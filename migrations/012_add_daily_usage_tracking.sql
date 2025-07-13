-- @ name: Add daily usage tracking for plan recommendations
-- @ description: Track daily usage per user for future plan recommendations
-- @ version: 12

-- Create daily usage tracking table
CREATE TABLE IF NOT EXISTS daily_usage_summary (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    date DATE NOT NULL,
    conversation_count INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0.0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    average_response_time REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, date)
);

-- Create index for efficient querying
CREATE INDEX IF NOT EXISTS idx_daily_usage_user_date ON daily_usage_summary(user_id, date);
CREATE INDEX IF NOT EXISTS idx_daily_usage_date ON daily_usage_summary(date);