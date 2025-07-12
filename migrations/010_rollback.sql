-- Rollback for migration 010_remove_local_credit_tracking
-- This script undoes the changes made by migration 010

-- Drop the indexes first
DROP INDEX IF EXISTS idx_conversation_usage_token_unique;
DROP INDEX IF EXISTS idx_conversation_usage_license_key;
DROP INDEX IF EXISTS idx_conversation_usage_timestamp;
DROP INDEX IF EXISTS idx_conversation_usage_verified;

-- Remove columns from conversation_usage
-- Note: SQLite doesn't support DROP COLUMN directly, so we need to recreate the table
-- First, create backup of the original table structure
CREATE TABLE conversation_usage_backup AS 
SELECT id, conversation_id, user_id, timestamp 
FROM conversation_usage;

-- Drop the modified table
DROP TABLE conversation_usage;

-- Recreate original table structure
CREATE TABLE conversation_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    user_id TEXT,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Restore the data
INSERT INTO conversation_usage (id, conversation_id, user_id, timestamp)
SELECT id, conversation_id, user_id, timestamp 
FROM conversation_usage_backup;

-- Drop backup table
DROP TABLE conversation_usage_backup;

-- Recreate original indexes from migration 005
CREATE INDEX idx_usage_conversation ON conversation_usage(conversation_id);
CREATE INDEX idx_usage_timestamp_legacy ON conversation_usage(timestamp);

-- Remove columns from users table (these may not exist if migration 010 failed)
-- Note: SQLite doesn't support DROP COLUMN, so this is informational
-- ALTER TABLE users DROP COLUMN subscription_tier;
-- ALTER TABLE users DROP COLUMN last_d1_sync;

-- Add back the removed columns if needed (per original rollback instruction)
-- ALTER TABLE users ADD COLUMN credits_remaining INTEGER DEFAULT 0;
-- ALTER TABLE users ADD COLUMN subscription_tier TEXT DEFAULT 'free';
-- ALTER TABLE users ADD COLUMN subscription_expires_at TEXT;