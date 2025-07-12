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

-- Create trigger to automatically update daily usage
CREATE TRIGGER IF NOT EXISTS update_daily_usage_after_conversation
AFTER INSERT ON conversation_usage
BEGIN
    INSERT INTO daily_usage_summary (
        id,
        user_id,
        date,
        conversation_count,
        created_at,
        updated_at
    ) VALUES (
        lower(hex(randomblob(16))),
        NEW.user_id,
        date(NEW.timestamp),
        1,
        datetime('now'),
        datetime('now')
    )
    ON CONFLICT(user_id, date) DO UPDATE SET
        conversation_count = conversation_count + 1,
        updated_at = datetime('now');
END;

-- Backfill existing data
INSERT INTO daily_usage_summary (id, user_id, date, conversation_count, created_at, updated_at)
SELECT 
    lower(hex(randomblob(16))) as id,
    user_id,
    date(timestamp) as date,
    COUNT(*) as conversation_count,
    datetime('now') as created_at,
    datetime('now') as updated_at
FROM conversation_usage
GROUP BY user_id, date(timestamp)
ON CONFLICT(user_id, date) DO NOTHING;